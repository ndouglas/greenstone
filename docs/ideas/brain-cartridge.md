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

Result (2026-10-14, terrain in the window): two runs, raw label. First
the block buffers appended as they sit in RAM, 416 fixed bytes: raw fell
from 0.859 to 0.854 with training up, and the search never picked a
terrain byte. A fixed byte there is "row r of column c of the current
page", which means nothing without Mario's position. Then the terrain
re-indexed relative to Mario: his own column and the seven ahead, 13 rows
each, 104 bytes, read from whichever block buffer holds that page.

```
condition          |  rows | train acc | held-out acc | held-out mse |  mean x | max x | resets
-------------------+-------+-----------+--------------+--------------+---------+-------+-------
clock              |  2012 |     0.861 |        0.808 |       0.1376 |   999.4 |  1442 |      0
raw                |  2012 |     0.871 |        0.854 |       0.1229 |   313.6 |   314 |      0
random-a           |  2012 |     0.779 |        0.779 |       0.2207 |   472.6 |   722 |      0
placement-onehot   |  2012 |     0.865 |        0.860 |       0.1134 |   313.6 |   315 |      0
placement-tetris   |  2012 |     0.854 |        0.794 |       0.1591 |   866.6 |  1512 |    889
searched-onehot    |  2012 |     0.869 |        0.867 |       0.1097 |   337.8 |   435 |      0
searched-tetris    |  2012 |     0.861 |        0.805 |       0.1534 |   423.2 |   898 |    774
placement-hash     |  2012 |     0.846 |        0.782 |       0.1617 |   937.8 |  1131 |    826
```

Raw with the terrain ahead of Mario holds out at 0.854, the same as
without it, and the search still picks the same four zero-page bytes.
Giving a linear readout the pits, pipes and stairs in front of Mario did
not move it. So the "missing state" explanation from the smoothed-label
run is not supported, at least not for a linear readout: whatever is
missing is not fixed by more bytes.

Taken together, the raw-label runs say this. Floors at 0.779 (random A)
and 0.808 (clock). Every linear readout of state, 256 bytes or 360, as
scalars, as bins, hand-picked or searched, lands between 0.854 and 0.867,
and Tetris in front of any of them costs six points. The ceiling belongs
to the policy class: a weighted sum cannot express "jump when the thing
ahead is close", however the thing ahead is written down.

That makes the next experiment the one the reservoir idea has been
begging for: a proper reservoir in software. A fixed random projection of
the window to 2,048 units through tanh, the echo state network's
expansion with no game in it, read out by the same ridge. If it clears
0.87 by a margin, nonlinearity was the missing piece and Tetris is a bad
nonlinearity. If it does not, the demonstrations' jump phase is the
ceiling after all, and the only way up is a different teacher.

Result (2026-10-14, software reservoirs and the ridge penalty): two new
conditions with no game in them. `random-tanh` is a fixed random
projection of the 360-byte window to 2,048 tanh units; `echo-state` is
the same with a random recurrent matrix at spectral radius 0.9 and leak
0.5, so the state carries history the way Tetris's board does. Both are
read out by the same ridge. Tetris rows are left out (`--no-tetris`), so
each table takes four minutes, and the ridge penalty is swept because
2,048 features against 2,012 rows at the byte-tuned penalty of 1 overfit
badly (echo-state trained at 0.936 and held out at 0.805).

```
held-out accuracy      lambda 1     10    100   1000
clock                     0.808  0.808  0.786  0.782
raw                       0.854  0.860  0.863  0.863
random-a                  0.779  0.779  0.779  0.779
random-tanh               0.829  0.853  0.861  0.863
echo-state                0.805  0.844  0.856  0.855
placement-onehot          0.860  0.861  0.863  0.863
searched-onehot           0.867  0.866  0.866  0.867
```

Regularised properly, the nonlinear expansions converge on exactly raw's
number, 0.863, and nothing in the sweep clears 0.867. The expansion can
fit the training rows much better than bytes can, 0.936 against 0.871,
which proves it has the capacity, and that capacity buys nothing on a
held-out demonstration. So the ceiling is not the policy class either.
Bytes, bins, terrain, a 2,048-unit tanh expansion, an echo state network
with memory: all stop at 0.86 to 0.87 on the raw label, with floors at
0.78 and 0.81.

That leaves the teacher. The demonstrations come from a search that
jumps whenever a jump survives, and among equally surviving jumps it
picks by a seeded coin, so the phase of each jump relative to its
obstacle is a random variable that no function of the state can predict.
The fix is in the search, not the readout: break ties toward the path
with the fewest A presses, so every demonstration jumps as late and as
little as it can. Then the jump phase becomes a function of what is in
front of Mario, and a state readout has something to learn. That is one
line in `prune_and_rank` and a re-search of the six demonstrations.

