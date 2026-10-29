# Inside the Machine: a video series on mechanistic interpretability, taught on the NES

(Working title; Nate's to change.)

## Intent

A series of short videos that teaches mechanistic interpretability from
the ground up, using Greenstone and NES games as the subject of every
experiment. Someone who watches the whole series in order should come out
able to take on an interpretability problem in a domain the series never
touches, such as image models. They would still need to learn that
domain's particulars (curve detectors, frequency detectors, dog-head
detectors), but they would have a total grasp of the foundations and of
the field's vocabulary and theories: features, monosemanticity,
polysemanticity, superposition, probing, causal intervention, circuits,
and the rest.

Why the NES: the game's state is fully known. Every frame, the 2KB of RAM
holds the ground truth that a network's internal features can be checked
against, and save states let us rewind and change one thing. Interpretability
work rarely has an answer key; here it always does. The NES is also
deterministic, so every result replays exactly.

The emotional promise: no one ever feels dumb watching these videos. They
come out feeling they can take on the world.

## The viewer

A middle-aged person in any line of work (blue-, pink- or white-collar)
who hasn't done algebra since high school and has never seen a line of
code. If the series works for them, it works for everyone else. Nate is
the first viewer: the pilot batch exists so he can follow every
explanation and animation and feel comfortable discussing all of it.

## How the series teaches

- **One narrow idea per video, and total clarity on it by the end.** Not
  a survey: one thing, made solid.
- **Every video builds on the earlier ones**, even if only to contrast
  ideas, topics and approaches. Watching in order works as spaced
  repetition: earlier ideas come back in new settings, are re-animated,
  and get stronger each time.
- **No warm-up video.** The series starts with the perceptron. Maths and
  code are introduced at the moment they're needed, then brought back and
  re-animated as often as possible. Examples: w·x + b arrives with the
  first perceptron, shown as the slope-intercept form of a line
  (y = mx + b) the viewer may half-remember from school; nonlinear
  activations arrive when a straight line stops being enough.
- **Code is shown in Python, purely as illustration** of the concept. It
  is never presented as Greenstone's actual code (which is Rust).
- **Animation (2D or 3D) beats pictures; pictures beat words.** Words go
  on screen only when a picture can't carry the idea.
- **Interpretability starts in video 1.** Every video both builds
  something and looks inside it.
- **Many games per idea.** Each idea is explained across several games
  (2, 3, 5, even 50), because games structure their data differently, and
  seeing a principle hold (or break) across different layouts is part of
  the lesson. A recurring side thread: how far one network can get across
  games that share structure, an engine or a publisher.
- **Every idea is pressure-tested.** After an idea is clear, the series
  turns on it: what it quietly assumes, what the alternatives are, what
  breaks it, and mad-science experiments that show where it holds and
  where it cracks. Each idea gets 5 to 6 ideonomy passes (negation,
  substitution, combination, abstraction-lift, cross-domain
  re-instantiation and the rest, via the `ideonomy-plain` skill) before
  its videos are written. A small idea keeps its pressure test in its own
  video; a large one spills into the next video(s).
- **Videos are numbered 1, 2, 3, … with no sub-numbering.** An idea that
  needs more room takes the next number.
- **An assumptions ledger.** Each idea's hidden assumptions are recorded
  (in the series bible, below) so later videos can call back to them
  ("remember when we assumed bigger byte means more?").

## Anti-goals (what would make Nate wince)

- LLM clichés in the scripts. (Nate will revise the scripts as needed,
  but drafts should not lean on him to remove them.)
- **Handwaving a concept away for later.** If an explanation needs
  "we'll see why later", something needs breaking down further or the
  order is wrong. Fix the order.
- Anything not absolutely supported or explained.
- Anything incorrect, or not responsibly cited. Every claim from the
  literature, and every borrowed technique, is cited in the video's
  description and credits.
- Clunky animation, or too much happening on screen at once.
- Slop. This is a slow, patient, thoughtful exploration of the subject.

## Production (decided)

- **Animation in Manim** (the Python library made for 3Blue1Brown's
  videos). Other tools may join where Manim isn't the right fit.
- **A synthesized narrator** for the pilot batch, with Nate's own voice
  kept possible: he doesn't like his voice or have a good recording
  setup yet, but knows his narration would be better for the audience and
  for him.
