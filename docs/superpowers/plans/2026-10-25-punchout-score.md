# Punch-Out!! Score Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the Punch-Out!! film's borrowed Tetris score with energetic, funny public-domain arrangements, cued per shot, with crowd roars at knockdowns and honest credits.

**Architecture:** The scorer learns cue *spans*: a chapter's entry in the film's cue map is either a cue name (as now, so Tetris is unchanged) or a list of `(shot, offset, cue)` spans. New cues are written in the existing tracker in `tools/punchout_film/music.py`. Knockdown times come from the takes and feed both the stings and the funeral march's start. The credits are generated from the cues' `credit` fields.

**Tech Stack:** Python 3.13, numpy, Pillow, ffmpeg; the tracker in `tools/reservoir_film/music/core.py`; `examples/apu_render.rs` (Rust) renders.

**Spec:** `docs/superpowers/specs/2026-10-25-punchout-score-design.md`

## Global Constraints

- No Nintendo music, nothing transcribed or paraphrased from any game score; no modern copyrighted works (no Rocky theme); no arrangement copied from a modern edition. Work from the public-domain melody.
- No mock-ethnic jingles. Humour comes from tempo, timing, instrumentation, a sour note.
- The Tetris film's score must not change.
- Music runs free: no cue is synced to the action except the funeral march, which starts at Mac's last knockdown in chapter 2.
- Each cue must fit each span it's used for within `TEMPO_RANGE` (0.85–1.15) without the trim/pad fallback; for the Punch-Out!! film a span that doesn't fit fails the build naming the span and the cue.
- Python tests: `cd tools && python3 -m unittest discover -s punchout_film/tests -t .` and `... -s reservoir_film/tests -t .`.
- Repo rules: never stage `src/main.rs` or `src/nes/bus/mod.rs`; leave `.claude/`, `AGENTS.md`, `.superpowers/` alone; commit with `git commit -F <file>`, message ending with a blank line then `Claude-Session: https://claude.ai/code/session_01GxcjqSAGjzrqp9ufw5r46e`; don't push.

## Review Focus

1. The Tetris film's score plan changes because of the span refactor. Expected: identical `(cue, seconds)` plan. Pinned by `test_tetris_plan_is_unchanged` (Task 1).
2. A span's offset lands past the end of its shot, or spans overlap or leave a gap, so the plan's lengths don't add up to the film's length. Expected: the plan's seconds sum to the film's seconds exactly. Pinned by `test_plan_covers_the_film_exactly` (Task 1).
3. Mac's *first* knockdown is used for the funeral march instead of his last (the take's `mac_down_at` is the first). Expected: the cut at step 1106, about 18.4 s into the idle shot. Pinned by `test_funeral_march_starts_at_macs_last_knockdown` (Task 2).
4. A cue too short or too long for a span is silently trimmed. Expected: the build fails naming it. Pinned by `test_every_span_fits` (Task 5) and `test_strict_film_rejects_an_unfit_span` (Task 1).
5. The credits omit a composer, or still credit the Minuet / Tetris cues that are no longer used. Expected: every non-original credit of a cue in the map, and nothing else. Pinned by `test_credits_name_exactly_the_cues_used` (Task 5).

---

### Task 1: Cue spans in the scorer

**Files:**
- Modify: `tools/reservoir_film/build.py` (the plan in `run`, stings unchanged)
- Modify: `tools/reservoir_film/model.py` (`Film` gains `strict_score: bool = False` as its last field)
- Test: `tools/reservoir_film/tests/test_build.py` (or the existing build test file)

**Interfaces:**
- Produces: `cue_plan(chapters, chapter_cues, cues, ctx) -> list[tuple[Cue, float, str]]` in `reservoir_film/build.py`: one `(cue, seconds, label)` per span in film order, `label` like `"3/defence"`. A chapter_cues value is either a cue name (one span for the whole chapter, title card included) or a list of `(shot_name, offset, cue_name)` where `offset` is seconds into that shot or a callable `offset(ctx) -> float`. The chapter's title card belongs to its first span; the first span must start at the first shot, offset 0.
- Produces: `unfit_spans(plan) -> list[str]`: labels (with cue names) of spans where `score.fit` fell back.
- Produces: `run()` uses `cue_plan`; when `film.strict_score` and `unfit_spans` is non-empty, `sys.exit` with the list.

- [ ] **Step 1: Write failing tests** in `test_build.py`:
  - `test_tetris_plan_is_unchanged`: for TETRIS, `[(c.name, s) for c, s, _ in cue_plan(...)]` equals `[(CUES[CHAPTER_CUES[ch.number]].name, chapter_seconds(ch, ctx)) for ch in chapters]`.
  - `test_spans_split_a_chapter_at_shots_and_offsets`: a fake chapter (number 1, shots `a` 10 s and `b` 20 s, so 34 s with the title) with spans `[("a", 0, "x"), ("b", 5, "y")]` gives seconds `[4 + 10 + 5, 15]` and labels `["1/a", "1/b"]`. A callable offset `lambda ctx: 5` gives the same.
  - `test_plan_covers_the_film_exactly`: for the fake film above, plus a second chapter with a plain cue name, the sum of the plan's seconds equals the sum of `chapter_seconds`.
  - `test_a_span_must_start_its_chapter`: spans whose first entry is not `(first shot, 0, …)` raise `ValueError`.
  - `test_strict_film_rejects_an_unfit_span`: `unfit_spans` on a plan holding a one-bar cue for 600 s returns that span's label.
