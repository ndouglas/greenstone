# The beat recorder: Nate narrates the series himself

## Why

The series is drafted with Kokoro's voice, and released in Nate's. Nate
wants to record a video's 130-odd beats quickly, with as many takes as
it needs ("I suck at this"), and have the result sound reasonably good:
compressed, filtered, even in level from beat to beat. The pipeline
already uses a recorded take wherever one exists and the narrator
elsewhere (`tools/series/voice.py`: `takes/<id>.wav` plus `takes.toml`
naming the text each take was read from, with stale takes reported). What
is missing is a fast way to make takes, and the polish on them.

## Goals

- One beat at a time, in script order, a keypress to start and stop a
  take; a botched take is redone at once with no dialogs or file names.
- Every take is kept; the newest is chosen, and any earlier one can be
  chosen instead.
- Takes are recorded raw. The sound (filtering, compression, level) is a
  fixed chain applied when the video is built, so it can be retuned
  without recording again.
- At a glance: which beats have no take, and which are stale because the
  script's text changed since.
- Problems caught at once: clipping, too quiet, and a take much longer
  or shorter than the narrator's reading.

## Non-goals

- Recording a chapter straight through and splitting it by speech
  recognition ("last take wins"). Maybe later, if stopping per beat is
  slow.
- Editing within a take (cutting a cough out of the middle). Redo it.
- Releasing a video in the narrator's voice. Every released video is in
  Nate's voice, every beat a current take (Nate, 2026-10-30); Kokoro is
  for storyboards and drafts. The recorder makes the gaps visible, the
  release build enforces it (below).
- Backing up the takes: they live only on Nate's machine, "more trouble
  than it's worth" (Nate, 2026-10-30).

## Anti-goals

- Anything that alters the raw recording (browser echo cancellation,
  noise suppression or automatic gain; OS "voice isolation"). Raw stays
  raw.
- Takes in git: the repo is public and the takes are large; they are
  already ignored (`/tools/series/videos/*/takes/`).

## The hardware and the room (Nate, 2026-10-30)

A FIFINE K669B, a USB cardioid condenser (Nate is buying it). His
office is small and quiet; the walk-in closet is the fallback. A
condenser hears the room more than a dynamic mic would, so the first
session starts with a test: a few beats recorded in each place, judged
by ear after the chain. The recorder doesn't assume a device: it lists
the inputs and remembers the one chosen.

## How it works (decided)

**Shape.** `python -m series.record NNN-slug` starts a small server on
localhost and opens a page in the browser. Python records (through
PortAudio, `sounddevice`) so the audio never passes through the
browser's voice processing; the page is the display and the keyboard.
Nothing leaves the machine.

**The page.** The current beat's text, large; the beat before and after
it dimmed; the chapter's title and the beat's place (e.g. "Chapter 3 ·
beat 7 of 15"); its board in small type, as a reminder of what's on
screen. A live level meter. The beat's takes as a row, the chosen one
marked, each with its length beside the narrator's. A strip along the
bottom: every beat in the video as a tick, coloured unrecorded / recorded
/ stale, clickable.

**Keys.**
- Space: start a take; space again: stop it. The new take is chosen and
  played back once, processed, unless playback is off.
- Enter or →: next beat. ←: previous beat.
- R: redo, i.e. start another take of this beat.
- 1–9: choose that take. P: play the chosen take (processed). N: play
  the narrator's reading of the beat, as a pacing guide.
- Esc: abandon a take in progress (it isn't kept).

**Files.** Raw takes at `takes/raw/<id>/<n>.wav`, numbered from 1,
never overwritten, 48 kHz, 24-bit, mono. `takes.toml` gains a table
form per beat: `[<id>]` with `text` (what was on screen when the chosen
take was recorded) and `take` (its number). The existing form,
`<id> = "<text>"` with `takes/<id>.wav`, keeps working. Each raw take
also records the text it was read from (in a sidecar or in
`takes.toml`), so choosing an older take after the script changes is
judged stale against its own text.

**Trimming.** Silence is trimmed from both ends of each take on record,
non-destructively: the raw file is kept whole and the trim points are
stored, with a short pad (about 0.15 s) kept at each end so breaths and
consonants aren't clipped.

**Checks, shown on the take.** Clipping (any sample at full scale);
too quiet (peak below about −20 dBFS); length more than 50% off the
narrator's reading of the same beat (a hint, not an error).

**The sound, at build time.** `voice.py` processes the chosen take with
one ffmpeg filter chain and caches the result like the narrator's
voices, the chain's version in the cache key:
1. trim to the stored points;
2. high-pass at about 80 Hz;
3. light noise reduction;
4. a de-esser;
5. compression, about 3:1;
6. a small presence lift, around 3–5 kHz;
7. loudness matched for every take (about −16 LUFS);
8. a limiter at −1 dB.

Starting values; Nate tunes them by ear on the first session. The
recorder plays takes through the same chain, so what he hears is what
the video gets.

**Room tone.** The recorder also takes one "room" recording per video,
10 seconds of the room's own quiet. The build loops it as a bed under
the whole narration track, from the first title screen to the credits,
so the cut between beats doesn't drop to digital silence. It goes
through the same chain as the takes, except the loudness matching:
instead it gets the median of the gains the takes received, so it sits
at the takes' own noise floor. It plays only when every beat is a take
(never under Kokoro), and a video with no room recording builds as now.
(My call, after the builder check; a gaps-only bed would need the scene
to report every pause's timing, for a difference under the music bed
that's hard to hear.)

**Release.** The full build (not the storyboard) reports how many beats
are narrated by Kokoro. A `--release` build refuses unless every beat
has a current take, naming the beats that don't.

## Left to the builder

- The server's plumbing (stdlib `http.server` or another small library),
  how the page gets the level meter (polling or a socket), and the page's
  look, kept plain and readable at a glance.
- How trim points and per-take text are stored, within `takes/`.
- The exact ffmpeg filters and parameters for the chain, within the
  list above.
- Tests: the take store, trimming, stale detection, the checks, the
  chain's cache, and the release refusal are testable without a
  microphone (fake input device or recorded fixtures); the page is
  checked by hand.

## Process

Small enough to build and test without the series' council: it changes
no video's content. Nate tries it in the first session, and its
starting sound values change from what he hears.
