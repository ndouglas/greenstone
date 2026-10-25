# Punch-Out!! reservoir film design

A chaptered video of the Reservoir Punch-Out!! experiment
(`docs/superpowers/specs/2026-10-21-punchout-reservoir-design.md`,
results in `docs/ideas/brain-cartridge.md` section 1a), the sequel to the
Tetris reservoir film (`docs/superpowers/specs/2026-10-17-reservoir-film-design.md`,
built by `examples/reservoir_film.rs` and `tools/reservoir_film/`).

## Intent

The audience is people with some machine-learning experience, possibly
AI researchers. The film is not a show of mastery; Nate says he doesn't
have it. It shows a habit: finding odd alternative ideas and poking at
them for fun. The Tetris film felt most like him because it was "odd and
ineffective". In Nate's words, the point is not "let's adopt a proven
technique and see how simple it is"; it is "let's do weird crap and see
what happens". It's mad science, but explained.

What's new over the Tetris film is theory. This film says what the
approach is, why it's weird, and how it differs from mainstream deep RL,
in terms an ML viewer would recognise.

Success: an ML viewer can say afterwards what each part of the brain is,
what a mainstream agent would have in its place, and what happened when
the weird version met the game. They should also enjoy it and find it a
bit funny.

The thing Nate most wants remembered is the Tetris games used as a
brain's memory, and more broadly anything that sets this approach apart
from typical learning.

## Goals

- Explain every component of the approach against its mainstream
  counterpart, using an animated diagram that builds up over the film.
- Show the experiment as it went, failures included, from real footage.
- Keep the Tetris film's tone: curious, honest, a little absurd.

## Non-goals

- Making the brains better before filming. Nate: "I'm more interested in
  trying new things than in creating a masterpiece here." No retraining,
  no widening of the bank's window, no velaryon runs.
- Teaching ML basics. Viewers know what a conv net, a policy, a reward
  and gradient descent are, so the film uses those terms without defining
  them. Terms specific to this project (reservoir, echo state network,
  readout, ridge, DAgger, held out, the bank) each get a sentence before
  they're used.

## Anti-goals

- Selling the approach as a simpler, proven alternative. It isn't one,
  and the film never implies it is.
- Hiding or softening a failure. The bank stands still; nobody wins a
  fight; DAgger made stack4 worse against Bald Bull. These stay on screen
  with the reason when there is one.
- A wall of theory with no game on screen. Theory sits next to footage
  or clips wherever possible.

## The explaining device: the build-up diagram

Nate chose this treatment over a static diagram card and over live
insides beside the fight.

The diagram is a row of boxes, left to right: what the brain sees, what
it remembers, how it decides, what it learns from. In chapter 1 it
appears in its mainstream form:

    screen -> conv net (learned) -> policy (learned) <- reward, ~10^7 frames

