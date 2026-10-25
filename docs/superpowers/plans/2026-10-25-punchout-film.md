# Punch-Out!! Reservoir Film Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build an eleven-chapter film of the Reservoir Punch-Out!! experiment that explains the approach against mainstream deep RL with a diagram that builds up over the film.

**Architecture:** Rust records the film's takes: fights with the brain's insides per step, the teacher, round-0 readouts, the sweep and the boot. They go to `target/punchout-film/takes/` through a new `film-takes` command. The Tetris film's Python compositor is parameterised by a `Film` descriptor. A new package, `tools/punchout_film/`, supplies the chapters, the build-up diagram, the numbers and the cue map, and builds `target/punchout-film/punchout-reservoir.mp4`.

**Tech Stack:** Rust (Greenstone, serde_json), Python 3 with Pillow and numpy, ffmpeg, Greenstone's APU for music.

**Spec:** `docs/superpowers/specs/2026-10-24-punchout-film-design.md`

## Global Constraints

- Never stage `src/main.rs` or `src/nes/bus/mod.rs`. Leave `.claude/`, `AGENTS.md` and `.superpowers/` alone.
- Format Rust with `rustfmt --edition 2021 <files>`, never `cargo fmt`; run `git status` afterwards.
- Rust tests: `RUSTFLAGS=-Awarnings cargo test --release --lib <filter>`.
- Python tests: `cd tools && python3 -m unittest discover -s <package>/tests -t .`
- Commit on `main` with `git commit -F <file>`. Each message ends with a blank line, then `Claude-Session: https://claude.ai/code/session_01GxcjqSAGjzrqp9ufw5r46e`. Don't push.
- The Tetris film still builds unchanged: it builds, its tests pass and it looks the same by eye. Byte-identical output is not required.
- Every number on screen comes from the ideas doc section 1a, `target/punchout/phase*/results.json`, `sweep.json` or the teacher records, and never contradicts the doc. "The teacher" on screen means the pure teacher (fight `index` 0).
- Project terms are explained on screen before use: reservoir, echo state network, readout, ridge, DAgger, held out, the bank. Standard ML terms are not.
- Echo leak: `h = (1-a)h + a*new`, so a higher leak forgets faster. stack4 spans 12 frames. A brain sees one frame in four.
- Captions: at least 4 s per 15 words, never under 3 s (`check_captions` enforces this).
- No narration, no game audio; scored on the APU.
- Record only the takes the film uses. Raw takes are about 1.8 GB per 2,400-step fight.

## Review Focus

1. A film take's score differs from the results file it should match (a refit that isn't the trained one). The captions would then contradict the doc. Pinned by `film_take_scores_match_results` (Task 2) and `test_numbers_match_the_ideas_doc` (Task 4).
2. A missing take. The build should fail naming the `punchout` command that records it, not the Tetris one. Pinned by `test_missing_take_hint_names_the_films_command` (Task 3).
3. The Tetris film is broken by the refactor. Pinned by `test_tetris_film_is_the_default` and the existing 55 tests (Task 3).
4. A frame index past a take's last step (a teacher KO take is shorter than 2,400 steps). The step lookup clamps instead of raising. Pinned by `test_step_at_clamps_past_the_end` (Task 4).
5. Stale cached segments after editing the diagram or a shared helper. Pinned by `test_sources_include_the_diagram_and_shared_modules` (Task 3).

---

### Task 1: Takes record the brain's insides

**Files:**
- Modify: `src/nes/gym/punchout/brains.rs`: `Brain::boards`
- Modify: `src/nes/gym/punchout/takes.rs`: step insides, `moves` in take.json

**Interfaces:**
- Produces:
  - `Brain::boards(&mut self) -> Option<Vec<Vec<u8>>>`: the bank's four 200-byte playfields, else `None`.
  - Each take.json step gains `"scores"`: one f64 per move of the phase, in `moves(phase)` order.
  - Each step gains `"proj"`: the 256 projection values, rounded to 3 decimals.
  - Bank takes add `"boards"` to each step: 4 arrays of 200 bytes.
  - take.json top level gains `"moves"`: the phase's move names, in order.
  - Idle takes write none of these.

