# Series Bible and Production Pipeline Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** The series bible, and a pipeline that turns a video's script into a finished, narrated, line-timed Manim video with Greenstone footage, credits and a description, where any line can be re-voiced by dropping in a recorded take, proven on a short test clip (video 000).

**Architecture:** A new Python package `tools/series/`, run in its own uv virtualenv (Manim and the narrator need Python 3.12 and heavy dependencies the films don't). A video is a directory `tools/series/videos/NNN-slug/` holding `script.toml` (title, lines), `citations.toml` and `scene.py` (a Manim scene). The build synthesizes each line with a local narrator (cached by text), prefers a recorded take when one matches the line's text, renders the scene with each line's animations lasting as long as its audio, mixes an optional music bed, and writes the video and its description to `target/series/NNN-slug/`.

**Tech Stack:** Python 3.12 (uv venv), Manim Community, Kokoro (local TTS, Apache-2.0) with espeak-ng, soundfile, ffmpeg, `tomllib`; unittest; LaTeX (already installed) for Manim's MathTex. Footage via `reservoir_film.model.Takes`.

**Spec:** `docs/superpowers/specs/2026-10-29-interpretability-series-design.md`

## Global Constraints

- Never stage `src/main.rs` or `src/nes/bus/mod.rs`; leave `.claude/`, `AGENTS.md` alone.
- Commit with `git commit -F /private/tmp/claude-501/-Users-nathan-Projects-ndouglas-greenstone/cf099f6b-6b2a-4507-8759-c42c3f7c29f4/scratchpad/msg.txt`; every message ends with a blank line then `Claude-Session: https://claude.ai/code/session_01GxcjqSAGjzrqp9ufw5r46e`. Don't push.
- The repo is public. Nothing downloads ROMs.
- "The narrator runs on this machine with a good free voice, not a paid service."
- "The script is the source of truth. Narration is written as text, line by line. Each line is voiced, and the animation for that line lasts as long as its audio."
- "Re-voicing uses one recorded take per script line ... Any line can be re-recorded any number of times and replaced alone."
- "Every line and fragment of every script is supported by a citation, listed in the video's description and credits."
- "A disclaimer in every video: Nate is not an expert, just a guy learning this stuff, and the series is him learning in public."
- "Videos are numbered 1, 2, 3, … with no sub-numbering." The test clip is `000`; real videos start at `001`.
- The films' tooling and the Tetris and Punch-Out!! films must keep working unchanged.
- Tests: `cd tools && series/.venv/bin/python -m unittest discover -s series/tests -t .`

## Review Focus

- A take recorded before its line's text was edited: it must not be used; the build falls back to the synthesized voice and lists the line as stale.
- A script line the scene never says, says twice, or says out of order: the render fails naming the line, rather than shipping a video missing narration.
- Narration with symbols the narrator can't read ("w·x + b", "0x3C", "2,048"): a line's optional `spoken` text is what gets voiced (and keys the cache), while `text` stays the script and caption.
- Footage from a take that hasn't been recorded: a clear error with the take's record hint, not a stack trace from deep in Manim.
- Swapping one take: only that line changes; unchanged lines reuse their cached synthesis (no re-synthesis of the whole video).

---

### Task 1: The series bible

**Files:**
- Create: `docs/series/bible.md`
- Create: `docs/series/ideonomy/001-one-neuron-draws-a-line.md`
- Create: `docs/series/review-prompt.md`

**Interfaces:**
- Produces: the bible every later video is written against, and the reviewer prompt Task 5 and every video use.

- [ ] **Step 1: Write `docs/series/bible.md`** with these sections, taking content from the spec: Curriculum (the pilot arc table, numbered 001–010, as revisable); Per-video entries for 001 and 002 (the one idea, prerequisites, callbacks, games, citations to find); Assumptions ledger (the five assumptions from the spec's "Video 1 and 2's pressure test", each tagged with the video that introduces it); Visual language (placeholder pointing at `tools/series/style.py`, for Nate's review); Rules (the spec's teaching rules and anti-goals, one line each); How a video is made (bible → 5–6 ideonomy passes → script → two adversarial reviews → Nate's review → animation → Nate's review).
- [ ] **Step 2: Write `docs/series/ideonomy/001-one-neuron-draws-a-line.md`**: the six passes run in the design conversation (procedure, spectrum, graph, matrix, notation, cycle), as recorded in the session transcript, with their method tuples.
- [ ] **Step 3: Write `docs/series/review-prompt.md`**: the adversarial reviewer's prompt. It is given a video's `script.toml`, `citations.toml` and the bible; it reports every line that is incorrect, unsupported by its citations, uncited (and not `framing`), handwaved ("we'll see later"), out of order against the bible's prerequisites, or an LLM cliché; each finding names the line id; "nothing substantial" is a valid answer.
- [ ] **Step 4: Commit** `docs/series/` with message "Start the series bible."

### Task 2: Environment and the script model

**Files:**
- Create: `tools/series/__init__.py`, `tools/series/requirements.txt`, `tools/series/script.py`
- Create: `tools/series/tests/__init__.py`, `tools/series/tests/test_script.py`
- Modify: `.gitignore` (add `/tools/series/.venv/` and `/tools/series/videos/*/takes/`)

**Interfaces:**
- Produces: `ScriptError(Exception)`; `Line(id: str, text: str, spoken: str | None, cite: tuple[str, ...], framing: bool)` with property `voiced -> str` (`spoken` or `text`); `Citation(key: str, title: str, authors: str, year: int, url: str)`; `Video(name: str, dir: Path, title: str, music: Path | None, lines: tuple[Line, ...], citations: dict[str, Citation])`; `load_video(dir: Path) -> Video`; `VIDEOS = Path(__file__).parent / "videos"`.

- [ ] **Step 1: Set up the environment.** `brew install cairo pango pkg-config espeak-ng` (Manim's and Kokoro's native dependencies); `uv venv --python 3.12 tools/series/.venv`; `requirements.txt` pins exact current versions of `manim`, `kokoro`, `soundfile`, `numpy`; `uv pip install --python tools/series/.venv -r tools/series/requirements.txt`. Expected: `tools/series/.venv/bin/python -c "import manim, kokoro, soundfile"` exits 0.
- [ ] **Step 2: Write the failing tests** in `test_script.py`, each building a temporary video directory:
  - `test_loads_lines_in_order`: two `[[line]]` entries load as `Line`s in file order, with `title` and `citations`.
  - `test_spoken_text_is_what_gets_voiced`: a line with `text = "w·x + b"` and `spoken = "w times x, plus b"` has `voiced == "w times x, plus b"`; without `spoken`, `voiced == text`.
  - `test_every_line_needs_a_citation_or_framing`: a line with neither `cite` nor `framing = true` raises `ScriptError` whose message contains the line id.
  - `test_unknown_citation_key_is_an_error`: `cite = ["nope"]` raises `ScriptError` naming `nope` and the line.
  - `test_line_ids_are_unique_and_simple`: a duplicate id, and an id `"Intro 1"` (not matching `^[a-z0-9-]+$`), each raise `ScriptError`.
  - `test_music_is_resolved_against_the_repo`: `music = "target/x.wav"` resolves to the repo root's `target/x.wav`; absent `music` gives `None`.
- [ ] **Step 3: Run** the test command. Expected: errors, `series.script` has no `load_video`.
- [ ] **Step 4: Implement `load_video(dir: Path) -> Video`** in `tools/series/script.py` with `tomllib`; `name` is the directory name.
- [ ] **Step 5: Run** the test command. Expected: 6 tests pass.
- [ ] **Step 6: Commit** the package, tests, requirements and `.gitignore`.

### Task 3: Voices: synthesis, cache, takes

**Files:**
- Create: `tools/series/voice.py`, `tools/series/tests/test_voice.py`, `tools/series/audition.py`

**Interfaces:**
- Consumes: `Video`, `Line`, `Line.voiced` (Task 2).
- Produces: `class Engine(Protocol): name: str; def synthesize(self, text: str, out: Path) -> None` (writes a WAV); `KokoroEngine(voice: str = DEFAULT_VOICE)` with `name = f"kokoro-{voice}"`; `Voice(line_id: str, path: Path, seconds: float, source: Literal["take", "synth"])`; `voices(video: Video, engine: Engine, out: Path) -> tuple[dict[str, Voice], list[str]]` (voices by line id, and the ids of stale takes); `TAKES = "takes"` (the per-video subdirectory holding `<id>.wav` and `takes.toml`, which maps each id to the text it was recorded from).

- [ ] **Step 1: Write the failing tests** with a `FakeEngine` (writes a silent WAV whose length is `len(text) / 20` seconds and counts calls):
  - `test_each_line_is_synthesized_once_and_cached`: two calls to `voices` on the same video call the engine once per line in total; `seconds` match the WAV lengths.
  - `test_the_cache_is_keyed_by_the_voiced_text`: changing one line's `spoken` re-synthesizes only that line.
  - `test_a_matching_take_wins`: a take `takes/<id>.wav` whose `takes.toml` text equals the line's `text` is used, `source == "take"`.
  - `test_a_stale_take_falls_back_and_is_reported`: a take recorded from different text is not used; the line's voice is `"synth"` and its id is in the stale list.
  - `test_swapping_one_take_leaves_other_lines_cached`: after adding one take, a second `voices` call makes zero engine calls.
- [ ] **Step 2: Run** the test command. Expected: errors, `series.voice` missing.
- [ ] **Step 3: Implement** `voices` in `tools/series/voice.py`. Cache files are `out / "tts" / f"{line.id}-{h}.wav"` with `h` the first 12 hex digits of sha256 of `engine.name + "\0" + line.voiced`. `KokoroEngine` uses `kokoro.KPipeline(lang_code="a")` at 24 kHz and writes with `soundfile`.
- [ ] **Step 4: Run** the test command. Expected: all pass.
- [ ] **Step 5: Audition voices.** `tools/series/audition.py` renders one passage (the first paragraph of the spec's Intent) in three Kokoro voices into `target/series/audition/<voice>.wav`. Run it; pick `DEFAULT_VOICE` as the clearest, and note the other two for Nate.
- [ ] **Step 6: Commit.**

### Task 4: The narrated scene, footage and style

**Files:**
- Create: `tools/series/scene.py`, `tools/series/footage.py`, `tools/series/style.py`
- Create: `tools/series/tests/test_scene.py`, `tools/series/tests/test_footage.py`

**Interfaces:**
- Consumes: `load_video` (Task 2); `Voice` (Task 3).
- Produces: `NarratedScene(manim.Scene)` with class attributes `video: Video`, `voices: dict[str, Voice]` (set by the build), and `say(self, line_id: str, *animations, pad: float = 0.25) -> None`; `ScriptOrderError(Exception)`; `GameFootage(take: str, stream: str, start_frame: int = 0, root: Path = TAKES_DIR)`, a Manim `ImageMobject` showing frame `start_frame + round(t * 60)` at scene time `t` after it is added, scaled with nearest-neighbour resampling; `style.py` constants `INPUT, WEIGHT, BIAS, OUTPUT, BACKGROUND` (hex colours) and `FONT`.

- [ ] **Step 1: Write the failing tests** (`test_scene.py` renders tiny scenes at Manim's lowest quality into a temp dir, using fake voices of known lengths):
  - `test_a_line_lasts_as_long_as_its_voice`: a scene saying lines of 1.0 s and 2.0 s renders a video whose duration (ffprobe) is 3.5 s ± 0.1 (with the two 0.25 s pads), and whose audio stream exists.
  - `test_animations_are_stretched_to_the_line`: `say("a", FadeIn(dot))` with a 2.0 s voice plays the fade over 2.0 s (scene time after the call is 2.25).
  - `test_an_unsaid_line_fails_the_render`: a scene that never says line `b` raises `ScriptOrderError` naming `b` at teardown.
  - `test_lines_out_of_order_or_twice_fail`: saying `b` before `a`, or `a` twice, raises `ScriptOrderError` naming the line.
  - `test_footage_shows_the_frame_for_the_time` (`test_footage.py`, a two-frame fake take written in the `Takes` format): at t = 0 the image is frame 0; after 1/60 s it is frame 1.
  - `test_missing_take_names_the_record_hint`: `GameFootage("nope", ...)` raises `FileNotFoundError` containing "record it with".
- [ ] **Step 2: Run** the test command. Expected: errors, modules missing.
- [ ] **Step 3: Implement** `NarratedScene.say` (adds the voice with `add_sound`, plays the animations with `run_time` equal to the voice's length, or waits that long when none are given, then waits `pad`) and its teardown check; `GameFootage` with an updater over `reservoir_film.model.Takes`; `style.py` with a palette readable on a dark background, in which input, weight and bias are distinguishable to colour-blind viewers.
- [ ] **Step 4: Run** the test command. Expected: all pass.
- [ ] **Step 5: Commit.**

### Task 5: Credits, build, and the test clip

**Files:**
- Create: `tools/series/credits.py`, `tools/series/build.py`
- Create: `tools/series/videos/000-test-clip/script.toml`, `citations.toml`, `scene.py`
- Create: `tools/series/tests/test_credits.py`, `tools/series/tests/test_build.py`

**Interfaces:**
- Consumes: everything above.
- Produces: `DISCLAIMER: str`; `description(video: Video) -> str`; `credits(video: Video) -> manim.Mobject`; `build(name: str, engine: Engine, draft: bool = False) -> Path` (returns the finished mp4) and the CLI `python -m series.build NAME [--draft]`.

- [ ] **Step 1: Write the failing tests:**
  - `test_description_lists_every_cited_source_once`: a video citing key `k` on two lines lists `k`'s title, authors, year and URL exactly once, in first-cited order.
  - `test_description_ends_with_the_disclaimer`: the description ends with `DISCLAIMER`, whose text is exactly: "I'm not an expert, just a guy learning this stuff, in public. If you spot a mistake, please tell me."
  - `test_build_reports_stale_takes` (`test_build.py`, fake engine, `draft=True`, a two-line video with one stale take): the build finishes, prints the stale line's id, and writes `description.txt` next to the mp4.
  - `test_music_is_mixed_under_the_narration`: with `music` set to a 1 kHz test tone, the output's audio is longer than zero and the tone is present at lower level than the narration's peak (ffmpeg `volumedetect` on each).
- [ ] **Step 2: Run** the test command. Expected: errors, modules missing.
- [ ] **Step 3: Implement** `credits.py` (the credits mobject lists the citations and ends on the disclaimer) and `build.py`: load the video; `voices` into `target/series/NAME/`; import `videos/NAME/scene.py`, set the scene's `video` and `voices`, and render it (1080p60, or low quality with `--draft`); then, when `music` is set, mix it with ffmpeg at −18 dB under the narration, trimmed to the video; write `NAME.mp4` and `description.txt`; print each line's source (take or synth) and any stale takes.
- [ ] **Step 4: Run** the test command. Expected: all pass.
- [ ] **Step 5: Write video 000.** Four lines, each cited or `framing`: an opening; "a neuron multiplies, adds, and compares", animating `w·x + b` in the style colours (with `spoken` text); a Greenstone clip (`GameFootage` from an existing Punch-Out!! film take) under a line about the game's memory being the answer key (cite the design's Intent as a `framing` line, since it's a claim about this series); the credits. Music: an existing APU cue WAV from `target/punchout-film/music/`.
- [ ] **Step 6: Build it** with `cd tools && series/.venv/bin/python -m series.build 000-test-clip`. Expected: `target/series/000-test-clip/000-test-clip.mp4` exists, plays with narration, footage and music; the report shows four synth lines.
- [ ] **Step 7: Prove the re-voice swap.** Record one line with macOS `say -o` into `videos/000-test-clip/takes/<id>.wav` (converted to WAV with ffmpeg), list it in `takes.toml` with the line's text, rebuild. Expected: the report shows that line as `take`, the other three reuse the cache, and the video's duration changes by that line's length difference. Then edit that line's `text` and rebuild. Expected: the line is reported stale and voiced by the narrator. Remove the test take afterwards.
- [ ] **Step 8: Commit** (not the takes, which `.gitignore` excludes).