Over the film, each box is swapped for this experiment's version, one
piece at a time, just before the chapter that tests that piece. Each swap
shows the mainstream box and the weird box side by side for a moment,
then the weird one takes its place. Small inset clips show the piece
working, or not. Learned parts and frozen parts are told apart at a
glance (the mockup used orange for learned and dashed blue for frozen;
the exact visual language is the builder's call). The finished diagram
comes back in the verdict.

The pieces, and the facts each must carry (check every one against the
code before it goes on screen):

1. **Seeing.** The 84x84 grey screen with the HUD cropped off (5,712
   pixels) is multiplied by a fixed uniform random matrix to give 256
   numbers. The matrix is rolled once from a seed and never trained
   (`src/nes/gym/punchout/brains.rs`, `Projection`). Mainstream: a conv
   net learned end to end.
2. **Remembering.** One of three memories, each picked by hand and none
   trained. Mainstream: a learned recurrent net, or a fixed frame stack
   feeding a learned net.
   - `stackN`: the last N projections side by side. A brain acts once
     per 4-frame step and sees only the step's last frame, so stack4
     spans 12 frames.
   - `echoL`: a 2,048-unit echo state network with random fixed weights
     (spectral radius 0.9). The update is `h = (1-a)h + a*new`, so a
     *higher* leak forgets *faster*.
   - `bank`: four Tetris games. Bytes made from the projection are
     poked into four running copies of Tetris, and the boards they build
     are the memory. This is the headline. It also needs its own
     honest beat (see chapter 4).
3. **Deciding.** One linear readout per move, fitted to one-hot teacher
   labels by ridge regression in a single solve; the move with the
   highest score is pressed. Mainstream: millions of gradient steps.
4. **Learning from.** No reward signal. A teacher with save states tries
   each move for 60 frames and labels the one that goes best. Then DAgger
   runs four rounds: the student fights, the teacher labels the
   student's own situations, and the readout is refitted.
   Mainstream: a reward signal and exploration.
5. **The yardstick.** A `cheat` brain gets all 2,048 bytes of RAM
   instead of the screen, as the reference no screen-reader should beat.
   It doesn't turn out to be an upper bound.

## Story

Eleven chapters, agreed with Nate as a structure; the details inside
each are the builder's within the facts above. All numbers come from
the ideas doc section 1a or from `target/punchout/phase*/results.json`
and `sweep.json`. The film never contradicts the doc.

The doc doesn't yet report the teacher's own fights. Before the film
uses them, add a short paragraph and table to section 1a from
`target/punchout/phase{1,2,3}/teacher.jsonl`. In those records, fight
`index` 0 is the pure teacher, and indexes 1 and 2 play a tenth of their
moves at random. The pure teacher's fights, for checking:

    phase 1, Defence   GJ 0 hits   VK 0 hits   PH 12 hits   (no punches, no wins)
    phase 2, Counter   GJ 0 hits, 4 knockdowns, no win at the cap
                       VK 0 hits, KO in 1,217 steps   PH 0 hits, KO in 1,389
    phase 3, Fight     GJ 0 hits, KO in 1,018   VK 0 hits, KO in 801
                       PH 0 hits, KO in 817

Claims on screen about "the teacher" mean the pure teacher.

1. **The cartridge boots.** Punch-Out!! starts on Greenstone, with the
   MMC2 (mapper 9) aside: it was built for this. Then the mainstream
   diagram appears and the premise: swap each box for something
   stranger and see what happens.
2. **Always-Idle gets flattened by Glass Joe.** The floor: standing
   still takes 20.98 hits a minute from Glass Joe, and more from the
   others.
3. **The teacher dodges everything, then wins.** *Piece: learning
   from* (the teacher half; DAgger waits for chapter 7). The teacher
   fights with save-state lookahead. In Defence it dodges, taking no hits
   from Glass Joe or Von Kaiser. In Fight it knocks out all three
   training opponents, three knockdowns each, taking no hits; show a
   KO. Nate chose to keep the knockouts in. Mention that its labels are
   mostly Idle (96.7%, 84.7% and 85.3% by phase), because that matters
   later. The students copy this teacher; that's the setup for
   everything after.
4. **Building a brain.** Mostly theory. *Pieces: seeing, remembering,
   deciding.* The bank gets the close-up: projection bytes dropping
   Tetris pieces, four boards as memory. Then the honest beat: each
   byte the bank feeds Tetris averages 16 projection numbers, which
   squeezes the spread so every piece lands in columns 3 to 6. The boards
   change every step but look alike whatever Glass Joe is doing, so
   the bank barely tells moments apart. That's a plumbing choice, not
   a test of the idea, and the film says so.
5. **Defence.** echo0.5 sidesteps; the bank stands there, identical
   to always-Idle, as chapter 4 predicted. The fights shown are the
   fully trained (round 4) brains, called simply "trained" here; what
   the rounds are waits for chapter 7.
6. **The memory sweep, side by side.** stack1 to stack8 and echo0.2 to
   echo0.9 on the same opponent. Crisp history helps on trained
   opponents; long blurry memory doesn't. Tie it to the question of how
   long a tell lasts against how much the brain remembers, and to the
   blind spot: a brain sees one frame in four.
7. **Counter: stack4 versus Von Kaiser.** *Piece: DAgger.* Counter is
   where stack4 comes closest to perfect (0 to 2.5 hits a minute on
   trained opponents where idle takes 21 to 38).
8. **Fight knockdowns.** stack4 knocks Glass Joe down twice: 277
   damage, no hits taken. No student wins within the 2,400-step cap,
   though the teacher they copied won every Fight in 800 to 1,020 steps.
9. **Held out: Bald Bull, round 0 versus round 4.** More teaching made
   stack4 worse against an opponent it never trained on (Counter 12.77
   to 22.76 hits a minute). echo0.5 got better against him, which is
   worth a sentence.
10. **The cheat versus the screen.** *Piece: the yardstick.* RAM under a
    linear readout takes more hits than stack4 and echo0.5 from every
    trained opponent in every phase.
11. **Verdict and credits.** The finished diagram, what each weird part
    did, the gap between a teacher that wins and students that don't,
    and the open questions: widening the bank, feeding all four
    frames of a step.

About 12 to 15 minutes. That's a guess Nate called reasonable, not a
requirement.

## Principles (inherited from the Tetris film)

- One idea per moment. Show, then say.
- Text stays up long enough to read twice (the existing rule in
  `tools/reservoir_film/model.py`: at least 4 s per 15 words, never under
  3 s).
- After every experiment, a few seconds of music and action with no
  text.
- Experiments are real: every fight shown is recorded from the emulator.
  Diagrams and the build-up are animated; footage is not.
- Methods and sample sizes live in the ideas doc, not in captions. One
  sentence of caveat stays on screen when it changes the meaning (for
  example: the opponents are scripted, so one fight per opponent is the
  whole test).
- Scored on Greenstone's APU, as before. No narration and no game audio.

## How it gets made

These are my calls (Claude's), made to build on what exists. Nate hasn't
weighed in on them.

- **Reuse the Tetris film's pipeline.** Its compositor
  (`tools/reservoir_film/`: `model.py`, `draw.py`, `build.py`, the music
  engine) gets reused by parameterising or factoring out what is
  hard-coded to Tetris: the takes directory, the output name and title,
  the chapter list, cues and credits. The Punch-Out!! film gets its own
  chapters and cues, and builds to
  `target/punchout-film/punchout-reservoir.mp4` plus `chapters.txt`.
  The Tetris film must still build unchanged. That means it builds, its
  tests in `tools/reservoir_film/tests/` pass, and its chapters look
  the same by eye. It doesn't have to be byte-identical, and a full
  re-render after a refactor is acceptable. (My call: Nate isn't
  re-publishing the Tetris film, so pixel drift costs nothing.)
- **Takes come from `examples/punchout.rs`** and
  `src/nes/gym/punchout/takes.rs`, which already write the TakeWriter
  format (`punchout.rgb` plus `take.json`) to `target/punchout/takes/`.
  Thirty-one takes exist. Missing ones are recorded locally. They replay
  fitted readouts and need no training:
  - the teacher fighting (chapter 3);
  - the sweep brains (chapter 6);
  - round-0 readouts, refitted from the teacher's records alone
    (chapter 9);
  - stack4 against Von Kaiser in Counter (chapter 7; the existing phase 2
    stack4 takes are Glass Joe and Don Flamenco).
- **Takes record the brain's insides** where a shot needs them. Today
  `take.json` has per-step move, health and hit only. Chapter 4 needs at
  least the bank's four boards per step (as a frame stream or as board
  bytes) and the projection values. The readout's per-move scores would
  let the diagram's "deciding" box move with the fight.
- **New widgets** for the build-up diagram and its inset clips live in
  the shared `draw.py` or a Punch-Out!! module beside it.

## Left to the builder

- Captions, shot lists, timings, music cues and stings.
- Which existing take or new take each shot uses.
- The diagram's visual language, animation timing and where the inset
  clips sit.
- Whether to factor the shared compositor into a library or parameterise
  it in place.
- Which opponent and phase the sweep chapter shows, as long as its claim
  matches the sweep table.

## Constraints

- Never stage `src/main.rs` or `src/nes/bus/mod.rs`; leave `.claude/`,
  `AGENTS.md` and `.superpowers/` alone.
- Format Rust with `rustfmt --edition 2021 <files>`, never `cargo fmt`.
- Test with `RUSTFLAGS=-Awarnings cargo test --release --lib <filter>`;
  the compositor has its own tests under `tools/reservoir_film/tests/`.
- The repo is public. Commit on `main`; don't push without asking.
- Raw takes are large: 4 frames per step and 2,400 steps make 9,600
  frames, about 1.8 GB per take. Record only what the film uses.
