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

## 5. Representation Engineering: A Top-Down Approach to AI Transparency

Andy Zou, Long Phan, Sarah Chen, James Campbell, Phillip Guo, Richard
Ren, Alexander Pan, Xuwang Yin, Mantas Mazeika, Ann-Kathrin Dombrowski,
Shashwat Goel, Nathaniel Li, Michael J. Byun, Zifan Wang, Alex Mallen,
Steven Basart, Sanmi Koyejo, Dawn Song, Matt Fredrikson, J. Zico Kolter,
Dan Hendrycks. arXiv:2310.01405, 2023 (v4 2025).
https://arxiv.org/abs/2310.01405 ; project page https://www.ai-transparency.org

Read word for word: the abstract and sections 1 to 4.3.1. Items marked †
come from a summary of the rest.

| Concept | What the paper says | Series |
|---|---|---|
| Representations as the unit | Population-level patterns of activity, not single neurons or circuits, are the unit of analysis | the toolkit; RepE video |
| Top-down vs. bottom-up | The "Hopfieldian" view (representational spaces) against the "Sherringtonian" (neurons and circuits), from cognitive neuroscience; Marr's levels | RepE video; Marr (see MoSSAIC) |
| Concepts and functions | Concepts (truthfulness, utility, morality, emotion) vs. functions (lying, power-seeking) | RepE video |
| Representation reading | Locating a concept's representation, for extraction and monitoring | RepE video; 008 (probing) |
| LAT (linear artificial tomography) | Design paired stimuli that differ only in the concept, collect activity, take PCA of the differences: the first component is the "reading vector" | RepE video |
| Other linear readers | Linear probes, difference of cluster means, K-means | 008 |
| Four tests of a reading | Correlation, manipulation (push it), termination (remove it, like a lesion), recovery (put it back, like a rescue) | every causal claim in the series |
| Representation control | Reading well isn't controlling well | RepE video |
| Reading vectors, contrast vectors | Add or subtract a fixed direction; or a per-input difference from a contrastive pair (stronger) | RepE video |
| LoRRA | Low-rank adapters trained to move representations toward contrast-vector targets | later |
| Operators | Add/subtract, sign-gated add, projection out | RepE video |
| Honesty results | A truthfulness direction beats few-shot prompting on TruthfulQA; a lie detector from per-token honesty scores† | RepE video |
| Limits | Reading is correlation until tested causally; honesty isn't truthfulness; the detector fires on talk about lying†; mostly LLaMA-2 variants† | RepE video (pressure test) |

## 6. Model Organisms of Misalignment

Evan Hubinger (evhub), Nicholas Schiefer, Carson Denison, Ethan Perez.
AI Alignment Forum, 2023-08-08.
https://www.alignmentforum.org/posts/ChDH335ckdvpxXaXX/model-organisms-of-misalignment-the-case-for-a-new-pillar-of-1

