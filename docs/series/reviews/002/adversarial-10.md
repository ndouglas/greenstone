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

# Adversarial review U (draft 18, commit f787943)

Two substantial findings.
1. tldr-3: "only by remembering its best pass" is false (the plain runs
   reached 90%, the averaged perceptron 90% in every order), "fewest
   mistakes did better" overstates (pocket's best order 91%), and
   "within a point" breaks the word rule. → "Fewest mistakes did as well
   as anything here. The perceptron matched it most reliably by
   remembering its best pass…"; remember says "about ninety percent
   every time".
2. perceptron-rest-3: "Even at zero it can't rest" leans on a step never
   given, so "nowhere to rest at all" is asserted, and bowl's "To see
   both" never shows the perceptron on the sheet. U and V both propose
   restoring the geometric step; under Nate's ruling (above) the
   promises are cut back to what was measured instead: staircase-outro
   asks "And what about the perceptron?", bowl says "To see where the
   square trick's slope leads", perceptron-rest-3 is cut, map-outro and
   tldr say "never came to rest" / "never did". perceptron-rest-2 now
   quotes a checked number: over its last hundred steps, its added-up
   pulls kept more than three quarters of their size (lowest 0.78;
   exp002 refuses under a half).
Minor, taken: "lowest point" → "lowest spot" off Mario's map; balance's
board "mark" → target; dropped names its start ("Starting from the run
that fell to 67%"); too-many no longer cites hoffgen1995; downhill split;
numbers "to the nearest pixel". Not taken: square-outro's question
(map-intro now promises "a picture of what that would take", round 5);
too-big's split (done in draft 18 as too-big / too-big-2).

# Adversarial review V (draft 18, commit f787943)

Six substantial findings; no wrong numbers.
1. perceptron-rest-3 (U2).
2. tldr-3 / remember (U1).
3. never-outro used "pulls cancel" before introducing it. → "A rule can
   only come to rest where its nudges, added up, come to nothing. What
   would a rule need, for that to happen?"
4. target-over: "for the same position and speed" is impossible with
   the same weights (the totals would match). → "Its miss is two and a
   half times the last one's, so, for numbers the same size, so is its
   pull."
5. how-far: "close to, but not on" the best border, against short-2's
   very different lean. → "Their scores land close to the
   fewest-mistakes border's, but the borders themselves lean quite
   differently."
6. downhill: a ground over three settings can't be pictured, and four
   steps in one beat. → `downhill` starts from the slice ("Hold every
   weight still but one…: a U"); `downhill-2` widens it to a valley and
   names following a slope; balance says "valley".
Minor, taken: search-2 "The best straight border is a compromise"
(no claim about the game's own border); shelf split, "on balance, faces
the wrong way"; bowl-2 drops the U callback; boards call the weights
"knobs", keeping "dial" for the total (001, bible).
