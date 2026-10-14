# Brain Cartridge and friends

Ideas for what to do with the `src/nes/gym/` module, produced by several
ideonomy-plain sessions on 2026-10-13. The picker was run without `--seed`,
so the tuples are not reproducible; the operators and organons used are
noted per section instead.

The thesis that survived every draw: **the brain and the game share a
substrate.** A policy that is itself a 6502 program, hosted beside the game
it plays, is the one thing a pixel-and-controller gym built on someone
else's emulator can never have. Everything below descends from that.

Status tags: `BUILD` means recommended and cheap, `LATER` means recommended
but depends on something else, `WRITEUP` means worth doing for the story
rather than the result, `DEPOSIT` means it belongs in ml-mad-science's grid.

## The six

### 1. Tetris reservoir  `BUILD`

Plug a commercial ROM in as the brain for Super Mario Bros. Hash the brain
ROM's 2KB of RAM into eight button bits. That is reservoir computing: a
fixed, high-dimensional, nonlinear dynamical system that is never trained,
with a small learned readout on top. People have built reservoirs from
buckets of water and octopus arms; nobody has used Tetris.

- Two `NesEmulator` instances, no new mapper.
- The readout is a linear map from 2048 reservoir bytes to 8 buttons.
  Train it with ordinary least squares, which is ml-mad-science stop 1.
- Feed the game's state into the reservoir through its controller port,
  so the reservoir is driven, not free-running.
- Then rank the whole ROM library: which 1980s game is the best Mario
  player.

Likely UNEXPLORED as an ML cell. Named family: echo state networks,
physical reservoir computing.

Result (2026-10-14): built as `src/nes/gym/reservoir/` and
`examples/tetris_reservoir.rs`. Demonstrations come from a beam search over
save states in 24-frame rounds of jump macros, with a one-round survival
peek so candidates committed to a collision do not crowd out the beam, and
one candidate per game state so the beam does not collapse to copies of
its best lineage. Six searches, each from a different no-op offset (0 to
25 frames, in steps of 5) and tie-break seed, all clear 1-1 in 492 to 540
steps. Steps where the game has control (flagpole slide, castle countdown)
are not training rows, which leaves 2012. Readout is ridge regression,
lambda 1, intercept unpenalised. Held-out columns are
leave-one-demonstration-out, pooled over the six folds. Closed loop is
five episodes from no-op offsets halfway between the demonstrations' (2,
7, 12, 17, 22 frames). The `clock` condition is a one-hot of the step
index and nothing else; because the demonstrations are offset from one
another, its held-out accuracy is close to the majority bit per button,
which makes it the floor.

```
condition          |  rows | train acc | held-out acc | held-out mse |  mean x | max x | resets
-------------------+-------+-----------+--------------+--------------+---------+-------+-------
clock              |  2012 |     0.861 |        0.808 |       0.1376 |   999.4 |  1442 |      0
raw                |  2012 |     0.896 |        0.891 |       0.0909 |   313.2 |   314 |      0
undriven           |  2012 |     0.806 |        0.797 |       0.1371 |   813.6 |  1129 |      0
driven-controller  |  2012 |     0.805 |        0.793 |       0.1435 |   764.8 |  1673 |      0
driven-playfield   |  2012 |     0.888 |        0.886 |       0.1015 |   313.6 |   314 |      3
```

The single-demonstration hint that the playfield drive beat raw does not
survive. Across six demonstrations raw is best on held-out error, the
playfield drive is a point behind it, and the undriven and
controller-driven reservoirs sit on the clock floor: Tetris RAM driven
through the controller carries nothing a linear readout can use, and
Tetris RAM with the window pasted into its playfield carries roughly what
the window itself carries, a little degraded.

The column that explains the rest is training accuracy. Raw fits its own
training rows at 0.896 and holds out at 0.891: the readout is not
overfitting, it is underfitting, and more demonstrations will not move it.
Three buttons are active (A, B, Right); B and Right are held almost always,
so nearly all the error is the A bit, and a linear map from byte values
cannot express "jump when the next enemy is this many pixels ahead". The
closed-loop column says the same: every state-based readout dies at the
first Goomba in every episode, and only the clock gets anywhere, by
replaying the demonstrations' timing.

So the bottleneck is the readout's encoding, not the number of
demonstrations and not which reservoir sits in front of it. The next
experiment is a `raw-bits` condition, the window expanded to one feature
per bit (2048 features, the same width as Tetris RAM): a fixed nonlinear
expansion with no game in it. If raw-bits learns the jump and Tetris does
not, Tetris is a worse expansion than bits, and the reservoir idea is
closed for this readout. If neither does, the readout needs a second
layer, or the DAgger-style corrections from the design note, before any
reservoir comparison means anything.

