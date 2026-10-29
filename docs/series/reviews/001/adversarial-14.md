# Video 001, adversarial reviews, fourteenth pass

Script reviewed: commit 4aaae0b. Reviewer 2: nothing substantial.
Reviewer 1: one substantial finding (reviewer 2 listed it first among its
minors), introduced by the thirteenth pass's rewrite. Fixed, with the
safe minor points.

| Finding (reviewer) | Change |
|---|---|
| one-dot-4b: "Any fixed number gives a neuron the same answers, but only a test against zero survives this tripling" contradicts itself heard literally, and carries two thoughts (1 substantial, 2) | Split: one-dot-4b "Here, zero matters. In the last chapter, moving the 100 into the bias gave the same answers as testing against 100." / one-dot-4c "Tripling is different. Test against 100 instead of zero, and a total of 50, a no, becomes 150, a yes. Only a test against zero survives tripling." |
| testing-end's "But" has nothing to push against; its last clause repeats recipe-intro (1, 2) | "So it goes wrong because all it knows about is the floor. A harder question needs two numbers, and for that we need the recipe." |
| two-intro oversells ("a single number really isn't enough"; speed alone gets 84%) (2) | "one where depth alone really isn't enough" |
| draw-intro's plural promises more than chapter 4 shows (1) | "shows a change that leaves its answers exactly the same" |
| depth's text and its offset ruler (1, 2) | "counted … from near the top" |
| Board nits (2): "a third bar" (it's the fourth); the-corner labels truth "lands soon" like the neuron's answer; w₀ never used again | Fixed |

Not acted on: why-zero-3's board holds the chip, the transformation and
the real instruction (2; the storyboard stage can drop the chip);
"found and tested the same way" vs `in_play` being Super Mario Bros.-only
(1: tiny); the two 92%s (2: every-session-games already sits between
them).