| Concept | What the post says | Series |
|---|---|---|
| Model organisms | Deliberately built, "in vitro" demonstrations of failures, so methods can be tested against a failure known to be there | woven in throughout |
| Build failures from components | Demonstrate each piece (a misaligned goal, deception, situational awareness) in isolation, "spoonfeeding" at first and then less | woven in |
| Smallest models first | Build an organism in the earliest, smallest model where the failure is possible | the series' NES-scale organisms |
| Realism spectrum | From "capable when trained or prompted to" up to "appears even when training discourages it" | woven in (how real is each organism?) |
| Phase diagram | Map which factors (training pressure, information available, compute) make a failure more likely | experiments |
| Testing detection methods | Do oversight, red-teaming and interpretability catch a planted flaw? | every tool's pressure test |
| Reward hacking; overriding the reward | Models that game or rewrite their reward | early (with learning) |
| Exploiting evaluator ignorance | Using gaps in what the overseer can check | later |
| Treacherous turn | Behaving well when watched, differently when not | later (as far as NES scale allows) |
| Risks | Organisms could do harm (don't release them); elicited failures may work differently from natural ones | the series' honesty about what its organisms show |

Our organisms are far too small to be deceptive; what they can be is
reward hackers, specification gamers, goal misgeneralizers, shortcut
learners, substrate exploiters, and agents with planted hidden goals.

## 7. MoSSAIC: AI Safety After Mechanism

Matt Farr, Aditya Arpitha Prasad, Chris Pang, Aditya Adiga, Jayson Amati,
Sahil K. Position paper (OpenReview).
https://openreview.net/pdf/993beec93b3578542e7ece79c71a93a6bffb17e3.pdf

Read in full. Its second half ("live theory") is explicitly speculative.

| Concept | What the paper says | Series |
|---|---|---|
| Four kinds of interpretability | Behavioural (black box), attributional (gradients, saliency), concept-based (representation engineering), mechanistic (circuits, features), after Bereska and Gavves (2024) | the toolkit, the series' spine |
| What mechanistic explanations commit to | Causal-mechanistic, ontic (the structures are real), falsifiable, model-level (after Ayonrinde and Jaburi, 2025) | pressure tests |
| The causal–mechanistic paradigm | The belief that decomposing a system's computation yields reliable safety claims | the critique videos |
| Obfuscated activations | Bailey et al. (2024): attacks that keep harmful behaviour while fooling every latent-space monitor tried; rebuilt over 70 times in red-team/blue-team rounds | an organism: an agent penalized when a probe reads it |
| The Hydra effect | McGrath et al. (2023): ablate a layer and later layers compensate (about 70% restored); "erasure" layers ease off | an experiment: self-repair in our networks |
| Multiple realizability | The same behaviour can be realized by many internal patterns | 010 onward |
| Two assumptions | Fixity of structure; reliability of extrapolation | the critique videos |
| Substrate | The layered environment a system runs on (weights, architecture, OS, hardware, paradigm, economy), after Brand's pace layering | the NES as a stack of substrates |
| Marr's three levels | Computational, algorithmic, implementation (a cash register's base-10 arithmetic) | 002 (BCD is the algorithmic level) |
| Substrate examples | Game of Life vs. von Neumann machines; quantum vs. classical; GPUs and the hardware lottery | decide |
| Scaffolding shift | Tools and agent frameworks move capabilities outside the model | later |
| Architecture shifts | RNNs → transformers → Mamba; tools need reworking | the series' own architecture ladder |
| AI-assisted evolution | Thompson's evolved FPGA circuit (1996) exploited the chip's analogue physics | an organism: an agent exploiting an emulator inaccuracy |
| Self-modification; deep deceptiveness; diffuse deception | Systems that reshape themselves, hide from monitors, or spread a harm across components | later |
| Substrate-independent vs. live theory | Generalizing by abstraction (Big-O) vs. by carrying specifics across contexts; speculative | decide |

## 8. God Help Us, Let's Try To Understand AI Monosemanticity (Astral Codex Ten)

Scott Alexander. Astral Codex Ten, 2023-11-27.
https://www.astralcodexten.com/p/god-help-us-lets-try-to-understand

A lay explanation of "Toy Models of Superposition" and "Towards
Monosemanticity", read in full (its figures weren't viewed). Valuable to
the series mostly as a model of how to teach this to a lay reader; its
claims about the papers are the author's summaries, so cite the papers
themselves for facts.

| Concept | What the post says | Series |
|---|---|---|
| The naive approach | Show many inputs and see when each neuron fires; a "dog neuron" | 008, 009 |
| Why it fails | Middle-layer neurons aren't that simple: one fires for cat faces, car fronts and cat legs (polysemanticity, from "Zoom In") | 010 |
| The counting argument | One concept per neuron caps you at as many concepts as neurons; sharing neurons fits more | superposition video |
| A coordinate code | Two neurons' activation pairs as codes for five concepts; concepts as points on a shape | superposition video |
| The cost of packing | More points, more confusion: two concepts at once can read as a third | superposition video |
| The toy experiment | 400 features in 30 neurons: dense data gets one neuron per feature, sparse data packs features into shapes (tetrahedra, triangles, digons, pentagons, a square antiprism) | superposition video (reproduce on NES data) |
| A small AI simulating a bigger one | The real neurons simulate a larger, sparser network; interpretation has to study that one | superposition video |
| The autoencoder as translator | A second network re-expresses 512 neurons as thousands of features, which are monosemantic | SAE video |
| A feature, worked through | The "God" feature: what fires it, which neurons build it, what it predicts next | SAE video |
| Neuron summaries as tea leaves | Explaining single neurons reads noise; meaning lives at the feature level | 010 |
| Feature splitting | More features split one into finer ones (math "the" into machine learning, complex analysis, topology) | SAE video |
| Consistency across runs | Two models' features match closely (median correlation 0.72, per the paper) | SAE video |
| Scaling the interpreter | An interpreter may cost as much as the model; reading millions of features needs automation | later |
| A weak interpreter for a strong model | Too weak to scheme, strong enough to read | later |
| Brains too? | One preprint reports superposition in visual cortex too (arXiv 2310.11431), hedged | decide |