Result (2026-10-14, the teacher, and where the errors are): the beam now
ranks by progress in 8-pixel buckets and then by the fewest A presses, so
each demonstration jumps as late and as little as it can. Six new
demonstrations, all clearing 1-1. It did not help: on the fast table at
penalty 100 every row came in a point or two under the old
demonstrations. So a diagnostic was written instead of another change
(`where_the_a_errors_are`, an ignored test): classify the raw readout's
held-out A errors by where they fall relative to the demonstrations'
presses.

Two things came out. The B and Right bits are 0.856 accurate for every
condition including the clock: they are the search's walk and wait
macros, chosen by nothing, and they are two thirds of the headline
number. And every press onset is missed, 120 of 120, by raw, by the
one-hot encoder and by the clock alike. The readouts never predict the
first step of a jump; they predict A once Mario is already rising. They
learned to recognise a jump, not to start one, which is why every one of
them dies at the first Goomba closed-loop. The table now reports the A
bit on its own, and the hand-picked encoder's enemy bins narrowed from
32 pixels to 8, a step's worth, in case resolution was the problem.

```
fewest-jumps demonstrations, raw label, penalty 100, A bit held out
condition          | train acc | held-out acc | A acc
clock              |     0.820 |        0.820 | 0.746
raw                |     0.854 |        0.846 | 0.827
random-a           |     0.783 |        0.783 | 0.634
random-tanh        |     0.865 |        0.846 | 0.837
echo-state         |     0.870 |        0.843 | 0.829
placement-onehot   |     0.846 |        0.843 | 0.816
searched-onehot    |     0.861 |        0.860 | 0.856
  ($9F vertical speed /8, $70 enemy page /8, $118 = terrain one column ahead, row 11: a pit detector /8, $F1 /4)
```

On the A bit the floor is 0.746, the majority class. Raw and the two
software reservoirs sit at 0.83, the searched encoder at 0.856, eleven
points over the floor. The search, now allowed terrain, picked a pit
detector on its third try. Narrowing the hand-picked enemy bins did
nothing, so resolution was not what the onsets were waiting for.

Where this leaves the idea. Twenty runs, and the reservoir question has
an answer: a game in the loop costs six points on the raw label whatever
is fed to it, and the echo state network, the thing Tetris was standing
in for, matches raw and no more. The imitation question has a sharper
one: a readout of the current state predicts that a jump is happening,
not that one should start, and the onset is the whole policy. Anything
that moves the onset number is a different experiment: a label that
marks onsets rather than presses, scored on the onset bit, or a
demonstrator that is itself a readable function of state. The video's
struggle chapter has its arc either way: six demonstrations, every
readout agreeing with them only once Mario has left the ground.

Result (2026-10-15, a teacher the readouts can read, runs 21 to 27):
the arbitrary jump phase was the ceiling, so the teacher was replaced.
`teacher.rs` jumps by a rule over the same window the readouts see: an
enemy within some pixels ahead on Mario's floor, or a pit or wall within
some columns, while Mario is on the ground; A is held a few steps and
the rule cannot fire again until he lands. The four numbers are searched
over a small grid by closed-loop progress from the six offsets. The
winner, an enemy within 40 px (hold 3) or an obstacle within 3 columns
(hold 5), clears 1-1 from all six, so every press onset is now a
function of the state by construction. The table gained an `onset`
column: the fraction of the held-out demonstration's press onsets at
which the readout pressed A. Starting a jump, not recognising one.

The first run on the rule teacher (ridge, penalty 100) raised every
held-out accuracy by eight to ten points and left the onset column at
zero for raw, one-hot and clock alike, with every episode still dying at
x 314. A diagnostic of the raw outputs at onsets explained it: the raw
readout's A output at a press onset was 0.200 and at the step before it
0.187, which is no signal at all; the one-hot encoder's was 0.277
against 0.121; the echo state's 0.326 against 0.264. Two things were
wrong. Bytes cannot express "any of five enemy slots within 40 px",
which is a minimum and a threshold, so the information is in the window
and not in the linear span of it. And least squares on 0/1 targets puts
a rare conjunction (157 onset rows in 2,419) under the 0.5 threshold
even when it is separable, because the easy rows outweigh it. The
second was fixed two ways: a logistic readout (`logistic.rs`, Newton
steps on the ridge solver, `--logistic`), and the ridge penalty dropped
to 1. The first was helped by adding the nearest-enemy distance, a
derived feature, to the encoder search pool.

