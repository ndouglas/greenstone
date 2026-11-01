# Ideonomy: the arc of the whole series

Six ideonomy-plain passes (2026-10-31) on the series as one thing: *Through
the Glass Cartridge*, taught on NES games where the RAM is an answer key,
read as a story about people and AI. The picker was run without `--seed`;
each pass names its tuple. The draft these feed is `docs/series/arc.md`.

Sources read first: the series design, the bible, the trailer, the
packing question, Nate's 001 interviews, `docs/ideas/brain-cartridge.md`,
`docs/ideas/model-organisms-of-misalignment.md`, and the sister project
`../ml-mad-science` (its design, README, grid and stop 1).

## Pass 1: an atlas of the series

Tuple: combination, abstraction-lift; atlas; animacy, size, purpose.

The same series, one page per perspective.

- **Teaching.** One neuron, then learning, assumptions, bends, layers,
  gradients, pixels, probes, feature pictures, mixed neurons, packing,
  dictionaries, circuits, intervention, steering, the critique, a small
  transformer. A ladder of ideas, each built on the last.
- **Knowledge (what can be checked).** Every claim in 001 is checked
  against the game's own memory. The bible already plants the first
  question the game can't answer ("will Mario land soon?", 001's
  `why-key`). Read this way the series is a slow walk from *full answer
  key* to *no answer key*, which is where real interpretability lives.
- **Safety.** The model organisms: the cheating byte (003), shortcuts,
  reward hacking, goal misgeneralization, substrate exploits, planted
  hidden goals, auditing games; and the real failures already in hand
  (Punch-Out!! brains that stand still, the bank that does nothing).
- **The field.** Two debates run through it: whether mechanistic
  interpretability is worth doing at all (Hendrycks and Hiscott, the
  theories of impact), and the packing question (superposition or
  generalization; "a real fork in the road for AI").
- **The person.** A software engineer moving into AI safety, learning
  in public: "corrections welcome", his own voice, his own 8-bit self
  chased by paperclips.
- **The material.** The NES itself: 2,048 bytes, a 6502, a
  deterministic machine with save states; greenstone; the Pis and
  velaryon. And from brain-cartridge: "the brain and the game share a
  substrate", a network that could run on the console it plays.
- **Myth.** A descent into something we made but don't understand,
  with a lamp (the answer key) that dims the deeper we go.

Lifted, the invariant every page agrees on: **a person learning to
understand a made mind by checking it against a world they understand
completely.** The pages disagree about where it ends. Teaching ends at a
transformer; knowledge ends where the answer key runs out; safety ends at
a flaw nobody planted; the field ends at a verdict on the fork; the person
ends somewhere new; the material ends with a mind inside the cartridge.
Those disagreements are the arc's open choices.

## Pass 2: a periodic grid of who reads whom

Tuple: substitution, tree-finding; periodic-grid; visibility, polarity,
scope.

Substitute the reader and the read. Every cell should exist.

```
+------------------+----------------------+----------------------+----------------------+
| reader \ read    | a game               | a network            | a person             |
+------------------+----------------------+----------------------+----------------------+
| a person         | RAM maps (Data       | the series' core     | teaching: viewers    |
|                  | Crystal), reverse    | (every video)        | reading Nate; Nate   |
|                  | engineering          |                      | reading himself      |
+------------------+----------------------+----------------------+----------------------+
| a network        | every network we     | probes, sparse       | ? a model that reads |
|                  | train (it models     | autoencoders,        | Nate's play and says |
|                  | the game)            | automated labelling  | what he wants        |
+------------------+----------------------+----------------------+----------------------+
| both together    | ? a network that     | auditing games:      | ? a person and a     |
|                  | finds RAM meanings   | a person with tools  | network reading a    |
|                  | we didn't know       | finds a planted flaw | third party together |
+------------------+----------------------+----------------------+----------------------+
```

The question marks are predictions:

- **A network reading a person** is ml-mad-science's motivating negation
  (reinforcement learning's reward in, policy out, turned into policy in,
  reward out: inverse RL), already sketched small in brain-cartridge
  ("RAM-address inverse reinforcement learning"): from a recording of
  play, find which byte the player was trying to raise. Pointed at Nate's
  own play, the machine reads the human. That cell is the mirror of the
  whole series, and inverse RL is a branch of value learning, which is an
  alignment topic in its own right.
- **A network finding meanings we didn't know** is the moment the answer
  key stops being ours: the network as a RAM-map author.
- **Visibility and polarity, as a second grid** (the safety arc):

```
+------------------+------------------------------+------------------------------+
| flaw \ seen      | visible                      | hidden                       |
+------------------+------------------------------+------------------------------+
| planted by us    | the cheating byte (003)      | an auditing game: Nate finds |
|                  |                              | a flaw he didn't plant       |
+------------------+------------------------------+------------------------------+
| arose by itself  | Punch-Out!! brains standing  | ? what we haven't noticed:   |
|                  | still                        | the real frontier            |
+------------------+------------------------------+------------------------------+
```

The last cell can't be shown on purpose. The series can only build the
habits that would find it, and say so.

## Pass 3: a dictionary, and its negations

Tuple: negation, cross-domain re-instantiation; dictionary; cardinality,
hierarchicalness, naturalness.

- **Answer key:** the game's own memory, a complete record of the world
  the network sees, against which any reading of the network can be
  checked.
- **Glass cartridge:** a made thing whose insides can be seen and
  checked. The opposite of a black box.
- **Understanding (a working definition the series could adopt):** we
  understand a network when we can predict what it will do on a frame it
  hasn't seen, change what it does on purpose, and explain it to someone
  else. Three tests: prediction, intervention, teaching.
