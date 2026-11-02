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

# Adversarial review Y (draft 20, commit 0421f70)

One substantial finding.
1. inside / tldr tied the perceptron's spread of endings to its pulls
   never cancelling, but that was measured only on the batch run (one
   ending, no order); in the one-at-a-time runs, whose 67–90% spread is
   being explained, each late pass's nudges add to about 0.1% of their
   length. → both now say what was seen: "the perceptron never came to
   rest".
Minor, taken: too-big "decides whether it gets there at all" (any small
enough step ends at the same border); settles-3 "makes one small move,
and does that over and over" (it takes about 2,000).

# Adversarial review Z (draft 20, commit 0421f70)

One substantial finding.
1. "step" meant both the step size and one added-up move, so just
   before tldr's "a rule whose step stays the same size", the viewer
   heard that the square trick's steps shrink: the backwards reading.
   → "step" is only the size we set; each added-up update is a "move"
   (settles-3, too-big, too-big-2, perceptron-rest to -3, tldr-2); the
   header's word rule updated; perceptron-rest-2 adds "with the same
   step size throughout".
Minor, taken: step-size "seeing the moments in the same order";
square-outro "adding up its nudges, settles in one place" (the batch
version; the per-moment one doesn't settle).
