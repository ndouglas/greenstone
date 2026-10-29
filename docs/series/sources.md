# Core sources

Works the series must incorporate and explore, chosen by Nate. Every
concept they cover is listed here. The "Series" column is where the
curriculum takes it up; "later" means beyond the pilot arc, and "decide"
means it hasn't been placed yet. Before any line cites one of these
works, check the claim against the work itself: these lists are
inventories, not substitutes for reading the source.

Inventories were taken on 2026-10-29: the BlueDot post and "Zoom In" from
their full text, the podcast from a partial read (see its note).

## 1. Introduction to Mechanistic Interpretability (BlueDot Impact)

Sarah (guest post), BlueDot Impact blog, 2024-08-19.
https://blog.bluedot.org/p/introduction-to-mechanistic-interpretability

| Concept | What the post says | Series |
|---|---|---|
| Mechanistic interpretability | Understanding a trained network's internal reasoning: how and why it produces its outputs | throughout |
| The black box | Inputs and outputs are visible; the layers between are largely unknown | 001 |
| Why it matters: decisions | Understanding reasons behind decisions (law, health, finance) can expose bugs and bias | decide |
| Why it matters: misalignment | Output checks can miss deception, sycophancy, reward hacking; interpretability is one path to detecting them | later |
| Steering | Isolating the parts responsible for a behaviour lets you switch them off or amplify them | later |
| Features | Meaningful, isolated concepts a network encodes, usually within one layer | 007, 008 |
| Circuits | Features connected across layers into more complex computations; a feature is what the model knows, a circuit how it comes to know it | later |
| Curves → dog ear → dog head | Worked example of features composing | later (CNNs) |
| Universality hypothesis | Different networks may converge on similar features and circuits; contested | later |
| Locating features | A neuron that fires on dogs may really respond to snouts or fur; isolating a feature takes repeated experiments | 008, 009 |
| Polysemanticity | One neuron, several features; one feature, several neurons | 010 |
| Superposition | A hypothesis for polysemanticity: more features than neurons, packed in | later |
| Safety and polysemanticity | A mostly harmless neuron may also carry something dangerous | later |
| Preventing superposition in training | Possible in toy models; likely costs performance at scale | later |
| Sparse autoencoders (SAEs) | A wider hidden layer trained to be sparse, so each unit tends toward one feature (e.g. 8,192 units to read a 512-unit layer) | later |
| SAEs at scale | Millions of features found in Claude 3 Sonnet (Anthropic, 2024) | later |
| Golden Gate Claude | Amplifying one feature made the model identify as the bridge | later |
| Open problems; caution about hype | Monosemanticity is unsolved; Amodei's "about 3%" estimate of understanding | later |

Works it cites: Zoom In (Olah et al., 2020); Feature Visualization (Olah,
Mordvintsev, Schubert, Distill 2017); Toy Models of Superposition
(Elhage et al., 2022); Towards Monosemanticity (Bricken et al., 2023);
Scaling Monosemanticity (Templeton et al., 2024); "Taking features out of
superposition" (Sharkey, Braun, Millidge, LessWrong 2022).

## 2. Zoom In: An Introduction to Circuits (Distill)

Chris Olah, Nick Cammarata, Ludwig Schubert, Gabriel Goh, Michael Petrov,
Shan Carter. Distill, 2020-03-10. doi:10.23915/distill.00024.001.
https://distill.pub/2020/circuits/zoom-in/

