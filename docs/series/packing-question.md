# The packing question: a strand of the series

When one neuron responds to several things, why? This strand of the
series studies that question empirically, one video at a time, on NES
games. It is a core topic for the series (Nate, 2026-10-30): "a real
fork in the road for AI."

## The fork

Polysemanticity is the observation: a neuron fires for several seemingly
different things. Two explanations compete:

- **Superposition (packing):** the network packs unrelated things into
  one neuron because they rarely occur together, so they don't
  interfere. A neuron's mixture then says nothing about how the network
  thinks, and the network must be untangled (by a second model, such as
  a sparse autoencoder) before it can be studied. This is the case for
  doom, or at least for a much harder job.
- **Generalization (including recursive generalization):** things share
  a neuron because, to the network, they share something: a V shape, a
  pose, "iy" growing into "do it yourself". The mixture is a clue, and
  the work is building better microscopes. This is the case for networks
  being more beautiful, and more powerful, than we feared.

The prompt is Nick Cammarata's essay "Neural Networks Are Already Clean"
(Goodfire, 2026; https://already-clean.goodfire.com), which argues for
generalization: the canonical cat-and-car neuron was a V-shape detector
misread, the canonical superposition circuit was shared pose plus a
reward-hacked microscope, and InceptionV1's illegible last layer was a
learning failure that architecture fixed. The opposing case is at full
strength in Toy Models of Superposition (Elhage et al., 2022) and
Towards Monosemanticity (Bricken et al., 2023), whose toy models do pack;
the essay agrees they do, and says they live in a world (features that
are ends in themselves, nothing built on them) that real networks don't.
Which world our networks are in is the question.

The two explanations predict nearly opposite things, which is what
makes the question testable:

- **Squeeze a network:** generalization predicts neurons that get
  coarser; superposition predicts neurons that get scrambled.
- **Watch a neuron during training:** generalization predicts it spreads
  into nearby things; superposition predicts it picks up distant ones.
- **Remove the data's structure:** packing should appear when nothing is
  shared, and fade as structure returns.

## Why us

- **Ground truth:** in an NES game we know what every RAM byte means,
  so "related" and "unrelated" are checked against the game's own
  variables (and their known relationships: position and speed, health
  and the hurt timer) rather than guessed from pictures.
- **Small networks:** cheap enough to train many times, at many widths,
  with checkpoints kept, so we can watch neurons grow.
- **Controls only a lab can run:** we can change the data itself, for
  example destroying the relationships between variables while keeping
  each one's values.

## How the strand works (Nate, 2026-10-30)

- One video at a time, guided by inquiry: each experiment is chosen in
  light of the last one's result, not from a master plan.
- Each experiment runs well before its video's script, with time to
  catch flaws in the method before anything is said on camera. Each gets
  its own design document when it's next.
- Results can come out either way, and the video says what we found.
  Both sides are presented at full strength (the bible's rule on
  critics).
- Many games, many phenomena, many levels of abstraction (RAM, pixels,
  learned features), many kinds of network.
- The strand has no fixed place in the curriculum. Its first video can
  replace 010 ("one neuron, several meanings"), which would otherwise
  have presented the cat-and-car neuron as textbook polysemanticity.
- Training runs on velaryon through Kubernetes, never on the Pis.

## Experiments

Starting points, not commitments; each will gain controls and changes
when it's designed. Games are chosen from those in `roms2/` whose memory
we can read with confidence, with a mix of related and unrelated
variables, rare events, and plenty of varied play.

| # | Experiment | The question | Game(s) | Status |
|---|---|---|---|---|
| 1 | **Squeeze** | Train a small network on the game's memory at shrinking widths, two ways side by side: copying the memory back out (the toy models' world, every variable an end in itself) and predicting the next frame's memory (variables useful to each other). Read each neuron against the known variables: one variable, related ones blended, or unrelated ones packed? Do neurons get coarser or scrambled as the network shrinks? | Castlevania (position and speed, health, hearts, score, timer, stage, enemy slots, whip level; its memory used in 001) | planned, first |
| 2 | **Grow** | Keep checkpoints from experiment 1's runs and watch each neuron form: does it spread into nearby things or distant ones? | Castlevania (the same runs) | planned |
| 3 | **Structure dial** | Shuffle each variable through time independently, keeping each variable's values but destroying the relationships between them, and train again, at several strengths. Does packing rise as structure falls? If neurons stay as clean with nothing to share, generalization can't be what kept them clean. | Castlevania | planned |
| 4 | **Rare events** | The essay predicts a rare fact rides on neurons related to it, not unrelated ones chosen because they rarely collide. | Punch-Out!! (stars, knockdowns, the knockout) | planned |
| 5 | **One network, many games** | Two games are never on at once, so for superposition they are the perfect pair to pack. Does a "falling" neuron serve all three platformers (one shared idea), or pair Mario's falling with something unrelated in Tetris? The cat-and-car question built on purpose. | Super Mario Bros., Super Mario Bros. 3, Castlevania, plus Tetris | planned |
| 6 | **Pixels** | The same question a level up: a small vision network on the screen, its neurons read against the memory we know. | Tetris, then Super Mario Bros. | planned |
| 7 | **Architecture** | The essay's InceptionV1-to-ConvNeXt claim, scaled down: the same task with and without residual connections and LayerNorm. Does architecture alone change how clean the neurons are? | experiment 1's data | planned |
| 8 | **Our own microscope** | In every video, at least one result checked for whether our tools fooled us: negative poles, weak activations, enough context and breadth of play. | all | standing rule |
| — | **A planted packed code** | Install packing by hand and see whether training keeps it (the essay tried this in InceptionV1). | if the results call for it | idea |