```
rule teacher, raw label, penalty 1, with Tetris (run 25); logistic columns from run 23
condition          | held-out acc | A acc | onset | mean x | max x | resets || logistic: A acc | onset | mean x
clock              |        0.914 | 0.743 | 0.465 |  778.4 |   899 |      0 ||           0.703 | 0.159 |  743.2
raw                |        0.956 | 0.869 | 0.197 |  313.6 |   314 |      0 ||           0.858 | 0.127 |  313.6
random-a           |        0.866 | 0.598 | 0.274 |  390.0 |   691 |      0 ||           0.598 | 0.274 |  390.0
random-tanh        |        0.961 | 0.883 | 0.471 |  594.2 |   595 |      0 ||           0.878 | 0.389 |  562.2
echo-state         |        0.976 | 0.928 | 0.701 | 1572.4 |  1796 |      0 ||           0.924 | 0.554 |  781.6
placement-onehot   |        0.951 | 0.853 | 0.293 |  434.0 |   434 |      0 ||           0.861 | 0.420 |  434.0
placement-tetris   |        0.939 | 0.816 | 0.325 |  689.0 |   709 |   1083 ||
searched-onehot    |        0.976 | 0.928 | 0.108 |  313.6 |   314 |      0 ||           0.964 | 0.637 |  313.6
searched-tetris    |        0.936 | 0.809 | 0.172 |  313.6 |   314 |    772 ||
placement-hash     |        0.882 | 0.645 | 0.070 |  473.8 |   723 |    846 ||
  searched encoder, ridge: $9F /8, $CE /4, $12F /8, $9B /8; logistic: $9F /8, $CE /4, $116 /4, $130 /8
```

Read the onset and mean-x columns; the B and Right bits are constant
under this teacher, so the headline accuracy is inflated and the A
column is the honest one. Three things happened.

The echo state network starts seven in ten held-out jumps and runs to a
mean x of 1572, five times the Goomba wall at 314 and more than half the
level, dying at the Goomba pair near x 1800. The memoryless tanh
expansion gets 0.471 and 594; raw bytes 0.197 and 314. So on a teacher
that can be read, the policy class matters after all, and memory
matters: the echo state's recurrence is the only thing in the table
that carries the previous step, and a jump is a decision about the
previous few steps as much as this one.

Tetris beat its own control for the first time. Through the game, the
readout starts 0.325 of onsets against 0.293 for the same four
placements read as one-hots, and its episodes reach 689 against 434,
past the first Goomba. Its held-out accuracy is lower (0.939 against
0.951) and it reset the board 1,083 times doing it, but the thing a
reservoir is for, turning an encoding into something a linear readout
can threshold, happened. The searched encoder shows the same ordering
on onsets (0.172 through Tetris against 0.108 one-hot) without reaching
the Goomba. The hash encoder, which destroys locality, is at the
random-A floor on both. Tetris is still a factor of two short of the
software echo state on onsets and more than that on distance.

The logistic readout helps the small encodings and hurts the large
ones. On the one-hot features it lifts onsets from 0.293 to 0.420 and
on the searched features from 0.108 to 0.637, the latter by picking two
wall tiles (one and three columns ahead, row 9) that are the rule's own
obstacle test; yet that readout still dies at 314 because none of its
four features is an enemy. On the 2,048-unit expansions the logistic fit
at penalty 1 is worse than ridge (echo state 0.554 against 0.701), so
the penalty is not matched to the likelihood scale there; at penalty
100 it was worse everywhere.