- [ ] **Step 1: Write the failing tests** in `takes.rs` tests.

```rust
#[test]
fn a_take_records_scores_and_projection_per_step() {
  // as a_take_has_one_metadata_step_per_screen_frame_group, Stack(1), Defence
  let step = &meta["steps"][0];
  assert_eq!(step["scores"].as_array().unwrap().len(), moves(Phase::Defence).len());
  assert_eq!(step["proj"].as_array().unwrap().len(), PROJ);
  assert!(step.get("boards").is_none());
  assert_eq!(meta["moves"][0], "Idle");
}

#[test]
fn a_bank_take_records_four_boards_per_step() {
  // Bank, Defence, tetris ROM via rom_or_skip("roms2/tetris.nes"), zero readout of
  // the bank's row width, budget: record only the first 3 steps (see Step 3)
  let boards = meta["steps"][0]["boards"].as_array().unwrap();
  assert_eq!(boards.len(), 4);
  assert!(boards.iter().all(|b| b.as_array().unwrap().len() == 200));
}
```

- [ ] **Step 2: Run them**

Run: `RUSTFLAGS=-Awarnings cargo test --release --lib punchout::takes`
Expected: both new tests fail (no `scores` key); the existing test passes.

- [ ] **Step 3: Implement.**
  - `record_take` gains a `max_steps: usize` parameter (callers pass `MAX_STEPS`) so the bank test stays short.
  - The policy closure computes the row, `readout.predict(&row)`, the projection and (for the bank) `brain.boards()`. It keeps them in a `RefCell<Vec<Value>>` indexed by step.
  - The `Watched::Step` handler merges entry `index` into the step's JSON.
  - Check that the policy's step index and `Watched::Step { index }` agree. The test asserts that every step has `scores`.

- [ ] **Step 4: Run the tests again.** Same command. Expected: all `punchout::takes` tests pass.

- [ ] **Step 5: Commit** `brains.rs` and `takes.rs`: "Record a Punch-Out!! brain's insides in its takes."

---

### Task 2: The film's takes

**Files:**
- Create: `src/nes/gym/punchout/film.rs` (and `pub mod film;` in `src/nes/gym/punchout/mod.rs`)
- Modify: `src/nes/gym/punchout/dagger.rs`: `round0_readout`
- Modify: `src/nes/gym/punchout/takes.rs`: `record_teacher_take`, `record_boot_take`
- Modify: `examples/punchout.rs`: the `film-takes` command and its usage doc

**Interfaces:**
- Consumes: Task 1's `record_take(..., max_steps)`.
- Produces:
  - `dagger::round0_readout(kind: BrainKind, phase: Phase, store: &Store, tetris: &[u8]) -> Result<Readout, String>`: fitted on `teacher.jsonl` alone, the same fit `test_with` scores as `"round 0"`.
  - `takes::record_teacher_take(rom: &[u8], phase: Phase, opponent: Opponent, out: &Path) -> Result<usize, String>`. The policy is `teacher::label(s.emu, phase)` when `can_act`, else `Idle`. Kind is `"teacher"`. Each step adds `"label": true`.
  - `takes::record_boot_take(rom: &[u8], frames: usize, out: &Path) -> Result<usize, String>`. Power-on frames 0..frames, pressing Start the way `boot_strip` does (every 240 frames), one stream `"punchout"`, `"steps": []`.
  - `film::Round { Zero, Final }`.
  - `film::FilmTake { Boot, Idle { phase, opponent }, Teacher { phase, opponent }, Brain { phase, kind, opponent, round } }`, with `fn name(&self) -> String`. The names are:
    - `boot`
    - `p1-idle-glass-joe`
    - `p3-teacher-von-kaiser`
    - `p2-stack4-r0-bald-bull` for `Round::Zero`, and `p2-stack4-bald-bull` for `Final`
  - `film::FILM_TAKES: &[FilmTake]`, in this order:
    1. `Boot` (1,800 frames)
    2. Idle: p1 Glass Joe
    3. Teacher: p1 Glass Joe, p3 Von Kaiser
    4. Brain, Final:
       - p1 bank Glass Joe
       - p1 echo0.5 Glass Joe
       - p1 Von Kaiser for each of the seven `SWEEP` brains
       - p2 stack4 Von Kaiser
       - p3 stack4 Glass Joe
       - p2 stack4 Bald Bull
       - p3 cheat Von Kaiser
       - p3 stack4 Von Kaiser
    5. Brain, Zero: p2 stack4 Bald Bull
  - `film::expected_score(take: &FilmTake, root: &Path) -> Option<Value>`. This is the matching entry:
    - sweep brains: `phase1/sweep.json`
    - other brains: `phaseN/results.json` under `kinds.<name>."round 0"` or the highest round
    - teacher: the index-0 record of `teacher.jsonl`
    - boot and idle: `None`
  - CLI: `punchout film-takes [--only NAME,...]` records each missing take into `target/punchout-film/takes/<name>/` (skipped if `take.json` exists), in parallel threads like `takes`. Readouts come from `root()` (target/punchout). After each recording it compares `hits`, `damage_dealt`, `knockdowns_dealt` and `won` with `expected_score`, and prints `MISMATCH <name>: ...` to stderr on any difference. An unknown `--only` name fails with the valid list.

