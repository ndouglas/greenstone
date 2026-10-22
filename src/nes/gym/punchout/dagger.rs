//! DAgger: the teacher labels its own fights, then each brain fights, the
//! teacher labels where the brain went, and the labels pile up round by round.

use std::collections::HashMap;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Sender};
use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::nes::gym::punchout::brains::{Brain, BrainKind, Projection};
use crate::nes::gym::punchout::fight::{run_fight_for, Score, MAX_STEPS};
use crate::nes::gym::punchout::moves::{moves, Move, Phase};
use crate::nes::gym::punchout::start::{Opponent, HELD_OUT, TRAIN};
use crate::nes::gym::punchout::teacher::label;
use crate::nes::gym::reservoir::demo::guarded;
use crate::nes::gym::reservoir::ridge::{fit, Readout};
use crate::nes::gym::reservoir::Lcg;
use crate::nes::gym::{NesEmulator, NesState};

/// DAgger rounds after the teacher's round 0.
pub const ROUNDS: usize = 4;
/// Teacher fights per training opponent in round 0.
pub const TEACHER_FIGHTS: usize = 3;
/// A student fight is labelled at every this-many-th step where Mac can act.
pub const STUDENT_LABEL_EVERY: usize = 4;
/// Ridge penalty for every readout.
const LAMBDA: f64 = 1.0;
/// Share of a seeded teacher fight's moves that are random.
const EXPLORE: f64 = 0.1;
/// Bytes of work RAM the cheat reads.
const RAM_LEN: u16 = 2048;
const TEACHER_FILE: &str = "teacher.jsonl";
const GATE_FILE: &str = "gate.json";
const RESULTS_FILE: &str = "results.json";

/// A fight's rows for one brain, and the teacher's labels for them.
type Rows = (Vec<Vec<f32>>, Vec<Move>);

/// How much a run does. `Budget::default()` is the experiment; a smaller one
/// is for smoke runs of the command line.
#[derive(Debug, Clone, Copy)]
pub struct Budget {
  pub max_steps: usize,
  pub rounds: usize,
  /// Run phase one's stop gate.
  pub gate: bool,
}

impl Default for Budget {
  fn default() -> Budget {
    Budget {
      max_steps: MAX_STEPS,
      rounds: ROUNDS,
      gate: true,
    }
  }
}

/// One fight: what the screen looked like at every step, and the teacher's
/// labels (with the game's RAM, for the cheat) at the steps it labelled.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FightRecord {
  pub opponent: Opponent,
  /// "teacher", or the name of the brain that fought.
  pub driver: String,
  pub round: usize,
  pub index: usize,
  pub projections: Vec<Vec<f32>>,
  pub labels: Vec<(usize, Move)>,
  pub rams: Vec<(usize, Vec<u8>)>,
  pub score: Score,
}

/// Who chooses Mac's moves in a recorded fight.
pub enum Driver<'a> {
  /// The teacher labels every step and plays its label; with a seed, a
  /// tenth of its moves are random instead (its labels stay its own), so a
  /// deterministic game gives more than one fight.
  Teacher(Option<u64>),
  /// A brain and its readout; the teacher labels afterwards.
  Student(&'a mut Brain, &'a Readout),
}

pub fn record_fight(rom: &[u8], opponent: Opponent, phase: Phase, driver: Driver, label_every: usize, threads: usize) -> Result<FightRecord, String> {
  record_with(rom, opponent, phase, driver, label_every, threads, MAX_STEPS).map(|(record, _)| record)
}

