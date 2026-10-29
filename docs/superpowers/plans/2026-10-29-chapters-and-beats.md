# Chapters, Beats and Breathing Room Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Every series video is built from sections (an Introduction, numbered chapters, Conclusions) of short beats with stillness after each and silent drifting title screens, narrated 10% slower; 001 is re-cut that way, storyboarded, reviewed, and ready for Nate's interview.

**Architecture:** `script.toml` gains `[[section]]` tables holding their beats (`[[section.line]]`); top-level `[[line]]`s after the sections are the closing (the disclaimer). `series/pacing.py` holds the timing settings and the length check; `series/title.py` builds the title screen; `NarratedScene` pauses after every beat and refuses a section's first beat before its title screen; the storyboard shows it all. The build refuses a section over the length limit before rendering.

**Tech Stack:** Python 3.12 (tools/series/.venv), Manim 0.22.1, Kokoro TTS, ffmpeg, unittest.

**Spec:** docs/superpowers/specs/2026-10-29-chapters-and-beats-design.md (and the series spec, docs/superpowers/specs/2026-10-29-interpretability-series-design.md)

## Global Constraints

- Applies to every video: the format, the scene and the build change, and the bible and council say so.
- Stillness after each beat: 2.0 s by default (a beat may set its own `pause`).
- Title screen: silent; about 4 s; the chapter number ("Chapter 3"; none for the Introduction and Conclusions) and title over a dimmed wall of uneven tiles from that section, the whole wall drifting slowly one way (mockup option A).
- No section longer than 240 s (4 minutes) as voiced, with pauses and its title screen; a longer one is split.
- Narration speed 0.9 (10% slower), one setting, part of the voice cache key.
- Every number in the script still comes from `numbers.json`; every line still cites or is `framing`; every beat has one board (one picture or animation).
- Never stage `src/main.rs` or `src/nes/bus/mod.rs`; commit with `git commit -F` and the session trailer; work on `main`; don't push without asking.
- Tests: `cd tools && series/.venv/bin/python -m unittest discover -s series/tests -t .` (pytest isn't installed).

## Review Focus

- A script with no `[[section]]` at all (the 000 test clip): loads and builds exactly as before, with stillness after its lines. Test: `test_a_script_without_sections_still_loads` (Task 2), `test_storyboard_needs_no_scene_file` stays green (Task 4).
- A chapter whose intro or conclusion beat is missing or out of place: a ScriptError naming the chapter. Test: `test_a_chapter_needs_its_intro_first_and_conclusion_last` (Task 2).
- A section that runs over 240 s once voiced: the build stops before rendering and names it, with its length. Test: `test_a_section_over_the_limit_stops_the_build` (Task 4).
- A beat said before its section's title screen, or a title screen shown twice: ScriptOrderError. Test: `test_a_sections_first_beat_needs_its_title_screen` (Task 3).
- A speed change silently reusing old voices: the cache key changes with speed. Test: `test_the_speed_is_part_of_the_cache_key` (Task 1).

---

### Task 1: Pacing settings and a slower narrator

**Files:**
- Create: `tools/series/pacing.py`
- Modify: `tools/series/voice.py` (KokoroEngine)
- Test: `tools/series/tests/test_voice.py`, `tools/series/tests/test_pacing.py` (new)

**Interfaces:**
- Produces: `pacing.STILLNESS = 2.0`, `pacing.TITLE_HOLD = 4.0`, `pacing.SECTION_LIMIT = 240.0`, `pacing.NARRATION_SPEED = 0.9`; `KokoroEngine(voice: str = DEFAULT_VOICE, speed: float = NARRATION_SPEED)` with `name == f"kokoro-{version('kokoro')}-{voice}-speed{speed}"`, passing `speed=self.speed` to `KPipeline.__call__`.

- [ ] **Step 1: Write the failing tests**: `test_kokoro_engine_names_its_version` extended to assert `"speed0.9" in KokoroEngine().name`; `test_the_speed_is_part_of_the_cache_key`: `KokoroEngine(speed=0.9).name != KokoroEngine(speed=1.0).name`; `test_pacing_defaults` asserting the four values above.
- [ ] **Step 2: Run** `series/.venv/bin/python -m unittest series.tests.test_voice series.tests.test_pacing`. Expected: FAIL/ERROR (no `pacing`, no `speed`).
- [ ] **Step 3: Implement** `pacing.py` (constants, each with a one-line comment citing the spec) and `KokoroEngine.speed`.
- [ ] **Step 4: Run** the same command. Expected: OK.
- [ ] **Step 5: Commit** "Slow the narrator by 10% and add the pacing settings."

### Task 2: Sections and beats in the script format

**Files:**
- Modify: `tools/series/script.py`
- Test: `tools/series/tests/test_script.py`

**Interfaces:**
- Consumes: nothing new.
- Produces:
  - `Line` gains `role: str | None = None` (`"intro"` or `"conclusion"`), `pause: float | None = None`, `section: str | None = None` (its section's id).
  - `@dataclass(frozen=True) class Section: id: str; kind: str; number: int | None; title: str; lines: tuple[Line, ...]`.
  - `Video` gains `sections: tuple[Section, ...]` (empty for a sectionless script). `Video.lines` stays the flat order: every section's lines in order, then the top-level `[[line]]`s (the closing).
  - `SECTION_KINDS = ("introduction", "chapter", "conclusions")`. Ids: `"introduction"`, `"chapter-<n>"`, `"conclusions"`; chapters are numbered 1, 2, … in file order.
- TOML shape:
  ```toml
  [[section]]
  kind = "chapter"
  title = "Building one neuron"
  [[section.line]]
  id = "building-intro"
  role = "intro"
  text = "…"
  framing = true
  ```

- [ ] **Step 1: Write the failing tests** (each builds a script with `video()` as the file already does):
  - `test_sections_hold_their_beats_in_order`: an introduction, two chapters and conclusions, plus a top-level closing line → `[s.id for s in v.sections] == ["introduction", "chapter-1", "chapter-2", "conclusions"]`, `[s.number for s in v.sections] == [None, 1, 2, None]`, `v.lines` in file order with the closing last, each line's `section` set (the closing's is `None`).
  - `test_a_chapter_needs_its_intro_first_and_conclusion_last`: a chapter whose first line lacks `role = "intro"`, and one whose last lacks `role = "conclusion"` → `ScriptError` mentioning `"chapter-1"`.
  - `test_sections_must_be_in_their_places`: conclusions before a chapter, or two introductions, or an unknown kind → `ScriptError`.
  - `test_roles_belong_to_chapters`: `role = "intro"` inside the introduction section → `ScriptError`.
  - `test_a_beat_can_set_its_own_pause`: `pause = 0.5` → `line.pause == 0.5`; absent → `None`.
  - `test_a_script_without_sections_still_loads`: the existing single-line scripts → `v.sections == ()`, lines unchanged.
  - Placeholders are filled in section beats as in top-level lines (extend `test_numbers_fill_the_script` with a section).
- [ ] **Step 2: Run** `series/.venv/bin/python -m unittest series.tests.test_script`. Expected: failures/errors on the new tests only.
- [ ] **Step 3: Implement** in `load_video`: read `script.get("section", [])`, build each section's lines with the existing per-line validation, assign ids and numbers, then the top-level lines; enforce: at most one introduction (first) and one conclusions (last, before the closing), chapters non-empty with intro first and conclusion last, `role` only in chapters, `title` required, `kind` in `SECTION_KINDS`, line ids unique across the whole script.
- [ ] **Step 4: Run** the whole suite. Expected: OK.
- [ ] **Step 5: Commit** "Give the script format sections and beats."

### Task 3: Stillness and silent title screens in the scene

**Files:**
- Create: `tools/series/title.py`
- Modify: `tools/series/scene.py`
- Test: `tools/series/tests/test_scene.py`

**Interfaces:**
- Consumes: `pacing.STILLNESS`, `pacing.TITLE_HOLD`; `Section`, `Line.pause`, `Line.section` (Task 2).
- Produces:
  - `title.title_card(section: Section, tiles: list[Mobject]) -> Group`: the wall (three rows of tiles of uneven widths, wider than the frame, under a black overlay at opacity 0.68) and, centred over it, "Chapter <n>" (style.INPUT, small, letter-spaced; omitted when `number is None`) above the title (style.TEXT). The wall is the group's first submobject.
  - `title.board_tiles(section: Section) -> list[Mobject]`: one tile per beat with a board: a rounded rectangle holding the first 40 characters of the board, for the storyboard until real pictures exist.
  - `NarratedScene.show_title(section_id: str, tiles: list[Mobject] | None = None) -> None`: silent; adds the card, shifts its wall left by 22% of the frame width, linearly, over `TITLE_HOLD` seconds; then removes it. `tiles` defaults to `board_tiles`.
  - `NarratedScene.say(line_id, *animations, pad: float | None = None)`: after the voice, waits `pad`, else `line.pause`, else `STILLNESS`.
  - Order: the first beat of a section raises `ScriptOrderError` unless that section's title was shown; showing a title twice, or for an unknown section, raises too.
  - `StoryboardScene.construct`: for each section, `show_title(section.id)` then each beat's `placeholder`; then the closing lines' placeholders; then `roll_credits()`.

- [ ] **Step 1: Write the failing tests** (60 fps renders, as the file does):
  - Update `test_a_line_lasts_as_long_as_its_voice`: voices 1.0 + 2.0 → duration ≈ 1.0 + 2.0 + 2 × 2.0 = 7.0 (delta 0.1).
  - `test_a_beat_can_shorten_its_stillness`: a line with `pause = 0.5` → that line's share is voice + 0.5.
  - `test_a_title_screen_is_silent_and_holds`: a one-chapter script; `show_title("chapter-1")` then its beats → duration includes 4.0 s before the first voice, and the audio's first 3.9 s are silent (`band_level`-style volumedetect ≤ −80 dB, or check the first sound's offset via `scene.renderer.file_writer.sections`; either is fine).
  - `test_a_sections_first_beat_needs_its_title_screen`: saying a chapter's first beat without `show_title` → `ScriptOrderError` naming the chapter; `show_title` twice → `ScriptOrderError`.
  - `test_the_storyboard_shows_every_title`: `StoryboardScene` over a two-chapter script renders, with duration = Σ(voice + 2.0) + 2 × 4.0 + credits (delta 0.2).
  - `test_title_card_numbers_only_chapters` (no render): `title_card` of a chapter contains a Text "Chapter 1"; of the introduction, no "Chapter" text.
