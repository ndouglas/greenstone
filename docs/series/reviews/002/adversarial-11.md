# Adversarial review W (draft 19, commit bec733e)

Two substantial findings.
1. perceptron-rest-2: "never came near cancelling" holds only for the
   last hundred steps exp002 measures; at steps 38–54 of the batch run
   the summed pulls fell to an eighth of their size (0.116 at step 51)
   with the score near 88%, before it broke away. → scoped: "Over those
   last hundred steps…", in its own beat (`perceptron-rest-3`).
2. inside: "the perceptron's pulls never cancel" as a law, against the
   guarantee chapter, where the rule stops. → "On Mario's moments, the
   perceptron's pulls never cancelled".
Minor, taken: how-far's "lean quite differently" covered the hinge
version too, whose border is fairly close (4.71 and 1.20 blocks against
3.98 and 1.56): now only the square trick's, "as we saw".

# Adversarial review X (draft 19, commit bec733e)

Two substantial findings.
1. tldr / never-outro: "a learning rule can only come to rest where its
   added-up pulls cancel out" is false for a shrinking step (halving each
   pass stops the perceptron; games/002.md). Draft 17 dropped "with a
   fixed step" as unexplained; it is back, in plain words: "a learning
   rule whose step stays the same size".
2. inside (W2).
Minor, taken: perceptron-rest-2's two thoughts (split, W1); dropped's
start now its own sentence ("Take the run that fell to 67%."); search-2
says what the compromise is ("it gets some moments wrong to get more
right"). Not taken: too-big's two sentences (the reason and the
overshoot share one picture, the ball on the U; the numbers are already
their own beat).
