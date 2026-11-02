# Video 002: animating every beat

**Goal:** Every beat of 002 (draft 25, 90 beats) drawn from the real
experiment data, so Nate can watch the video and judge it, and the
interview can go on with pictures in front of him.

**Why:** Nate, 2026-11-02, after text-only interview questions confused
him: "The text is not sufficient for me to understand what is going on."
Then: "create the best animations/images/whatever for the current
script. If we think of a way to improve it, then we'll improve it."

**Spec:** the visual language
(`docs/superpowers/specs/2026-10-30-visual-language-design.md`) and each
beat's `board` in `tools/series/videos/002-every-mistake/script.toml`.
Where a board disagrees with draft 25's words (some were written for
earlier drafts), the words win and the board is updated.

## Global constraints

- Ember: near-black ground, warm = yes/above zero, cool = no/below,
  hairlines, Charter type, glow for continuous values, crisp ring and
  dot for answers. One picture per beat.
- The screen shows nothing that isn't true: every dot, border, score and
  surface comes from exp002's data and rules, computed the same way the
  numbers in the script were. Illustrative examples (`rule-wrong`'s
  moment at 4 and 1) are labelled as examples.
- Game footage only from the frame-exact smb1 footage take (RAM every
  frame), its lag applied.
- Up means up (memory `intuitive-directions`).
- Renders, takes and media stay out of git; ROMs never touched.

## Architecture

- **Film data** (`tools/series/lab/film002.py`): the trajectories the
  beats animate, computed from exp002's data and rules and cached to
  `target/series/002-every-mistake/film.npz`. The shown perceptron run
  nudge by nudge; the ten orders' borders and scores per pass; the
  control run; the pocket; the batch perceptron's and batch square
  trick's moves; the U over one weight; the points of the map with
  their counts of each answer; nudges per moment. Tested where it
  restates a number in `numbers.json` (it must agree).
- **Pieces** (in `tools/series/`, each with tests of what's measurable):
  - `pointmap.py`: Mario's map drawn by point, not by moment: each
    point a dot whose light grows with its count, warm for screen moved,
    cool for not, split warm/cool where both happened. A border set by
    (w, b) that can follow a trajectory; highlights; dots that fade.
  - `sheet.py`: the sheet of borders (angle across, wrapping; offset
    up) as a lit heightfield rendered to an image (numpy + PIL,
    painter's order, flat tops and vertical walls, so ledges read as
    ledges); camera tilt and a moving marker; the spot ↔ ruler link.
  - `chart.py`: line charts in Ember (score per pass, move sizes, the
    U, squared miss per move).
  - `knobs.py`: the formula's knobs (weights and bias) as dials whose
    needles turn, with a total bar.
- **Beats**: `tools/series/videos/002-every-mistake/beats/<section>.py`,
  wired by `scene.py`, as 001's are.

## Order of work

1. Film data, then the pieces, each tried as a sketch.
2. Introduction and chapter 1; render each (`series.build
   002-every-mistake --draft --only SECTION`), check stills against the
   board and the numbers, fix, then show Nate.
3. Chapters 2–5 and the conclusions, the same way.
4. A full render; Nate watches it; the interview resumes with pictures.