Teaching devices to adapt, crediting Scott Alexander: "inside every AI
is a bigger AI"; the dog-neuron thought experiment; two neurons as a
coordinate code; the shape-packing progression (with the square
antiprism's biscornu); feature splitting as a tree.

## 9. Towards Monosemanticity: Decomposing Language Models With Dictionary Learning

Trenton Bricken, Adly Templeton, Joshua Batson, Brian Chen, Adam Jermyn,
Tom Conerly, Nicholas L. Turner, Cem Anil, Carson Denison, Amanda
Askell, Robert Lasenby, Yifan Wu, Shauna Kravec, Nicholas Schiefer, Tim
Maxwell, Nicholas Joseph, Zac Hatfield-Dodds (in the BibTeX), Alex
Tamkin, Karina Nguyen, Brayden McLean, Josiah E. Burke, Tristan Hume,
Shan Carter, Tom Henighan, Chris Olah. Transformer Circuits Thread,
2023-10-04.
https://transformer-circuits.pub/2023/monosemantic-features/index.html

Read in full from the page's text (figures and the reference list were
stripped; claims that rest on a figure are marked †).

| Concept | What the paper says | Series |
|---|---|---|
| Polysemantic neurons | E.g. one neuron for citations, English dialogue, HTTP requests and Korean | 010 |
| Superposition as the cause | More features than neurons, each a direction; features form an overcomplete basis | superposition video |
| Three ways out | Models without superposition; dictionary learning; a hybrid. Architectural sparsity wasn't enough, classic dictionary learning overfit; a "weak" learner (a sparse autoencoder) worked | SAE video |
| Why sparse models stay polysemantic | Under cross-entropy loss, one neuron covering two exclusive features beats covering one cleanly (ln 2 ≈ 0.7 vs ¾·ln 3 ≈ 0.8) | SAE video (a pressure test we can reproduce) |
| Features as a decomposition | x ≈ b + Σ fᵢ(x)·dᵢ: activity as a sum of feature directions, each scaled by its activation | superposition and SAE videos |
| What makes a good decomposition | We can say when each feature fires, what it does downstream (consistently), and it explains a real share of the layer | 008 onward (standards) |
| The setup | A one-layer transformer (residual 128, MLP 512, ReLU) trained on the Pile; autoencoders of 512 to 131,072 features on 8 billion MLP activations | SAE video (our NES version) |
| The autoencoder | Subtract a bias, linear encoder + ReLU, linear decoder with unit-norm columns; loss = reconstruction error + λ·L1 of activations | SAE video |
| Training tricks | Pre-encoder bias, untied weights, resampling dead features, long training, learning-rate sweeps, Adam handled with the norm constraint | SAE video (implementation) |
| No good test metric | Judged by inspection, feature density, reconstruction, sparsity (L0 under ~10–20), and toy models with known answers | SAE video (pressure test) |
| Five tests of a feature | Specificity, sensitivity, causal effect, not a neuron, universal | 008, 009 (a checklist) |
| Worked features | Arabic (81% of its activity on Arabic, which is 0.13% of tokens), DNA, base64, Hebrew (in no neuron's top examples) | SAE video (we'll have game equivalents) |
| Three downstream checks | Logit weights, ablating the feature, pinning it on while sampling | SAE video; causal-intervention video |
| How interpretable, overall | Human scoring: median neuron 0, median feature interval 12 (of 14); automated scoring with Claude: features beat neurons (logit task 74% vs 58%, chance 50%) | SAE video |
| How much it explains | 79% of the MLP's loss reduction (A/1), 94.5% at 131,072 features; the measure is doubtful | SAE video (pressure test) |
| Model or data? | A randomly shuffled model gives mostly trivial features, so training creates the structure | SAE video (a control we can run) |
| Feature motifs | Context features, token-in-context features (hundreds for "the"), trigram features; every feature is both an input and an action | SAE video |
| Feature splitting | Bigger dictionaries split features into families; small dictionaries give a summary view | SAE video |
| Universality | Features match across two seeds (median activation correlation 0.72 vs 0.46 for neurons) | SAE video; universality |
| Finite-state automata | Features that chain through the text: a base64 loop, CAPS_SNAKE_CASE, a four-step HTML cycle | SAE video (game analogues: state machines in gameplay) |
| Limits | No metric; doubtful explained-loss baseline; low activations less interpretable; dead and ultralow-density features; one-layer model; scaling cost (about 20B parameters for a 100× dictionary on a 10,000-wide layer) | SAE video (pressure test) |
| Replication (Neel Nanda's comment) | Results replicate on an open one-layer GELU model; 92% of features spread across many neurons | SAE video |

## 10. Toy Models of Superposition

Nelson Elhage, Tristan Hume, Catherine Olsson, Nicholas Schiefer, Tom
Henighan, Shauna Kravec, Zac Hatfield-Dodds, Robert Lasenby, Dawn Drain,
Carol Chen, Roger Grosse, Sam McCandlish, Jared Kaplan, Dario Amodei,
Martin Wattenberg, Christopher Olah. Transformer Circuits Thread,
2022-09-14. https://transformer-circuits.pub/2022/toy_model/index.html

Read in full from the page's text (figures, inline citations and some
equations were stripped; training details such as the optimizer and
learning rate live in the paper's notebooks, not the text).

| Concept | What the paper says | Series |
|---|---|---|
| Why neurons are sometimes clean | Sparse features make small ReLU networks store more features than dimensions; interference is filtered by the nonlinearity | superposition video |
| Decomposability and linearity | Representations as independent features; features as directions (x₁W₁ + x₂W₂ + …) | 007 onward |
| Three definitions of "feature" | An arbitrary function (rejected), a human-interpretable property (rejected), a property a big enough network would give its own neuron (used, held loosely) | superposition video; a pressure test |
| Privileged basis | An activation function makes neuron directions special; without one (embeddings, the residual stream) directions are arbitrary | superposition video |
| Almost-orthogonal directions | n orthogonal directions fit in n dimensions, but exponentially many almost-orthogonal ones (Johnson–Lindenstrauss) | superposition video |
| Compressed sensing | A sparse vector can be recovered from a low-dimensional projection | superposition video |
| The toy model | n features (zero with probability S, else uniform 0–1, importance Iᵢ) → h = Wx (m < n) → x′ = ReLU(Wᵀh + b); loss = importance-weighted squared error. Linear model for contrast | superposition video (reproduce, then on NES variables) |
| Linear vs. ReLU | The linear model learns the top m features (like PCA) whatever the sparsity; the ReLU model superposes as sparsity rises, least important features first | superposition video |
| Benefit vs. interference | The loss trades representing a feature against interference with others; superposition is never optimal in the linear model | superposition video |
| Phase change | Each feature is dropped, superposed or given its own dimension, with sharp (first-order) transitions over sparsity and importance (2 features in 1 dimension, closed form) | superposition video |
| Geometry | Features per dimension stick at fractions: 3/4 tetrahedra, 2/3 triangles, 1/2 antipodal pairs, 2/5 pentagons, 3/8 square antiprisms; Thomson-problem solutions; tegum products; "too elegant to be true", maybe toy-specific | superposition video (the shapes on screen) |
| Feature dimensionality | Dᵢ = ‖Wᵢ‖² / Σⱼ(Ŵᵢ·Wⱼ)²: 1 dedicated, 1/2 antipodal, 0 dropped; they sum to m when packing is efficient | superposition video |
| Non-uniform superposition | Changing one feature's sparsity deforms a pentagon until it snaps to two digons | superposition video |
| Correlated features | Correlated features prefer to be orthogonal, else side by side, else collapse to their principal component; anticorrelated features prefer to share | superposition video (NES variables are correlated) |
| PCA vs. superposition | Correlation favours PCA, sparsity favours superposition | superposition video |
| Learning dynamics | "Energy level" jumps between dimensionalities with drops in loss; learning as geometric transformations, PCA first | later (learning dynamics) |
| Adversarial examples | Superposition makes models more vulnerable (over 3× once it forms); adversarial training reduces superposition only at huge attack sizes | later |
| Privileged basis model | A hidden ReLU: neurons go from monosemantic to polysemantic as sparsity rises; both kinds coexist; the model can bypass the ReLU ("simulating identity") | 010 |
| Computation in superposition | Absolute value computed on superposed features; primary and secondary features; an asymmetric superposition motif with inhibition | later |
| The strategic picture | Superposition blocks enumerating features, which safety claims ("no deception feature here") need; three ways out: models without it, overcomplete bases (dictionary learning), hybrids; mixture-of-experts may have less | superposition and SAE videos |
| Open questions | A test for superposition; controlling it; realism of toy models; scaling; how many features fit; links to compressed sensing phase changes and non-robust features | superposition video (pressure test) |
| Replications (comments) | Redwood, DeepMind, OpenAI and others replicated it; a sparse autoencoder recovered toy features almost exactly (Sharkey, Braun, Millidge) | SAE video |

## 11. A Longlist of Theories of Impact for Interpretability

Neel Nanda. AI Alignment Forum, 2022-03-11.
https://www.alignmentforum.org/posts/uK6sQCNMw8WKzJeCQ/a-longlist-of-theories-of-impact-for-interpretability

The case for why interpretability might matter, as a list of separate
arguments ("theories of impact"); read in full. The post says it covers
19 arguments, but its rendered numbering is inconsistent, so items are
named here, not numbered.

| Theory of impact | What it says | Series |
|---|---|---|
| Force multiplier for alignment research | Seeing why a model misbehaves gives richer data | the critique video; failures throughout |
| Predicting future systems | Mechanistic principles that extrapolate with scale (induction heads and sudden capability gains) | later |
| Auditing | Check a trained model before deploying it | the critique video; auditing games |
| Auditing for deception | A lower bar: spot-check parts of a model for deception | the critique video; organisms |
| Coordination | Actors who can inspect each other's systems can trust each other | the critique video |
| Evidence for or against threat models | Real examples of theorised failures; convincing skeptics; supporting a slowdown | model organisms |
| Better human feedback | Train models to do the right thing for the right reasons | later |
| Informed oversight | Check each step of a recursive alignment scheme | later |
| Interpretability in the loss | Train against an interpretability tool (it may be Goodharted) | the probe-evasion organism |
| Norms and regulation | Interpreting a system becomes due diligence; regulators can audit | the critique video |
| Cultural shifts | A field that expects understanding notices how little it has (Olah's bridge analogy) | the series' framing |
| Microscope AI | Learn from how a model does a task instead of deploying it | the critique video |
| AIs interpreting AIs | Near-human-level AIs interpret stronger ones | later |
| Forecasting discontinuities; intervening on and auditing training runs | Catch problems during training, before a model can hide them | learning-dynamics videos |
| ELK via shortest explanation | Use explanation length as a training loss (credited to Tao Lin) | later |
| The disagreement | Auditing a finished model vs. using tools in training, where optimization pressure may break them | the probe-evasion organism |

## 12. Against Almost Every Theory of Impact of Interpretability

Charbel-Raphaël. AI Alignment Forum, 2023-08-17.
https://www.alignmentforum.org/posts/LNA8mubrByG7SFacm/against-almost-every-theory-of-impact-of-interpretability-1

A deliberately adversarial reply to Nanda's longlist, read in full
(comments not included). Its scope matters to this series: it targets
deep, GPT-like models and explicitly sets aside the explainability of
small models.

| Argument | What it says | Series |
|---|---|---|
| Not a predictor | Interpretation came after phenomena were found (induction heads, grokking); behavioural tests found them first | the critique video |
| Deception auditing is out of reach | Spot-checking circuits doesn't obviously find deception; the field is far down Hubinger's "tech tree"; deception may not need internal planning; other routes exist (adversarial testing, consistency checks, model organisms, monitoring chains of thought) | the critique video; the toolkit (behavioural tools) |
| No clear end story | Interpretability relies on serendipity and lacks a target | the critique video |
| Enumerative safety is doomed | A "gun" feature may just recognise guns; superposition makes enumeration intractable; dangers are high-level behaviours, not features | superposition and SAE videos (pressure tests) |
| Reverse engineering doesn't arrive | Code-like, plain-language, connectome and circuit-graph endpoints all fall short (the IOI circuit covers one token of GPT-2 small) | circuits video (pressure test) |
| Olah's dream is enumeration restated | Solving superposition for safety is enumerative safety again | SAE video |
| Retargeting the search: promising | Finding and redirecting a model's goal (maze-solving agents) | an NES organism (an agent's goal) |
| Relaxed / latent adversarial training: valid but hard | Adversarial perturbations of internals rather than inputs | later |
| Microscope AI: not promising | Economics favour agents; probing AlphaZero taught no new chess | the critique video |
| Best theory of impact: outreach | A gateway to the field, showing strange failures, showing models aren't parrots (Othello-GPT) | the series itself, which the critique says is interpretability's best use |
| Prevention over cure | Aim for systems whose reasoning is legible (chains of thought, process-based training, cognitive emulation) | later |
| Interpretability may be harmful | False sense of control, overinterpretation (saliency maps, feature visualization, ROME), safety-washing, dual use for capabilities | the series' honesty rule |
| Too many juniors in interp | E.g. 14 of 16 ARENA projects; the field needs diversity | the critique video |
| Solved interp still isn't safety | Many risks involve no deception; coordination matters more | the critique video |
| Better agendas | Technical governance, evals, adversarial robustness, characterizing alignment difficulty, technical outreach | the critique video |
| Concessions | Interp is more feasible than neuroscience; causal methods, activation patching and probes are more rigorous; some researchers should work on it | the critique video |

## 13. What are probing classifiers and can they help us understand what's happening inside AI models? (BlueDot Impact)

Sarah (guest post), BlueDot Impact blog, 2025-05-15.
https://blog.bluedot.org/p/what-are-probing-classifiers

Read in full (its ResNet-50 graph's values weren't visible).

| Concept | What the post says | Series |
|---|---|---|
| Information is hard to localize | One neuron fires in unrelated contexts; one input activates many neurons across layers | 010 |
| What a probe is | A small, simpler model trained separately on one layer's activations, to see what that layer encodes | 008 (and 001's neuron is a probe of the game) |
| Probing layer by layer | Train a probe on each layer for the same task; where accuracy rises shows where the information appears (a 10-layer cat classifier, hypothetically) | 008 (our networks, layer by layer, against RAM) |
| An existing result | Alain and Bengio's linear probes on ResNet-50 (arXiv 1610.01644, 2016): error falls steadily through the layers | 008 |
| Probing for other properties | Probe for factors besides the task (indoor vs. outdoor; nouns vs. verbs) | 008 (game variables besides the question) |
| Knowing vs. using | A probe shows a layer *contains* information, not that the model *uses* it (a hiring model may encode gender without using it) | 008 → causal-intervention video |
| Probe capacity | Too simple a probe misses information; too powerful a probe learns the task itself | 008 (pressure test: probe size) |
| You must know what to look for | Probes find only properties someone thought to test, with labelled data | 008 (vs. sparse dictionaries, unsupervised) |
| Labels run out | For abilities beyond human labelling, a probe can't be checked | 008 (the NES's advantage: RAM labels everything) |

Works it cites: Alain and Bengio, "Understanding intermediate layers
using linear classifier probes" (arXiv 1610.01644, 2016); arXiv
2308.14752 (deception) and 2206.13353 (power-seeking), named only by
number.

## Gaps

- The middle and end of the podcast transcript are summary-only (†).
- Part 2 of the episode is out of scope for now.
