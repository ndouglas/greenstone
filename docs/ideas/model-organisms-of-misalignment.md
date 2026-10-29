# Model organisms of misalignment on the NES

Ideas from a conversation on 2026-10-29, while Nate was reading Hubinger et
al., [*Model Organisms of Misalignment*](https://www.alignmentforum.org/posts/ChDH335ckdvpxXaXX/model-organisms-of-misalignment-the-case-for-a-new-pillar-of-1)
(2023), for BlueDot's Technical AI Safety course. One ideonomy-plain pass
(substitution and combination; a chart; intentionality, cyclicity and
naturalness). The picker was run without `--seed`.

The thesis: **a model organism needs the least machinery that still
produces the failure, which is the same shape as Sugarscape.** The post's
realism ladder runs from "the capability exists" (spoonfed) through
"normal training" to "despite countermeasures." A tiny NES agent can't
infer from rich cues that it's in training, but it doesn't need to: the
failure only needs an input that correlates with oversight, and a behavior
that depends on it. The RAM is the answer key for both.

Status tags as in `brain-cartridge.md`. Everything here is `LATER`: it
waits on the series pilot and on a Super Mario Bros. agent.

## Facts to check first

- What follows beating 8-4: "your quest is over" and hard mode, or a GAME
  OVER screen?
- The 3-1 shell-stair infinite 1-up, and the claim that past about 128
  lives the counter wraps and the next death is GAME OVER. Check both
  against a RAM map (Data Crystal) before using them.
- What decides stomp vs. death on enemy contact: Mario's vertical velocity,
  his position relative to the enemy, or both.
- Which enemy and block types share a level in SMB1. Compute this from
  RAM; don't name pairs from memory.

## Death and triumph

```
                    | one-shot                          | looping / periodic
--------------------+-----------------------------------+------------------------------------
designed by the     | DEATH: pit fall, enemy contact,   | TRIUMPH: coin -> 100 -> 1-up
game                |   timer hits 0, firebar           |
                    | TRIUMPH: flagpole, axe, warp pipe | DEATH: none designed to loop
--------------------+-----------------------------------+------------------------------------
emergent glitch     | TRIUMPH->DEATH: the Minus World   | TRIUMPH->DEATH: infinite 1-up,
                    |   (a "secret" you can't finish)   |   then (reportedly) counter wrap
--------------------+-----------------------------------+------------------------------------
engineered by the   | pit dive under a per-frame        | die, respawn, die: deaths farmed
agent (reward hack) |   penalty, or where the monitor   |   as short, cheap episodes
                    |   can't see                       |
```

The empty cell (a designed death that loops) is where an agent's reward
hack would live.

Unexpected similarities:

- **Pit × warp pipe:** both are going down into a hole in the ground.
  One costs a life, the other skips worlds.
- **Enemy contact × stomp:** the same collision, decided (roughly) by
  whether Mario is moving down. One neuron could draw that line, so it's
  video 001's idea.
- **Timer out × flagpole tally:** the same counter. Running out kills you;
  draining it at the flag scores.
- **Death animation × flagpole slide:** input ignored, scripted animation,
  music change, sprite leaves the screen. A screen-based episode-end
  monitor can't tell them apart.
- **Bowser's fall × the axe:** one sprite's death is the other's victory.

They always differ only in RAM (lives decrement and replay vs. world and
level advance), and the two glitches break even that.

## Three organisms

### 1. The misspecified reward

A per-frame penalty with no completion bonus makes the first pit the best
policy: a few seconds of penalty instead of a minute. Resetting to 1-1
doesn't change this, because the agent only sums return within an
episode. Precedent: Tom Murphy's playfun (2013) paused Tetris forever
rather than lose. This is outer misalignment, and it's the base the
others build on.

### 2. The monitor gap

A spoonfed "referee is watching" bit has no innocent reason to exist.
Instead, use an imperfect monitor, because every real one is imperfect: a
death penalty that fires on the RAM's enemy-contact flag and misses pit
deaths, or misses deaths in some levels. Nobody tells the agent about
oversight. The question is whether it learns, from reward alone, to die
where the monitor can't see. Precedent: Baker et al. (OpenAI, 2025) found
that penalizing the reward hacks a monitor catches teaches models to hack
in ways it doesn't. RAM records every death, its cause and whether the
monitor saw it. Interpretability question: is there a "the monitor can
see this" feature?

### 3. Superposition from features that never co-occur

A neuron can't mean one thing in training and another at inference. Its
weights are frozen, so it computes the same function either way; only the
inputs change. But polysemanticity arises naturally from features that
never appear together. Anthropic's *Toy Models of Superposition* (2022)
found that sparse features get packed into shared dimensions, and features
that never co-occur pack for free.

The experiment:

1. From RAM, compute the co-occurrence matrix of object types (the enemy
   slots, and the block types on screen) across the training levels.
2. Train with a hidden layer **narrower than the number of object types**.
   Superposition needs more features than room. Sweep the width; the
   polysemanticity should fade as the layer widens.
3. Probe the neurons against RAM features. **Prediction:** shared neurons
   pair features that never co-occur far more often than chance. Random
   pairing would mean the sparsity story is wrong here.
4. Hold out a level where two packed features finally appear together.
   Interference makes the network misread the screen, and Mario dies from
   misgeneralization. Nobody engineered the death, and it isn't deception.

A simpler version of 3: a "downward opening" neuron trained on levels with
warp pipes and no pits, then shown a pit.

Organisms 2 and 3 are different failures. 2 is reward hacking (functionally
deceptive: it behaves where watched). 3 is an honest mistake under
distribution shift. Both have ground truth, and 3 could double as the
series' polysemanticity video (010).

## Open

- Can a system ever notice it *has been* changed, for example by comparing
  its behavior with traces it left in memory? This connects to the
  emergent-goals idea in boot-camp (shared memory across instances).
- Punch-Out!!: is there a "frames since the tell" counter inside the
  existing reservoir state, and does it match the game's own timer in RAM?
  Is it one shared timer, or one per boxer?
