# The council

Before the adversarial reviews (`review-prompt.md`), each script goes to
a council of reviewers, one per lens. Each gets the script
(`tools/series/videos/NNN-slug/script.toml`), its citations, and the
bible (`docs/series/bible.md`), plus the shared preamble and its own
lens below. Each returns findings by line id, ranked, or "nothing
substantial".

## The procedure

1. Run every lens, independently and in parallel.
2. Revise the script against their findings. A finding not acted on gets
   a one-line reason, kept with the round's notes.
3. Repeat, up to three rounds, until no lens reports anything
   substantial.
4. Then the two adversarial reviews.
5. Then the interview (below): Nate's review of the script.

## The interview

Nate's review is not reading the script over. The council questions him,
lens by lens, to find problems while the script is still cheap to change.
It is not adversarial; it is how gaps surface at the script stage rather
than in animation, or after the video is live.

- Each lens asks Nate its own questions, a few at a time, and follows up
  on his answers. Examples:
  - *Pedagogy:* "In your own words, what is a sparse autoencoder?" "Why
    does the bias move the line but not tilt it?"
  - *The cross-examiner:* "Line `neuron` says a perceptron checks whether
    the total is above zero. Why zero?"
  - *The blind listener:* "Line `footage` shows the game. What would a
    listener who can't see it be missing?"
  - *Animation:* "How will we show w·x + b sliding the line? What's on
    screen, and what isn't?" "Which frames of which game prove this?"
  - *Skepticism and empiricism:* "What would make this claim false? Which
    experiment would show it?"
- Questions cover the concepts (so Nate can discuss everything in the
  video), the script's claims, and how each idea will be animated.
- An answer Nate can't give, or that needs the video to make sense, is a
  finding: the script (or the plan for its animation) changes, and the
  question is asked again.
- The interview ends when Nate can answer every lens's questions from the
  script and its sources, and the animation of every line has a plan he
  agrees with.

## Shared preamble

```
You are one reviewer on a council reviewing the script of an educational
video about mechanistic interpretability. The audience is a lay viewer
who hasn't done algebra since high school and has never seen code. The
script must stand on its own, without the video. Review only through
your lens, below. For each finding give the line id, what is wrong, and
what would fix it, ranked by importance. If you find nothing
substantial, say so; don't invent findings.
```

## The lenses

- **Clarity, legibility and coherence.** Is every sentence plain and
  easy to say aloud? Does each line follow from the last? Is there one
  idea, made clear by the end?
- **Pedagogy.** Is each new thing introduced when it's needed, with one
  new thing at a time? Are earlier ideas called back and reinforced? Is
  the one idea solid by the end? Would a viewer ever feel dumb?
- **Skepticism.** Which claims are stronger than the evidence? What
  would an expert in interpretability object to? What's oversimplified
  to the point of being wrong?
- **Empiricism.** Is every claim about the games or the networks
  demonstrated, or at least demonstrable, by an experiment we run? Are
  the experiments fair: controls, baselines, and the result honestly
  reported even when dull?
- **The lay viewer.** You are a middle-aged person who hasn't done
  algebra since high school and has never seen code. Where were you
  lost, bored, talked down to, or made to feel dumb?
- **The blind listener.** You hear the narration but can't see the
  screen. Where does the script lean on a picture it doesn't describe
  ("as you can see", "this one here")?
- **The cross-examiner.** You question the narrator about the script, as
  in a courtroom or a job interview. Which statements couldn't they
  defend from the script and its sources alone?
- **Continuity and pacing.** You read the script as sections of beats,
  with the storyboard's section lengths beside it
  (`docs/series/reviews/NNN/pacing.md`). Does each beat carry one thought
  and one picture, short enough to take in? Does the stillness after a
  beat do its job (none wasted after a bridge, none missing after a dense
  beat; a beat can set its own `pause`)? Does each chapter's introduction
  say what problem it solves, and does its conclusion zoom out: from the
  last chapter's X to this one's Y, why it mattered, what's next? Does
  the Introduction set the video up and the Conclusions TL;DR it? Is
  every section under four and a half minutes? Does anything arrive before it's set
  up, and does each beat lead to the next?

## The picture council

The council above reads the script. The pictures get their own council
(Nate, 2026-11-02: animations had been judged only by whoever built
them, and too many were cheap: a clock for "slow", a blank sheet, a
static view of a 3D surface).

It runs when beats are first designed, and again on any beat Nate
flags after watching a render. Each lens gets the beat's line and its
neighbours, its board, stills of the current render (if any), Nate's
notes on it, the visual language
(`docs/superpowers/specs/2026-10-30-visual-language-design.md`), and
what data and pieces exist. Each proposes designs for every beat it's
given, ranked, or says the current one works. The builder merges the
proposals into one or two candidates per beat, prototypes the
strongest, and Nate chooses by watching, not by reading.

Shared preamble:

```
You are one designer on a council designing the animation for beats of
an educational video about how neural networks learn, worked out on NES
games. The audience is a lay viewer who hasn't done algebra since high
school. Each beat is one line of narration and one picture. The rule:
the screen shows nothing that isn't true; every dot, line, score and
surface comes from the real data. Design only through your lens, below.
For each beat give: what's on screen and how it moves, second by second
against the narration; why it teaches the line better than what's there;
what data it needs; and how hard it is to build. Be concrete enough to
build from. If the current picture already works through your lens, say
so; don't invent changes.
```

The lenses:

- **The idea.** What is the one thing the line says, and what picture
  would make a viewer understand it without the narration? Does the
  picture show the idea itself, or only a symbol of it (a clock for
  "slow", a counter for "many")?
- **Scale and intuition.** Where the line is about size (how many, how
  fast, how far), does the picture make the size felt, by comparison
  with something the viewer already knows, rather than just stated?
- **Space and motion.** Is the picture a scene the camera explores, or a
  diagram held still? For a surface or space: does the viewer get
  enough angles, depth and movement to know its shape? Does every
  motion mean something?
- **Honesty.** Does every mark on screen come from the data, at its true
  scale? Could the picture mislead (a stretched axis, a smoothed
  staircase, a made-up example not labelled as one)?
- **Legibility.** Can a viewer read it on a phone: contrast, size,
  colour (warm is yes and above zero, cool is no and below), clutter?
  Is the eye led to the one thing that matters at each moment?
- **Continuity.** Does it grow out of the last beat's picture and into
  the next's, with a shared vocabulary (the same map, the same sheet,
  the same knobs), and transitions that carry the eye rather than
  cutting?