Result (2026-10-14, placement drive): the first version where Tetris is a
real reservoir. Each SMB step is encoded as four tetromino placements, a
column and a rotation each: the distance to the nearest enemy ahead in
32-pixel bins and its height, the same for the second enemy, Mario's
height with his airborne state, and his speed with his state byte. Tetris
runs at level 19, selected through its own menu, and a scripted player
steers each piece by controller taps: rotate, slide, drop, and wait for
the lock. Game over resets the board. Two controls: `placement-onehot`
feeds the same four placements to the readout as 56 one-hot features with
no Tetris in between, and `placement-hash` drives Tetris from a hash of
the whole window. Same six demonstrations, same leave-one-out.

```
condition          |  rows | train acc | held-out acc | held-out mse |  mean x | max x | resets
-------------------+-------+-----------+--------------+--------------+---------+-------+-------
clock              |  2012 |     0.861 |        0.808 |       0.1376 |   999.4 |  1442 |      0
raw                |  2012 |     0.896 |        0.891 |       0.0909 |   313.2 |   314 |      0
placement-onehot   |  2012 |     0.865 |        0.860 |       0.1134 |   313.6 |   315 |      0
placement-tetris   |  2012 |     0.854 |        0.794 |       0.1591 |   866.6 |  1512 |    889
placement-hash     |  2012 |     0.846 |        0.768 |       0.1707 |   934.8 |  1433 |    868
```

Four things, in order of how much they matter.

Tetris subtracts. The one-hot control holds out at 0.860 and the same
placements played through Tetris hold out at 0.794, below the clock
floor, with training accuracy above held-out for the first time. Tetris
turned 56 clean features into 2,048 noisy ones, and with a game over
every nine pieces the board never held more than two steps of history.
The dynamics scrambled the signal rather than enriching it.

Locality matters, as predicted. The hash drive is the worst row at 0.768,
below the floor: unrelated stacks for similar Mario states give the
readout nothing but features to overfit.

The encoder is weaker than the bytes. The one-hot control, which is the
encoder's ceiling, sits three points under raw. The four features know
about enemies and Mario, and nothing about terrain, and most A presses in
1-1 are for pipes, pits and stairs, not Goombas. The encoder search in
the design note's second phase is the fix, and it needs no Tetris to run.

The closed-loop column has been flattered by noise. The two Tetris
readouts travel to 867 and 935 on average while raw and one-hot die at
the first Goomba, and the hash does better than the salient drive. A
readout whose A output hovers around one half jumps at random, and
jumping at random while running right gets further through the start of
1-1 than any faithful copy of the demonstration does. Closed loop needs a
random-A baseline before it says anything, and this table is the reason.

Result (2026-10-14, encoder search and random-A floor): two additions and
one correction. The correction first: the SMB zero page holds the game's
own reading of the controller at `$0A` to `$0D`, and the encoder search's
first pick was `$0A`, last frame's A and B buttons. Every condition had
been reading the previous action back. The window now zeroes those four
bytes; the table below is the first honest one. The search then picks a
feature at a time, each a window byte, optionally relative to Mario's x,
binned by a power of two, scored by leave-one-out accuracy of the one-hot
readout, with no Tetris in the loop. The random-A row has no readout: it
runs right and presses A by a two-state chain fitted to how often and how
long the demonstrations hold A.

```
searched encoder: $9F /4, $6F-x /4, $CE /4, $CE-x /32
  (vertical speed; enemy slot 1 x page relative to Mario; Mario y; Mario y relative to Mario x)

condition          |  rows | train acc | held-out acc | held-out mse |  mean x | max x | resets
-------------------+-------+-----------+--------------+--------------+---------+-------+-------
clock              |  2012 |     0.861 |        0.808 |       0.1376 |   999.4 |  1442 |      0
raw                |  2012 |     0.865 |        0.859 |       0.1184 |   313.2 |   314 |      0
random-a           |  2012 |     0.779 |        0.779 |       0.2207 |   472.6 |   722 |      0
placement-onehot   |  2012 |     0.865 |        0.860 |       0.1134 |   313.6 |   315 |      0
placement-tetris   |  2012 |     0.854 |        0.794 |       0.1591 |   866.6 |  1512 |    889
searched-onehot    |  2012 |     0.869 |        0.867 |       0.1097 |   337.8 |   435 |      0
searched-tetris    |  2012 |     0.861 |        0.805 |       0.1534 |   423.2 |   898 |    774
placement-hash     |  2012 |     0.843 |        0.782 |       0.1668 |   791.0 |  1127 |    793
```

