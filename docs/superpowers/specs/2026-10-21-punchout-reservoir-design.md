# Reservoir Punch-Out!!: can a brain with memory learn to dodge?

Status: approved design, 2026-10-21. Follows the Tetris-reservoir film
(`docs/superpowers/specs/2026-10-13-tetris-reservoir-design.md`), whose
verdict card asked for "a task that needs memory".

## What it is

On Mario, a weighted sum of the current RAM did as well as any reservoir,
because RAM is the whole game state: nothing needed remembering. Mike Tyson's
Punch-Out!! is built on memory. Every opponent telegraphs a punch (a rear-back,
a flash, an eyebrow), and the punch lands a fixed number of frames later. A
brain that reads the *screen*, not RAM, must see the tell and act after it has
gone: the classic delayed-response task reservoirs exist for.

We teach four brains that differ only in their memory to fight Little Mac's
fights, in three phases: defence, defence plus counter, whole fight. A
lookahead teacher built on save states labels any state a brain reaches
(DAgger), so brains are corrected where they actually go, not only where the
teacher went. Then we make a film.

## Deliverable

A narrated film in the style of the Tetris-reservoir film, its captions
written after the results, plus the results behind it.

- `target/punchout/gate/`: mapper gate screenshots.
- `target/punchout/phase<N>/`: per-brain label files, per-round results,
  `results.json`.
