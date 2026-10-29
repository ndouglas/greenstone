# The series bible

The living document every video is written against. The design behind it
is `docs/superpowers/specs/2026-10-29-interpretability-series-design.md`;
this file is where the series' decisions accumulate as videos are made.

## Curriculum

The pilot arc. It will be revised as ideas are pressure-tested: an idea
that needs more room takes the next number, and everything after it moves
up. Numbers are plain whole numbers, never sub-numbered.

| # | The one idea | Games and moments (examples) |
|---|---|---|
| 001 | One neuron draws a line (w·x + b, slope and intercept; a balance scale) | "Is Mario in the air?" from two numbers; across several games |
| 002 | When the line lies: the assumptions a single neuron makes | ID bytes, BCD scores (Tetris), a wrapping frame counter, the "cheating" byte |
| 003 | Learning is nudging the line after each mistake | The same neuron, learning |
| 004 | Some questions no single line can answer | A question that is not linearly separable |
| 005 | Bends and layers: hidden neurons redraw the space | Solving 004's question |
| 006 | Learning is rolling downhill (loss and gradients) | The landscape of 005's network |
| 007 | A neuron's weights are a picture of what it wants | A neuron that sees screen pixels |
| 008 | Check a neuron's meaning against the answer key (probing) | Does a neuron track Mario's speed in RAM? |
| 009 | Play a neuron backwards: what excites it most | Dataset examples vs. synthesized inputs |
| 010 | One neuron, several meanings (polysemanticity) | A neuron firing for two unrelated things |

Beyond the pilot (direction, not commitment): superposition; causal
intervention by rewinding and editing the game; circuits; convolutional
networks; sparse dictionaries and sparse autoencoders; a small
transformer; Greenstone's own reservoir brains.

## Videos

### 001: One neuron draws a line

- **The one idea:** a single neuron multiplies each input by a weight,
  adds them and a bias, and says yes or no by which side of zero the
  total lands. With two inputs, that is a line across a picture of all
  possible inputs (w·x + b, which is school's y = mx + b in disguise).
- **Prerequisites:** none. Algebra and code are introduced here as they're
  needed.
- **Introduces:** input, weight, bias, w·x + b, the decision line, yes/no
  output; the perceptron as a balance scale; place value as a neuron.
- **Callbacks:** none (first video).
- **Games:** Super Mario Bros. for "is Mario in the air?"; at least two
  more games asking a question of the same shape (to research).
- **Plants for later:** the distance from the line as confidence (paid
  off with sigmoid); shuffled inputs change nothing (paid off with
  convolutional networks); weights over pixels as a picture (007); the
  widest-gap line (margin).
- **Citations to find:** Rosenblatt's perceptron (1958); McCulloch and
  Pitts (1943); a source on the NES's 2KB RAM and the games' RAM maps
  (for every address used).

### 002: When the line lies

- **The one idea:** a single neuron quietly assumes things about its
  inputs, and when the game's data breaks those assumptions, the line
  fails, in ways that teach how data is shaped.
- **Prerequisites:** 001.
- **Introduces:** the first five entries of the assumptions ledger below;
  encoding (the same fact stored as a magnitude, an ID, BCD digits or a
  wrapping counter); sine and cosine as a way to unroll a circle.
- **Callbacks:** 001's line, weights, bias and balance scale; place value.
- **Games:** Tetris (BCD score), Super Mario Bros. (frame counter, enemy
  IDs), others to research.
- **Experiments:** feed it the game's own "airborne" flag (instant
  perfection, and cheating); a random byte; an ID byte; BCD scores across
  games; a wrapping frame counter, fixed by sine and cosine; one game's
  weights transplanted onto another game's RAM.
- **Citations to find:** BCD on the 6502/NES; the games' RAM maps for
  every address used; a source for encoding cyclic features with sine and
  cosine.

## Assumptions ledger

Each assumption a new idea quietly adds, tagged with the video that names
it, so later videos can call back to it.

| Assumption | Named in | Broken by |
|---|---|---|
| A human picked the inputs | 002 | Learned features (the series' spine) |
| A bigger byte means "more" | 002 | ID bytes, bit flags, BCD digits |
| The evidence adds up | 002 | Effects that only work together (→ 004) |
| A hard cut is fine | 002 | Inputs near the line (→ sigmoid) |
| One frame is enough | 002 | Questions about motion ("rising or falling?") |

## Visual language

One consistent look per concept, so the eye learns it. The palette and
fonts live in `tools/series/style.py`, for Nate's review in the pilot:
input, weight, bias and output each keep their colour in every video.

## Rules

- One narrow idea per video, with total clarity on it by the end.
- Every video builds on the earlier ones, even if only to contrast.
- No warm-up video; maths and code arrive when needed and come back often.
- Code on screen is the working Python, stripped of what's irrelevant
  (error handling, optional arguments), with type hints kept and a note
  when it's simplified.
- Animation beats pictures; pictures beat words.
- Interpretability starts in 001: every video builds something and looks
  inside it.
- Many games per idea; prefer games people know.
- Every idea gets 5 to 6 ideonomy passes, recorded in `docs/series/ideonomy/`.
- The script stands on its own: a blind listener, a cross-examiner or an
  interviewer could follow it without the video.
- Every line is cited, demonstrated by a reproducible experiment, or
  marked `framing` (no factual claim); otherwise it's cut.
- A disclaimer in every video: not an expert, learning in public.
- Never: LLM clichés; "we'll see why later" (reorder or break it down
  instead); anything unsupported, incorrect or uncited; clunky animation;
  too much on screen at once; slop.

## How a video is made

1. **Ideonomy:** the idea gets 5 to 6 passes, saved in
   `docs/series/ideonomy/NNN-slug.md`.
2. **Script:** written in `tools/series/videos/NNN-slug/script.toml`, then
   revised through the council (`docs/series/council.md`), then the two
   adversarial reviews (`docs/series/review-prompt.md`). Nate reviews.
3. **Storyboard:** `python -m series.build NNN-slug --storyboard`, with
   placeholders, real narration, music and credits, to test pacing. Nate
   reviews.
4. **Sketches:** each animation explored alone with
   `python -m series.sketch`, judged from its contact sheet and iterated.
   Nate reviews.
5. **Full render:** `python -m series.build NNN-slug`. Nate reviews.