Raw fell from 0.891 to 0.859 once it could not see the controller: three
points of its score, a third of its margin over the floor, was the last
action. With that gone, raw, the hand-picked encoder and the searched
encoder all land within a point of each other, 0.859 to 0.867, with
training accuracy the same as held-out for every one. That is a ceiling,
and it is the same ceiling for 256 scalars, four hand-picked bins and
four searched bins. The search's picks after the first add a thousandth
each, and the last two are Mario's y binned two ways. There is no better
byte to send.

The ceiling is the demonstrations, not the encoder. The search that
produced them jumps whenever a jump survives, so when it presses A within
a window of equivalent moments is arbitrary, and six runs disagree with
one another by phase. A readout that sees only the current state cannot
know which of several equally good moments this run chose. About an
eighth of the A bits are unpredictable from state in principle, and every
condition stops there. The way past it is either a smoother target (jump
within the next k steps, rather than this step) or demonstrations from a
policy that is itself a function of state.

Tetris still subtracts, by the same six points as before: 0.867 to 0.805
for the searched encoder, 0.860 to 0.794 for the hand-picked one. The
hash control stays worst.

The random-A row settles the closed-loop column. Jumping at the
demonstrations' rate with no state at all reaches a mean x of 473. The
three Tetris readouts land at 867, 423 and 791 around it, and the hash,
which cannot understand anything, beats the searched encoder. Those
numbers are the spread of a noisy jump rate, not a ranking of policies.
The clock's 999 remains a replay.

Result (2026-10-14, smoothed jump label): the A label at a step became "A
is pressed within the next 4 steps" (16 frames), which covers the three
jump phases the search chooses between, so demonstrations that jump at
different phases of the same obstacle now agree. Every condition, the
random-A chain and the encoder search use the smoothed label. Absolute
numbers are not comparable with the raw-label table because the label's
base rate changed; compare each row with this table's floors.

```
searched encoder: $F9 /4, $CE /4, $CF /32, $9F /16
  ($F9; Mario y; enemy slot 0 y; vertical speed)

condition          |  rows | train acc | held-out acc | held-out mse |  mean x | max x | resets
-------------------+-------+-----------+--------------+--------------+---------+-------+-------
clock              |  2012 |     0.890 |        0.866 |       0.1129 |   313.6 |   314 |      0
raw                |  2012 |     0.875 |        0.862 |       0.1143 |   313.6 |   316 |      0
random-a           |  2012 |     0.819 |        0.819 |       0.1811 |   387.2 |   682 |      0
placement-onehot   |  2012 |     0.871 |        0.860 |       0.1098 |   313.6 |   316 |      0
placement-tetris   |  2012 |     0.892 |        0.851 |       0.1339 |   623.8 |   713 |    839
searched-onehot    |  2012 |     0.881 |        0.877 |       0.1054 |   313.8 |   315 |      0
searched-tetris    |  2012 |     0.894 |        0.858 |       0.1294 |   385.8 |   678 |    779
placement-hash     |  2012 |     0.887 |        0.849 |       0.1398 |   544.0 |   710 |    697
```

The smoother target did not open the ceiling; it raised the floors. The
best readout went from 0.867 to 0.877, one point. The random-A floor went
from 0.779 to 0.819, four points, and the clock from 0.808 to 0.866, six.
The clock now beats raw. Under a label that forgives four steps of phase,
knowing roughly where you are in the level predicts "a jump is due" as
well as any reading of the state does, and the best state readout adds
one point over it. Every margin over the floors shrank.

That is the finding. A linear readout of the zero page, in bytes, in
bins, or through Tetris, knows nothing about when to jump that a clock
does not. The reason is in the memory map rather than the readout: pits,
pipes and stairs are not in the zero page. The terrain Mario is about to
meet lives in the metatile buffer at `$0500` to `$069F`, two screens of
tile columns, and the window has never included it. The state a jumping
policy needs was never in the 256 bytes we gave it, so the readouts fell
back on timing and on Mario's own vertical speed, which is a jump already
in progress.

Tetris subtracts less under the forgiving label, one to two points
instead of six, with training accuracy now above held-out by three to
four points: the 2,048 noisy features overfit, and the label stopped
punishing noise as hard. Closed loop collapsed for everyone: holding A
early and long dies at the first Goomba, including the clock's replay,
which had reached 999 under the raw label.

Next: widen the window to the metatile buffer, so a readout can see the
terrain ahead, and rerun the raw-label table. If raw with terrain beats
the clock by a clear margin, the state was the problem all along, and the
reservoir question can be asked again on a readout that has something to
work with.