- [ ] **Step 2: Run** `cd tools && python3 -m unittest reservoir_film.tests.test_build`. Expected: errors, `cue_plan` not defined.
- [ ] **Step 3: Implement** `cue_plan`, `unfit_spans`, `Film.strict_score`, and switch `run()`'s `plan` to `[(cue, seconds) for cue, seconds, _ in cue_plan(...)]`.
- [ ] **Step 4: Run** both Python suites. Expected: OK.
- [ ] **Step 5: Commit** `tools/reservoir_film`. Message: "Let a film's score change cue partway through a chapter."

### Task 2: Knockdown times, the roar, and the funeral march's cut

**Files:**
- Modify: `tools/punchout_film/chapters/common.py`, `c03_teacher.py`, `c08_knockdowns.py`
- Modify: `tools/punchout_film/music.py` (the `roar` sting)
- Test: `tools/punchout_film/tests/test_chapters.py`, `test_music.py`

**Interfaces:**
- Produces: `downs(take, field) -> list[int]` in `chapters/common.py`: steps where `field` (`"opponent_health"` or `"mac_health"`) reaches 0 from above. `c08.bells` uses it.
- Produces: `step_seconds(step, speed) -> float` = `step * MOVE_FRAMES / speed / 60`.
- Produces: `knockdown_stings(take, speed) -> list[tuple[float, str]]`: a `"roar"` at each `downs(take, "opponent_health")`. In the teacher's Von Kaiser take those are steps 251, 426 and 632; the KO itself is the take's end (step 801), where chapter 3's `ko_bell` already rings, so chapter 3 adds one more `"roar"` at the bell's time. Chapter 8's stack4 never wins, so it gets roars at its knockdowns only.
- Produces: `c02_idle.funeral_offset(ctx) -> float`: `step_seconds(downs(take, "mac_health")[-1], IDLE_SPEED)` for `p1-idle-glass-joe`.
- Produces: `STINGS["roar"]`: a short crowd-roar cue on the noise channel (a swelling then fading hiss, about 1.5 s), credit "original".

