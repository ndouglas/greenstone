# The Beat Recorder Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Nate records a video's beats one at a time on a local page, every take kept raw, and the build gives them one consistent sound, lays room tone under them, and refuses to release a video with any beat not in his voice.

**Architecture:** `series/takes.py` is the take store (raw takes, their text and trim points, which one is chosen, the room recording). `series/sound.py` is the filter chain, run with ffmpeg and cached. `voice.py` voices each line from its chosen take through the chain, else the narrator. `build.py` gains `--release` and the room-tone bed. `series/record.py` is the recorder: a `Session` that records from an input device into the store (testable with a fake input), and a stdlib HTTP server serving `record.html`, whose keys drive the session and whose `<audio>` plays takes back.

**Tech Stack:** Python 3.12 (tools/series/.venv), sounddevice (PortAudio), soundfile, numpy, ffmpeg, stdlib `http.server`, plain HTML/JS, unittest.

**Spec:** docs/superpowers/specs/2026-10-30-beat-recorder-design.md

## Global Constraints

- Raw stays raw: takes are written exactly as recorded (48 kHz when the device allows, 24-bit PCM, mono), never overwritten, never altered; trims are stored, not applied.
- Raw takes at `takes/raw/<id>/<n>.wav`, numbered from 1. Takes live in `tools/series/videos/NNN/takes/`, which git ignores; never commit them.
- `takes.toml`: table form `[<id>]` with `text` and `take`; the old `<id> = "<text>"` with `takes/<id>.wav` keeps working.
- A take is stale when the text it was recorded from differs from the beat's text now.
- Trim pad: 0.15 s at each end.
- Checks: clipping (any sample at full scale), too quiet (peak below −20 dBFS), length more than 50% off the narrator's reading (a hint).
- The chain, in order: trim; high-pass ~80 Hz; light noise reduction; de-esser; compression ~3:1; presence lift ~3–5 kHz; loudness matched to −16 LUFS; limiter at −1 dB. Its version is part of the cache key.
- Room tone: 10 s; looped under the whole narration from the first title screen to the credits; the chain without loudness matching, at the median of the takes' gains; only when every beat is a take; absent room recording means no bed.
- Release: `--release` refuses unless every beat has a current take, naming the beats that don't. Every full build reports how many beats Kokoro narrates.
- Nothing leaves the machine: the server binds 127.0.0.1 only.
- Never stage `src/main.rs` or `src/nes/bus/mod.rs`; commit with `git commit -F` and the session trailer; work on `main`; don't push without asking.
- Tests: `cd tools && series/.venv/bin/python -m unittest discover -s series/tests -t .`

## Review Focus

- The script's text changes after a beat was recorded: the take is stale; drafts use the narrator, `--release` refuses and names the beat, the recorder marks it. Test: `test_a_take_goes_stale_when_the_text_changes` (Task 3), `test_release_refuses_a_stale_take` (Task 4).
- Choosing an older take that was read from different text than the newest: stale is judged against that take's own text. Test: `test_staleness_follows_the_chosen_take` (Task 1).
- Space pressed twice quickly, or a take with no speech in it: kept, flagged "no speech", and not chosen. Test: `test_a_silent_take_is_kept_but_not_chosen` (Task 1).
- An input device that can't do 48 kHz (many run at 44.1): records at the device's rate; the chain's output is 48 kHz all the same. Test: `test_the_chain_outputs_48k_from_any_rate` (Task 2).
- The recorder stopped (Ctrl-C) mid-take: no half-written take and no gap in numbering. Test: `test_an_abandoned_take_leaves_nothing` (Task 5).

---

### Task 1: The take store

**Files:**
- Create: `tools/series/takes.py`
- Test: `tools/series/tests/test_takes.py`