- [ ] **Step 1: Write the failing tests** in `film.rs`:
  - `film_take_names_are_unique_and_match_the_plan`: assert the names above, the count (19 takes), and that there are no duplicates.
  - `round0_differs_from_final`: skipped without `target/punchout/phase2/teacher.jsonl`. stack4's round-0 and final readouts differ in weights.
  - `film_take_scores_match_results`: skipped without the ROM and `target/punchout/phase2`. Record `p2-stack4-r0-bald-bull` to a temp dir and assert that `hits` and `damage_dealt` equal `expected_score`. This takes about a minute and a half in release mode; mark it `#[ignore]` and run it explicitly in Step 4.

- [ ] **Step 2: Run them**

Run: `RUSTFLAGS=-Awarnings cargo test --release --lib punchout::film -- --include-ignored`
Expected: compile failure (no `film` module).

- [ ] **Step 3: Implement** the interfaces above. `round0_readout` is `fit_cached(rows_of(kind, &store.load(TEACHER_FILE), tetris)?.iter(), phase)`.

- [ ] **Step 4: Run the tests again.** Same command. Expected: all three pass.

- [ ] **Step 5: Record the takes**

Run: `cargo run --release --example punchout -- film-takes`
Expected:
- 19 directories under `target/punchout-film/takes/`, and no `MISMATCH` lines.
- `p3-teacher-von-kaiser/take.json` has `score.won == true`, with fewer than 2,400 steps.
- `p1-bank-glass-joe` has `boards` on each step.
- If a `MISMATCH` appears, stop and debug it (superpowers:systematic-debugging). Don't film a fight whose numbers differ from the doc.

- [ ] **Step 6: Commit** the Rust files: "Record the Punch-Out!! film's takes."

---

### Task 3: A compositor that builds either film

**Files:**
- Modify: `tools/reservoir_film/model.py`: `Film`; `Takes(root, hint)`
- Modify: `tools/reservoir_film/build.py`: `run(film, argv)`; module paths come from the film
- Create: `tools/reservoir_film/film.py`: `TETRIS = Film(...)`
- Test: `tools/reservoir_film/tests/test_film.py`

**Interfaces:**
- Produces:
  - `model.Film` (a dataclass) with these fields:
    - `name: str`
    - `title: str` (the ffmetadata title)
    - `takes_dir: Path`
    - `out: Path`, the final mp4
    - `record_hint: str`, a format string with `{name}`
    - `numbers: Path`
    - `chapters: Callable[[], list]`
    - `cues: Callable[[], tuple]`, returning `(CUES, CHAPTER_CUES, STINGS)`
    - `sources: list[Path]`, the directories and files whose text keys the cache
    - `segments: Path`, the cached segments: `TAKES_DIR / "segments"` for Tetris, `REPO/target/punchout-film/segments` for Punch-Out!!
  - `Takes(root, hint="...")`: a missing take raises `FileNotFoundError` containing `hint.format(name=name)`.
  - `build.run(film: Film, argv: list[str])`: today's `main`, with every hard-coded path taken from `film`. Segments go to `film.takes_dir.parent / "segments"` for the Punch-Out!! film. For the Tetris film, keep `TAKES_DIR / "segments"`; put a `segments` field on `Film`. `chapters.txt` goes next to `film.out`. `ffmetadata(chapters, ctx, title)` gains `title`.
  - `build._sources(film)`: the concatenated text of every `.py` under each path in `film.sources`, sorted.
  - `python3 -m reservoir_film.build` runs `run(TETRIS, sys.argv[1:])`, with the same flags as today.