| Concept | What the article says | Series |
|---|---|---|
| Zooming in | Science advances when a tool lets it see a finer level (microscope → cells); interpretability might be at such a point | decide (framing) |
| Neurons and weights as objects of study | Treat single neurons, even single weights, as worth serious investigation | 007 onward |
| Three speculative claims | Stated strongly on purpose, as Schwann's cell theory claims were (one of which was wrong) | decide |
| Claim 1: features | Features are the fundamental unit; they correspond to directions; they can be rigorously studied | 007, 008 |
| Directions | A feature may be a neuron or a linear combination of neurons | later |
| The disagreement about features | Some think meaningful neurons are trivial, others that they're mistaken (texture, imperceptible patterns) | 008 |
| Curve detectors | Present in every non-trivial vision model examined; come in families spanning orientations | later (CNNs) |
| Seven arguments for a feature | Feature visualization, dataset examples, synthetic examples, tuning curves, and three from circuits: how it's implemented, how it's used, a hand-written version (figure content; verify against the curve-detector articles) | 008, 009 |
| High-low frequency detectors | An unexpected but simple feature: low frequency on one side, high on the other; helps find object boundaries | later (CNNs) |
| Pose-invariant dog head | A high-level feature, shown with feature visualization, dataset examples and synthetic 3D renders | later (CNNs) |
| Feature visualization vs. dataset examples | Visualization shows what causes firing; dataset examples show what it fires on in practice | 009 |
| Polysemantic neurons | E.g. one neuron for cat faces, car fronts and cat legs; a major obstacle (5 meanings × 5 meanings = 25 tangled connections) | 010 |
| Claim 2: circuits | Features connected by weights form circuits that can be studied; you can read algorithms off weights | later |
| Positive and negative weights | Excite or inhibit the later neuron | 001 (sign of a weight), 007 |
| Curve circuit | Curves built from earlier curves at each point along the curve | later (CNNs) |
| Equivariance | The symmetry of a problem shows up as symmetry in the weights | later |
| Unioning over cases | Left-facing and right-facing dog heads detected separately, then combined; with mutual inhibition, XOR-like | later (and 004 for XOR) |
| Superposition (cars) | A pure car feature spread across dog neurons; works because cars and dogs rarely co-occur; high-dimensional space has exponentially many almost-orthogonal directions | later |
| Circuit motifs | Recurring patterns (equivariance, unioning, superposition), as in systems biology | later |
| Claim 3: universality | Analogous features and circuits form across models and tasks; Gabor filters in first layers; evidence so far anecdotal | later |
| Universality and biology | Artificial features may match, or even predict, biological ones | later |
| Interpretability as natural science | Pre-paradigmatic (Kuhn); circuits are falsifiable: understand one and you can predict the effect of editing weights | decide (framing; ties to our save-state edits) |
| Glossary | Circuit, motif, client neuron, direction, upstream/downstream, family, feature, meaningful/pure/polysemantic/universal feature, representation | throughout |

## 3. Neel Nanda on the race to read AI minds (80,000 Hours podcast, part 1)

Rob Wiblin with Neel Nanda (Google DeepMind). 80,000 Hours, 2025
(recorded July 2025).
https://80000hours.org/podcast/episodes/neel-nanda-mechanistic-interpretability/

**Note:** this page is long, and only about a third of it was read word
for word. Items marked † come from a summary of the rest and must be
checked against the transcript before use.

| Concept | What the episode says | Series |
|---|---|---|
| What mech interp is | Using a model's internals to understand it; the numbers are all visible, their meaning isn't | 001 |
| Not a black box | Parameters and activations can be read; the challenge is meaning | 001 |
| Biology of AI | Training, like evolution, produces complex systems with no aim of being understandable | decide (framing) |
| Partial understanding | Full reverse-engineering is unlikely; partial understanding is still useful (evaluation, monitoring, incident analysis) | decide |
| Causal intervention / activation patching | Change one part, watch the output; easy in networks, impossible in brains | later (our save-state edits) |
| Interpretability won't reliably find deception | Nothing can; interpretability belongs in a portfolio | later |
| ROME and factual recall | Facts stored like a database in early layers; edits can make "the Eiffel Tower is in Rome" | later |
| Competing explanations | The same behaviour can come from different mechanisms; easy to be overconfident | 002 onward (pressure tests) |
| Unlearning as suppression | Unlearning adds a suppressor rather than removing knowledge | later |
| Start with the simplest technique | E.g. shutdown resistance explained by reading the chain of thought | throughout (method) |
| Chain-of-thought faithfulness and monitorability | Stated reasoning may not drive the answer; can a model scheme without showing it? | later |
| How chain-of-thought monitoring could break | Thinking in numbers, scheming without a scratchpad, noticing monitoring, shorthand | later |
| Evaluation awareness | Models can sometimes tell they're being tested | later |
| Limited serial depth | About one step per layer; chains of thought extend it | later (transformer) |
| Training against a probe | May hide the behaviour rather than remove it | later |
| "Wrong level of analysis" objection | Neurons may be too fine-grained; every level has value, the field should diversify | decide |
| Abstract directions | E.g. a direction for "this is true" | later |
| SAE intuition and assumptions | Concepts are directions; concepts are sparse; sparsity as a proxy for interpretability | later |
| Unsupervised vs. supervised | SAEs find concepts without labels; probes need a target and miss what's not on the list | 008 (probes), later (SAEs) |
| Single neurons don't have clear meanings | Hence the need to find directions | 010 |
| SAE lineage and examples | 2021 and 2023 papers; Base64 features; Golden Gate Claude | later |
| SAE limits | Sparsity isn't interpretability; imprecise; feature absorption ("starts with E but isn't elephant"); Matryoshka SAEs | later |
| SAEs on real tasks† | Simple probes beat SAEs at detecting harmful intent and generalise better; SAEs good for generating hypotheses | later |
| Auditing games† | Models with hidden goals as benchmarks; secret-word recovery | later |
| Lessons from the hype† | Flashy results, missing baselines, selection bias | throughout (pressure tests) |
| Understanding vs. control† | An x-ray: strong at diagnosis, weak at changing behaviour | decide |
| Research philosophy† | Simple baselines first; ground truth; expect most hypotheses to be false | throughout (method) |
| Universality in toy models† | A series of papers revising a group-composition hypothesis | later |