- `target/punchout/sweep.json`: the memory sweep.
- `target/punchout/takes/`: film takes (the reservoir film's take format).
- `target/punchout/film.mp4`: the film.

## Decisions

- **Screen in, RAM only for scoring.** No brain ever reads RAM, except the
  RAM-cheat reference, which is not a contestant.
- **Three phases**, each from fresh brains with its own teacher and moves:
  1. Defence: Mac never punches. Isolates the memory question.
  2. Defence plus counter: adds both jabs.
  3. Whole fight: adds the star uppercut.
  All three run whatever the earlier ones show (except the stop gate below).
- **Four brains, one readout.** All share the same input and a ridge readout
  (one score per move, highest wins); only memory differs.
- **Training is DAgger** against a lookahead teacher. Round 0 (pure
  imitation) is reported as the imitation-only baseline.
- **Opponents.** Train: Glass Joe, Von Kaiser, Piston Honda. Held out: Don
  Flamenco, Bald Bull.

## Step zero: mapper 9 (MMC2)

Greenstone supports mappers 0, 1, 2, 3, 4, 7; Punch-Out!! is mapper 9, which
panics today. `src/nes/cartridge/mapper/mapper9.rs`, registered in
`cartridge/mod.rs`:

- PRG: `$A000-$AFFF` write selects the 8 KB bank at `$8000`; `$A000-$FFFF`
  are fixed to the last three 8 KB banks.
- CHR: two 4 KB halves, each with two bank registers chosen by a latch:
  `$B000` (half 0, latch FD), `$C000` (half 0, FE), `$D000` (half 1, FD),
  `$E000` (half 1, FE). `$F000` bit 0: mirroring (0 vertical, 1 horizontal).
- Latches: a CHR read at `$0FD8` sets latch 0 to FD, `$0FE8` to FE; reads in
  `$1FD8-$1FDF` set latch 1 to FD, `$1FE8-$1FEF` to FE. The latch changes
  after the read; the triggering read uses the old bank.
- Latches live in `Cell`s, so `Mappable::read_chr_u8(&self)` is unchanged.
- Power-on: both latches FE, all registers 0.

**Gate.** Boot the ROM; save the title screen and Glass Joe mid-fight as
PNGs in `target/punchout/gate/`; show them to Nate. Nothing below starts
until he has seen them. The risk is sprite pattern fetches: if Greenstone
fetches sprite patterns at a different time than hardware (dots 257-320),
latch flips land in the wrong place and the opponent draws with wrong tiles.
If so, correcting the PPU's sprite fetch timing becomes part of this step,
with its own test.

## The harness: `src/nes/gym/punchout/`

- `ram.rs`: Mac's health and hearts, opponent's health, knockdowns, round,
  clock, opponent id, Mac's got-hit state, fight over. Found by RAM diffs
  around known events, cross-checked against community memory maps. Every
  address is pinned by a test on a recorded state.
- `start.rs`: `fight_start(rom, opponent) -> Result<(state, frame), String>`:
  boot, select the opponent (poke its id into RAM; fallback: the circuit
  passwords), step to the first frame Mac can act, save there.
- `moves.rs`: a move is held for `MOVE_FRAMES` = 4 frames.
  - Phase 1: idle, dodge left, dodge right, duck/block.
  - Phase 2: adds left jab, right jab.
  - Phase 3: adds star uppercut.
  - `hit_taken`: Mac's health fell or he entered the got-hit state during a
    move or its trailing frames.

## Brains: `brains.rs`

Input: the gym's 84x84 grey frame (`preprocess.rs`) with the HUD rows cropped,
then a fixed random projection (fixed seed) to `PROJ` = 256 numbers. All
brains get the same 256 numbers each step.

1. **No memory**: the current 256.
2. **Frame stack**: the last `depth` steps' 256 each; default depth 4.
3. **Echo state network**: `soft.rs`'s 2,048-unit `Expansion`, recurrent,
   generalised to take 256 inputs; default leak 0.5.
4. **Tetris bank**: four Tetris games, each fed one quarter (64) of the
   projection through the bank's existing encoder.
5. **RAM cheat** (reference): the RAM map's fields, including the opponent's
   attack state and timer.

Readout: `ridge::fit` on one-hot move targets, argmax at play time.

## Teacher: `teacher.rs`

`label(state, phase) -> Move`: from the saved state, for each move, play it
for one move then each of a few fixed continuations, `HORIZON` = 60 frames in
all; score each move by its best continuation.

- Phase 1 score: hits taken (fewer is better); ties go to idle, so labels ask
  for a move only when one is needed. Continuation: idle.
- Phases 2-3 score: damage dealt - 2 x damage taken. Continuations: idle;
  jab-jab-jab; in phase 3 also the uppercut when Mac has a star.

At most about 1,000 frames per label.

## Training: `dagger.rs`

Per phase, per brain:

- Round 0: the teacher fights 3 fights per training opponent, labelling its
  own states; the brain fits on those (shared by all brains, since the
  states are the teacher's).
- Rounds 1-4: the brain fights 1 fight per training opponent; the teacher
  labels every 4th step it reached; the brain refits on all labels so far.
- Labels append to `target/punchout/phase<N>/<brain>.jsonl`; a stopped run
  resumes at the first unfinished round.

Test: after round 4, 3 fights per opponent, training and held out, no
labelling.

## Memory sweep (phase 1 only)

Frame stack at depths 1, 2, 4, 8; echo state network at three leak rates
(0.2, 0.5, 0.9). Chart: dodge rate against memory reach. If dodging improves
past what depth 4 sees, the game really needs memory.

## Scores

- Phase 1: hits taken per minute, share of punches dodged, survival time.
- Phase 2: adds damage dealt, knockdowns.
- Phase 3: wins, time to knockout.

Per brain, per opponent, per round, in `results.json`.

## Budget

About 6 h for phase 1 including the sweep, 3-4 h each for phases 2 and 3:
about 13 h. Everything resumable.

## Failure handling

- Each emulator step runs under the Prospector's `guarded`; a panic costs a
  fight, not the run.
- Fixed seeds everywhere.
- **Stop gate**: if after phase 1 round 0 no brain takes fewer hits than
  always-idle, stop and show Nate: the teacher or the input is broken.

## Command line: `examples/punchout.rs`

`gate`, `train --phase N`, `test --phase N`, `sweep`, `takes`. Output under
`target/punchout/`.

## Film: `tools/punchout_film/`

Reuses the reservoir film's `draw`, `model` and music approach. Chapters:

1. Cold open: a random-weight brain against Glass Joe.
2. The emulator couldn't run it: the mapper and its tile-triggered banks.
3. Tells: each opponent's cue, then the punch.
4. Four brains: same 256 numbers, four kinds of memory.
5. Defence: results, the memory chart, the teacher correcting round by round.
6. Counter.
7. Whole fight.
8. Held-out opponents.
9. Verdict and credits.

Captions that state results are chosen from the results, never written ahead.

## Testing

- Mapper 9 unit tests, no ROM needed.
- RAM map and fight-start tests on the real ROM, skipped when it is missing
  (`rom_or_skip("roms2/mike_tysons_punch_out.nes")`).
- Teacher: prefers a dodge when a punch is coming, idle otherwise.
- Brains: frame-stack window size, projection determinism, ESN input width.
- DAgger: the label set grows each round; resume skips finished rounds.
- Film: Python unit tests as for the reservoir film.

## Out of scope

- Mac's own punches in phase 1; passwords beyond selecting opponents.
- King Hippo, Mike Tyson, and anything past Bald Bull.
- Training on RAM (beyond the cheat reference) or on pixels at full size.