- [ ] **Step 1: Write failing tests**:
  - `test_funeral_march_starts_at_macs_last_knockdown`: `downs(idle_take, "mac_health")[-1] == 1106` and `abs(funeral_offset(ctx) - 18.43) < 0.05`.
  - `test_knockdowns_get_a_roar`: chapter 8's shot stings hold a `"roar"` at each `"bell"` time; chapter 3's knockout shot stings hold three roars before its bell and one at it (the teacher's three knockdowns and the KO), and still exactly one bell.
  - In `test_music.py`, `test_roar_is_a_sting` : `"roar" in stings` and it compiles (`compile_cue(stings["roar"], 1)`).
- [ ] **Step 2: Run** the punchout suite. Expected: failures/errors for exactly these.
- [ ] **Step 3: Implement.** The 4 s title card is not part of the shot, so offsets are shot-relative as the stings are now.
- [ ] **Step 4: Run** both suites. Expected: OK.
- [ ] **Step 5: Commit.** "Roar at every knockdown, and find Mac's last one."

### Task 3: The Marseillaise family, the circus and the funeral march

**Files:**
- Modify: `tools/punchout_film/music.py`
- Create: `tools/punchout_film/audition.py` (same as `reservoir_film.music.audition` but over the Punch-Out!! cues and stings, writing to `target/punchout-film/music/`; reuse its `spectrogram` and `render_cue`)
- Test: `tools/punchout_film/tests/test_music.py`

**Interfaces:**
- Produces in `music.py`: new instruments as needed (at least `BRASS`, a punchy duty-2 pulse with a fast attack, and a crowd-noise drum letter if the tracker needs one — add it to `core.DRUMS` only if no existing letter will do, and confirm the Tetris suite still passes). Cues, each with a `credit`:
  - `ring_entrance` (original fanfare)
  - `gladiators` (credit `"Julius Fučík, Entry of the Gladiators (1897)"`): circus oom-pah, one pulse slightly detuned for sourness (e.g. vibrato offset or an arp of +1 semitone on held notes).
  - `funeral` (credit `"Frédéric Chopin, Funeral March from Piano Sonata No. 2 (1839)"`): deadpan, slow, a few bars that can loop.
  - `marseillaise` (credit `"Claude Joseph Rouget de Lisle, La Marseillaise (1792)"`), `marseillaise_fast`, `marseillaise_elevator` (slow, soft, SOFT/HARM instruments, no drums or brushed hats), `marseillaise_stall` (the opening phrase builds, cuts to a rest or a deflating slide, restarts; never reaches the cadence). All four share the melody constant.
- [ ] **Step 1: Write failing test** `test_cues_compile_and_credit_their_composers`: for each of the names above, `compile_cue(CUES[name], 2)` succeeds and `CUES[name].credit` is `"original"` only for `ring_entrance`.
- [ ] **Step 2: Run.** Expected: KeyError on the first name.
- [ ] **Step 3: Write the cues.** Arrange from the public-domain melody (see Global Constraints). Keep loops 4–16 bars so they fit 15–60 s spans.
- [ ] **Step 4: Run** the suite, then `cd tools && python3 -m punchout_film.audition ring_entrance gladiators funeral marseillaise marseillaise_fast marseillaise_elevator marseillaise_stall`. Expected: OK; one WAV and spectrogram each in `target/punchout-film/music/`. Listen-check the spectrograms for all four channels sounding and no clipping (peak below 0 dBFS as the audition reports).
- [ ] **Step 5: Commit.** "Write the Marseillaise, the circus and the funeral march for the Punch-Out!! film."

### Task 4: The marches, the Rondo and the Valkyries

**Files:** `tools/punchout_film/music.py`, `tests/test_music.py`

**Interfaces:**
- Produces cues: `radetzky` (credit `"Johann Strauss I, Radetzky March (1848)"`), `triumph` (a heroic, over-the-top variation on the Radetzky melody: major, brass, big triangle runs; credit `"Johann Strauss I, Radetzky March (1848)"`, since it varies that tune), `alte_kameraden` (credit `"Carl Teike, Alte Kameraden (1889)"`, faster than a parade march), `turca` (credit `"Wolfgang Amadeus Mozart, Rondo alla Turca (1783)"`), `valkyries` (credit `"Richard Wagner, Ride of the Valkyries (1856)"`).
- [ ] **Step 1: Extend** `test_cues_compile_and_credit_their_composers` with these five names.
- [ ] **Step 2: Run.** Expected: KeyError on `radetzky`.
- [ ] **Step 3: Write the cues.**
- [ ] **Step 4: Run** the suite and the audition for the five. Expected as Task 3.
- [ ] **Step 5: Commit.** "Write the marches, the Rondo and the Valkyries."

### Task 5: The cue map, the credits, and the film

**Files:**
- Modify: `tools/punchout_film/music.py` (`CHAPTER_CUES` becomes the span map), `tools/punchout_film/film.py` (`strict_score=True`), `tools/punchout_film/chapters/c11_verdict.py` (credits)
- Test: `tools/punchout_film/tests/test_music.py`, `test_chapters.py`

**Interfaces:**
- Consumes: `cue_plan`, `unfit_spans` (Task 1); `funeral_offset` (Task 2); the cues (Tasks 3–4).
- The map, exactly as the spec's table:
  - 1: `ring_entrance`
  - 2: `[("idle", 0, "gladiators"), ("idle", funeral_offset, "funeral")]`
  - 3: `[("swap", 0, "lab"), ("defence", 0, "marseillaise"), ("knockout", 0, "triumph")]`
  - 4: `[("see_swap", 0, "lab"), ("bank", 0, "korobeiniki"), ("decide_swap", 0, "lab")]`
  - 5: `[("sidestep", 0, "marseillaise_fast"), ("bank", 0, "marseillaise_elevator")]`
  - 6: `radetzky`; 7: `alte_kameraden`; 8: `marseillaise_stall`; 9: `turca`; 10: `valkyries`; 11: `reprise`
  - `lab`, `korobeiniki` and `reprise` are imported from `reservoir_film.music.cues` unchanged.
- Produces: `music_credits(chapter_cues, cues) -> list[tuple[str, str]]` in `c11_verdict.py` (or `music.py`): one `(title, credit)` per distinct non-original credit among cues in the map, in film order, then `("Everything else", "Original.")`. `CREDITS` = the "Made with" and "Music" lines, then these, then the Nintendo line. The credits shot lasts long enough for every line to scroll past: `seconds = (H + len(CREDITS) * 110) / 60 + 2`.
- [ ] **Step 1: Write failing tests**:
  - `test_every_span_fits`: `unfit_spans(cue_plan(...))` for PUNCHOUT is `[]`.
  - `test_credits_name_exactly_the_cues_used`: the set of credit bodies in `CREDITS` (minus the fixed lines) equals the set of non-original credits of cues in the map; `"Minuet"` appears nowhere; `"Korobeiniki"` does.
  - `test_credits_scroll_fully`: at the credits shot's last frame, the last line's `y + 34` is above the top (`< 0`), i.e. everything has scrolled past.
  - Update `test_every_chapter_has_a_cue_and_every_sting_exists` to accept span lists (every named cue exists; every span's shot exists in its chapter).
- [ ] **Step 2: Run.** Expected: failures for these.
- [ ] **Step 3: Implement** the map, `strict_score=True`, the credits.
- [ ] **Step 4: Run** both suites, then `cd tools && python3 -m punchout_film.build`. Expected: suites OK; the build writes the MP4; `ffprobe -show_chapters` lists 11 chapters. Listen to chapters 2, 3, 5 and 8 in the MP4 around the cut and the knockdowns.
- [ ] **Step 5: Commit.** "Score the Punch-Out!! film with its own music, and credit it."