- **The script is the source of truth.** Narration is written as text,
  line by line. Each line is voiced, and the animation for that line
  lasts as long as its audio. Re-voicing is a swap: Nate records lines
  (some or all), the timing adjusts, and the video re-renders without
  re-editing. Plan for re-rendering the whole pilot batch after the swap.
- **The narrator runs on this machine** with a good free voice, not a
  paid service (builder's choice of engine).
- **Videos are short: about 5 to 12 minutes.**
- Game footage and any score come from Greenstone, composited as the
  reservoir films are (`tools/reservoir_film/`). The APU-rendered music
  approach carries over.
- **Greenstone grows as videos need it**: new mappers for new games, and
  new observability, display and debugging tools. Built when a video
  needs them, not ahead of time.
- **Any NES ROM is available**: official, unlicensed, hacks, homebrew.
  The builder researches (with web searches) which games best demonstrate
  each idea and asks Nate for them; Nate supplies the files into `roms2/`.
  When in doubt, pick a game people know over one they don't. The repo
  has 9 today (Super Mario Bros. 1–3, Tetris, Punch-Out!!, Castlevania,
  Arkanoid, Adventures of Lolo, Marble Madness). Existing mappers
  (0, 1, 2, 3, 4, 7, 9) cover most of the library; a game on another
  mapper means writing that mapper.
- The repo is public; the videos will be too. ROMs live in `roms2/`,
  which is gitignored, and are never committed.

## Review and correctness

- **Nate reviews very often for now**: the series bible, each script
  before it is animated, early animation passes, and each finished
  video. The right cadence will settle as the series finds its feet;
  start frequent.
- **Every line and fragment of every script is supported by a
  citation**, listed in the video's description and credits. A claim
  that can't be cited is either demonstrated by a reproducible
  experiment (and the experiment cited) or cut.
- **The script stands on its own.** It must be coherent by itself and
  carry enough to understand the topic without the video: imagine a
  blind audience member, someone cross-examining Nate on it in a
  courtroom, or an interviewer asking him about it. The video adds the
  richness a text script can't; it never carries meaning the script
  lacks.
- **A council of expert reviewers, iterated.** Each script is reviewed by
  several expert agents, each with one lens (clarity, legibility and
  coherence; pedagogy; skepticism; empiricism; and others as needed),
  and revised over a few rounds until their takes converge.
- **Two adversarial reviews per script.** Two independent reviewer
  agents, each told to find anything incorrect, unsupported, uncited,
  handwaved or out of order, read the script against its sources. A
  script passes when neither turns up anything substantial. There is no
  outside expert; these reviews stand in for one.
- **A disclaimer in every video**: Nate is not an expert, just a guy
  learning this stuff, and the series is him learning in public.

## How a video is made

1. The idea, from the bible's curriculum, gets 5–6 ideonomy passes.
2. The script, revised through the council's rounds, then the two
   adversarial reviews. Nate reviews.
3. **A storyboard render**: the whole video with placeholders where the
   animations will go, but real narration, music and credits, to test
   pacing. Nate reviews.
4. **Each animation explored on its own**, outside the video (no
   narration, music or context), rendered short and iterated until it
   works. Nate reviews them.
5. The full render. Nate reviews.

## The pilot batch

5 to 10 videos, made with the synthesized narrator, for Nate to judge the
structure, the explanations and the animations before anything is
re-voiced or published.

A draft arc (a starting point to revise as ideas are pressure-tested;
the ideonomy passes may split an idea across several videos, as they
already have for the first one):

| # | The one idea | Games and moments (examples) |
|---|---|---|
| 1 | One neuron draws a line (w·x + b, slope and intercept; a balance scale) | "Is Mario in the air?" from two numbers; across several games |
| 2 | When the line lies: the assumptions a single neuron makes | ID bytes, BCD scores (Tetris), a wrapping frame counter, the "cheating" byte |
| 3 | Learning is nudging the line after each mistake | The same neuron, learning |
| 4 | Some questions no single line can answer | A question that is not linearly separable |
| 5 | Bends and layers: hidden neurons redraw the space | Solving video 4's question |
| 6 | Learning is rolling downhill (loss and gradients) | The landscape of video 5's network |
| 7 | A neuron's weights are a picture of what it wants | A neuron that sees screen pixels |
| 8 | Check a neuron's meaning against the answer key (probing) | Does a neuron track Mario's speed in RAM? |
| 9 | Play a neuron backwards: what excites it most | Dataset examples vs. synthesized inputs |
| 10 | One neuron, several meanings (polysemanticity) | A neuron firing for two unrelated things |

Beyond the pilot (direction, not commitment): superposition, causal
intervention by rewinding and editing the game, circuits, convolutional
networks, sparse dictionaries / sparse autoencoders that pull features
apart, a small transformer, and finally Greenstone's own reservoir brains.

### Video 1 and 2's pressure test (already run)

Six ideonomy passes on "one neuron draws a line" produced:

- **The assumptions it adds:** a human picked the inputs; bigger byte
  means more; the evidence adds up; a hard cut is fine; one frame is
  enough.
- **Experiments:** the perceptron as a literal balance scale (weights are
  weights, the bias a counterweight); feeding it the game's own
  "airborne" flag (instant perfection, and cheating); a random byte; an ID
  byte (enemy type 7 isn't "more" than 6); scores as place value (weights
  1000/100/10/1) and where BCD breaks it; a wrapping frame counter that no
  line can split, fixed by sine and cosine; transplanting one game's
  weights onto another game's RAM.
- **Plants for later:** shuffling the inputs changes nothing (the neuron
  doesn't know which bytes or pixels are neighbours, paid off with
  convolutional networks); the distance from the line as confidence (paid
  off with sigmoid); the widest-gap line (margin); weights over pixels
  as a picture (video 7).
- **Notations to animate between:** algebra, school slope-intercept,
  Python, a spreadsheet formula, a balance scale, a circuit; and the
  0D → 1D → 2D → 3D opener (a point is a thermostat, a line, a sheet),
  with 2,048 inputs stated honestly as beyond drawing.

## The series bible

A living document in the repo that every video is written against. It
holds:

- the curriculum (the arc above, as revised);
- per video: the one idea, its prerequisites (which earlier videos it
  builds on), its callbacks, the games used, and its citations;
- the assumptions ledger;
- the visual language: one consistent look per concept (for example the
  same colours for weight, input and bias in every video), so the eye
  learns them;
- the ideonomy passes run for each idea.

## Calls made by the designer (Claude), reviewed by Nate

- **Experiments run in Python, on data recorded from Greenstone.**
  Greenstone (Rust) plays the games and records RAM, frames and inputs;
  the networks are trained and inspected in a Python harness. Greenstone's
  Rust is not where the series' networks live.
- **The code on screen is the working code, stripped down** (Nate's
  call): the real Python that produced the numbers, with irrelevant code
  and concepts removed (error handling, optional arguments and the like)
  and, where it helps, a note on screen that it is simplified to focus on
  what matters. Type hints stay, so viewers keep that context.
- **Gameplay is simulated or replayed, never played by Nate** (Nate's
  call: no controller hooks up cleanly here, and he'd rather not):
  scripted, random or simulated-human input policies per game, and
  existing input recordings (for example published input movies) replayed
  in Greenstone. Replaying a recording format may need building.
- **Depth wins over coverage.** The pilot is 5 to 10 videos wherever
  that lands in the arc. If pressure tests split ideas so the pilot ends
  before polysemanticity, that is fine; pressure tests are not thinned to
  reach a table row.
- **Re-voicing uses one recorded take per script line** (Nate agreed),
  read from the script, dropping straight into the line-timed pipeline.
  Any line can be re-recorded any number of times and replaced alone. Supporting one continuous take
  (forced alignment) is deferred until Nate actually records.

## Left to the builder

- The narrator engine and voice (local and free).
- How scripts, timing and re-voicing are represented in files.
- The visual language itself (colours, typography, recurring motifs),
  for Nate's review in the pilot.
- Where the series' tooling lives in the repo (following `tools/` as the
  reservoir films do), and where the series bible lives.
- The exact games per video, chosen from the ROMs available.
- Music: whether and how each video is scored.

## Order of work

1. This design, then the series bible.
2. The production pipeline: script → narrator → timed Manim animation →
   composited with Greenstone footage and score, with the re-voice swap
   working from day one.
3. Video 1, with whatever Greenstone features it needs. Then video 2, and
   on through the pilot batch.
