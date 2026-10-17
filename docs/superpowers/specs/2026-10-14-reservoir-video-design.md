# Reservoir video design

> Superseded by `docs/superpowers/specs/2026-10-17-reservoir-film-design.md`.

A captioned MP4 that explains the Tetris reservoir experiment piece by
piece, then composes the pieces and runs the whole of SMB 1-1 in real time.
No narration audio; captions carry the words.

## Pipeline

1. `examples/reservoir_dump.rs` (Rust) loads the six cached demonstrations
   from `target/tetris-reservoir-demos/`, fits the raw, playfield and clock
   readouts on demonstrations 1 to 5, replays demonstration 0 recording
   every SMB frame, every Tetris frame per step, the window, the board and
   each readout's outputs per step, and records one closed-loop episode
   each for the raw and clock readouts. It writes a directory
   (`target/tetris-reservoir-dump/` by default):
   - `demo.smb.rgb`: 256x240 RGB24 frames, one per emulated frame.
   - `demo.tetris.rgb`: 256x240 RGB24 frames, one per step.
   - `demo.json`: steps `[{x, y, buttons, cutscene, window, board, enemies,
     outputs: {raw, playfield, clock}}]` with `window` and `board` as hex,
     plus `weights: {raw_a: [257], playfield_a: [2049]}` and demonstration
     metadata.
   - `episode-raw.smb.rgb`, `episode-raw.json`, `episode-clock.smb.rgb`,
     `episode-clock.json`: one closed-loop episode each, steps
     `[{x, buttons, dead}]`.
2. `tools/reservoir_video.py` (Python, Pillow, numpy) reads the dump,
   composes 1280x720 frames chapter by chapter, and pipes them to ffmpeg,
   which writes `tetris-reservoir.mp4` (H.264, 60 fps).

The library gains `src/nes/gym/reservoir/trace.rs`: the fit-on-several-
demonstrations helper, the replay that records frames and outputs, the
closed-loop trace, and pure helpers for reading enemy slots from the window.

## Chapters

1. The game is 2,048 numbers. Mario's screen beside his RAM as a 32x64 grid
   of cells lit by value, the window outlined, changed bytes flashing, a
   few bytes labelled.
2. A demonstration. The level strip (pits, enemies seen, flagpole) with
   the path playing at step rate and A ticks dropping onto the strip.
3. A readout. Window cells, weight bars, the A gauge with its 0.5 line,
   then the gauge running along the held-out demonstration beside the true
   A ticks.
4. A reservoir. The board drawn from RAM beside the window; the paste,
   four frames of Tetris, the 2,048 bytes as the new input. Note that only
   the first 160 window bytes fit the board.
5. The whole level in real time: SMB screen, Tetris board and screen, the
   strip with the position marker, three live A gauges (truth, raw,
   playfield). Then the coda: raw driving Mario and dying at the first
   Goomba; the clock readout replaying further.
6. The verdict: the table and three sentences.

## Captions

Each caption stays for at least three seconds, or longer when the chapter's
content runs longer. Text is drawn with a system font; no external assets.

## Out of scope

Narration audio, controller-driven condition, interactive playback.
