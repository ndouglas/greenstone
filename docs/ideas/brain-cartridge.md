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

Result (2026-10-13): built as `src/nes/gym/reservoir/` and
`examples/tetris_reservoir.rs`. The demonstration is a beam search over
save states (228 steps, x=2014 in 1-1, not cleared). Readout is ridge
regression, lambda 1, trained on the first 80% of steps, tested on the
last 20% (46 steps, 138 button bits). Closed loop is five episodes with
random no-op starts. The `clock` condition is a one-hot of the step index
and nothing else, the most any readout can get from a clock. The intercept
is unpenalised; the solver uses the dual form since rows are fewer than
features.

```
condition          |  rows | held-out accuracy | held-out mse |  mean x | max x | reservoir resets
-------------------+-------+-------------------+--------------+---------+-------+-----------------
clock              |   228 |             0.870 |       0.0818 |  1135.4 |  1563 |                0
raw                |   228 |             0.971 |       0.0468 |   313.6 |   314 |                0
undriven           |   228 |             0.775 |       0.1550 |   781.4 |   899 |                0
driven-controller  |   228 |             0.768 |       0.1919 |   314.0 |   316 |                0
driven-playfield   |   228 |             0.971 |       0.0406 |   313.6 |   314 |                3
```

Tetris adds nothing. The playfield-driven reservoir ties the raw baseline
on held-out error, and both die at the first Goomba in every closed-loop
episode. The controller drive is worse than raw on every column.

The clock row explains the undriven row. A readout on a one-hot step index
cannot generalise to unseen steps, so its held-out numbers are the bias
alone, yet in closed loop it replays the training segment and travels
further than anything else, past the pipes to x=1563 at best. Undriven
Tetris RAM advances deterministically from its snapshot, so it is a clock
too, and its 781 is the same replay effect through a noisier clock. Both
rows are tape recorders, not policies.

Closed-loop distance is measured on the same stretch of the same level the
readout was trained on, so no row here measures generalisation. A fair
test of any reservoir needs either a held-out level or a clock-subtracted
score, and neither exists yet. Tetris left play mode three times under the
playfield drive and was reset each time.
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