A captioned explainer video of the experiment, six chapters ending with
the whole level in real time, is built by `examples/reservoir_dump.rs`
followed by `tools/reservoir_video.py`; see
`docs/superpowers/specs/2026-10-14-reservoir-video-design.md`. Two facts
the video surfaced: only the first 160 window bytes fit the 16 playable
board rows, so the last 96 never reach Tetris, and the Tetris screen never
shows the pasted window, because the game redraws rows only when a piece
lands; the board the readout sees is RAM.

Earlier tables from this day, including the single-demonstration run
where the playfield drive led raw on a time split, are in the git history
of this file.

### 2. Second-mover polarity  `DEPOSIT`

ml-mad-science stop 1 tags "data moves freely, no tether" as DEGENERATE.
That holds only when the second mover shares the first mover's objective.
Flip the polarity and the same cell is a GAN. So **polarity of the second
mover: cooperative / adversarial** is a dimension that splits one
degenerate cell into a degenerate one and a famous one.

Found by walking the "who moves" table from stop 1 onto brain-vs-game
(see the scale under "Game distillation" below). Belongs in `GRID.md` when
stop 9 (RL) runs, and arguably as a retroactive row on stop 1, since
adversarial examples already sit in variant 1 pointing at it. Not added
yet: the grid's rule is that dimensions are deposited only when a stop's
expansion surfaces them.

### 3. RAM-address inverse reinforcement learning  `BUILD`

Take a recording of a human playing (or any TAS). For each of the 2048 RAM
bytes, ask how well "maximize this byte" explains the inputs. Rank. The
answer for SMB is probably the x-position byte, but it might not be, and
the misses are the interesting part.

- Search space is 2048 candidates, so brute force is fine.
- Scoring a candidate: train a tiny policy against that byte as reward,
  compare its action distribution to the human's.
- This is ml-mad-science's motivating negation (reward in, policy out,
  becomes policy in, reward out) made small enough to finish.

### 4. Quine Mario  `WRITEUP`

A TAS input log is one byte per frame. A 4KB brain is 68 seconds of play.
Find a brain that clears 1-1 and whose recorded inputs, read as a file,
are the brain. Most frames of a clear are slack where any input works, so
there is room to encode. Constraint satisfaction, not learning. Built for
the blog post.

### 5. Game distillation and adversarial ROM patch  `LATER`

The scale: fraction of each generation's byte mutations spent on the game
ROM rather than the brain ROM.

```
budget on game | name                               | polarity    | tag
---------------+------------------------------------+-------------+------------
   0%          | Brain Cartridge                    | n/a         | KNOWN
  ~5%          | game jittered, brain adapts        | neutral     | KNOWN (domain randomization)
  50%          | both move, game tethered           | cooperative | collapses to a smoothed env
  50%          | both move, no tether               | cooperative | DEGENERATE: game sets the win flag
  50%          | both move, no tether               | adversarial | KNOWN (co-evolution)
 ~95%          | brain nearly fixed, game breaks it | adversarial | adversarial ROM patch
 100%          | brain fixed, game moves            | cooperative | game distillation
```

**Game distillation**: hold a trivial brain fixed and mutate SMB until the
brain beats it. Output: the smallest ROM a one-byte mind can win, a
curriculum discovered backwards. **Adversarial ROM patch**: find the fewest
bytes to change so a good brain fails. The one-pixel attack with a CHR tile
as the pixel. Both need a working brain first.

### 6. Forward-Forward is mandatory on a NES  `WRITEUP`

A modular brain is a pipeline of cartridges: one compresses the frame, one
decides, one times the presses. There is no way to pass a gradient between
two consoles, so each cart can only train on a local loss. Forward-Forward,
evolution strategies, Hebbian rules, and predictive coding are native here.
Anything needing backprop is not. This is the reason the lab is worth
having, not a project in itself.

## Level demon

Seeded by a Super Mario Maker clip of a stage carpeted in fire bars on
anchor blocks and track rails. The clip is not hard in the kaizo sense. It
is punishing to look at and deceptive about which bars matter. That split
is the tree.

### Tree of awful, by what the level costs the player

```
awful level
+-- lethal (costs lives)
|   +-- fair-lethal: precise but learnable          kaizo
|   +-- unfair-lethal: no reaction window           blind drops
|   +-- impossible                                  DEGENERATE (fails the clear check)
+-- tedious (costs time)
|   +-- long, repetitive, labyrinthine
|   +-- empty: nothing happens                      EMPTY BRANCH: nobody builds these
+-- deceptive (costs trust)
|   +-- hidden blocks, betrayal routes, fake goals
+-- disgusting (costs attention)                    the clip
+-- broken (costs the game)
    +-- softlock, crash                             World -1 is a natural member
```

