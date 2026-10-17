# Reservoir film design

A 10 to 12 minute chaptered video, *Can Tetris play Mario?*, that explains
the Tetris reservoir experiment as it actually went: the hope, the bugs,
the three ways the protocol fooled us, the bank of boards, the level it
never saw, and the verdict. It replaces the 2026-10-14 explainer
(`docs/superpowers/specs/2026-10-14-reservoir-video-design.md`), which
stops before the rule teacher and ends on a verdict the later runs
reversed.

The audience is the user and a few friends: curious, not specialists.
Success is that a viewer can say afterwards what a reservoir is, why
Tetris was a reasonable thing to try, what went wrong along the way, and
what the answer turned out to be, and that they enjoyed it.

## Principles

Borrowed from Flump Studio's planning rules (`../SugarScape/studio/README.md`):

- One idea per moment. One sentence per caption, and one visual question
  per shot.
- Show, then say. The rhythm of a shot is: what you are about to see,
  watch it, what it meant.
- No term on screen before it is explained. "Readout", "reservoir",
  "onset", "held out" each get a sentence before they are used.
- Text cards stay up long enough to read twice: at least 4 seconds per
  15 words, never under 3 seconds.
- After every experiment, a few seconds of music and action with no text.
- Honesty stays on screen when it changes the meaning (15 onsets cannot
  separate the leaders; we poked the lives counter rather than doing the
  staircase trick). Methods and sample sizes live in
  `docs/ideas/brain-cartridge.md`, not in captions.
- Experiments are real. Every intervention, buggy build and closed-loop
  run shown is recorded from the emulator, not animated.

## Story

Each chapter opens on a title card, which is also its chapter marker.
Durations are targets, not quotas.

0. **Cold open (about 30 s).** 1-1 plays, four Tetris boards churn
   beside it. Title. Then the "MARIO × N" screen with the lives counter
   climbing: 9, then a crown, then letters, then the game's own tiles,
   question blocks and brick fragments, as the ones digit. Caption: we
   poked the counter rather than bouncing a shell on the 3-1 staircase.
   The garbage digits start to fall: Mario's RAM as rain, drawn in SMB's
   own tiles.
1. **Mario is just numbers (about 90 s).** The rain slows and resolves:
   bytes light up and get labels as Mario moves (x, on-ground/jumping/
   falling, enemy slots, the terrain ahead). "After a while you don't see
   the code. You see Goomba, pipe, pit." Experiments: move a Goomba by
   writing its x; write the terrain buffer so Mario stands on an
   invisible block or falls through solid-looking ground (the numbers are
   the world, the picture is a drawing of them); spray noise into a few
   bytes. Then the brain's view: the game beside its 360-byte window, the
   game fading out until only the grid plays.
2. **The simplest brain (about 50 s).** One weight per byte, add them up,
   jump if over one half. The gauge against real jumps. Experiment: it
   drives, and dies at the first Goomba.
3. **Pour Mario into Tetris (about 90 s).** Reservoir card: people have
   computed with buckets of water; a messy system scrambles the input so
   a simple brain can read it. Animated ripple tank. First version: paste
   bytes onto the board, junk, nothing for Tetris to do, and "the screen
   lies" (Tetris's screen against the board in RAM). Second version: four
   tetromino placements. Experiment, the echo: two boards fed the same
   pieces but one, their difference in red, fading over about twenty
   pieces.
4. **The teacher was flipping coins (about 60 s).** Six search-made
   examples overlaid at the same moment, jumping at different times.
   "Copy that, if you can." The rule teacher as three tripwires (enemy
   close, pit ahead, wall ahead) lighting as Mario plays. The honest
   score: did it start the jump when the teacher did.
5. **Three ways we fooled ourselves (about 100 s).** Each is a real buggy
   build, told as cheat, symptom, fix, with a lab-note card:
   - The controller leak: a near-perfect score; it plays and goes
     nowhere; it was reading back the button it had just pressed.
   - The clock: knowing only the time, it plays well; start Mario two
     seconds late and it jumps into nothing.
   - The memoriser: the search's perfect 1.000, death at x 316, the
     features it chose highlighted, Mario's position in disguise.
