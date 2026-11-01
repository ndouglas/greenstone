# How we work

*Through the Glass Cartridge* is made by me, Nathan Douglas, with a lot of
help from an AI: Claude, made by Anthropic. This page says who does what,
and which parts I've made sure I understand. Like the
[corrections lists](errata/), it's here so you can judge what you're
watching.

## Why I'm doing this

I'm making this series to learn machine learning and interpretability
properly, in public. The ideas are what I care about: they're what my
future work and research will rest on. So the parts of the project that
carry those ideas are the parts I hold myself to understanding. A lot of
the rest is machinery, and I'm happy to let the AI build it.

## What I make sure I understand

- **Every idea a video teaches**, well enough to explain it without the
  script.
- **Every claim, number and citation on screen.** If a video says a
  neuron is right 94% of the time, I know where that number comes from
  and how we checked it wasn't a fluke.
- **How each experiment is designed**: what it measures, what it's
  compared against, and what result would have shown we were wrong.
- **The scripts.** Claude drafts; I read, question and rewrite. Each
  script goes through rounds of review (you can read them in
  [`reviews/`](reviews/)), including me answering questions about it
  until the gaps show.
- **The narration** is my own voice.

## What the AI builds, and I mostly don't read

- **The animations.** They're written in Python with Manim, by Claude. I
  judge them by watching, not by reading the code.
- **The production tools**: the build that turns a script into a video,
  the voice recorder, the sound processing, the music.
- **Much of the emulator added in 2026.** Greenstone began in 2022 as my
  own hand-written NES emulator. A lot of what's been added since, to film
  games and run experiments, was written with Claude, and I've tested it
  more than I've read it.

## The line between them

It isn't a clean line, and I'd rather say so than pretend it is. The
code that runs an experiment sits on it: Claude often writes it, but its
results are checked against things that don't depend on that code being
right, such as the game's own memory, recordings the result wasn't chosen
on, and other games. When a result matters to what a video claims, I go
through the code until I understand why it gives the number it does.

## How claims are checked

- **The answer key.** On the NES, the game's memory says what's really
  happening on every frame, so most claims can be checked against it.
- **Held-out play.** A result is chosen on some recordings and tested on
  others.
- **Reviews.** Each script is read by several reviewers with different
  jobs (a cross-examiner, a skeptic, a first-time viewer, and others),
  and their findings are kept.
- **Citations.** Every idea taken from someone else is cited in the
  video's description and credits.
- **Corrections.** Mistakes found after release go on that video's
  corrections list.

Everything is public: the scripts, the experiments, the reviews and the
code are all in this repository.