- [ ] **Step 1: Write the failing tests** in `test_film.py`:
  - `test_tetris_film_is_the_default`: `TETRIS.out == TAKES_DIR / "tetris-reservoir.mp4"`, and `TETRIS.title == "Can Tetris play Mario?"`.
  - `test_missing_take_hint_names_the_films_command`: `Takes(tmpdir, hint="cargo run --release --example punchout -- film-takes --only {name}")["nope"]` raises with `--only nope` in the message.
  - `test_ffmetadata_uses_the_films_title`: `build.ffmetadata([], CTX, "X")` contains `title=X`.
  - `test_sources_include_the_diagram_and_shared_modules`: for a `Film` whose `sources` are `[HERE, tmpdir]` and with `tmpdir/diagram.py` written, `_sources` contains the text of `draw.py` and of `diagram.py`.
  - Update `test_build.py` callers of `ffmetadata` and `_sources` to the new signatures.

- [ ] **Step 2: Run them**

Run: `cd tools && python3 -m unittest discover -s reservoir_film/tests -t .`
Expected: the new tests fail with `ImportError` or `TypeError`.

- [ ] **Step 3: Implement** the interfaces above.

- [ ] **Step 4: Run the tests again.** Same command. Expected: all pass (55 old + 4 new).

- [ ] **Step 5: Check the Tetris film by eye**

Run: `cd tools && python3 -m reservoir_film.build --contact --chapter 4`
Expected: `target/reservoir-film/contact/c04.png` looks as it did before the change.

- [ ] **Step 6: Commit** the compositor: "Let the reservoir compositor build more than one film."

---

### Task 4: The Punch-Out!! film package, its numbers and the teacher in the ideas doc

**Files:**
- Create:
  - `tools/punchout_film/__init__.py`
  - `film.py`: `PUNCHOUT = Film(...)`
  - `build.py`: `run(PUNCHOUT, sys.argv[1:])`
  - `numbers.py`: writes `numbers.json`
  - `numbers.json` (checked in)
  - `insides.py`
  - `chapters/__init__.py`: `CHAPTERS = []` for now
  - `music.py`
  - `tests/__init__.py`, `tests/test_numbers.py`, `tests/test_insides.py`
- Modify: `docs/ideas/brain-cartridge.md`, section 1a: the teacher's own fights.

**Interfaces:**
- Consumes: `Film`, `Takes`, `build.run` (Task 3); take.json fields (Tasks 1 and 2).
- Produces:
  - `PUNCHOUT` has these fields:
    - `takes_dir = REPO/target/punchout-film/takes`
    - `out = REPO/target/punchout-film/punchout-reservoir.mp4`
    - `title = "Reservoir Punch-Out!!"`
    - `record_hint = "cargo run --release --example punchout -- film-takes --only {name}"`
    - `sources = [reservoir_film dir, punchout_film dir]`
  - `insides.step_at(take, i: int) -> dict`: the step for output frame `i` (`steps[min(i // 4, len - 1)]`).
  - `insides.moves(take) -> list[str]`.
  - `numbers.json`, written by `python3 -m punchout_film.numbers` from `target/punchout/`, with these keys:
    - `idle.<opponent>.hpm`
    - `phase<N>.<kind>.r0|r4.<opponent>`: `{hpm, dodge, dealt, knockdowns, won}`
    - `sweep.<kind>.<opponent>`: `{hpm, dodge}`
    - `teacher.phase<N>.<opponent>`: `{hits, won, knockdowns, steps, dealt}`, from the index-0 record
    - `teacher_idle_share.phase<N>`: the share of Idle labels over every fight in `teacher.jsonl`, as the doc computes it
  - Opponent keys use the take names (`glass-joe`).
  - `music.py`: `CUES`, `CHAPTER_CUES` and `STINGS`, reusing the cue objects in `reservoir_film.music.cues`. It also adds a `"bell"` sting: an original three-note ring on the pulse channel, with `PLINK`.
