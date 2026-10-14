# Placement drive design

Tetris as a hidden layer. Mario's state is encoded as tetromino placements,
Tetris plays them with its own physics, and a readout turns the resulting
board back into Mario's buttons. The game's dynamics are the nonlinearity;
the board is the hidden activation, and the video can show it.

Phase 1 below is the build. Phase 2 is the roadmap it is designed to
support, where the encoder and the player are learned rather than fixed.

## Why

The current playfield drive overwrites the board every step, so Tetris
contributes four frames of a falling piece and nothing else, and its
column sits one point under raw. The leave-one-out table shows the real
bottleneck is the linear readout's encoding: a weighted sum of bytes
cannot express "the next enemy is close". A placement drive turns a few
quantised bytes into stack geometry, which is exactly the kind of
nonlinear, one-hot-like feature the readout lacks, and it does so through
the game rather than around it.

## Phase 1: fixed encoder, scripted player, linear decoder

### The channel

Each SMB step (four frames) sends `PIECES_PER_STEP = 4` placements to
Tetris. A placement is a target column `0..9` and a target rotation
`0..3`. The encoder derives one placement per salient feature:

| piece | column from | rotation from |
|---|---|---|
| 1 | distance to the nearest enemy ahead of Mario, 32-pixel bins, clamped to 0..9 (9 = none) | that enemy's screen y, four bands |
| 2 | distance to the second enemy ahead, same bins | its y band |
| 3 | Mario's screen y (`$00CE`), 24-pixel bins | airborne flag (`$001D`) and y-speed sign |
| 4 | Mario's x speed (`$0057`), signed, binned | Mario's state byte (`$000E`) low bits |

Enemy slots come from `trace::enemies`. The exact bin edges are constants
in one place and reported in the table header. "Nearest ahead" means the
smallest positive `enemy.x - mario.x`.

### The player

Tetris starts at a fast level so a piece drops in a few dozen frames.
First choice: select level 19 through the A-type menu (nine Right taps on
the level select, then Start with A held). If the menu script cannot be
made reliable within the boot budget, write the level byte (`$0044`) once
at boot and say so in the table header; it is a one-time setup
intervention, not a per-step one.

Steering is by controller only. Each frame while a piece is falling:

- if the piece's orientation (`$0042`) is not the target rotation, tap A;
- else if the piece x (`$0040`) is left of the target column, tap Right;
  if right of it, tap Left;
- else hold Down.

A tap is one frame pressed, one frame released. The piece is placed when
the play state leaves "falling" or the piece id changes. One placement is
bounded at `PIECE_FRAME_LIMIT = 120` frames; if the piece has not locked
by then, the step moves on. Game over resets the reservoir as now and
increments `resets`; a reset mid-step continues with the next piece.

Orientation ids and the x semantics of `$0040` are verified empirically
by a ROM-backed test before any of this is relied on; the spec lists them
as assumptions, not facts.

### Conditions

| condition | readout input | what it tests |
|---|---|---|
| clock | one-hot step | floor |
| raw | the 256-byte window | the existing baseline |
| placement-onehot | the four placements as one-hot columns and rotations, 56 features | what the quantisation alone gives |
| placement-tetris | Tetris RAM after the four placements | what Tetris's dynamics add to, or take from, that |
| placement-hash | Tetris RAM, placements from a hash of the whole window | that locality in the encoding matters |

The undriven, controller-driven and playfield conditions stay available
but leave the default table, which should fit on one screen.

### Measurement

Unchanged: six cached demonstrations, leave-one-demonstration-out, train
and held-out accuracy and mse over the active buttons, five closed-loop
episodes, reset counts. `resets` is now expected to be large and is a
reported property of the encoding, not a fault.

### Cost

At level 19 a piece takes roughly fifty frames including spawn delay, so
a step is about 200 Tetris frames. A 540-step demonstration is about
110,000 frames, six or seven minutes on one core; collection runs one
demonstration per thread. The whole table is under twenty minutes on the
cached demonstrations.

### Video

A seventh chapter: the encoder's four numbers, the pieces being steered
into place at speed, the board after the step, and the readout's A gauge,
beside Mario. The board should read as a picture of what is around Mario.

## Phase 2: learned encoder and player

The structure is a three-layer network whose middle layer is a running
game: encoder weights choose what to send, the player chooses where to put
it, Tetris applies a fixed nonlinearity with memory, and the decoder reads
the result. The emulator is not differentiable, so the learned parts are
small and trained by search.

1. **Encoder search.** Parametrise piece `k` as (window address, subtract
   Mario x or not, bin width, bin offset). Greedy forward selection of
   four such tuples by the placement-onehot condition's held-out
   accuracy, which does not need Tetris in the loop and runs in seconds
   per candidate. Then evaluate the chosen encoder through Tetris.
2. **Player search.** Give the player a small table from (feature value)
   to (column, rotation) per piece, and tune it by evolution against the
   placement-tetris held-out accuracy, then against closed-loop x. This is
   the stage where the player "learns to play Tetris a specific way".
3. **Piece shapes as a channel.** Overriding the next-piece byte gives
   seven shapes per piece, three more bits, chosen by a fifth feature.
   Flagged as an intervention in the table header if used.

Each stage keeps the one-hot control, so at every point the table says
whether Tetris added anything beyond the encoding.

## Out of scope

Pixels as input, a nonlinear decoder, training the decoder closed-loop.