6. **Four Tetris games (about 100 s).** The window's four regions, colour
   coded, each flowing into its own board. The tidy piece as a little
   housekeeper. Experiment: the bank drives 1-1 in real time, boards
   beside Mario, past the Goomba pair (best run x 2470). Experiment: noise
   on the bank's inputs and what it costs.
7. **A level it never saw (about 60 s).** 4-1 plays. A bar chart of jumps
   started builds one bar at a time; error bars arrive last with the
   caveat. The clock jumping its way far by luck, as a joke.
8. **So what is Tetris doing? (about 60 s).** Theory 1, stack heights hold
   the memory: shown, fails. Theory 2, it just remembers "I was jumping":
   one bit, fails. The echo again as the answer: the brain mostly reads
   where this step's pieces landed.
9. **Verdict and credits (about 40 s).** One sentence: Tetris learns to
   make Mario jump about as well as an echo state network, and is not a
   memory. The three experiments for more hardware. Credits, including
   that the music is played on Greenstone's own APU and the pulse-channel
   bug the score found.

## Architecture

### Takes (Rust, `examples/reservoir_film.rs`)

Each on-screen experiment is a take: one function that runs the
emulators with its intervention and writes `target/reservoir-film/<take>/`:
frames as raw RGB24 (`*.rgb`, 256x240), a `take.json` of per-step records
(x, buttons, window, boards, readout outputs, pokes) and any numbers the
take measures. `--take NAME` re-records one take; with no argument every
missing take is recorded. Takes reuse the library: `trace.rs` replay and
episode functions, `demo.rs` start states and windows, the bank and
search code.

Takes: `lives_glyphs`, `matrix_walk`, `poke_enemy`, `poke_terrain`,
`noise_bytes`, `simple_brain_drives`, `paste_board`, `placements`,
`echo_twins`, `coin_flip_teachers`, `rule_tripwires`, `controller_leak`,
`clock_late_start`, `memoriser`, `bank_drive`, `bank_noise`, `unseen_4_1`,
`heights`, `hold_bit`, `cold_open_bank`.

`lives_glyphs` sweeps $075A from 0 to 255 on the intermission screen and
saves the ones-digit tile for each value; the compositor builds the rain
from these. Observed on 2026-10-17: at 10 or more the tens digit is a
crown and the ones digit is tile (lives + 1 - 10), which walks the
letters and then the background tiles; 255 wraps to "0".

The buggy builds are real code paths behind explicit options, off by
default and used only by the film:

- `smb_window` gains a variant that leaves the controller bytes
  (`SMB_INPUT_BYTES`) in place.
- The encoder search gains a pool option that admits Mario's x bytes
  (`POSITION_BYTES`).
- The clock's late start is an ordinary episode at an offset outside the
  trained range.

Numbers that need a long run (the run 38 table behind chapters 7 and 8)
come from `tools/reservoir_film/numbers.json`, checked in, each entry
citing its run and commit. Numbers a take can measure in seconds it
measures itself.

### Compositor (Python, `tools/reservoir_film/`)

Replaces `tools/reservoir_video.py` with a package:

- `draw.py`: shared widgets. Byte grid, Tetris board from RAM, A gauge,
  level strip, region colours, bar chart with error bars, title and text
  cards, captions, the rain in SMB tiles, the ripple tank.
- `chapters/c00_cold_open.py` to `c09_verdict.py`: each lists its shots
  with their takes, captions and timings, and draws its frames.
- `build.py`: renders each chapter to a cached segment under
  `target/reservoir-film/segments/`, joins them with ffmpeg, lays in the
  music, and embeds chapter markers (ffmetadata). Writes
  `chapters.txt` in YouTube timestamp form. `--chapter N` renders one
  chapter; `--quick` renders at a tenth of the length for layout checks.
- 1280x720, 60 fps, so NES frames map one to one. H.264 and AAC.

`examples/reservoir_dump.rs` and `tools/reservoir_video.py` are deleted
once the film covers their ground.

### Music

Played on Greenstone's APU.

- **Tunes** are written in Python (`tools/reservoir_film/music/`), as
  Flump Studio writes its tunes: notes per channel, sections, an intro,
  a loop and an ending per cue. **Instruments** are FamiTracker-style
  per-frame macros: volume envelope, duty sequence, delayed vibrato,
  arpeggio, pitch slide.
