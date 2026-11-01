# The arc

Drafted by Claude (2026-10-31) for Nate to push against; Nate's answers
to its questions are below, and decide what changed. The six ideonomy
passes behind it are in `docs/series/ideonomy/000-series-arc.md`.

## Decided (Nate, 2026-10-31)

- **What a viewer should feel at the end:** "the overpowering urge to
  hire me for AI engineering work." The series is evidence of what Nate
  can do: rigorous, original, ambitious. That raises the bar on depth
  (already pushing away from ELI5) and on results nobody else has, which
  the packing strand and the mad-science experiments supply. The lay
  viewer stays the test for clarity; the person deciding whether to hire
  Nate is the test for substance.
- **It doesn't end.** It runs "for as long as we can think up
  important, interesting, provocative, bizarre, or cursed things to
  try." Seasons, with no finale; the finale candidates below become
  season-enders or episodes.
- **Safety is a thread**, not the destination.
- **Nate stays offstage** ("none, unless I become a whole lot sexier"):
  no personal journey on screen. The mirror reads *a player*, not Nate.
- **ml-mad-science merges into the series:** the series walks that
  course's spine in public (below).
- **How it's made is said plainly** in `docs/series/how-we-work.md`,
  linked from every video's description beside the corrections list:
  Nate's goal is to understand the ML and interpretability; much of the
  machinery (animation, tools, much of 2026's emulator code) is built by
  the AI, and the line between them is stated honestly as vague.

## Why this exists

The series has a curriculum (the pilot, 001 to 010), a list of topics
beyond it, one strand with a real question (packing), and, as of today, a
trailer. It has never had an arc: what the whole thing is *about*, as a
story about people and AI, and where it ends. The trailer problem (001's
introduction promising pictures from videos years away) came from writing
the series' promises before its shape. This draft proposes a shape.

## What it's about, in one sentence

Candidates, from the passes:

1. **A person learning to understand a made mind, by checking it against
   a world they understand completely, until the world runs out.**
2. Looking into the machine keeps showing us our own instructions.
3. From a neuron we built to a neuron we found.

The first is the spine; the second is the theme; the third is the
bookend. They fit together.

## The spine: the answer key runs out

Interpretability almost never has an answer key. The NES always does,
which is why the series is on the NES. But that's also the series'
biggest gap from the real thing, and the arc can turn the gap into the
story: **start with the answer key in hand, and lose it a piece at a
time**, so the viewer arrives where real researchers work, having
practised every tool somewhere they could check it.

001 already plants this. Its `why-key` beat says: we start with a question
the game can answer, "before we ask it things the game doesn't know, like
whether Mario's about to land."

The stages, as the passes found them:

1. **The key in hand.** Every claim checked against RAM.
2. **The key behind glass.** The network sees only the screen; the key
   still exists, but the network never touches it.
3. **Questions the key can't answer.** What happens next, what an agent
   is about to do, what it wants.
4. **No key.** A flaw nobody planted, in a network nobody fully
   understands. Only the habits are left.

## The theme: the mirror

Pass 6 found a hub nobody chose: **responsibility**. The cheating byte
is a shortcut we allowed. The Punch-Out!! brains that stand still
learned a reward we wrote. "It found an answer to the letter of its
goal" points back at whoever set the goal. Nearly every "what they
shouldn't do" in the series is a picture of what we asked for.

That's the people-and-AI theme, and it's honest rather than alarmist:
the machine is not a villain, and not innocent either, because what it
does is ours to understand. It also matches the "corrections welcome"
stance: a person who looks for their own mistakes on camera.

The mirror can be made literal. The empty cell in pass 2's who-reads-whom
grid is **a network reading a person**: from a recording of someone
playing, find which byte they were trying to raise (inverse
reinforcement learning, ml-mad-science's motivating negation, already
sketched small in brain-cartridge). The series spends its length reading
machines, then a machine reads a player.

## A proposed shape: seasons

The videos stay numbered 1, 2, 3, … as now; seasons are only a grouping,
and nothing below renumbers anything.

**Season 1: With the key** (the pilot, roughly 001 to 010).
One neuron, learning, when the line lies (the first model organism, the
cheating byte), questions no line can answer, layers, gradients, a
neuron's weights as a picture, probing, playing a neuron backwards, one
neuron with several meanings. Every claim is checked against RAM. The
viewer learns that we can check, and how. Ends on a mixed neuron: the
first thing the answer key can't explain away.

**Season 2: The fork** (the packing strand).
Superposition or generalization, tested: squeeze, grow, the structure
dial, rare events, one network across many games, pixels, architecture;
sparse dictionaries; first circuits. The series changes kind here: the
teacher doesn't know the answer, and says so. Lessons become a lab
notebook. Ends with what we found, either way.

**Season 3: Losing the key.**
Causal intervention by rewinding and editing the game; questions the
game can't answer; agents and what they want; model organisms with
hidden goals; an **auditing game** where Nate looks for a flaw he didn't
plant; representation engineering; the critique's full case (after the
viewer can weigh it, as the bible already says) and the series' answer.
The viewer learns what's left when the key is gone: the tools, the
controls, the habit of doubting your own microscope.

