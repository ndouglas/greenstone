//! The film's takes: which fights the Punch-Out!! film shows, how each is
//! recorded, and the score each should reproduce.

use std::path::Path;

use serde_json::{json, Value};

use crate::nes::gym::punchout::brains::BrainKind;
use crate::nes::gym::punchout::dagger::{final_readout, round0_readout, Store};
use crate::nes::gym::punchout::fight::MAX_STEPS;
use crate::nes::gym::punchout::moves::Phase;
use crate::nes::gym::punchout::start::Opponent;
use crate::nes::gym::punchout::takes::{record_boot_take, record_idle_take, record_take, record_teacher_take};

/// Frames of the boot take: thirty seconds from power-on.
pub const BOOT_FRAMES: usize = 1800;

/// Which of a brain's readouts a take fights with.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Round {
  /// Fitted on the teacher's labels alone.
  Zero,
  /// Fitted on every label, after the last DAgger round.
  Final,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FilmTake {
  Boot,
  Idle {
    phase: Phase,
    opponent: Opponent,
  },
  Teacher {
    phase: Phase,
    opponent: Opponent,
  },
  Brain {
    phase: Phase,
    kind: BrainKind,
    opponent: Opponent,
    round: Round,
  },
}

use FilmTake::{Boot, Brain, Idle, Teacher};
use Opponent::{BaldBull, GlassJoe, VonKaiser};
use Phase::{Counter, Defence, Fight};

const fn final_take(phase: Phase, kind: BrainKind, opponent: Opponent) -> FilmTake {
  Brain {
    phase,
    kind,
    opponent,
    round: Round::Final,
  }
}

/// Every take the film uses, in recording order.
pub const FILM_TAKES: [FilmTake; 19] = [
  Boot,
  Idle {
    phase: Defence,
    opponent: GlassJoe,
  },
  Teacher {
    phase: Defence,
    opponent: GlassJoe,
  },
  Teacher {
    phase: Fight,
    opponent: VonKaiser,
  },
  final_take(Defence, BrainKind::Bank, GlassJoe),
  final_take(Defence, BrainKind::Echo(0.5), GlassJoe),
  final_take(Defence, BrainKind::Stack(1), VonKaiser),
  final_take(Defence, BrainKind::Stack(2), VonKaiser),
  final_take(Defence, BrainKind::Stack(4), VonKaiser),
  final_take(Defence, BrainKind::Stack(8), VonKaiser),
  final_take(Defence, BrainKind::Echo(0.2), VonKaiser),
  final_take(Defence, BrainKind::Echo(0.5), VonKaiser),
  final_take(Defence, BrainKind::Echo(0.9), VonKaiser),
  final_take(Counter, BrainKind::Stack(4), VonKaiser),
  final_take(Fight, BrainKind::Stack(4), GlassJoe),
  final_take(Counter, BrainKind::Stack(4), BaldBull),
  final_take(Fight, BrainKind::Cheat, VonKaiser),
  final_take(Fight, BrainKind::Stack(4), VonKaiser),
  Brain {
    phase: Counter,
    kind: BrainKind::Stack(4),
    opponent: BaldBull,
    round: Round::Zero,
  },
];

impl FilmTake {
  /// The take's directory name, e.g. `p2-stack4-r0-bald-bull`.
  pub fn name(&self) -> String {
    match *self {
      Boot => "boot".to_string(),
      Idle { phase, opponent } => format!("p{}-idle-{}", phase.number(), opponent.name()),
      Teacher { phase, opponent } => format!("p{}-teacher-{}", phase.number(), opponent.name()),
      Brain {
        phase,
        kind,
        opponent,
        round,
      } => {
        let r0 = if round == Round::Zero { "r0-" } else { "" };
        format!("p{}-{}-{}{}", phase.number(), kind.name(), r0, opponent.name())
      }
    }
  }
}

/// Record `take` into `out`, fitting any readout from the training records
/// under `root`. Returns the number of steps (frames, for the boot).
pub fn record(take: &FilmTake, rom: &[u8], tetris: &[u8], root: &Path, out: &Path) -> Result<usize, String> {
  match *take {
    Boot => record_boot_take(rom, BOOT_FRAMES, out),
    Idle { phase, opponent } => record_idle_take(rom, phase, opponent, out),
    Teacher { phase, opponent } => record_teacher_take(rom, phase, opponent, out),
    Brain {
      phase,
      kind,
      opponent,
      round,
    } => {
      let store = Store::new(root, phase);
      let readout = match round {
        Round::Zero => round0_readout(kind, phase, &store, tetris)?,
        Round::Final => final_readout(kind, phase, &store, tetris)?,
      };
      record_take(rom, tetris, phase, kind, opponent, &readout, MAX_STEPS, out)
    }
  }
}

fn read_json(path: &Path) -> Option<Value> {
  serde_json::from_str(&std::fs::read_to_string(path).ok()?).ok()
}