- **Channels:** pulse 1 melody, pulse 2 harmony or arpeggio, triangle bass
  and kick (by a fast pitch drop), noise snare and hats. Mostly 25% and
  50% duty at moderate volume; 12.5% only as colour.
- **The compiler** (`music/compile.py`) turns a cue into a register log,
  one line per write: frame, address, value. Note periods use the NTSC
  formulas (pulse `1789773 / (16 f) - 1`, triangle `1789773 / (32 f) - 1`).
- **The renderer** (`examples/apu_render.rs`) reads a log, writes each
  frame's registers into `APU`, ticks 29,781 CPU cycles per frame, and
  writes 44.1 kHz 16-bit WAV.
- **Fitting:** the build reads each chapter's length, repeats the cue's
  loop so it ends with the chapter, and crossfades at chapter joins.
  Stings are compiled separately and mixed at the frame of their event.
  The final mix is normalised with ffmpeg `loudnorm` to -16 LUFS.

Cues:

| Cue | Character | Where |
|---|---|---|
| *Coin Run* (original) | bouncy, syncopated, calypso-ish C major | cold open, chapter 6's drive |
| *Underground Numbers* (original) | low, sparse, minor arpeggios | chapter 1 |
| *The Lab* (original) | plinky, curious, mid tempo | chapters 2 and 4 |
| *Korobeiniki* (traditional) | straight, then a fast triumphant variation | chapters 3 and 6 |
| *Fooled Again* (original) | sneaky staccato minor | chapter 5 |
| Bach, Minuet from French Suite No. 3 | thoughtful | chapters 7 and 8 |
| *Coin Run* reprise | slow, triangle and one pulse | chapter 9 |

Stings: a 1-up blip per poked life, a sweep-unit "wah-wah-wahhh" when a
buggy build fails, a fanfare when the bank passes the Goomba pair, a
death jingle when the simple brain dies. All originals.

Melodies are original except Korobeiniki and Bach, both public domain.
Nothing is taken from Koji Kondo's or Hirokazu Tanaka's scores; the
Mario-flavoured cues borrow a style, not a tune.

### APU fix

A probe on 2026-10-17 found pulse 1 silent with sweep register $08
(negate on, shift 0, the usual way to turn sweep off): the target period
is computed as `period - period - 1` with `wrapping_sub`, which wraps to
0xFFFF and trips the "target > $7FF" mute. On hardware a negated target
cannot overflow that way. Pulse 2 and pulse 1 with $00 play. The fix
mutes on target > $7FF only when not negating, with a test that fails
first. It is mentioned in the credits.

## Testing

Rust:

- APU: pulse 1 with sweep $08 is audible (fails before the fix); a
  negated sweep still mutes when the period is under 8.
- `apu_render`: a log holding A440 on pulse 1, A220 on the triangle and
  noise renders with spectral peaks at 440 and 220 Hz (within 1 Hz) and
  non-silent noise; the WAV length matches the log's frames.
- The leaky window keeps the controller bytes and the default hides them;
  the permissive pool contains $6D and $86 and the default does not.
- Each take (ignored, needs the ROMs) writes its `take.json` and a
  non-empty frame file.

Python (`unittest`, no new dependencies beyond Pillow and numpy):

- Note to period: A4 on pulse gives 253, A3 on triangle gives 253.
- Instrument macros expand to the expected per-frame writes.
- Fitting: a cue fitted to a chapter ends within one frame of it.
- Caption timing: every caption meets the reading-time rule.
- Chapter markers match the segment boundaries.

Verification before calling it done:

- A contact sheet per chapter (a frame every two seconds), read and
  checked for layout, overlap and legibility.
- Each cue rendered alone and its spectrogram checked for clipping and
  silence where there should be sound; the final mix's loudness
  measured.
- The user watches the full video at normal speed. Pacing and whether it
  is funny are their call, not a test's.

## Out of scope

Narration audio, the DMC channel (unimplemented in the APU), new
experiments beyond those listed, publishing the video anywhere.

## Risks

- Takes depend on the cached demonstrations
  (`target/tetris-reservoir-demos-*`); a missing cache means a long
  re-search. The film's takes name the caches they read.
- Render time: at 60 fps, 12 minutes is about 43,000 frames. Per-chapter
  segments keep iteration cheap.
- The APU may have further bugs the score exposes; each gets the same
  treatment as the sweep: a probe, a failing test, a fix.
