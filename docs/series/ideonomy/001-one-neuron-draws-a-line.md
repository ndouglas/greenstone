# Ideonomy: one neuron draws a line (videos 001 and 002)

Six passes, run with the `ideonomy-plain` skill on 2026-10-29. Each pass
used a randomly picked method tuple. These passes split the idea across
two videos: 001 (the line) and 002 (when the line lies).

## Pass 1: procedure

Tuple: operators tree-finding, abstraction-lift; organon procedure;
dimension prompts materiality, reversibility, autonomy.

```
step                        precondition              how it fails
1 choose a yes/no question  answerable from 1 frame   "rising or falling?" needs 2 frames
2 choose the inputs         a human picks the bytes   pick the answer byte itself -> trivial
3 turn bytes into numbers   bigger byte = "more"      byte is an ID or bit flags -> nonsense
4 weigh, sum, add bias      the inputs add up         effects that only work together
5 compare to zero           a hard cut is fine        everything near the line is a coin flip
6 read the answer           someone trusts it         no one asks why
```

- Up the tree: a weighted vote against a threshold (a jury, a credit
  score, a thermostat, a referee's points card).
- Siblings: a lookup table, a decision rule ("if byte 0x1D is 1"),
  nearest-example matching.
- Materiality: a perceptron can be a literal balance scale; the weights
  are weights, the bias a counterweight.
- Reversibility: it can't be run back to one input; every frame on the
  same side of the line gives the same answer.
- Autonomy: step 2 is the hidden human. Most of the "intelligence" is in
  what we choose to show it.
- Experiments: the game's own "airborne" flag (instant perfection, and
  cheating); a random byte (fails); an ID byte (fails strangely).

## Pass 2: spectrum

Tuple: operators combination, organon-construction; organon spectrum;
dimension prompts age, hierarchicalness, distribution.

```
one byte decides   a few bytes share   all 2048 bytes share
(lookup)           (readable weights)  (smeared, unreadable)
```

- The middle ("a few loud voices", 2 to 5 weights) is where
  human-readable explanations live.
- Crossed with the age of the game: simple early games tend to keep a
  variable in one named byte; later games use object tables, where a
  fact lives in "slot k of the table". The same question lands at
  different points on the spectrum in different games.
- Hierarchy: one neuron is flat; two layers is where structure starts.
- False discreteness: yes/no hides a spectrum, the distance from the line
  (confidence). Leads to sigmoid.

## Pass 3: graph

Tuple: operators cross-domain re-instantiation, negation; organon graph;
dimension prompts predictability, complexity, cyclicity.

```
perceptron --[straight -> curved]--> bent boundary
           --[one answer -> many]--> a layer of neurons
           --[yes/no -> how much]--> regression: predict Mario's x
           --[sum -> product]-------> an AND gate built from multiplication
           --[fixed -> drifting]----> weights that change mid-game
           --[chosen inputs -> learned]-> features (the whole series)
           --[same every time -> random]-> a noisy neuron
```

- The hub: "chosen inputs -> learned" connects to almost everything; it's
  the series' spine.
- Cyclicity: frame counters wrap from 255 to 0. "Is the counter in the
  first half of its cycle?" can't be answered by one line on the raw
  byte; with the sine and cosine of the counter it's trivial.
- Predictability: the NES is deterministic; every result replays exactly.
- The same line elsewhere: a tide line (the bias is the tide), football's
  offside line, an exam's pass mark.

## Pass 4: matrix

Tuple: operators substitution, dimension-identification; organon matrix;
dimension prompts symmetry, naturalness, polarity.

Swapped: the question. Held: one neuron, two RAM bytes.

```
question                Mario 1      Tetris         Punch-Out!!     Arkanoid
player in the air?      line works   n/a            n/a             n/a
left half of screen?    line works   piece x: yes   n/a             paddle x: yes
score over 1,000?       place value  BCD breaks it  BCD-ish         ?
Mac about to be hit?    n/a          n/a            no single line  ball near paddle?
```

(Cells are predictions to check, not results.)

- Place value is a perceptron: weights 1000, 100, 10, 1 over digit bytes
  read a score. Two digits packed per byte (BCD) break "bigger byte =
  more": 0x19 is followed by 0x20.
- Symmetry: shuffling the inputs changes nothing; the neuron doesn't know
  which bytes or pixels are neighbours (pays off with convolutional
  networks).
- Polarity: a weight's sign is evidence for or against; flip all signs
  and the answer flips.
- Naturalness: "is byte 0x3C odd?" is as good a question as any to the
  neuron.

## Pass 5: notation

Tuple: operators cross-domain re-instantiation, negation; organon
notation; dimension prompts size, connectivity, purpose.

```
algebra     y = w1*x1 + w2*x2 + b ; yes if y > 0
school      y = m*x + b   (the line is where they're equal)
Python      yes = w1*x1 + w2*x2 + b > 0
spreadsheet =IF(A2*$B$1 + A3*$B$2 + $B$3 > 0, "yes", "no")
scale       pans, weights, counterweight, which side tips
circuit     resistors summing voltages into a comparator
```

- Required slots: units. Bytes are 0 to 255; what unit is a weight in?
  Only the scale and the circuit force you to say what physically moves.
- Inexpressible in all of them: where the inputs came from, and why.
- Size: 2 inputs is a line, 3 a sheet; 2,048 can't be drawn, and the
  series says so. Negating the dimension: in 1D the line is a point, a
  thermostat (a 0D -> 1D -> 2D -> 3D opener).
- Purpose: weights mean nothing without the question. Transplant Mario's
  weights onto Tetris's memory and it answers a question nobody asked.

## Pass 6: cycle

Tuple: operators combination, substitution; organon cycle; dimension
prompts discovery-vs-invention, visibility, homogeneity.

```
guess a line -> test it on real frames -> find where it's wrong
     ^                                          |
     |                                          v
rewind and change the game <- predict what it'll say <- look at the weights
```

- Learning (003) is this loop run automatically.
- Discovered or invented: many lines separate the data equally well; the
  gap is in the data, the line is our choice. The widest-gap line
  (margin) is a free pressure test.
- Visibility: two inputs, everything is visible; with pixels, the
  weights are a picture (007).
- Homogeneity: the neuron treats every input alike, but RAM bytes aren't
  alike (positions, flags, timers, IDs); pixels are. Memory inputs vs.
  pixel inputs, across games, is a ready experiment.

## What it adds up to

- Assumptions (in the bible's ledger): a human picked the inputs; bigger
  byte means more; the evidence adds up; a hard cut is fine; one frame is
  enough.
- Experiments: the balance-scale neuron; the cheating byte; the ID byte;
  BCD scores across games; the wrapping frame counter; weight transplants
  between games.
- The split: 001 is the line (balance scale, place value, several games);
  002 is when the line lies (IDs, BCD, wrapping counters, cheating bytes).