/// `record_fight` for at most `max_steps` steps, also returning the
/// student's rows at the labelled steps (none for the teacher).
fn record_with(
  rom: &[u8],
  opponent: Opponent,
  phase: Phase,
  mut driver: Driver,
  label_every: usize,
  threads: usize,
  max_steps: usize,
) -> Result<(FightRecord, Vec<Vec<f32>>), String> {
  let projection = Projection::new();
  let mut rng = match driver {
    Driver::Teacher(Some(seed)) => Some(Lcg::new(seed)),
    _ => None,
  };
  let (mut projections, mut labels, mut rams, mut rows, mut saved) = (vec![], vec![], vec![], vec![], vec![]);
  let mut active = 0;
  let score = run_fight_for(rom, opponent, max_steps, &mut |s| {
    let proj = projection.project(&s.emu.framebuffer_grayscale_84x84());
    let chosen = match &mut driver {
      Driver::Teacher(_) if !s.can_act => Move::Idle,
      Driver::Teacher(_) => {
        let mv = label(s.emu, phase);
        labels.push((s.index, mv));
        rams.push((s.index, s.emu.ram_read_range(0, RAM_LEN)));
        let explore = rng.as_mut().map(|r| (r.next_f64() < EXPLORE, r.next_f64()));
        match explore {
          Some((true, pick)) => {
            let options = moves(phase);
            options[(pick * options.len() as f64) as usize % options.len()]
          }
          _ => mv,
        }
      }
      Driver::Student(brain, readout) => {
        let ram = s.emu.ram_read_range(0, RAM_LEN);
        let row = brain.row(&proj, &ram);
        let mv = choose(readout, &row, phase);
        if s.can_act {
          if active % label_every == 0 {
            saved.push((s.index, s.emu.save_state()));
            rams.push((s.index, ram));
            rows.push(row);
          }
          active += 1;
        }
        mv
      }
    };
    projections.push(proj);
    chosen
  })?;
  if !saved.is_empty() {
    labels = label_saved(rom, phase, &saved, threads)?;
  }
  let driver = match driver {
    Driver::Teacher(_) => "teacher",
    Driver::Student(..) => "student",
  };
  let record = FightRecord {
    opponent,
    driver: driver.to_string(),
    round: 0,
    index: 0,
    projections,
    labels,
    rams,
    score,
  };
  Ok((record, rows))
}

/// The teacher's label for each saved state, on `threads` threads.
fn label_saved(rom: &[u8], phase: Phase, saved: &[(usize, NesState)], threads: usize) -> Result<Vec<(usize, Move)>, String> {
  let chunk = saved.len().div_ceil(threads.max(1)).max(1);
  std::thread::scope(|scope| {
    let handles: Vec<_> = saved
      .chunks(chunk)
      .map(|part| {
        scope.spawn(move || {
          guarded(|| {
            let mut emu = NesEmulator::new(rom);
            part
              .iter()
              .map(|(i, state)| {
                emu.load_state(state);
                (*i, label(&mut emu, phase))
              })
              .collect::<Vec<_>>()
          })
        })
      })
      .collect();
    let mut out = Vec::with_capacity(saved.len());
    for h in handles {
      out.extend(h.join().map_err(|_| "a labelling thread panicked".to_string())??);
    }
    Ok(out)
  })
}

/// Replay a fight's screens through a fresh brain: its rows and the
/// teacher's labels at the labelled steps.
pub fn rows_for(kind: BrainKind, record: &FightRecord, tetris: &[u8]) -> Result<(Vec<Vec<f32>>, Vec<Move>), String> {
  let labels: HashMap<usize, Move> = record.labels.iter().copied().collect();
  let rams: HashMap<usize, &Vec<u8>> = record.rams.iter().map(|(i, r)| (*i, r)).collect();
  let mut brain = Brain::new(kind, tetris)?;
  let (mut rows, mut out) = (vec![], vec![]);
  for (i, proj) in record.projections.iter().enumerate() {
    let label = labels.get(&i);
    if kind == BrainKind::Cheat {
      // The cheat has no memory: it needs only the labelled steps' RAM.
      if let Some(&mv) = label {
        let ram = rams.get(&i).ok_or_else(|| format!("no RAM at labelled step {}", i))?;
        rows.push(brain.row(proj, ram));
        out.push(mv);
      }
      continue;
    }
    let row = brain.row(proj, &[]);
    if let Some(&mv) = label {
      rows.push(row);
      out.push(mv);
    }
  }
  Ok((rows, out))
}

/// Ridge on one-hot targets over the phase's moves.
fn fit_rows(rows: &[Vec<f32>], labels: &[Move], phase: Phase) -> Result<Readout, String> {
  let k = moves(phase).len();
  let targets: Vec<Vec<f32>> = labels
    .iter()
    .map(|m| {
      let mut t = vec![0.0; k];
      t[m.index_in(phase)] = 1.0;
      t
    })
    .collect();
  fit(rows, &targets, LAMBDA)
}

