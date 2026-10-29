# The series bible

The living document every video is written against. The design behind it
is `docs/superpowers/specs/2026-10-29-interpretability-series-design.md`;
this file is where the series' decisions accumulate as videos are made.

## Why look inside at all?

The series takes the strongest criticism of its subject seriously.
Hendrycks and Hiscott argue that mechanistic interpretability rests on a
clockwork assumption that complex systems don't satisfy, and that it has
delivered little (`sources.md`, section 4). Much of what they say is
right, and the series says so out loud: complex systems resist
reduction, single-neuron stories are often fragile, feature
visualizations mislead, and the field has been prone to hype.

The series' answer:

- **Understanding a complex system is worth attempting, at every
  level.** Weather, cells and minds are studied both top-down (patterns,
  statistics, the whole) and bottom-up (mechanisms, parts). Neither
  replaces the other; each checks the other. Systems thinking asks how
  the parts and the whole relate, not which one is "real".
- **Holistic and reductionist approaches belong together.** The series
  teaches both: neurons and circuits, and representations, probes and
  directions spread over many neurons. Representation engineering, the
  critics' preferred approach, gets its own video.
- **The NES is a system with ground truth at both levels.** A game has
  high-level variables (Mario's speed, the score) and the bytes and
  frames they're built from. That lets the series test claims the
  critique says are hard to test: does a neuron really track a
  variable, or only seem to on this data? Does a top-down direction
  capture it better?
- **Being honest about limits is part of the method.** Every pressure
  test asks where an idea breaks. The critique's arguments are the
  pressure tests' raw material, cited where they bite.
- **Learning the field is worth it even if the strongest hopes fail.**
  The viewer should be able to read an interpretability claim and judge
  it, which needs both the methods and their criticism.

The debate over what interpretability is *for* runs the same way.
Nanda's longlist of theories of impact (2022) and Charbel-Raphaël's
reply (2023) are taught as a pair: each theory, and the case against
it. The reply's own conclusions shape the series: its best theory of
impact is outreach and education, which is what this series is; it
exempts small models from its critique, which is all this series uses;
and it praises causal methods, probes, model organisms and adversarial
testing, which the toolkit teaches alongside circuits.

Where it lands in the videos:

- **001** says briefly why anyone would look inside a network, and that
  some researchers doubt it's the right approach, a doubt the series
  will take up properly.
- **Each pressure test** cites the critique's specific arguments where
  they apply (feature visualization in 009, fragile single-neuron
  stories in 008 and 010, the long tail in 002).
- **A dedicated video** (after the series has covered features,
  superposition and sparse autoencoders, so the viewer can weigh it)
  makes the critique's full case, with the disclosure that its lead
  author created the alternative it proposes. Then it gives the
  series' answer.
- **A representation-engineering video** teaches the top-down
  alternative on the NES.

## Interpretability as a toolkit

The series teaches interpretability as a whole, not one school of it.
Each approach is a tool with its own strengths and blind spots, and the
NES lets the series hold several of them up to the same answer key:

- **Behavioural:** treat the network as a black box; test what it does.
- **Attributional:** which inputs drove an answer (gradients, saliency).
- **Probing:** is a variable readable inside the network?
- **Mechanistic:** neurons, features and circuits, bottom-up.
- **Concept-based (representation engineering):** top-down directions
  read from contrasting examples, and used to steer.
- **Dictionary learning (sparse autoencoders):** pull apart features
  packed into too few neurons.
- **Causal intervention:** change one thing (an activation, or the game
  itself through a save state) and see what follows. Every reading is
  tested by pushing it, removing it and restoring it.

(After Bereska and Gavves, 2024, as summarized in MoSSAIC, and Zou et
al., 2023; see `sources.md`.)

## Failures and model organisms

"The model didn't work" is where an investigation starts, not where it
ends. Why did it fail, and what does that teach about learning? The
series builds failures on purpose, too: **model organisms**
(Hubinger et al., 2023), small networks and agents with a flaw we
planted or know is there, so each tool can be tested against a failure
we know exists.

Our models are too small to be deceptive. They can be reward hackers,
specification gamers, goal misgeneralizers (an agent that learned "run
right", not "reach the flag"), shortcut learners, exploiters of the
substrate they run on (an emulator inaccuracy a real NES wouldn't have),
probe evaders, and agents with planted hidden goals (auditing games).

Rule (Nate's): organisms are woven in from early on. A concept is
explored with an organism whenever it first can be, in a sensible order,
without crowding the video's one idea. 002's "cheating byte" is the
first: a shortcut the neuron takes when we let it.

Real failures we already have to dissect: the Punch-Out!! brains that
stand still, the bank brain that does nothing, and the cheat.

## Curriculum

The pilot arc. It will be revised as ideas are pressure-tested: an idea
that needs more room takes the next number, and everything after it moves
up. Numbers are plain whole numbers, never sub-numbered.

| # | The one idea | Games and moments (examples) |
|---|---|---|
| 001 | One neuron draws a line (w·x + b, slope and intercept; a balance scale) | "Is Mario in the air?" from his depth, in three games; "will he land soon?" from depth and falling speed |
| 002 | When the line lies: the assumptions a single neuron makes | ID bytes, BCD scores (Tetris), a wrapping frame counter, the "cheating" byte; place value (moved from 001): Mario's score digits read by weights of a million down to ten, and why Tetris's two-digits-per-byte breaks it |
| 003 | Learning is nudging the line after each mistake | The same neuron, learning |
| 004 | Some questions no single line can answer | Mario's speed says "in the air" only when it isn't zero, on either side: two cuts, 97% (from 001's experiments), which no one neuron can draw |
| 005 | Bends and layers: hidden neurons redraw the space | Solving 004's question |
| 006 | Learning is rolling downhill (loss and gradients) | The landscape of 005's network |
| 007 | A neuron's weights are a picture of what it wants | A neuron that sees screen pixels |
| 008 | Check a neuron's meaning against the answer key (probing) | Does a neuron track Mario's speed in RAM? |
| 009 | Play a neuron backwards: what excites it most | Dataset examples vs. synthesized inputs |
| 010 | One neuron, several meanings (polysemanticity) | A neuron firing for two unrelated things |

Beyond the pilot (direction, not commitment): superposition; causal
intervention by rewinding and editing the game; circuits; convolutional
networks; sparse dictionaries and sparse autoencoders; representation
engineering (the top-down alternative); the case against mechanistic
interpretability, and the series' answer; multiple realizability
(self-repair after ablation, probe evasion); substrates and Marr's
levels (the NES as a stack of them); a small transformer; Greenstone's
own reservoir brains. Model organisms are woven through all of it.

## Videos

### 001: One neuron draws a line

The script is `tools/series/videos/001-one-neuron/script.toml`; its
experiments and their results are in `docs/series/games/001.md`.

- **The one idea:** a single neuron multiplies each input by a weight,
  adds them and a bias, and says yes if the total is above zero. On a
  picture of its inputs, that's a straight border (school's y = mx + b is
  the same kind of line).
- **Prerequisites:** none. Algebra is introduced as it's needed.
- **Introduces:** input, weight, bias, w·x + b, the border, yes/no
  output; the balance scale; depth and falling speed as numbers in a
  game's memory; the answer key; testing on a session the setting wasn't
  chosen on.
- **Callbacks:** none (first video).
- **Games:** Super Mario Bros., Super Mario Bros. 3, Castlevania.
- **Chosen by search from outside, not learned** (pass 7, revised):
  every setting tried on recordings 1–2, the best kept and tested on 3;
  the landing border found the same way among many leaning borders.
  Learning (a neuron adjusting its own weights) is 003.
- **Experiments:** "is Mario in the air?" from his depth alone (a
  setting; 94% against 55%), in three games; "will he land within 10
  frames?" from depth and falling speed (a leaning border; 91% against
  75%), written in whole numbers as 1 × depth + 5 × falling speed − 152.
  A behavioural test and attribution (the balance-scale blocks) appear
  unnamed.
- **Said out loud:** we can see all of this neuron and know exactly what
  it does; networks of many are much harder to see into; scaling every
  weight and the bias by the same positive amount doesn't move the border
  (called back in 003, where weight size matters).
- **Organisms:** none (the first is 002's cheating byte).
- **Moved out:** place value (to 002); "speed isn't zero" and its two
  cuts (to 004); Castlevania's jump counter (to 002, encoding).
- **Plants for later:** the distance from the border (the modern neuron's
  graded output, paid off with sigmoid and ReLU).
- **Ideas from the passes not used in 001** (for later videos): shuffled
  inputs change nothing (convolutional networks); weights over pixels as
  a picture (007); the widest-gap border (margin).

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

## Core sources

`docs/series/sources.md` inventories every concept in the works Nate
chose as core: BlueDot's introduction, "Zoom In", and Neel Nanda on
80,000 Hours. The curriculum must take up each one; the inventory says
where.

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
- The field's critics are presented fairly, at full strength, where
  their arguments bite; the series never pretends a method works better
  than the evidence shows.
- Never: LLM clichés; "we'll see why later" (reorder or break it down
  instead); anything unsupported, incorrect or uncited; clunky animation;
  too much on screen at once; slop.

## How a video is made

1. **Ideonomy:** the idea gets 5 to 6 passes, saved in
   `docs/series/ideonomy/NNN-slug.md`.
2. **Script:** written in `tools/series/videos/NNN-slug/script.toml`, then
   revised through the council (`docs/series/council.md`), then the two
   adversarial reviews (`docs/series/review-prompt.md`). Then the
   interview: the council questions Nate about the concepts, the claims
   and how each line will be animated, and the script changes until he
   can answer everything (`council.md`, "The interview").
3. **Storyboard:** `python -m series.build NNN-slug --storyboard`, with
   placeholders, real narration, music and credits, to test pacing. Nate
   reviews.
4. **Sketches:** each animation explored alone with
   `python -m series.sketch`, judged from its contact sheet and iterated.
   Nate reviews.
5. **Full render:** `python -m series.build NNN-slug`. Nate reviews.
