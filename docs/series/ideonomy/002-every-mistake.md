# Ideonomy: Every mistake moves the line (002)

Six ideonomy-plain passes (2026-11-01) on 002's idea: a neuron learns its
border by nudging after each mistake (the perceptron), which never
settles on data that's only mostly separable; the square trick
(Widrow–Hoff) settles, but aims at squared error, not mistakes, and stops
short of the best line. Data: 001's landing question (depth, vertical
speed). The picker was run without `--seed`; each pass names its tuple.

Read first: `docs/series/games/002.md` (the experiment), and
ml-mad-science's stop 1 expansion
(`../ml-mad-science/spine/01-linear-regression/expansion.md`), which this
video absorbs as its pressure test (arc: "merged").

Numbers are from `exp002.py` (first seen in a scratch run the same day;
some moved slightly). Citations were checked 2026-11-01; sources are in
`games/002.md`.

## Pass 1: a tree of what a nudge can be

Tuple: dimension-identification, abstraction-lift; tree; reversibility,
naturalness, polarity.

Lifted: a learning rule is *a feedback loop that moves a setting by an
amount decided from one signal*. The tree branches on what the signal is
and how the amount depends on it.

- Feedback loop that moves a setting
  - Moves only when wrong (the signal is a yes/no: "wrong")
    - Fixed amount, times the input: **perceptron**
    - Fixed amount, sign of the input only: **simple trick**
    - Only when wrong *by too little*, i.e. too close to the line:
      perceptron with a margin, hinge loss (support vector machines,
      Cortes & Vapnik 1995)
  - Moves always (the signal is a size: "how far from the target")
    - In proportion to the error: **square trick** (Widrow–Hoff)
    - In proportion to a squashed error that stops caring once it's
      confidently right: **logistic trick**
    - By a fixed amount in the error's direction: the absolute trick for
      fitting (Nate's repo)
  - Moves without any answer at all (no teacher)
    - Strengthen what fires together: Hebb's rule (1949)
    - Move toward the nearest cluster: k-means (ml-mad-science stop 3)

Reversibility: every perceptron nudge is undone by an equal and opposite
mistake later. On data no line splits, that's why it never stops: its weights stay
bounded but keep moving (stated by Minsky & Papert 1969; proved by Block
& Levin 1970; "bounded" is what's proven, not a repeating cycle). The square trick's nudges are not
mirror images (their size depends on where the line is), so they can
cancel to zero: a resting point.

Naturalness: the square trick is essentially the same rule as the
Rescorla–Wagner model of how animals learn to expect a reward (1972), a
near identity first recognised by Sutton & Barto (1981). The rule that
settles was found twice, once for machines and once for animals.

Polarity: a mistake has two signs, a false alarm and a miss. The
perceptron weighs them equally; with landings only 25% of moments, the
misses are rarer, so false alarms do more of the pushing. *Experiment:*
count each sign's nudges in a run.

## Pass 2: a map of the landscape the line walks on

Tuple: negation, tree-finding; map; complexity, hierarchicalness, age.

The best picture in the video may be a map. Because scaling a line's
weights doesn't move it (001), every border is one point on a flat map:
its *angle* across, its *offset* up. Colour each point of the map by how
many mistakes that border makes, and again by its squared error.

- **The mistakes map is a staircase.** The count only changes when the
  border crosses a dot, so the map is flat terraces with cliffs between:
  a mistake count has no slope anywhere. There's nothing to roll down.
  The perceptron feels only the cliff edge it's standing on, steps over,
  and stumbles on the next.
- **The squared-error map is a smooth bowl.** It has a slope everywhere,
  and one bottom (the least-squares line, which can be solved directly:
  stop 1's "teleport edge").
- **The two lowest points are in different places.** Search's 91% sits
  at the bottom of the staircase; the bowl's bottom is 35° away, at 87%.
  Map regions: a wide basin, a terraced valley, and the border zone
  between them where the two disagree (6% of moments).

Negating the idea's definitional properties:

```
property                         | negated                        | what it is
---------------------------------+--------------------------------+------------------------------------
the line moves                   | the dots move                  | adversarial examples; dataset
                                 |                                | distillation (stop 1 expansion)
it moves after a mistake         | after every moment             | the square trick
                                 | never; solve it once           | least squares (the teleport)
a teacher says what's right      | no teacher                     | Hebb, k-means (stop 3)
                                 | the teacher lies sometimes     | label noise (pass 4)
one moment at a time             | all moments at once            | batch: tested below
a fixed step                     | a shrinking step               | Robbins–Monro (1951);
                                 |                                | the perceptron then stops, somewhere
it forgets where it's been       | it remembers its best          | the pocket algorithm (Gallant
                                 |                                | 1990): 89–90%
                                 | it averages where it's been    | voted/averaged perceptron
                                 |                                | (Freund & Schapire 1999): 89%
```

Batch, tested (in `exp002.py`): computing every mistake's nudge at once and
taking one averaged step (the batch perceptron) **still wobbles**,
between 43% and 90% over its last 100 steps. So batching isn't what makes
the square trick settle. What does is that its nudges shrink as it nears
its target. **This corrects what Claude told Nate** (and
`games/002.md`'s spine): the perceptron can be run on a GPU too. What the
settling family has that the perceptron doesn't is a *slope*: the
squared-error map has one everywhere, the mistakes map has none. That's
also what learning through many layers needs (later: gradients, 006),
because a hard yes/no passes no slope back through it.

Age: the perceptron (1958) and Widrow–Hoff (1960) are the same age; the
staircase problem has a modern name, too: finding the fewest-mistakes
line is NP-hard in general (Höffgen, Simon & Van Horn 1995), with
the number of inputs part of the problem. In two inputs, search can try every line (001 did); with 2,048
inputs it can't. That's *why* everyone aims at a bowl instead of the
staircase.

## Pass 3: a dictionary

Tuple: organon-construction, cross-domain re-instantiation; dictionary;
connectivity, modularity, decomposability.

- **Mistake:** a moment where the line's answer and the answer key
  differ. The only thing the perceptron ever sees.
- **Nudge:** a change to the weights and bias made after looking at one
  moment (or at a batch, averaged).
- **Step size:** how big a nudge is. For the perceptron from a zero
  start it changes nothing about the border (only the weights' size);
  for the square trick it changes the path but not the resting point.
- **Settle:** stop moving, or move ever less. Not the same as "find the
  best line": the square trick settles on a line that isn't the best.
- **Error (square trick):** how far the neuron's raw sum is from the
  target code (−1 or +1). Not a mistake: a right answer can have an
  error, if it's right by too little *or by too much*.
- **Right by too much:** the square trick pulls the line toward moments
  it already answers right, if their sum overshoots the target. 16% of
  the moments learned from are like this: 97% of them rising
  (median speed −3 against −1 overall). A defensible guess for
  the 35° lean, to test.
- **Loss / landscape:** a score for every possible line; the staircase
  (mistakes) and the bowl (squared error) are two.
- **Batch:** many moments' nudges averaged into one.

Decomposability: both scores are sums over moments, so both can be
split across a GPU. Connectivity: the square trick's nudge for one
moment depends on every weight (the error does); the perceptron's
depends on every weight only through a yes/no.

Across domains, the cleanest translation for a lay viewer is steering:

- **Bang-bang steering:** jerk the wheel a fixed amount whenever you're
  out of the lane. You never stop correcting; you chatter. That's the
  perceptron. (In control engineering, bang-bang control is known to
  chatter.)
- **Proportional steering:** turn the wheel in proportion to how far off
  you are. You settle, but a proportional controller can settle with a
  steady offset (a known property of P-control), which is a fair picture
  of settling on the wrong line.
- **The shower:** turning the tap a fixed amount each time it's too hot
  or cold, against turning it by how wrong it feels.

## Pass 4: a lattice of who decides what moves

Tuple: combination, substitution; lattice; direction, intentionality,
source.

Ordered by how much is decided outside the neuron (top: everything
decided by us; bottom: everything left to the data):

```
                     search (001): we try every line
                    /                                \
     least squares, solved                  pocket perceptron:
     (we choose the score,                  we choose the score
      the math finds the point)             (mistakes), the walk finds
                    \                        candidates, we keep the best
                     \                      /
                    square trick, walking     perceptron, walking
                    (the score chooses        (the last mistake
                     the direction)            chooses the direction)
                                 \            /
                               the order of the moments
                               (what's left to chance)
```

The perceptron and the square trick are incomparable: neither decides
more than the other, they decide different things. Their meet (what
both leave to the data) is the order of the moments, which is exactly
what moved the perceptron from 70% to 90%.

Substitution on the order (source of the moments):

- **Shuffled** (what we ran).
- **In the order they were played:** Nate's "watch it learn over a whole
  playthrough". Learning online in play order isn't the same experiment:
  the moments come in runs (one level, one jump), so the line chases
  whatever Mario did recently. With levels beyond 1-1 (underground,
  water, castles) the line could visibly drift as the level changes:
  learning, then forgetting. Needs recordings past 1-1.
- **Easy moments first:** curriculum learning (Bengio et al. 2009).
- **Chosen by an adversary:** the order that makes the perceptron end
  worst. How bad can it get on our data?

Substitution on intentionality (the teacher): flip a share of the
answer key at random (a teacher who's sometimes wrong). Which rule
suffers more? The perceptron reacts to every flipped mistake forever.

## Pass 5: a spectrum from knob to pressure

Tuple: dimension-identification, substitution; spectrum; materiality,
autonomy, purpose.

Spectrum: how much a rule *cares how wrong* it is, from 0 (only whether
it's wrong) to all (in proportion, without limit).

```
cares 0% ----------------------------------------------------------- 100%
 simple  perceptron   hinge     logistic           square trick
 trick   (on mistakes,(near the (fades once right) (right by too much
         fixed size)  line too)                     still pulls)
```

Nothing named sits between "only when wrong" and "always": hinge and
logistic do. Each step right buys settling and costs listening to
moments that don't matter for mistakes.

Materiality: the Mark I Perceptron learned by motors turning
potentiometers (Rosenblatt 1961/62); the first ADALINE had hand-turned
knobs, later ones electrochemical "memistors" (Widrow 1960; the order
rests on Wikipedia). Learning was a knob being turned. 001's
neuron already has dials on screen; a nudge can be a turn of them.

Autonomy: substitute the console as the learner. The simple trick needs
only adds and compares, which a 6502 does natively: a neuron that learns
*while you play*, running beside the game (brain-cartridge's thesis; the
arc's "network on the cartridge" season-ender). Not for 002; worth
noting that 002's crudest rule is the one that fits on the cartridge.

Purpose: the rule's purpose isn't the viewer's. The viewer wants fewest
mistakes; the square trick wants smallest squared error; nobody asked it
to want that. A first small mirror for the arc's theme: it found an
answer to the score we wrote.

## Pass 6: an atlas of the wobble

Tuple: abstraction-lift, negation; atlas; distribution,
discovery-vs-invention, visibility.

The same wobble, one page per view:

- **On the dots:** the line swinging across the cloud of moments,
  pass after pass.
- **On the score:** 79, 86, 75, 73, 86, 83, 85, 88, 87, 90%: one
  perceptron run's last ten passes.
- **On the map (pass 2):** the walker hopping across terraces, never
  resting; the square trick rolling to the bowl's bottom.
- **On the moments:** which moments keep causing nudges. Distribution:
  are the endless mistakes spread evenly, or the same few moments
  (bounces, pits, raised landings: 001's named mistakes) again and
  again? *Experiment:* count nudges per moment over a run. If it's the
  same few, the wobble has a cause the answer key can name.
- **On chance:** the ending moved by an order or 30 dropped moments.

Discovery vs invention: the bowl's bottom is discovered (it's there,
solvable directly); the walk is invented. The staircase's bottom is also
there to be discovered, but no walk is guaranteed to find it, and in
many inputs nothing efficient is known to.

Visibility: the wobble is invisible in a final score. A single run
reported as "84%" (as 001's notes once did) hides that the next pass
might have said 73%. That's the lesson for interpretability later: a
network is one stop on a walk, and the walk could have stopped elsewhere.

## What the passes add up to

- **A correction, first.** Batching isn't what makes the square trick
  settle, and it isn't what the perceptron lacks: the batch perceptron
  still wobbles (43–90%). What the settling family has is a *slope*:
  its nudges shrink near its target. "Why the settling kind won" becomes:
  a slope can be followed through many layers, and the mistakes count
  has none.
- **The picture:** the map of every border (angle × offset), as a
  staircase (mistakes) and a bowl (squared error), with their lowest
  points 35° apart. It explains the spine in one image.
- **Why not aim at mistakes directly:** in 2 inputs we can (001's
  search); in 2,048 nobody can, so everyone aims at a bowl.
- **The cause of the lean, to test:** moments the square trick answers
  right *by too much* (mostly high and rising) still pull the line.
- **Mad-science candidates** (stop 1's expansion supplies the first
  two):
  1. *The dots move, not the line* (stop 1 variant 1): the smallest edit
     to Mario's state that flips the neuron's answer, then play the game
     forward from it with a save state, so the answer key says what
     really happens. Only the NES can do this. Probably season 3
     (intervention); plant it.
  2. *The teleport* (stop 1 variant 7): least squares solved in one
     step, landing where the walk ends. Already checked in `exp002.py`.
  3. Learn in play order across a whole playthrough: the line drifts
     with the level (Nate's idea; needs recordings past 1-1).
  4. A lying teacher: flip some answers; which rule suffers more?
  5. Which moments keep the perceptron moving: count nudges per moment.
  6. Remember the best (pocket) or the average: 89–90% and 89%,
     nearly search's 91%. The cheapest fix for the wobble.
- **Cross-domain:** bang-bang vs proportional steering; the square trick
  is essentially Rescorla–Wagner, how animals learn to expect.
- **A seed for the arc:** the crudest rule (sign-only nudges) fits on a
  6502 (Nate's repo's simple trick: a fixed step by the input's sign;
  Serrano's simple trick uses small random steps instead): a neuron that learns on the cartridge while you play.