## 4. The Misguided Quest for Mechanistic AI Interpretability (AI Frontiers)

Dan Hendrycks and Laura Hiscott. AI Frontiers, 2025-05-15.
https://newsletter.ai-frontiers.org/p/the-misguided-quest-for-mechanistic

The series' main critic, and the one it answers (see the bible's "Why
look inside at all?"). Disclosure worth making on screen: Hendrycks
co-created representation engineering, the alternative the article
proposes.

Thesis: mechanistic interpretability assumes a complex system can be
understood bottom-up, like clockwork; after a decade it has produced no
lasting results, and top-down, representation-level work should take
priority.

| Argument | What the article says | Series |
|---|---|---|
| The clockwork premise | "Mechanistic" assumes nuts-and-bolts explanations, which may not suit AI | 001 (why), the critique video |
| Complex systems resist reduction | Meteorology, biology and psychology work with higher-level patterns, not every molecule or neuron | the critique video; the NES as a two-level system |
| Emergence | Many weak nonlinear connections make the whole more than its parts; bigger is more capable and less transparent | the critique video |
| We can't explain ourselves | Polanyi's paradox: we know more than we can tell (choosing a parking spot) | the critique video |
| Brevity vs. faithfulness | Terabytes of weights won't fit a paragraph-long explanation; if they did, the model could be far simpler (Hinton, 2018) | 002 onward (what a line can't say) |
| Compression loses the long tail | Simplified explanations drop the exceptions that matter for safety (Rehder) | 002 (when the line lies) |
| Feature visualization is unreliable | Visualizations are abstract and selected; natural examples explain activations better (arXiv 2010.12606) | 009 |
| Saliency maps fail sanity checks | Explanations barely change when the model's weights are randomized (arXiv 1810.03292) | later (saliency) |
| Single-neuron stories are fragile | A BERT neuron's apparent role vanished on another dataset (arXiv 2104.07143) | 008, 010 |
| It doesn't scale | Months of work to find a partial circuit in Chinchilla 70B, broken by a small format change (arXiv 2307.09458) | later |
| SAEs disappointed | Hard to get working; DeepMind deprioritized them; beaten by a simple baseline at detecting harmful intent | later (SAEs) |
| Little return | A decade and millions of dollars for "roughly nonexistent" returns | the critique video |
| The alternative: top-down | Work at the level with the most invariances (Gell-Mann); representation engineering studies patterns across many neurons | later (a representation-engineering video) |
| Concessions | Some understanding is needed; an "MRI for AI" would be useful; mechanistic work needn't stop, but should be judged by its record | the critique video |

The arXiv papers above are cited by number in the article; check each
one's title and authors before citing it.

## Gaps

- The middle and end of the podcast transcript are summary-only (†).
- Part 2 of the episode is out of scope for now.