Two caveats. The clock readout now reaches 0.465 onsets and x 778,
because the rule teacher is deterministic and the six offsets differ by
at most six steps, so a one-hot of the step index memorises when the
level's jumps happen; it is a floor that only the echo state clears,
and the next protocol change is to spread the demonstration offsets
over a hundred frames or more so the clock has nothing to memorise. And
the greedy encoder search optimises held-out accuracy, which the hold
steps dominate, so it picks jump-recognition bytes first and the enemy
distance last and at the wrong width (4 px bins top out at 36 px, under
the rule's 40); scoring the search by onset recall, or by closed-loop
x, is the obvious follow-up.

What this says about the idea, 27 runs in: the reservoir question had
been answered on a teacher nobody could learn from. With a teacher that
is a function of the state, a game in the loop does add to its encoding
on the one measure that is the policy, and a software echo state adds
much more. The video's struggle chapter has its resolution: six
demonstrations from a rule, the readouts that only recognised a jump,
and the echo state and Tetris starting them.

Result (2026-10-15, a bank of four boards, runs 28 to 31): the
follow-ups above were done, and two of them found memorisers before
they found anything else. The demonstration offsets were spread over
120 idle frames (`noop_max`, a cache at other offsets is searched
again). The rule search on the new offsets kept the same trigger and
took a nine-step hold on obstacles over the five-step one, on a tie of
x sums; all six clear 1-1 in 551 to 568 steps. The encoder search was
rescored, twice. Scored by onset recall plus A accuracy it picked a byte
that pressed A nearly everywhere (0.696 onsets at 0.656 A accuracy),
because never pressing A already scores 0.7 on accuracy, so presses
were cheap. Scored by onset recall plus A specificity (always-A and
never-A both score one) over the old pool, it picked a terrain tile
minus Mario's x pixel, which is Mario's position within the page in
16-px bins, and on one fixed level that predicts where the jumps are:
0.717 onsets by memorising the layout. The pool now leaves out Mario's
x bytes and reads only the five enemy x bytes relative to him. With
that, the search picked the rule's own inputs in order: the wall row
three columns ahead (`$131`), the nearest-enemy distance at 8 px, the
wall row one column ahead, and the floor three columns ahead (the pit
test), reaching 0.935 held-out onsets on the one-hot control.

The bank (`bank-tetris`) is four Tetris boards, each steered by four
placements from its own part of the window: Mario himself (airborne
byte, vertical speed, y, x speed), the enemies (nearest distance and
three slots' x relative to him), the floor one to four columns ahead,
and the row above the floor over the same columns, which every pipe,
stair and wall fills. The readout sees the four playfields, 800 cells.
Its control (`bank-onehot`) is the same sixteen placements as one-hots,
so the bank's gain over its control is the boards and nothing else.

The clock had to be fixed too. Counted from the first button, with the
idle frames before it, every offset had the same step at every x,
because Mario starts in one place and nothing moves until he does; on
the new offsets the clock readout started 0.971 of onsets and cleared
the level (mean x 2902, max 3267). It now counts macro steps since the
level started, idle frames included, and is a floor again. Every other
row is deterministic and identical between runs 30 and 31.

```
rule teacher (hold 9 on obstacles), offsets over 120 frames, ridge penalty 1 (run 30; clock from run 31)
condition          | held-out acc | A acc | onset | mean x | max x | resets
clock              |        0.857 | 0.572 | 0.449 |  429.6 |   898 |      0
raw                |        0.973 | 0.920 | 0.007 |  312.4 |   314 |      0
random-a           |        0.841 | 0.524 | 0.420 |  387.2 |   685 |      0
random-tanh        |        0.987 | 0.960 | 0.384 |  837.4 |   898 |      0
echo-state         |        0.995 | 0.984 | 0.841 | 1399.6 |  1525 |      0
placement-onehot   |        0.978 | 0.934 | 0.348 |  434.0 |   434 |      0
placement-tetris   |        0.951 | 0.854 | 0.449 | 1126.0 |  1130 |   1102
searched-onehot    |        0.912 | 0.737 | 0.935 |  722.0 |   722 |      0
searched-tetris    |        0.943 | 0.829 | 0.746 |  432.4 |   898 |    864
bank-onehot        |        0.985 | 0.954 | 0.594 |  898.0 |   898 |      0
bank-tetris        |        0.981 | 0.943 | 0.855 | 1494.8 |  1790 |   4701
placement-hash     |        0.844 | 0.532 | 0.355 |  792.6 |  1524 |    959
  searched encoder: $131 /8, enemy /8, $115 /8, $132 /4
```

The bank starts 0.855 of the held-out onsets and its episodes reach a
mean x of 1495 and a best of 1790, which is level with the 2,048-unit
echo state (0.841, 1400, 1525). It does that from sixteen binned
features that carry no position; the echo state reads the whole window,
Mario's x included, and could be memorising the level the way the clock
did, so the bank's number is the cleaner of the two. The same sixteen
placements read as one-hots, with no game behind them, start 0.594 and
reach 898; the single hand-picked board starts 0.449 and reaches 1126.
So of the bank's gain over the old single board, going from four
features to sixteen is worth 0.348 to 0.594 and the boards are worth
0.594 to 0.855, and the boards added more over their encoding (0.26)
than the single board added over its (0.10). The boards reset 4,701
times, about once every five steps per board, so what Tetris remembers
is the last twenty pieces; a jump decision needs about that much.

The searched one-hot shows what a memoryless readout of the right four
features cannot do: it starts 0.935 of onsets and dies at x 722 in every
episode, with A accuracy 0.737 and specificity 0.815, because four
one-hots have no record of having pressed, so A stays down for as long
as the trigger is in view. The same features through one board start
0.746 and reach 898 at best. The hash control stays at the floor on
onsets. Raw bytes start 0.007 of jumps on this teacher.

What this says about the idea, 31 runs in: a bank of four Tetris games,
each watching its own part of Mario's world, is a reservoir that
matches a software echo state on this task, on an encoding whose
memoryless readout sits at 0.594. The next things to try are inside the
bank: fewer resets (a housekeeping piece per step placed to keep the
stack low, or reading column heights alongside the playfield), more
boards or a search for each board's four features within its region,
and a second level so no condition can memorise a layout.

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