pub fn fit_brain(kind: BrainKind, records: &[FightRecord], phase: Phase, tetris: &[u8]) -> Result<Readout, String> {
  let (mut rows, mut labels) = (vec![], vec![]);
  for record in records {
    let (r, l) = rows_for(kind, record, tetris)?;
    rows.extend(r);
    labels.extend(l);
  }
  fit_rows(&rows, &labels, phase)
}

/// The readout's best move; ties go to the earlier move.
pub fn choose(readout: &Readout, row: &[f32], phase: Phase) -> Move {
  let out = readout.predict(row);
  let mut best = 0;
  for (i, &v) in out.iter().enumerate() {
    if v > out[best] {
      best = i;
    }
  }
  moves(phase)[best]
}

/// One phase's fight records, one JSON line per fight.
pub struct Store {
  root: PathBuf,
}

impl Store {
  pub fn new(root: &Path, phase: Phase) -> Store {
    Store {
      root: root.join(format!("phase{}", phase.number())),
    }
  }

  pub fn path(&self, file: &str) -> PathBuf {
    self.root.join(file)
  }

  /// Append one record as a line. A line left unfinished by a killed run is
  /// closed first, so the new record starts a line of its own.
  pub fn append(&self, file: &str, record: &FightRecord) -> Result<(), String> {
    let path = self.path(file);
    std::fs::create_dir_all(&self.root).map_err(|e| format!("create {}: {}", self.root.display(), e))?;
    let mut f = std::fs::OpenOptions::new()
      .create(true)
      .read(true)
      .append(true)
      .open(&path)
      .map_err(|e| format!("open {}: {}", path.display(), e))?;
    let mut line = serde_json::to_string(record).map_err(|e| e.to_string())?;
    line.push('\n');
    if ends_unfinished(&mut f).map_err(|e| e.to_string())? {
      line.insert(0, '\n');
    }
    f.write_all(line.as_bytes()).map_err(|e| e.to_string())
  }

  /// Every whole record in `file`; lines that do not parse are skipped.
  pub fn load(&self, file: &str) -> Vec<FightRecord> {
    std::fs::read_to_string(self.path(file))
      .unwrap_or_default()
      .lines()
      .filter_map(|line| serde_json::from_str(line).ok())
      .collect()
  }
}

fn ends_unfinished(f: &mut std::fs::File) -> std::io::Result<bool> {
  if f.metadata()?.len() == 0 {
    return Ok(false);
  }
  f.seek(SeekFrom::End(-1))?;
  let mut last = [0u8];
  f.read_exact(&mut last)?;
  Ok(last[0] != b'\n')
}

fn threads() -> usize {
  std::thread::available_parallelism().map(|n| n.get()).unwrap_or(1)
}

/// The teacher's random-move seed for fight `index`: none for the first.
fn teacher_seed(opponent: Opponent, index: usize) -> Option<u64> {
  (index > 0).then(|| 0x7EAC_4E50_0000 + opponent as u64 * 1000 + index as u64)
}

/// Run `fight(index)`; on an error, log it and try once more with
/// `retry_index`, then give up on this fight.
fn with_retry<T>(what: &str, index: usize, retry_index: usize, tx: &Sender<String>, fight: impl Fn(usize) -> Result<T, String>) -> Option<T> {
  for i in [index, retry_index] {
    match fight(i) {
      Ok(done) => return Some(done),
      Err(e) => {
        let _ = tx.send(format!("{} (index {}) failed: {}", what, i, e));
      }
    }
  }
  let _ = tx.send(format!("{}: skipped after a retry", what));
  None
}

/// `work` on a scoped thread per item; results in item order.
fn parallel<I: Sync, T: Send>(items: &[I], work: impl Fn(&I) -> T + Sync) -> Vec<T> {
  std::thread::scope(|scope| {
    let work = &work;
    let handles: Vec<_> = items.iter().map(|item| scope.spawn(move || work(item))).collect();
    handles.into_iter().map(|h| h.join().expect("workers return their errors")).collect()
  })
}