/// The score `take` should reproduce: its entry in the phase's results (or
/// phase 1's sweep), or the teacher's index-0 training fight. None for the
/// boot and always-Idle, and when the records are missing.
pub fn expected_score(take: &FilmTake, root: &Path) -> Option<Value> {
  match *take {
    Boot | Idle { .. } => None,
    Teacher { phase, opponent } => Store::new(root, phase)
      .load("teacher.jsonl")
      .into_iter()
      .find(|r| r.index == 0 && r.opponent == opponent)
      .map(|r| json!(r.score)),
    Brain {
      phase,
      kind,
      opponent,
      round,
    } => {
      let dir = root.join(format!("phase{}", phase.number()));
      ["results.json", "sweep.json"].iter().find_map(|file| {
        let results = read_json(&dir.join(file))?;
        let rounds = results["kinds"][kind.name()].as_object()?;
        let key = match round {
          Round::Zero => "round 0".to_string(),
          // A kind that failed has an "error" key beside its rounds.
          Round::Final => rounds
            .keys()
            .filter_map(|k| Some((k.strip_prefix("round ")?.parse::<usize>().ok()?, k)))
            .max()?
            .1
            .clone(),
        };
        let entry = rounds.get(&key)?.get(opponent.name())?;
        entry.get("hits").map(|_| entry.clone())
      })
    }
  }
}

/// The fields of a fight's score a take must reproduce.
const CHECKED: [&str; 4] = ["hits", "damage_dealt", "knockdowns_dealt", "won"];

/// None when `got` matches `expected` on every checked field, else what
/// differs.
pub fn score_mismatch(expected: &Value, got: &Value) -> Option<String> {
  let diffs: Vec<String> = CHECKED
    .iter()
    .filter(|k| expected[**k] != got[**k])
    .map(|k| format!("{} {} != {}", k, got[*k], expected[*k]))
    .collect();
  (!diffs.is_empty()).then(|| diffs.join(", "))
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::nes::gym::punchout::PUNCH_OUT;
  use crate::nes::gym::reservoir::rom_or_skip;

  const ROOT: &str = "target/punchout";

  #[test]
  fn expected_score_skips_a_kind_that_errored() {
    let root = std::env::temp_dir().join(format!("punchout-film-errored-{}", std::process::id()));
    let dir = root.join("phase1");
    std::fs::create_dir_all(&dir).unwrap();
    let results = json!({"kinds": {"stack4": {"error": "fit failed", "round 4": {"glass-joe": {"hits": 3}}}}});
    std::fs::write(dir.join("results.json"), results.to_string()).unwrap();
    let take = final_take(Defence, BrainKind::Stack(4), GlassJoe);
    let got = expected_score(&take, &root);
    std::fs::remove_dir_all(&root).unwrap();
    assert_eq!(got, Some(json!({"hits": 3})));
  }

  #[test]
  fn film_take_names_are_unique_and_match_the_plan() {
    let names: Vec<String> = FILM_TAKES.iter().map(FilmTake::name).collect();
    assert_eq!(names.len(), 19);
    let mut sorted = names.clone();
    sorted.sort();
    sorted.dedup();
    assert_eq!(sorted.len(), names.len(), "{:?}", names);
    for name in [
      "boot",
      "p1-idle-glass-joe",
      "p1-teacher-glass-joe",
      "p3-teacher-von-kaiser",
      "p1-bank-glass-joe",
      "p1-echo0.5-glass-joe",
      "p1-stack8-von-kaiser",
      "p1-echo0.2-von-kaiser",
      "p2-stack4-von-kaiser",
      "p3-stack4-glass-joe",
      "p2-stack4-bald-bull",
      "p2-stack4-r0-bald-bull",
      "p3-cheat-von-kaiser",
      "p3-stack4-von-kaiser",
    ] {
      assert!(names.iter().any(|n| n == name), "missing {}", name);
    }
  }

  #[test]
  fn round0_differs_from_final() {
    let root = Path::new(ROOT);
    if !root.join("phase2/teacher.jsonl").exists() {
      eprintln!("skipping: no phase 2 records");
      return;
    }
    let store = Store::new(root, Phase::Counter);
    let r0 = round0_readout(BrainKind::Stack(4), Phase::Counter, &store, &[]).unwrap();
    let r4 = final_readout(BrainKind::Stack(4), Phase::Counter, &store, &[]).unwrap();
    assert_ne!(r0.weights, r4.weights);
  }

  #[test]
  #[ignore]
  fn film_take_scores_match_results() {
    let Some(rom) = rom_or_skip(PUNCH_OUT) else { return };
    let root = Path::new(ROOT);
    if !root.join("phase2/results.json").exists() {
      eprintln!("skipping: no phase 2 results");
      return;
    }
    let take = FilmTake::Brain {
      phase: Phase::Counter,
      kind: BrainKind::Stack(4),
      opponent: Opponent::BaldBull,
      round: Round::Zero,
    };
    let dir = std::env::temp_dir().join(format!("punchout-film-take-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    record(&take, &rom, &[], root, &dir).unwrap();
    let meta: Value = serde_json::from_str(&std::fs::read_to_string(dir.join("take.json")).unwrap()).unwrap();
    let expected = expected_score(&take, root).expect("an expected score");
    assert_eq!(score_mismatch(&expected, &meta["score"]), None);
    let _ = std::fs::remove_dir_all(&dir);
  }
}