- **Model organism:** a small network with a flaw we know is there, kept
  so our tools can be tested against it.
- **Mad science:** an experiment chosen because it negates one property
  of an idea, to see whether the idea survives.

Negating the series' definitional properties:

```
property                      | negated                          | what it is
------------------------------+----------------------------------+------------------------------------
there is an answer key        | there is none                    | real interpretability; the ending
the answer key is honest      | the key lies                     | substrate exploits, emulator quirks
networks are small            | networks are large               | the transformer video; the critique
a person does the reading     | a network does the reading       | automated interpretability, SAEs
the network is read           | the person is read               | inverse RL on Nate's play
the teacher knows the answer  | nobody knows the answer          | the packing strand: research live
it is taught in order         | it is revisited in a circle      | spaced repetition, already a rule
the flaw is planted           | the flaw arose by itself         | Punch-Out!!; the frontier
```

Two of these are already rules but not yet an arc. "Nobody knows the
answer" says the series changes kind as it goes: early videos are
lessons, later ones are lab notebooks. Re-instantiated in medicine:
anatomy (dissecting a known body), then physiology (watching it work),
then pathology (what goes wrong), then a clinic where the patient walks
in with something no textbook names. The series is a medical education
for made minds, and the NES is the cadaver lab.

## Pass 4: a chart of what lasts

Tuple: dimension-identification, organon-construction; chart;
predictability, longevity, modularity.

```
+------------------------+---------------------------------+---------------------------------+
| how long it stays true | predictable result              | open result                     |
+------------------------+---------------------------------+---------------------------------+
| for good               | w·x + b; gradients; probes;     | does packing happen in our      |
|                        | the answer key; causal tests    | networks? what does a neuron    |
|                        |                                 | inside a transformer do?        |
+------------------------+---------------------------------+---------------------------------+
| a few years            | which tools are standard        | the field's debates: Hendrycks  |
|                        | (SAEs today)                    | and Hiscott, Cammarata          |
+------------------------+---------------------------------+---------------------------------+
```

The foundations last; the debates date. An arc anchored on a debate ages
with it; an arc anchored on a question ("can we understand what we
made?") doesn't. Debates belong in episodes, the question belongs in the
spine. Modularity: each video already stands alone (one idea), so the
arc can be grouped into seasons without rewriting anything.

## Pass 5: a cycle

Tuple: dimension-identification, organon-construction; cycle;
reversibility, direction, age.

Every video is one turn of the same cycle:

```
ask a question -> build a network -> look inside -> make a claim
      ^                                                   |
      |                                                   v
 a new question <- it breaks somewhere <- pressure-test it (mad science)
```

The series is the same cycle run as a spiral, each turn one level up.
Closure asks where the spiral comes back over its start. A natural
bookend: **001 builds one neuron that draws a line; the last video finds
one neuron that draws a line inside something huge** (a small
transformer, or a network running on the console). The first neuron is
made and fully known; the last is found, and known only as far as the
series' tools reach. Direction reverses: building, then dissecting.

Age: the viewer starts as someone who hasn't done algebra since school,
and ends able to read an interpretability paper and judge it. Nate
starts as an engineer moving into AI safety. Both age along the spiral;
the series could let that show.

## Pass 6: a graph of hubs and side effects

Tuple: cross-domain re-instantiation, negation; graph; side-effect,
homogeneity, materiality.

Walking the series' nodes (neuron, answer key, model organism, critique,
packing, Nate, viewer, NES, mad science, trust) and labelling edges, two
hubs carry more edges than anything chosen on purpose:

- **The answer key** (enables every check, defines the method, sets up
  the ending by running out).
- **Responsibility.** The cheating byte is a shortcut *we* allowed. The
  brains that stand still learned a reward *we* wrote. "It found an
  answer to the letter of its goal" (the trailer's old caption) points
  back at the person who set the goal. Nearly every "what they shouldn't
  do" in the series is a mirror of what we asked for. That is the
  people-and-AI theme hiding in the graph: looking into the machine keeps
  showing us ourselves.

Side effects: teaching how networks work also teaches how to build them
(the series is dual-use only in the mildest way: tiny models, public
knowledge). The series is made with an AI's help, a fact that sits on
the graph's edge and may belong on screen. Nate's own move into AI
safety is a side effect of the series as much as its cause.

Homogeneity: every subject is an NES game. The risk is monotony; the
strength is that one world, known completely, lets every tool be
compared on the same ground. Materiality: the cartridge is a real
object. brain-cartridge's thesis (a network as a 6502 program, hosted
beside the game it plays) would make the title literal: a glass
cartridge, a network inside a cartridge that we can read down to the
instruction. Marr's levels (computation, algorithm, implementation)
would all be on screen at once.

## What the passes add up to

- **A spine that ages well:** the answer key runs out. Full key, then a
  partial key, then questions the game can't answer, then none.
- **A kind-change:** lessons become a lab notebook as the series reaches
  questions nobody can answer yet.
- **A theme:** looking into the machine shows us our own instructions.
- **A bookend:** a neuron we built, then a neuron we found.
- **A mirror:** the machine reading the person (inverse RL on Nate's
  play), the empty cell of the who-reads-whom grid.
- **A literal title:** a network on the cartridge itself.
- **ml-mad-science:** its moves are already the series' pressure tests,
  and its spine (regression, logistic, k-means, trees, MLP, CNN,
  attention, autoencoders, RL) is nearly the series' foundation
  curriculum. The open question is whether the series *is* where that
  course's spine is walked in public, or whether the two stay separate.