/// `parallel`, with each worker's log lines passed to `log` as they come.
fn fan_out<I: Sync, T: Send>(items: &[I], log: &mut dyn FnMut(&str), work: impl Fn(&I, &Sender<String>) -> T + Sync) -> Vec<T> {
  let (tx, rx) = channel::<String>();
  std::thread::scope(|scope| {
    let work = &work;
    let handles: Vec<_> = items
      .iter()
      .map(|item| {
        let tx = tx.clone();
        scope.spawn(move || work(item, &tx))
      })
      .collect();
    drop(tx);
    for line in rx {
      log(&line);
    }
    handles.into_iter().map(|h| h.join().expect("workers return their errors")).collect()
  })
}

/// Round 0: three teacher fights per training opponent, recorded in parallel.
fn teacher_fights(rom: &[u8], phase: Phase, budget: Budget, store: &Store, log: &mut dyn FnMut(&str)) -> Vec<FightRecord> {
  let have = store.load(TEACHER_FILE);
  let missing: Vec<(Opponent, usize)> = TRAIN
    .iter()
    .flat_map(|&o| (0..TEACHER_FIGHTS).map(move |i| (o, i)))
    .filter(|&(o, i)| !have.iter().any(|r| r.opponent == o && r.index % TEACHER_FIGHTS == i))
    .collect();
  if !missing.is_empty() {
    log(&format!("phase {}: recording {} teacher fights", phase.number(), missing.len()));
  }
  let lock = Mutex::new(());
  fan_out(&missing, log, |&(opponent, index), tx| {
    let what = format!("teacher vs {} #{}", opponent.name(), index);
    let record = with_retry(&what, index, index + TEACHER_FIGHTS, tx, |i| {
      let (mut r, _) = record_with(rom, opponent, phase, Driver::Teacher(teacher_seed(opponent, i)), 1, 1, budget.max_steps)?;
      r.index = i;
      Ok(r)
    });
    if let Some(record) = record {
      let _guard = lock.lock().unwrap_or_else(|e| e.into_inner());
      let saved = store.append(TEACHER_FILE, &record);
      let _ = tx.send(match saved {
        Ok(()) => format!("{}: {} labels, {:.2} hits/min", what, record.labels.len(), record.score.hits_per_minute()),
        Err(e) => format!("{}: could not save: {}", what, e),
      });
    }
  });
  store.load(TEACHER_FILE)
}

/// Rows and labels for each record, in parallel over the records.
fn rows_of(kind: BrainKind, records: &[FightRecord], tetris: &[u8]) -> Result<Vec<Rows>, String> {
  parallel(records, |record| guarded(|| rows_for(kind, record, tetris)).and_then(|r| r))
    .into_iter()
    .collect()
}

fn fit_cached<'a>(cached: impl Iterator<Item = &'a Rows>, phase: Phase) -> Result<Readout, String> {
  let (mut rows, mut labels) = (vec![], vec![]);
  for (r, l) in cached {
    rows.extend(r.iter().cloned());
    labels.extend(l.iter().copied());
  }
  fit_rows(&rows, &labels, phase)
}

/// One fight by a fitted brain, unrecorded.
fn student_score(
  rom: &[u8],
  tetris: &[u8],
  kind: BrainKind,
  readout: &Readout,
  opponent: Opponent,
  phase: Phase,
  max_steps: usize,
) -> Result<Score, String> {
  let projection = Projection::new();
  let mut brain = Brain::new(kind, tetris)?;
  run_fight_for(rom, opponent, max_steps, &mut |s| {
    let proj = projection.project(&s.emu.framebuffer_grayscale_84x84());
    let ram = s.emu.ram_read_range(0, RAM_LEN);
    choose(readout, &brain.row(&proj, &ram), phase)
  })
}

fn idle_score(rom: &[u8], opponent: Opponent, max_steps: usize) -> Result<Score, String> {
  run_fight_for(rom, opponent, max_steps, &mut |_| Move::Idle)
}