- Doc paragraph: the pure teacher's fights, using the table in the spec's Story section. In Counter and Fight it knocks out every training opponent it beats with no hits taken; Counter against Glass Joe ends at the cap after 4 knockdowns. Defence: 0, 0 and 12 hits. Note that fights 1 and 2 play a tenth of their moves at random.

- [ ] **Step 1: Write the failing tests**
  - `test_numbers_match_the_ideas_doc`: these numbers must equal the doc (rounded to 2 places):
    - `phase2.stack4.r0.bald-bull.hpm == 12.77` and `r4 == 22.76`
    - `phase1.echo0.5.r4.von-kaiser.hpm == 17.39`
    - `sweep.stack8.von-kaiser.hpm == 20.31`
    - `idle.glass-joe.hpm == 20.98`
    - `teacher.phase3.von-kaiser == {won: True, steps: 801, hits: 0, knockdowns: 3, ...}`
    - `teacher_idle_share.phase1 == 0.967` (3 places)
  - `test_step_at_clamps_past_the_end`: given a fake take whose `meta` has 3 steps, `step_at(take, 400)` returns the last step.
  - `test_numbers_cover_every_film_take`: every Brain take named in Task 2's list has a `numbers.json` entry.

- [ ] **Step 2: Run them**

Run: `cd tools && python3 -m unittest discover -s punchout_film/tests -t .`
Expected: ImportError.

- [ ] **Step 3: Implement.** Then run `python3 -m punchout_film.numbers` and check in `numbers.json`. Write the doc paragraph.

- [ ] **Step 4: Run the tests again.** Same command. Expected: all pass.

- [ ] **Step 5: Commit** the package, `numbers.json` and the doc: "Start the Punch-Out!! film: numbers, insides and the teacher's fights."

---

### Task 5: The build-up diagram

**Files:**
- Create: `tools/punchout_film/diagram.py`
- Test: `tools/punchout_film/tests/test_diagram.py`

**Interfaces:**
- Produces:
  - `PIECES = ["see", "remember", "decide", "learn", "yardstick"]`.
  - `MAINSTREAM[piece]` and `OURS[piece]`: `Box(label: str, sub: str, learned: bool)`. Use exactly these labels and subs:
    - `see`
      - mainstream: "conv net" / "learned end to end", learned
      - ours: "random projection" / "5,712 pixels to 256 numbers, rolled once", frozen
    - `remember`
      - mainstream: "recurrent net" / "learns what to keep", learned
      - ours: "a memory I picked" / "stack, echo, or four Tetris games", frozen
    - `decide`
      - mainstream: "policy" / "millions of gradient steps", learned
      - ours: "one linear readout" / "one ridge solve", learned
    - `learn`
      - mainstream: "reward" / "about 10^7 frames", no learned flag
      - ours: "a teacher with save states" / "tries every move, 60 frames ahead", no learned flag; from chapter 7, the sub becomes "... then DAgger"
    - `yardstick`: absent in the mainstream form; ours: "the cheat" / "reads all 2,048 bytes of RAM", frozen
  - `pieces_by_chapter(n: int) -> dict[str, str]`: each piece's state at the start of chapter `n`, one of `"mainstream"`, `"ours"` or `"hidden"`:
    - chapters 1–3: all mainstream; yardstick hidden.
    - chapter 4: learn is ours.
    - chapter 5: see, remember and decide are ours.
    - chapter 8: DAgger sub.
    - chapter 11: yardstick is ours.
    - Swaps happen inside a chapter, so `pieces_by_chapter(n)` is the state when chapter n *begins*.
  - `draw_diagram(img, rect, states: dict[str, str], swap: tuple[str, float] | None = None, focus: str | None = None)`:
    - Learned boxes are drawn in `LEARNED = draw.GOLD`; frozen boxes in `FROZEN = (90, 150, 250)`, dashed.
    - `swap=(piece, p)` cross-fades that piece from mainstream to ours as p goes 0→1, with a moment where both are shown side by side.
    - `focus` dims the other pieces.