**Interfaces:**
- Produces:
  - `RAW = "raw"`; `PAD = 0.15`; `ROOM = "room.wav"` (in `takes/`, beside `takes.toml`).
  - `@dataclass(frozen=True) class Take: line_id: str; number: int; path: Path; text: str; start: float; end: float; rate: int` — `number` 0 means an old-form take (`takes/<id>.wav`, trim the whole file); `property seconds -> float` (end − start).
  - `trim(audio: np.ndarray, rate: int) -> tuple[float, float] | None` — speech start and end in seconds with PAD, clamped to the file; None when there's no speech.
  - `class TakeStore(dir: Path)` (`dir` is the video's `takes/` folder):
    - `takes(line_id) -> list[Take]` (by number);
    - `add(line_id, audio, rate, text) -> Take` — writes `raw/<id>/<n>.wav` (24-bit) and `raw/<id>/<n>.toml` (`text`, `start`, `end`, `rate`; a silent take stores no start/end and `silent = true`), via a temporary name renamed into place; chooses the new take unless it's silent;
    - `choose(line_id, number) -> None` (ValueError for an unknown or silent take);
    - `chosen(line_id) -> Take | None` — the table form's `take`, else the old form;
    - `silent(take) -> bool`;
    - `set_room(audio, rate) -> None`, `room() -> Path | None`.

- [ ] **Step 1: Write the failing tests** in `test_takes.py`, with synthetic audio (silence, then a 0.5 s 200 Hz tone at 0.3 amplitude, then silence):
  - `test_trim_finds_the_speech_with_a_pad`: tone from 1.0 to 1.5 s → `trim` ≈ (0.85, 1.65), within 0.03.
  - `test_trim_clamps_to_the_file`: tone from 0.05 s → start 0.0.
  - `test_trim_of_silence_is_none`: zeros (and noise at −60 dBFS) → None.
  - `test_takes_are_numbered_and_kept`: three `add`s → numbers [1, 2, 3], files `raw/a/1.wav`… exist, each `soundfile.info(...).subtype == "PCM_24"`, take 3 chosen.
  - `test_choose_an_earlier_take`: `choose("a", 1)` → `chosen("a").number == 1`, and `takes.toml` has `[a]` with `take = 1` and that take's text.
  - `test_staleness_follows_the_chosen_take`: take 1 read from "Old.", take 2 from "New."; `choose("a", 1)` → `chosen("a").text == "Old."`.
  - `test_a_silent_take_is_kept_but_not_chosen`: add a voiced take, then a silent one → two takes, the first chosen, `silent(takes[1])`; `choose("a", 2)` raises ValueError.
  - `test_the_old_form_still_reads`: `takes/a.wav` + `a = "Hello."` → `chosen("a")` has number 0, text "Hello.", start 0, end = its length.
  - `test_room_tone_is_stored`: `set_room` then `room()` is `takes/room.wav`; before it, None.

- [ ] **Step 2: Run** `cd tools && series/.venv/bin/python -m unittest series.tests.test_takes` — Expected: ImportError (no `series.takes`).

- [ ] **Step 3: Implement `takes.py`.** `trim`: RMS over 20 ms frames; a frame is speech when above max(−50 dBFS, loudest frame − 35 dB); first and last speech frames, padded by PAD. `takes.toml` is written whole each time (read, update, write; `tomllib` reads, write the simple table form by hand with TOML string escaping).

- [ ] **Step 4: Run** the same command — Expected: OK.

- [ ] **Step 5: Commit** `tools/series/takes.py tools/series/tests/test_takes.py` — "Add the take store for recorded narration."

### Task 2: The sound chain

**Files:**
- Create: `tools/series/sound.py`
- Test: `tools/series/tests/test_sound.py`

**Interfaces:**
- Consumes: `Take` (Task 1).
- Produces:
  - `CHAIN_VERSION = 1`; `TARGET_LUFS = -16.0`; `LIMIT = 0.89` (−1 dB); `RATE = 48000`.
  - `FILTERS = "highpass=f=80,afftdn=nr=10:nf=-50,deesser=i=0.4,acompressor=threshold=-20dB:ratio=3:attack=5:release=80:makeup=2,equalizer=f=4000:t=q:w=1:g=2"` (starting values; Nate tunes them).
  - `loudness(path: Path) -> float` — integrated LUFS (ffmpeg `ebur128`).
  - `process(take: Take, cache: Path) -> tuple[Path, float]` — the processed wav (48 kHz, 16-bit, mono) and the gain in dB that matched it to TARGET_LUFS; cached at `cache/<id>-<n>-<key>.wav` where key hashes CHAIN_VERSION, FILTERS, the raw file's bytes, start and end; the gain is stored beside it (`.json`) so a cache hit returns it too.
  - `process_room(path: Path, gain_db: float, cache: Path) -> Path` — FILTERS, then `volume=gain_db`, then the limiter; cached the same way.

- [ ] **Step 1: Write the failing tests** (`ffmpeg` available; synthetic takes made with `TakeStore`):
  - `test_takes_come_out_at_the_same_loudness`: a tone at 0.05 and one at 0.5 amplitude → `loudness` of both outputs within 1 LU of −16.
  - `test_the_chain_outputs_48k_from_any_rate`: a take recorded at 44100 → output `soundfile.info().samplerate == 48000`.
  - `test_the_trim_is_applied`: take with start 1.0, end 1.5 → output length ≈ 0.5 s (±0.05).
  - `test_the_limiter_holds`: a take at 0.99 amplitude → output peak ≤ 0.9.
  - `test_processing_is_cached`: second `process` call returns the same path and gain without running ffmpeg (patch `subprocess.run` to fail after the first call); changing `CHAIN_VERSION` (patch) gives a new path.
  - `test_room_tone_takes_the_given_gain`: room noise at −50 dBFS processed with gain +6 is louder than with gain 0 by about 6 dB (±1.5).

- [ ] **Step 2: Run** `series.tests.test_sound` — Expected: ImportError.

- [ ] **Step 3: Implement `sound.py`.** Two ffmpeg passes per take: trim (`atrim`) + FILTERS, measured with `ebur128`; then `volume=<gain>dB,alimiter=limit=0.89`, resampled to 48 kHz. Write to a temporary name and rename, as voice.py's cache does.

- [ ] **Step 4: Run** — Expected: OK.

- [ ] **Step 5: Commit** — "Add the sound chain for recorded takes."

### Task 3: Voicing lines from takes

**Files:**
- Modify: `tools/series/voice.py` (`voices`, `take_problems`, `prune`; `Voice`)
- Test: `tools/series/tests/test_voice.py`

**Interfaces:**
- Consumes: `TakeStore`, `Take` (Task 1); `process` (Task 2).
- Produces:
  - `Voice` gains `gain_db: float | None = None` (a take's chain gain).
  - `voices(video, engine, out) -> tuple[dict[str, Voice], list[str]]` — unchanged signature; a line's voice is its chosen take, processed (cache `out/takes/`), when that take's text equals the line's text; else the narrator, and a stale chosen take's id is in the second list. The narrator's reading is synthesized and kept for every line, take or not.
  - `narrator(video, engine, out) -> dict[str, Path]` — every line's narrator reading (the cache voices() uses), for the recorder.
  - `take_problems(video)` also reports raw take folders for ids the script doesn't have.

- [ ] **Step 1: Write the failing tests** (existing helpers; `ToneEngine`-style fake engine):
  - `test_a_chosen_take_is_processed_and_used`: a table-form take → `Voice.source == "take"`, path under `out/takes/`, `gain_db` not None.
  - `test_a_take_goes_stale_when_the_text_changes`: take read from "Old." for a line now "New." → narrator used, id in stale.
  - `test_the_narrator_is_kept_for_recorded_lines`: after `voices`, `narrator(...)[id]` exists for a recorded line, and a second `voices` call doesn't synthesize again.
  - `test_raw_takes_for_unknown_ids_are_reported`: `raw/ghost/1.wav` → a problem naming `ghost`.
  - The existing old-form tests stay green.

- [ ] **Step 2: Run** `series.tests.test_voice` — Expected: the new tests fail.

- [ ] **Step 3: Implement.** `prune` keeps narrator files for every line and processed takes in use.

- [ ] **Step 4: Run** the whole suite — Expected: OK.

- [ ] **Step 5: Commit** — "Voice lines from chosen takes through the sound chain."

### Task 4: Release builds and room tone

**Files:**
- Modify: `tools/series/build.py` (`build`, `mix`, `main`)
- Test: `tools/series/tests/test_build.py`

**Interfaces:**
- Consumes: `voices` (Task 3), `TakeStore.room` (Task 1), `process_room` (Task 2).
- Produces:
  - `class NotNarrated(Exception)`.
  - `build(..., release: bool = False)`; CLI `--release`.
  - `mix(movie, music, out, room: Path | None = None)` — the room bed looped under the narration from 0 to the movie's length minus the credits' seconds, at its own level (already set by `process_room`), before the music and the limiter.

- [ ] **Step 1: Write the failing tests:**
  - `test_release_refuses_beats_in_the_narrators_voice`: one of two lines has a take → `NotNarrated` naming the other, raised before rendering (no `media` folder).
  - `test_release_refuses_a_stale_take`: both recorded, one stale → `NotNarrated` naming it.
  - `test_release_builds_when_every_beat_is_a_take`: both current → the movie exists.
  - `test_the_build_counts_the_narrators_beats`: a full (non-storyboard) build with one take prints "1 of 2 beats narrated by Kokoro".
  - `test_room_tone_plays_only_under_takes`: with a room recording (noise at a known band, e.g. a 3 kHz tone) and every beat a take → that band is present in a pause; with one beat in the narrator's voice → absent; and absent during the credits.

- [ ] **Step 2: Run** `series.tests.test_build` — Expected: the new tests fail.

- [ ] **Step 3: Implement.** The room gain is the median of the voices' `gain_db`. The release check runs right after `voices()`, before the length check and the render.

- [ ] **Step 4: Run** the whole suite — Expected: OK.

- [ ] **Step 5: Commit** — "Refuse release builds with narrator beats; lay room tone under takes."

### Task 5: The recording session

**Files:**
- Create: `tools/series/record.py` (the `Session` and inputs; the server comes in Task 6)
- Test: `tools/series/tests/test_record.py`

**Interfaces:**
- Consumes: `TakeStore` (Task 1); `process` (Task 2); `narrator` (Task 3).
- Produces:
  - `class Input(Protocol)`: `rate: int`; `start(callback: Callable[[np.ndarray], None]) -> None`; `stop() -> None`.
  - `class DeviceInput(name: str | None)` — sounddevice, mono, 48 kHz if the device supports it, else its default rate; float32 blocks to the callback.
  - `ROOM_SECONDS = 10.0`; `LENGTH_HINT = 0.5`; `QUIET_DB = -20.0`.
  - `checks(audio: np.ndarray, rate: int, take: Take, narrator_seconds: float) -> list[str]` — any of "clipped", "too quiet", "no speech", "much longer than the narrator", "much shorter than the narrator".
  - `class Session(video: Video, store: TakeStore, input: Input, narration: dict[str, Path], cache: Path)`:
    - `index: int` (into `video.lines`), `goto(i)`, `next()`, `prev()` (clamped);
    - `start()`, `stop() -> Take`, `abandon()`; `recording: bool`; `level: float` (peak dBFS of the latest block);
    - `record_room()` — records ROOM_SECONDS then stops by itself into `store.set_room`;
    - `choose(n)`;
    - `state() -> dict` — JSON-ready: `beat` (`id`, `text`, `board`, `section` title, `position` e.g. "Chapter 3 · beat 7 of 15"), `before`/`after` texts, `takes` (each `number`, `seconds`, `chosen`, `stale`, `problems`), `narrator_seconds`, `recording`, `level`, `strip` (one of "unrecorded", "recorded", "stale" per line), `room` (bool).

- [ ] **Step 1: Write the failing tests** with a `FakeInput` that feeds a given array in 1024-sample blocks on `start` (synchronously) and records `stop` calls:
  - `test_a_take_is_recorded_into_the_store`: `start(); stop()` → take 1 of the current beat, chosen, text = the beat's text.
  - `test_an_abandoned_take_leaves_nothing`: `start(); abandon()` → no takes, no files under `raw/`; the next take is number 1.
  - `test_moving_between_beats`: `next()` twice, `prev()` once → index 1; `prev()` at 0 stays 0; `goto(999)` clamps to the last line.
  - `test_checks_flag_problems`: clipped audio → "clipped"; peak −30 dBFS → "too quiet"; a 0.5 s take against a 2.0 s narrator → "much shorter than the narrator"; silence → "no speech".
  - `test_the_strip_shows_each_beat`: one beat recorded, one stale (store a take, then load the video with that beat's text changed) → strip has "recorded", "stale", and "unrecorded" for the rest.
  - `test_room_tone_stops_by_itself`: `record_room()` with 12 s of fake input → `store.room()` exists and is 10 s (±0.05).
  - `test_state_is_json`: `json.dumps(session.state())` succeeds and includes the position string.

- [ ] **Step 2: Run** `series.tests.test_record` — Expected: ImportError.

- [ ] **Step 3: Implement.** Blocks accumulate in memory while recording; `stop()` concatenates and calls `store.add`; `abandon()` drops them. `DeviceInput` is not unit-tested (no microphone in tests).

- [ ] **Step 4: Run** — Expected: OK.

- [ ] **Step 5: Commit** — "Add the recording session behind the beat recorder."

### Task 6: The recorder page and command

**Files:**
- Modify: `tools/series/record.py` (server, `main`)
- Create: `tools/series/record.html`
- Modify: `tools/series/requirements.txt` (add `sounddevice==<latest at install>`, pinned)
- Test: `tools/series/tests/test_record.py`

**Interfaces:**
- Consumes: `Session` (Task 5).
- Produces:
  - `python -m series.record NAME [--device NAME] [--list-devices] [--port 8765]` — the device is remembered in `takes/recorder.toml`; opens the page with `webbrowser`.
  - `serve(session: Session, port: int) -> ThreadingHTTPServer` bound to 127.0.0.1: `GET /` (the page), `GET /state`, `POST /action` (`{"do": "start"|"stop"|"abandon"|"next"|"prev"|"goto"|"choose"|"room", "n": int}`; 400 on anything else), `GET /take/<id>/<n>.wav` (processed), `GET /narrator/<id>.wav`.
  - The page's keys, as the spec lists them: Space start/stop (stop plays the take back, processed, once); Enter or → next; ← previous; R another take; 1–9 choose; P play the chosen take; N play the narrator; Esc abandon; a "Play back after each take" checkbox (on by default, remembered in `localStorage`); and a "Record room tone" button. Polls `/state` about 15 times a second for the level meter.

- [ ] **Step 1: Write the failing tests** (a `Session` over `FakeInput`, `serve` on port 0 in a thread, `urllib`):
  - `test_the_server_drives_the_session`: POST start then stop → `/state` shows one take; GET `/take/<id>/1.wav` is a 48 kHz wav.
  - `test_the_server_refuses_unknown_actions`: `{"do": "delete"}` → 400, nothing changed.
  - `test_the_server_listens_only_locally`: `server.server_address[0] == "127.0.0.1"`.
  - `test_the_page_names_every_key`: `record.html` mentions Space, Enter, R, P, N, Esc.

- [ ] **Step 2: Run** — Expected: the new tests fail.

- [ ] **Step 3: Implement**, and install sounddevice into the venv, pinning the installed version in requirements.txt.

- [ ] **Step 4: Run** the whole suite — Expected: OK. Then by hand: `cd tools && series/.venv/bin/python -m series.record 001-one-neuron --list-devices` lists the Mac's inputs; with the built-in mic, record two takes of the first beat, step forward and back, choose take 1, play it and the narrator; the strip and the level meter move. Screenshot the page for Nate.

- [ ] **Step 5: Docs.** The bible's "How a video is made" gains a step between Sketches and Full render: **Narration:** `python -m series.record NNN-slug`; the release build (`--release`) refuses any beat still in the narrator's voice. Commit — "Add the beat recorder's page and command."