**Then more seasons, for as long as there are things worth trying.** Three
candidates for season-enders, from the passes:

- **The bookend:** a small transformer, and inside it one neuron that
  draws a line. Found, not built, and known only as far as our tools
  reach.
- **The mirror:** a network reads a player's play and says what they wanted.
- **The literal title:** a network that runs on the console itself, a
  6502 program beside the game it plays (brain-cartridge's thesis), read
  down to the instruction: every level from "what it computes" to "which
  bytes it touches" on screen at once.

The trailer then has a clear home: made at the end of season 1, and
remade each season.

## Where ml-mad-science fits (merged: Nate)

The series has been absorbing it already:

- The series' pressure tests are ml-mad-science's move (negate one
  definitional property, see what the sibling is), run with the same
  ideonomy-plain picker.
- brain-cartridge tags ideas `DEPOSIT` for ml-mad-science's grid, and
  its RAM-address inverse RL is that course's motivating negation made
  small.
- That course's nine stops (linear regression, logistic regression,
  k-NN and k-means, trees, MLP and backprop, CNNs, attention,
  autoencoders, RL) are nearly the series' foundation curriculum in the
  same order. The course stopped after stop 1 (2026-10-09).

Three ways to settle it:

1. **Merge:** the series is where the course's spine gets walked in
   public. Each foundation video's pressure test is that stop's
   expansion; the grid of ML becomes a recurring picture on screen;
   question-mark cells become mad-science episodes.
2. **A recurring segment:** keep the courses separate, but give each
   video a short "mad science" turn that borrows one negation.
3. **Separate:** the course stays a private notebook; the series cites
   it when a result travels.

Nate chose 1. How it lands without breaking "one narrow idea per video":
each foundation video is one stop, its pressure test is that stop's
expansion, and the grid grows on screen one row at a time, only as the
stops surface dimensions (the course's own rule). Question-mark cells are
candidates for the "bizarre or cursed" episodes between foundations. The
course's repository keeps its notebooks; the series is where its spine is
walked.

## What would change now if this were adopted

Very little, which is the point of deciding early:

- The bible gains the spine, the theme and the seasons; the curriculum
  keeps its numbers.
- 001 needs nothing new: `why-key` already plants the spine.
- 003's cheating byte gets one line pointing at the theme (a shortcut we
  allowed).
- The trailer is made at the end of season 1.
- One thing is worth starting early, because it takes time: the
  Punch-Out!! standing-still investigation (season 3's natural flaw).

## The questions, as asked

Answered above, under "Decided". Kept as asked:

1. **At the end, what should a viewer feel?** Hopeful, sober, urgent,
   capable? (The design's promise is "they can take on the world"; the
   spine above lands on "only the habits are left", which is capable,
   but sober.)
2. **Does the series end, or run in seasons for as long as it's
   alive?** A finale needs an ending; seasons only need a next question.
3. **How central is safety?** Model organisms as a thread through an
   interpretability course, or the destination the course is for?
4. **How much of you is in it?** Your move into AI safety could be a
   visible thread (the mirror finale depends on it), or stay offstage.
5. **ml-mad-science: merge, segment, or separate?**
6. **The series is made with an AI's help.** Say so on screen, in the
   description only, or not at all? Some viewers will care, and the
   theme (understanding what we made) makes it unusually relevant.