- [ ] **Step 1: Write the failing tests**
  - `test_pieces_by_chapter`: assert the states listed above for chapters 1, 4, 5, 8 and 11.
  - `test_learned_boxes_are_gold_and_frozen_boxes_blue`: draw the all-ours state into a blank image and sample a pixel on the `decide` box border (`LEARNED`) and on the `see` border (`FROZEN`). Expose `box_rects(rect, states) -> dict[str, tuple]` for the test.
  - `test_swap_renders_at_every_progress`: for p in 0, 0.5 and 1, `draw_diagram` with `swap=("see", p)` returns without error and changes pixels between p=0 and p=1.

- [ ] **Step 2: Run them**

Run: `cd tools && python3 -m unittest discover -s punchout_film/tests -t .`
Expected: ImportError for `diagram`.

- [ ] **Step 3: Implement** with `draw.panel`, `draw.text`, `draw.arrow` and `ImageDraw`.

- [ ] **Step 4: Run the tests again.** Same command. Expected: all pass.

- [ ] **Step 5: Commit**: "Draw the Punch-Out!! film's build-up diagram."

---

### Tasks 6–9: The chapters

Each chapter is a module `tools/punchout_film/chapters/cNN_<slug>.py` exporting `CHAPTER = Chapter(n, title, subtitle, shots)`, in the style of `tools/reservoir_film/chapters/c04_teacher.py`. Register each in `chapters/__init__.py` as it lands, and add it to `music.CHAPTER_CUES`. Captions, timings, layouts and stings are the implementer's. The required content below is what each chapter must show and say. Every number is filled from `{numbers.…}` or `{<take>.score.…}`, never typed in.

Each task follows the same steps:

- [ ] **Step 1: Write the failing test.** In `tools/punchout_film/tests/test_chapters.py`, add `test_chapter_N` for each chapter in the task. Use the `check` helper copied from `reservoir_film/tests/test_chapters.py`, skipped unless `target/punchout-film/takes/boot/take.json` exists.
- [ ] **Step 2: Run it.** `cd tools && python3 -m unittest discover -s punchout_film/tests -t .` Expected: fails with "chapter N is not in CHAPTERS".
- [ ] **Step 3: Write the chapter modules.**
- [ ] **Step 4: Run it again.** Same command. Expected: pass.
- [ ] **Step 5: Look at it.** Run `cd tools && python3 -m punchout_film.build --contact --chapter N` and read `target/punchout-film/contact/cNN.png` (Read the PNG). Fix anything unreadable, overlapping or off-frame before committing.
- [ ] **Step 6: Commit**: "Add the Punch-Out!! film's chapters N–M."

**Task 6: chapters 1–3**
1. **"The cartridge boots"** (take `boot`, then the mainstream diagram, `pieces_by_chapter(1)`).
   - Punch-Out!! boots on Greenstone.
   - One shot on MMC2 (mapper 9): it swaps the picture tiles mid-frame when the PPU reads certain tiles, and it was built for this experiment.
   - The usual pipeline as boxes: screen → conv net → policy ← reward.
   - The premise: we'll swap each box for something stranger and see what happens.
2. **"Always-Idle"** (`p1-idle-glass-joe`): standing still takes `{numbers.idle.glass-joe.hpm}` hits a minute from Glass Joe. That's the floor.
3. **"The teacher"** (`p1-teacher-glass-joe`, `p3-teacher-von-kaiser`).
   - Swap `learn` (`swap=("learn", p)`).
   - It's a save-state lookahead that tries each move for 60 frames.
   - In Defence it takes no hits from Glass Joe.
   - In Fight it knocks out Von Kaiser in `{numbers.teacher.phase3.von-kaiser.steps}` steps with no hits taken. Show the KO, with the `bell` sting.
   - Its labels are mostly Idle: `{numbers.teacher_idle_share.phase1:.1%}` in Defence.

**Task 7: chapters 4–5**