- [ ] **Step 2: Run** `series/.venv/bin/python -m unittest series.tests.test_scene`. Expected: the new and updated tests fail.
- [ ] **Step 3: Implement** `title.py` and the scene changes. The placeholder card also shows the section's title small at the top, so a storyboard frame says where it is.
- [ ] **Step 4: Run** the whole suite. Expected: OK (the build tests' duration assertions may need the new stillness added; update their expected values, not the behaviour).
- [ ] **Step 5: Commit** "Pause after every beat and give sections silent title screens."

### Task 4: The section length check in the build

**Files:**
- Modify: `tools/series/pacing.py`, `tools/series/build.py`
- Test: `tools/series/tests/test_pacing.py`, `tools/series/tests/test_build.py`

**Interfaces:**
- Consumes: `Video.sections`, `Voice.seconds`, Task 1's constants.
- Produces:
  - `pacing.section_seconds(section: Section, voices: dict[str, Voice]) -> float` = `TITLE_HOLD` + Σ over its lines of (voice seconds + the line's pause or `STILLNESS`).
  - `pacing.too_long(video: Video, voices: dict[str, Voice]) -> list[str]`: one message per section over `SECTION_LIMIT`, e.g. `"chapter-2 (Is it any good?) runs 4:12, over 4:00; split it"`.
  - `class SectionTooLong(Exception)`; `build()` prints every section's length (`chapter-1 Building one neuron 2:41`), then raises `SectionTooLong` with all messages if any, after voicing and before rendering.

- [ ] **Step 1: Write the failing tests**: `test_section_seconds_adds_voices_pauses_and_the_title` (fake voices 10 s and 20 s, one with `pause = 0.5` → 4.0 + 12.0 + 20.5 = 36.5); `test_too_long_names_the_section` (a 250 s voice → one message containing `"chapter-1"` and `"4:"`); `test_a_section_over_the_limit_stops_the_build` (in `test_build.py`, `ToneEngine` and a chapter whose one beat's text is 5,000 characters, so 250 s → `SectionTooLong`, and no `media` directory was made).
- [ ] **Step 2: Run** the two test modules. Expected: errors (no functions).
- [ ] **Step 3: Implement.**
- [ ] **Step 4: Run** the whole suite. Expected: OK.
- [ ] **Step 5: Commit** "Refuse a section longer than four minutes before rendering."

### Task 5: The process, for every video

**Files:**
- Modify: `docs/series/council.md` (a new lens), `docs/series/bible.md` ("How a video is made" and "Rules"), `docs/series/review-prompt.md` (beats and sections), `docs/superpowers/specs/2026-10-29-interpretability-series-design.md` (a pointer to the chapters design)

**Interfaces:** none (documents). The lens text is used by Task 8.

- [ ] **Step 1:** Add the **Continuity and pacing** lens to `council.md`, in the lenses' format: one thought and one picture per beat; beats short enough to absorb; the stillness doing its job (no padding after bridges, none missing after dense beats); each chapter's introduction states the problem it solves and its conclusion zooms out (from X to Y, why it mattered, what's next); sections under 4 minutes; nothing used before it's set up; the flow from beat to beat. It reads the storyboard's section lengths as well as the script.
- [ ] **Step 2:** In the bible: every video has an Introduction, numbered chapters (each with a silent title screen, an introduction beat and a conclusion beat) and Conclusions that TL;DR it; no section over 4 minutes; one thought and one picture per beat; 2 s of stillness after each; the storyboard comes before the council (Nate, 2026-10-29). In `review-prompt.md`, tell reviewers the script is sections of beats, and add "a beat carrying two thoughts or two pictures" to what they report.
- [ ] **Step 3:** Run `grep -n "Continuity and pacing" docs/series/council.md`. Expected: the lens heading.
- [ ] **Step 4: Commit** "Make chapters, beats and a pacing lens part of every video."

### Task 6: Re-cut 001 into sections and beats

**Files:**
- Modify: `tools/series/videos/001-one-neuron/script.toml`, `tools/series/videos/001-one-neuron/citations.toml`, `docs/series/games/001.md` (the 6502 example)
- Test: the suite's `test_every_video_loads`

**Interfaces:**
- Consumes: Task 2's format.
- Produces: 001 as sections: Introduction (`memory`…`start-with-one`); Chapter 1 Building one neuron (`thermostat`…`the-neuron`); Chapter 2 Is it any good? (`baseline`…`castlevania`); Chapter 3 The recipe (`why-weights`…`one-dot`); Chapter 4 Two numbers (`new-question`…`land-mistakes`); Conclusions (a TL;DR of the whole video, then `the-idea`, `modern`, `harder`, and "what's next" pointing at 002); closing: `disclaimer`. Titles are working titles.

- [ ] **Step 1: Split every line into beats**, one thought and one board each, keeping the narration's wording where it survives; new pictures where a thought needs its own (the spec's `land-mistakes` example is the model, verbatim). Beat ids: the old id for the first beat, `-2`, `-3` suffixes for the rest. Every number stays a placeholder.
- [ ] **Step 2: Write each chapter's intro beat** (the problem it solves; for a listener, what the chapter is about, since the title is silent) and **conclusion beat** (recap; from the previous chapter's X to this one's Y; why it mattered; what's next). No "In this chapter, we'll…".
- [ ] **Step 3: Rewrite `why-zero`'s reasons** per the spec: the learning one (the bias as one more weight on an input that's always 1, so the learning rule nudges it like the others; one sentence, stated, not deferred), and the machine aside (one sentence: arithmetic itself tells a processor whether a result is zero or negative, so comparing with zero needs no extra step). Find a real zero- or negative-flag branch straight after arithmetic in a published Super Mario Bros. disassembly (cite it), check its bytes against the ROM in `roms2/` (the opcode at the address, from the iNES file's PRG), record the address and instructions in `games/001.md`, and make it the beat's board. Add citations: a 6502 instruction reference for the flags and branches, the disassembly, and a general source if the sentence claims more than the 6502.
- [ ] **Step 4: Verify.** Run `series/.venv/bin/python -c "from series.script import load_video, VIDEOS; v = load_video(VIDEOS / '001-one-neuron'); print([(s.id, s.title, len(s.lines)) for s in v.sections])"`. Expected: six sections as above, each chapter starting with an intro beat and ending with a conclusion beat. Then the whole suite. Expected: OK.
- [ ] **Step 5: Commit** "Re-cut video 001 into sections and beats."

### Task 7: The chaptered storyboard

**Files:**
- Modify (only if a section is too long): `tools/series/videos/001-one-neuron/script.toml`

**Interfaces:** consumes Tasks 1–6.

- [ ] **Step 1: Build** `cd tools && series/.venv/bin/python -m series.build 001-one-neuron --storyboard` (background it; it takes minutes). Expected: each section's length printed; no `SectionTooLong`.
- [ ] **Step 2: If a section is too long**, split it into two chapters (each with its own title, intro and conclusion beats) at a natural break, and rebuild. Expected: every section under 4:00.
- [ ] **Step 3:** Record each section's length and the total in `docs/series/reviews/001/pacing.md` (the pacing lens reads it), and make a contact sheet with `series.sketch`'s `contact_sheet` (one still per title screen and a few beats) to check the title screens render.
- [ ] **Step 4: Commit** "Storyboard 001 in chapters."

### Task 8: The council and the adversarial reviews

**Files:**
- Create: `docs/series/reviews/001/round-4.md` (and further rounds), `docs/series/reviews/001/adversarial-12.md` (and further passes)
- Modify: the script and its sources as findings require

**Interfaces:** consumes `council.md` (with Task 5's lens), `review-prompt.md`, `pacing.md`.

- [ ] **Step 1: Run the council**: every lens, including Continuity and pacing, in parallel, each given the script, citations, numbers, bible, sources file, games doc and `pacing.md` (as `council.md` says). Fix, then repeat, up to three rounds, until no lens reports anything substantial. Record each round.
- [ ] **Step 2: Run the adversarial reviews** (two independent reviewers per pass, `review-prompt.md`), fixing and repeating until both find nothing substantial. Read every fix skeptically: earlier passes found fixes introducing errors.
- [ ] **Step 3: Rebuild the storyboard** after the last fix (Task 7's command). Expected: no `SectionTooLong`.
- [ ] **Step 4: Commit** each round's record with its fixes.

### Task 9: Nate's interview, with the storyboard

**Files:**
- Create: `docs/series/reviews/001/interview-2.md`

- [ ] **Step 1:** Prepare questions by lens (as `council.md`'s interview section says), weighted to what changed: the chapter breaks and titles, each intro and conclusion, the new beats and their pictures, the zero beats, the pacing. Point Nate at the storyboard and a fresh reading copy.
- [ ] **Step 2:** Ask Nate the first few questions and stop. The interview continues in conversation; changes it produces go back through the reviews if substantial.
- [ ] **Step 3: Commit** the questions file.