**The kind of awfulness a demon can discover is bounded by what its victim
can perceive.** A death-rate reward against a RAM-reading brain finds only
the lethal branch. Deception needs a pixel-observing victim with beliefs.
Tedium needs a victim with a quit action.

### Substitutions

```
dimension        | value                           | what it is
-----------------+---------------------------------+----------------------------------
victim           | evolved brain                   | lethal only
                 | pixel brain with quit action    | can reveal tedium and deception
                 | a human                         | reward = deaths, retries, quits
                 | you specifically                | profiled torment; sign flip = tutor
                 | the demon itself                | Mario Maker's clear check
source of levels | generator net                   | PCG, KNOWN
                 | mutate 1-1's bytes              | the demon as proposed
                 | wrong ROM offsets               | World -1; the Prospector
                 | random byte flips, filtered     | corruption with a solvability gate
solvability      | human clears it first           | Mario Maker upload rule
certificate      | learned antagonist clears it    | PAIRED (Dennis et al. 2020)
                 | brute-force search clears it    | BFS over save states
demon-victim     | offline, static                 | the clip
connectivity     | adapts between attempts         | Mario Maker iteration
                 | live director during play       | Left 4 Dead's AI Director
                 | reacts every frame by RAM poke  | the fire bars chase you
```

Two more deposits for ml-mad-science: POET, PAIRED, and Mario Maker's
upload rule are the same minimal-criterion structure with three different
certificate sources (learned, searched, human). And dynamic difficulty
adjustment is troll-level generation with the reward sign flipped.

### Build order

1. **The Prospector**  `BUILD`. SMB loads an area by indexing a pointer
   table into level data; World -1 exists because a glitch indexed it
   wrong. So every byte offset in the ROM is already an accidental level.
   Point the area pointer at each offset, BFS over save states for
   solvability, run a brain for lethality, rank. No generator. Output: a
   catalog of SMB's hidden levels with the worst solvable ones on top.
2. **Level Demon v1**  `LATER`. Mutate 1-1's object and enemy bytes, keep
   only what BFS clears, reward the brain's death rate. Fire bars are an
   SMB1 object type, so the demon can rediscover the clip's aesthetic on
   real SMB1 data. Needs a brain.
3. **Level Demon v2**  `LATER`. Pixel victim with a quit action, so the
   demon can find deception and tedium.
4. **The tutor**  `LATER`. Flip the sign on v2 and point it at a human.

## Supporting pieces worth building

Cheap things from earlier draws that either unblock the above or pay for
themselves.

- **Headless speed bench**  `BUILD`. Frames per second of
  `NesEmulator::step_frame` on one core. Bounds every training budget here.
- **Fuzzer Mario**  `BUILD`. Reward is one point per emulator panic. The
  dumbest idea in the thread and the most useful: it is a test generator
  for Greenstone itself.
- **Canary brain**  `LATER`. An evolved brain adapts to the emulator it was
  raised in, bugs included. Run it on Mesen or real hardware; where it
  desyncs, Greenstone is inaccurate. Differential testing in a Mario
  costume.
- **Byte Mario**  `BUILD`. The policy is a fixed function of one byte and
  the frame counter. There are 256 brains, so evaluate all of them. Then
  two bytes and 65,536. The complete catalog of every two-byte mind, ranked
  by distance reached. An afternoon.
- **MMC-GPU**  `LATER`. A fictional mapper whose registers are a neural
  net's input and output ports. The brain ROM becomes a 200-byte stub. The
  first rung toward a brain that runs on real hardware via an FPGA cart.
  Greenstone's `Mappable` trait makes this cheap.
- **Brain Cartridge proper**  `LATER`. The policy is a 6502 program in a
  second emulator instance with the game's RAM and framebuffer mapped into
  cartridge space and its output wired to the game's controller port.
  Trained by mutation and selection. Weights live in battery SRAM, so the
  save file is the brain. Deliverable is a `.nes` file.

## Prerequisites

Everything above assumes the emulator renders correctly. The uncommitted
framebuffer-sampling diff in `src/nes/bus/mod.rs` suggests it does not yet.
Fix that first, then the speed bench, then pick from the `BUILD` list.

One structural note for anyone training against a deterministic emulator
with `noop_max = 0`: scores are comparable across brains only because the
environment is deterministic, and that same determinism means every brain
overfits one trajectory and collapses into a TAS. The loop needs randomness
injected somewhere. The adversarial fix is a second brain that perturbs the
game's RNG seed, which is why co-evolution is required rather than
optional.