/// After round 0 of defence: does any brain fitted on the teacher's fights
/// take fewer hits per minute from Glass Joe than standing still? The
/// answer is kept in gate.json, so a resumed run asks only once.
fn stop_gate(
  rom: &[u8],
  tetris: &[u8],
  per_kind: &[(BrainKind, Vec<Rows>)],
  budget: Budget,
  store: &Store,
  log: &mut dyn FnMut(&str),
) -> Result<(), String> {
  let path = store.path(GATE_FILE);
  if let Some(gate) = std::fs::read_to_string(&path).ok().and_then(|s| serde_json::from_str::<Value>(&s).ok()) {
    return if gate["passed"] == json!(true) {
      Ok(())
    } else {
      Err(format!("stop gate: {}", gate))
    };
  }
  let idle = idle_score(rom, Opponent::GlassJoe, budget.max_steps)?.hits_per_minute();
  log(&format!("stop gate: always-Idle takes {:.2} hits/min from Glass Joe", idle));
  let results = fan_out(per_kind, log, |(kind, rows), tx| {
    let hpm = fit_cached(rows.iter(), Phase::Defence)
      .and_then(|readout| student_score(rom, tetris, *kind, &readout, Opponent::GlassJoe, Phase::Defence, budget.max_steps))
      .map(|score| score.hits_per_minute());
    let _ = tx.send(format!("stop gate: {} {:?}", kind.name(), hpm));
    (kind.name(), hpm)
  });
  let passed = results.iter().any(|(_, h)| matches!(h, Ok(h) if *h < idle));
  let kinds: serde_json::Map<String, Value> = results
    .into_iter()
    .map(|(name, h)| (name, h.map_or_else(|e| json!({ "error": e }), |h| json!(h))))
    .collect();
  let gate = json!({ "idle_hits_per_minute": idle, "hits_per_minute": kinds, "passed": passed });
  std::fs::create_dir_all(&store.root).map_err(|e| e.to_string())?;
  std::fs::write(&path, serde_json::to_string_pretty(&gate).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
  if passed {
    Ok(())
  } else {
    Err(format!("stop gate: no brain beats always-Idle: {}", gate))
  }
}

/// Rounds 1..=ROUNDS for one kind: fit on everything so far, fight each
/// training opponent once, let the teacher label every fourth step.
fn train_kind(
  rom: &[u8],
  tetris: &[u8],
  phase: Phase,
  kind: BrainKind,
  teacher_rows: &[Rows],
  budget: Budget,
  store: &Store,
  tx: &Sender<String>,
) -> Result<(), String> {
  let file = format!("{}.jsonl", kind.name());
  let mut own = store.load(&file);
  let mut own_rows: Vec<(usize, Rows)> = own.iter().map(|r| r.round).zip(rows_of(kind, &own, tetris)?).collect();
  for round in 1..=budget.rounds {
    let missing: Vec<Opponent> = TRAIN
      .iter()
      .copied()
      .filter(|&o| !own.iter().any(|r| r.round == round && r.opponent == o))
      .collect();
    if missing.is_empty() {
      continue;
    }
    let earlier = own_rows.iter().filter(|(r, _)| *r < round).map(|(_, rows)| rows);
    let readout = fit_cached(teacher_rows.iter().chain(earlier), phase)?;
    let fought = parallel(&missing, |&opponent| {
      let what = format!("{} round {} vs {}", kind.name(), round, opponent.name());
      let done = with_retry(&what, 0, 1, tx, |i| {
        let mut brain = Brain::new(kind, tetris)?;
        let driver = Driver::Student(&mut brain, &readout);
        let (mut record, rows) = record_with(rom, opponent, phase, driver, STUDENT_LABEL_EVERY, 1, budget.max_steps)?;
        record.index = i;
        Ok((record, rows))
      });
      (what, done)
    });
    for (what, done) in fought {
      let Some((mut record, rows)) = done else { continue };
      record.driver = kind.name();
      record.round = round;
      store.append(&file, &record)?;
      let _ = tx.send(format!(
        "{}: {:.2} hits/min, {} labels",
        what,
        record.score.hits_per_minute(),
        record.labels.len()
      ));
      let labels = record.labels.iter().map(|&(_, m)| m).collect();
      own_rows.push((round, (rows, labels)));
      own.push(record);
    }
  }
  Ok(())
}

/// Whether `kind` has fought every training opponent in every round.
fn rounds_done(kind: BrainKind, budget: Budget, store: &Store) -> bool {
  let own = store.load(&format!("{}.jsonl", kind.name()));
  (1..=budget.rounds).all(|round| TRAIN.iter().all(|&o| own.iter().any(|r| r.round == round && r.opponent == o)))
}

pub fn train(rom: &[u8], tetris: &[u8], phase: Phase, kinds: &[BrainKind], store: &Store, log: &mut dyn FnMut(&str)) -> Result<(), String> {
  train_with(rom, tetris, phase, kinds, Budget::default(), store, log)
}

pub fn train_with(
  rom: &[u8],
  tetris: &[u8],
  phase: Phase,
  kinds: &[BrainKind],
  budget: Budget,
  store: &Store,
  log: &mut dyn FnMut(&str),
) -> Result<(), String> {
  let teacher = teacher_fights(rom, phase, budget, store, log);
  if teacher.is_empty() {
    return Err("no teacher fights were recorded".to_string());
  }
  // Replaying the teacher's fights is the slow part of a resume: skip the
  // brains with every round done, unless the stop gate still needs them all.
  let gate_pending = phase == Phase::Defence && budget.gate && !store.path(GATE_FILE).exists();
  let kinds: Vec<BrainKind> = kinds
    .iter()
    .copied()
    .filter(|&k| gate_pending || !rounds_done(k, budget, store))
    .collect();
  if kinds.is_empty() {
    log(&format!("phase {}: every round is done", phase.number()));
    return Ok(());
  }
  log(&format!(
    "phase {}: replaying {} teacher fights through {} brains",
    phase.number(),
    teacher.len(),
    kinds.len()
  ));
  let kinds = &kinds[..];
  let replayed = fan_out(kinds, log, |&kind, _| rows_of(kind, &teacher, tetris));
  let per_kind: Vec<(BrainKind, Vec<Rows>)> = kinds
    .iter()
    .copied()
    .zip(replayed)
    .map(|(k, rows)| rows.map(|r| (k, r)))
    .collect::<Result<_, _>>()?;
  if phase == Phase::Defence && budget.gate {
    stop_gate(rom, tetris, &per_kind, budget, store, log)?;
  }
  let errors: Vec<String> = fan_out(&per_kind, log, |(kind, rows), tx| {
    train_kind(rom, tetris, phase, *kind, rows, budget, store, tx).map_err(|e| format!("{}: {}", kind.name(), e))
  })
  .into_iter()
  .filter_map(Result::err)
  .collect();
  if errors.is_empty() {
    Ok(())
  } else {
    Err(errors.join("; "))
  }
}

/// A kind's readout fitted on every label it has: the teacher's and its own.
pub fn final_readout(kind: BrainKind, phase: Phase, store: &Store, tetris: &[u8]) -> Result<Readout, String> {
  let teacher = rows_of(kind, &store.load(TEACHER_FILE), tetris)?;
  let own = rows_of(kind, &store.load(&format!("{}.jsonl", kind.name())), tetris)?;
  fit_cached(teacher.iter().chain(own.iter()), phase)
}

/// A fight's score as results.json reports it.
fn score_json(score: &Score, idle_hits_per_minute: f64) -> Value {
  let hpm = score.hits_per_minute();
  let dodge = (idle_hits_per_minute > 0.0).then(|| 1.0 - hpm / idle_hits_per_minute);
  json!({
    "fights": 1,
    "steps": score.steps,
    "active_steps": score.active_steps,
    "hits": score.hits,
    "damage_taken": score.damage_taken,
    "damage_dealt": score.damage_dealt,
    "knockdowns_dealt": score.knockdowns_dealt,
    "mac_down_at": score.mac_down_at,
    "won": score.won,
    "hits_per_minute": hpm,
    "dodge_rate": dodge,
    "survival_seconds": score.survival_seconds(),
  })
}

/// Fit each kind on the teacher alone (round 0) and on all its labels (its
/// last round), fight every opponent once with each, and write results.json.
/// The game and the brains are deterministic, so a second fight with the
/// same readout would be the same fight.
pub fn test(rom: &[u8], tetris: &[u8], phase: Phase, kinds: &[BrainKind], store: &Store) -> Result<Value, String> {
  test_with(rom, tetris, phase, kinds, Budget::default(), store, RESULTS_FILE)
}

/// `test`, within `budget`, writing to `file` in the phase's directory.
pub fn test_with(rom: &[u8], tetris: &[u8], phase: Phase, kinds: &[BrainKind], budget: Budget, store: &Store, file: &str) -> Result<Value, String> {
  let opponents: Vec<Opponent> = TRAIN.iter().chain(HELD_OUT.iter()).copied().collect();
  let idle: Vec<Score> = parallel(&opponents, |&o| idle_score(rom, o, budget.max_steps))
    .into_iter()
    .collect::<Result<_, _>>()?;
  let idle_hpm: Vec<f64> = idle.iter().map(Score::hits_per_minute).collect();
  let teacher = store.load(TEACHER_FILE);
  let per_kind = parallel(kinds, |&kind| -> Result<Value, String> {
    let own = store.load(&format!("{}.jsonl", kind.name()));
    let last = own.iter().map(|r| r.round).max().unwrap_or(0);
    let teacher_rows = rows_of(kind, &teacher, tetris)?;
    let own_rows = rows_of(kind, &own, tetris)?;
    let mut fits = vec![(0, fit_cached(teacher_rows.iter(), phase)?)];
    if last > 0 {
      fits.push((last, fit_cached(teacher_rows.iter().chain(own_rows.iter()), phase)?));
    }
    let mut by_round = serde_json::Map::new();
    for (round, readout) in &fits {
      let scores = parallel(&opponents, |&o| student_score(rom, tetris, kind, readout, o, phase, budget.max_steps));
      let mut by_opponent = serde_json::Map::new();
      for ((o, score), idle) in opponents.iter().zip(scores).zip(&idle_hpm) {
        by_opponent.insert(
          o.name().to_string(),
          score.map_or_else(|e| json!({ "error": e }), |s| score_json(&s, *idle)),
        );
      }
      by_round.insert(format!("round {}", round), Value::Object(by_opponent));
    }
    Ok(Value::Object(by_round))
  });
  let mut kinds_json = serde_json::Map::new();
  for (kind, result) in kinds.iter().zip(per_kind) {
    kinds_json.insert(kind.name(), result.unwrap_or_else(|e| json!({ "error": e })));
  }
  let idle_json: serde_json::Map<String, Value> = opponents
    .iter()
    .zip(&idle)
    .map(|(o, s)| (o.name().to_string(), score_json(s, s.hits_per_minute())))
    .collect();
  let results = json!({ "phase": phase.number(), "idle": idle_json, "kinds": kinds_json });
  std::fs::create_dir_all(&store.root).map_err(|e| e.to_string())?;
  let text = serde_json::to_string_pretty(&results).map_err(|e| e.to_string())?;
  std::fs::write(store.path(file), text).map_err(|e| e.to_string())?;
  Ok(results)
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::nes::gym::punchout::brains::PROJ;
  use crate::nes::gym::punchout::PUNCH_OUT;
  use crate::nes::gym::reservoir::rom_or_skip;

  fn record(round: usize, labels: usize) -> FightRecord {
    FightRecord {
      opponent: Opponent::GlassJoe,
      driver: "stack1".into(),
      round,
      index: 0,
      projections: vec![vec![0.0; PROJ]; labels * 4],
      labels: (0..labels).map(|i| (i * 4, Move::Idle)).collect(),
      rams: vec![],
      score: Score::default(),
    }
  }

  #[test]
  fn resume_after_a_truncated_line_keeps_every_whole_record() {
    let dir = std::env::temp_dir().join(format!("punchout-store-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let store = Store::new(&dir, Phase::Defence);
    store.append("stack1.jsonl", &record(1, 2)).unwrap();
    let path = dir.join("phase1").join("stack1.jsonl");
    let mut f = std::fs::OpenOptions::new().append(true).open(&path).unwrap();
    std::io::Write::write_all(&mut f, b"{\"opponent\":").unwrap();
    store.append("stack1.jsonl", &record(2, 3)).unwrap();
    let back = store.load("stack1.jsonl");
    assert_eq!(back.iter().map(|r| r.round).collect::<Vec<_>>(), vec![1, 2]);
  }

  #[test]
  fn a_kind_is_done_when_every_round_has_every_training_opponent() {
    let dir = std::env::temp_dir().join(format!("punchout-done-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let store = Store::new(&dir, Phase::Defence);
    let budget = Budget {
      max_steps: 10,
      rounds: 1,
      gate: false,
    };
    assert!(!rounds_done(BrainKind::Stack(1), budget, &store));
    for (n, &o) in TRAIN.iter().enumerate() {
      assert!(!rounds_done(BrainKind::Stack(1), budget, &store), "{} of 3 fought", n);
      store.append("stack1.jsonl", &FightRecord { opponent: o, ..record(1, 1) }).unwrap();
    }
    assert!(rounds_done(BrainKind::Stack(1), budget, &store));
    assert!(!rounds_done(BrainKind::Stack(1), Budget { rounds: 2, ..budget }, &store));
    let _ = std::fs::remove_dir_all(&dir);
  }

  #[test]
  fn rows_come_only_from_labelled_steps() {
    let (rows, labels) = rows_for(BrainKind::Stack(1), &record(0, 5), &[]).unwrap();
    assert_eq!(rows.len(), 5);
    assert_eq!(labels.len(), 5);
  }

  #[test]
  fn choose_takes_the_argmax_and_breaks_ties_to_idle() {
    let r = Readout {
      d: 1,
      k: 4,
      weights: vec![0.0, 0.0, 0.0, 0.0],
      logistic: false,
    };
    assert_eq!(choose(&r, &[1.0], Phase::Defence), Move::Idle);
    let r = Readout {
      d: 1,
      k: 4,
      weights: vec![0.0, 0.0, 1.0, 0.0],
      logistic: false,
    };
    assert_eq!(choose(&r, &[1.0], Phase::Defence), Move::DodgeRight);
  }

  #[test]
  fn the_label_set_grows_each_round() {
    let records = vec![record(0, 3), record(1, 2), record(2, 2)];
    let n = |upto: usize| records.iter().filter(|r| r.round <= upto).map(|r| r.labels.len()).sum::<usize>();
    assert!(n(0) < n(1) && n(1) < n(2));
    let fitted = fit_brain(BrainKind::Stack(1), &records, Phase::Defence, &[]).unwrap();
    assert_eq!(fitted.k, 4);
  }

  #[test]
  fn a_student_fight_labels_only_steps_where_mac_can_act() {
    let Some(rom) = rom_or_skip(PUNCH_OUT) else { return };
    let mut brain = Brain::new(BrainKind::Stack(1), &[]).unwrap();
    let readout = Readout {
      d: 1 + PROJ,
      k: 4,
      weights: vec![0.0; (1 + PROJ) * 4],
      logistic: false,
    };
    let rec = record_fight(&rom, Opponent::GlassJoe, Phase::Defence, Driver::Student(&mut brain, &readout), 4, 4).unwrap();
    assert_eq!(rec.projections.len(), rec.score.steps);
    assert!(!rec.labels.is_empty());
    assert!(rec.labels.len() <= rec.score.active_steps / 4 + 1);
  }

  fn record_fight_steps(rom: &[u8], opponent: Opponent, phase: Phase, driver: Driver, steps: usize) -> Result<FightRecord, String> {
    record_with(rom, opponent, phase, driver, 1, 1, steps).map(|(record, _)| record)
  }

  /// The game is deterministic, so a second teacher fight differs from the
  /// first only through its seeded random moves; its labels are still the
  /// teacher's, which the first fight's labels at the same steps confirm.
  #[test]
  fn a_seeded_teacher_fight_differs_but_labels_like_the_teacher() {
    let Some(rom) = rom_or_skip(PUNCH_OUT) else { return };
    let cut = |rec: &mut FightRecord| {
      rec.projections.truncate(40);
      rec.labels.retain(|&(i, _)| i < 40);
    };
    let pure = record_fight_steps(&rom, Opponent::GlassJoe, Phase::Defence, Driver::Teacher(None), 40).unwrap();
    let noisy = record_fight_steps(&rom, Opponent::GlassJoe, Phase::Defence, Driver::Teacher(Some(7)), 40).unwrap();
    let (mut pure, mut noisy) = (pure, noisy);
    cut(&mut pure);
    cut(&mut noisy);
    assert_ne!(pure.projections, noisy.projections, "the seeded fight went elsewhere");
    let first_divergence = pure.projections.iter().zip(&noisy.projections).position(|(a, b)| a != b).unwrap();
    for (&(i, a), &(j, b)) in pure.labels.iter().zip(&noisy.labels) {
      if i == j && i + 1 < first_divergence {
        assert_eq!(a, b, "same screen, same teacher label at step {}", i);
      }
    }
  }
}