4. **"Building a brain"** (`p1-bank-glass-joe`, `p1-echo0.5-glass-joe`).
   - Swap `see`, `remember` and `decide` in turn, each with an inset clip.
   - **See:** the screen beside its 256 `proj` values from the take.
   - **Remember:** three cards.
     - stack: last N glimpses; stack4 spans 12 frames.
     - echo: 2,048 random fixed units with a leak; a higher leak forgets faster.
     - the bank.
   - **The bank close-up:** four boards drawn from `boards` with `draw.board`, next to the fight.
   - **The honest beat:** each byte fed to Tetris averages 16 projection numbers, so pieces land only in columns 3 to 6. The boards change every step but look alike whatever Glass Joe does. It's a plumbing choice, not a test of the idea.
   - **Decide:** the per-move `scores` as moving bars. One ridge solve; the highest score is pressed.
   - **Define:** readout, ridge, reservoir.
5. **"Defence"** (`p1-echo0.5-glass-joe`, `p1-bank-glass-joe`).
   - echo0.5 takes `{numbers.phase1.echo0.5.r4.glass-joe.hpm}`.
   - The bank is identical to standing still, as chapter 4 predicted.
   - These are the trained brains; rounds wait for chapter 7.

**Task 8: chapters 6–8**

6. **"How much memory?"** (the seven `p1-*-von-kaiser` sweep takes).
   - Side by side, with `bar_chart` of `sweep.*.von-kaiser.hpm`.
   - Crisp history helps on trained opponents (stack8 is best of the stacks).
   - echo0.2, which remembers longest, is the worst echo.
   - A brain sees one frame in four, so a tell shorter than 4 frames can fall between samples.
7. **"Counter"** (`p2-stack4-von-kaiser`).
   - Swap the `learn` sub to DAgger. Define DAgger: the student fights, the teacher labels the student's own situations, refit, four rounds.
   - stack4 takes `{numbers.phase2.stack4.r4.von-kaiser.hpm}` where idle takes `{numbers.idle.von-kaiser.hpm}`.
8. **"Knockdowns"** (`p3-stack4-glass-joe`).
   - Two knockdowns, `{p3-stack4-glass-joe.score.damage_dealt}` damage, no hits.
   - No student wins in 2,400 steps. The teacher won every Fight in 801 to 1,018 steps (from `numbers.teacher.phase3`).

**Task 9: chapters 9–11, the cue map, and the full build**

9. **"Held out"** (`p2-stack4-r0-bald-bull`, `p2-stack4-bald-bull` side by side).
   - Define held out.
   - Round 0: `{numbers.phase2.stack4.r0.bald-bull.hpm}`; round 4: `…r4…`. More teaching made stack4 worse.
   - echo0.5 got better against him (`numbers.phase2.echo0.5.r0/r4.bald-bull`).
10. **"The cheat"** (`p3-cheat-von-kaiser`, `p3-stack4-von-kaiser`).
    - Reveal the `yardstick` piece. RAM under a linear readout takes `{numbers.phase3.cheat.r4.von-kaiser.hpm}` against stack4's `…`.
    - The cheat isn't an upper bound: it takes more hits than stack4 and echo0.5 from every trained opponent in every phase.
11. **"Verdict"**.
    - The finished diagram, one line per piece on what it did.
    - The teacher wins; no student does.
    - Open questions: widen the bank's window, feed all four frames of a step.
    - Credits.

Extra steps for Task 9, after the shared steps:
- [ ] **Cue map test.** Add `test_every_chapter_has_a_cue_and_every_sting_exists` to `tests/test_music.py`:
  - every chapter number has a cue in `CHAPTER_CUES`;
  - every name in `CHAPTER_CUES` is in `CUES`;
  - every sting name in any shot's `stings` is in `STINGS`.
  - Run it, watch it fail on missing entries, fill the map, and run it again.
- [ ] **Full build.** Run `cd tools && python3 -m punchout_film.build`.
  - Expected: `target/punchout-film/punchout-reservoir.mp4` and `chapters.txt`, with 11 chapters, about 12 to 15 minutes long.
  - Check `ffprobe -show_chapters` lists eleven chapters.
- [ ] **Tetris film still builds.** Run `cd tools && python3 -m reservoir_film.build --quick`. Expected: completes.
- [ ] **Both suites.** Run both Python suites and `RUSTFLAGS=-Awarnings cargo test --release --lib punchout`. Expected: all pass.
- [ ] **Commit**: "Finish the Punch-Out!! film."