## Tools

The essay's lesson is that "polysemantic" often meant a poor
microscope, so the tools are part of the science, and they pay off
across every experiment. Built as the experiments need them, reused
throughout, and in time shown on screen:

- **The neuron microscope.** Browse every neuron of a network. For
  each: its strongest *and* most negative moments (the negative pole
  matters about as much as the positive: the essay's V has an O
  opposite it), its activation range in bands (strong vs weak), and,
  our advantage, its correlation with every known game variable and
  derived one (speeds, timers, events). Moments are game states we can
  replay in the emulator and show as footage.
- **Connections.** A neuron's strongest inputs and readers, line width
  as strength, so a neuron is read through what builds it and what uses
  it.
- **Shared-input view** (the essay's "generic feature visualization"):
  cluster what feeds a neuron and show what each cluster has in common;
  the shared pattern, not the list of nouns.
- **The training map.** One neuron followed through every checkpoint:
  what it responds to at each stage of training, so a branch can be
  seen growing (experiment 2).
- **Attribution and ablation.** Which neurons carry a given prediction;
  remove them and see what the network predicts instead (the essay's
  four-neuron diagnosis hub; its Santa on a negative pole).
- **Steering.** Turn a neuron up or down and see what the network now
  predicts the game will do next.
- **Explanations that can lose.** Every label is tested where it could
  fail: one agent writes the explanation, another constructs or finds
  game states it predicts should fire the neuron hardest ("blind
  maximum"), plus near misses written to fool it; a neuron with several
  bundles of input gets a test per bundle.
- **The dictionary check.** Train a sparse autoencoder on a layer and
  ask whether it finds anything the neurons lack (the essay: 88% of its
  activity near-copies of single neurons, the rest removable without
  changing an answer); and search the layer's rotations for the one
  where a few units carry the most activity, to see whether it lands
  back on the neurons.
- **Sign regimes,** for networks with gated units: show a neuron's four
  sign combinations separately, since a double negative can fill its
  noisy middle.
- **Feature visualization,** for the pixel experiments: synthesize the
  screen a neuron wants, with transformation robustness, watching for
  the reward hacking the essay found (the optimizer falling into the
  easiest path).

## Open questions

- How "related" is defined: by the game's own semantics (what the bytes
  mean), by the data (variables that move together), or both, compared.
- How to score a neuron without a single cleanliness number that hides
  the difference between generalization and packing (the essay's
  warning about top-k scores).
- What gameplay to record for each game, so rare and varied states are
  covered.
