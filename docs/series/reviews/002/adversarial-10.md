# Adversarial review S (draft 17, commit 471077b)

Four substantial findings; every number reproduces and every check
passes.
1. perceptron-rest: "takes back part of the weights; every pull leans
   toward zero" is true only as an angle: 58.7% of nudges make the
   weights bigger, and nearly every one moves some dial away from zero.
2. remember against tldr-3/inside: the pocket perceptron (90–91%)
   matched the best border and beat the square trick, yet the
   Conclusions dropped it. → new tldr-3: it got within a point "only by
   remembering its best pass, scoring every pass on every moment: a
   search of its own"; the staircase line moves to tldr-4.
3. unseen-2: measured only the position fraction, and "below a pixel" is
   wrong for speed. → cut (T2).
4. too-big: reversed step-size-2 in one clause, and carried two
   thoughts. → split: the reason ("the targets stay at plus and minus
   one, so doubling the step can't just double everything, the way it
   did for the perceptron") and the overshoot, then `too-big-2` with the
   numbers.
Minor, taken: batch's two thoughts (now perceptron-rest and -2, with a
picture of the step sizes); square-intro shortened, ending "asks how
far off every moment was, right or wrong"; square-why's one-moment
clause cut; unseen's data set (T4); rule-flip "each by the step times
its own number"; rule-name "published".

# Adversarial review T (draft 17, commit 471077b)

Four substantial findings.
1. perceptron-rest/-2: the lean is tiny (median 91.6° from the
   weights) and 58% of nudges grow the weights, so "takes back", arrows
   to zero and the one-sided tug of war mislead, though the half-space
   argument holds.
2. unseen-2: the hidden fraction varies but doesn't predict the outcome
   (115 of 224, chance). → cut.
3. search-2 describes the line where it's mostly wrong (4% and 28% yes
   there). → framed as a compromise: "A straight border can't follow
   the game exactly. The best one says yes from…".
4. unseen quoted the test points after spots counted the training
   points. → the training points' 88% and 59%, computed in exp002.
Minor, taken: this-video "one that can stop"; too-big's missing link
(S4); square-intro's who-pulls-whom (S). perceptron-rest's "ground from
before": gone with the rewrite.

## Nate's ruling (2026-11-02) on perceptron-rest

The geometric argument (no nudge leans further out along the weights,
so they can't cancel) had drawn a substantial finding in four phrasings.
Asked to choose between measured facts only, the exact geometry, or
cutting it, Nate chose measured facts only: the added-up perceptron's
score calms (83–86% over its last hundred steps), but its steps never
shrink (its last hundred within 5% of their median size) while the
square trick's shrink toward nothing; and at zero it says no to
everything and is pushed back out. The geometry stays in games/002.md
and in exp002's `pulls_lean_back` check.
