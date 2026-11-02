# Adversarial review AA (draft 21, commit 85fe220)

One substantial finding.
1. never-outro / tldr: "a rule with a fixed step only rests where its
   nudges, added up, come to nothing", pointed at the one-at-a-time
   runs, whose late passes' nudges add to 0.02–0.17% of their length
   (they nearly cancel; single nudges are what never stop). → never-outro
   only asks "What would a rule need, to come to rest?"; tldr limits the
   law to added-up nudges ("adding up its nudges, a learning rule with a
   fixed step can only come to rest where they cancel out") and says the
   perceptron never came to rest "one moment at a time or added up".
Minor, taken: staircase-outro "a few points short"; shelf-2 "mostly
faces the wrong way"; best-size "doubling them all, and the bias".

# Adversarial review BB (draft 21, commit 85fe220)

Three substantial findings.
1. perceptron-rest-2/-3: from about move 100 the batch perceptron rocks
   between two borders, each move almost undoing the one before, so
   "pulls never came near cancelling" (true of one move) pictured a
   drifting border. Measured in exp002 and checked: two moves cover 1.9%
   of one move's length (refused above 10%); standing still, the two
   borders say yes from 3.7 and 5.8 blocks. → "it rocked back and forth
   between two borders, each move almost undoing the one before"; the
   pulls-share beat is gone (its number stays in exp002).
2. never-outro: "its nudges, added up" could be heard as the weights
   (step-size-2's definition). → the principle is gone from never-outro
   (AA1).
3. map-axes: a half turn about the map's middle gives the mirror-image
   line, not the same line. → "turn the ruler around the middle of the
   map, and half way round, its yes and no sides have swapped".
Minor, taken: too-many "only works easily with two numbers; each extra
number multiplies the borders to try"; tldr-3 drops "most reliably"
(averaging was as steady). Not taken: too-big's two sentences (as
before); the "So" openers (most are outros, whose format asks for it);
remember's "about ninety percent" hardcoded (pocket_lowest and
pocket_highest are 90% and 91%; the board shows both).

# Adversarial review CC (draft 22, commit 7b485c5)

Two substantial findings.
1. staircase-outro's "a few points short" (and tldr-3's board, "a point
   apart") used "point" for a share of the score, against the word rule.
   → "about three in a hundred short"; "about one in a hundred apart".
2. shelf: "on balance faces the wrong way" is heard as "gets most moments
   wrong", true of only 82% of the shelf. → "Some borders do best by
   shrinking to nothing… Mostly, though not always, they're borders that
   get most moments wrong."
Minor, taken: no-slope "around the best border we found" (search's
border makes one training mistake fewer than the sheet's lowest grid
spot; both score 91%).

# Adversarial review DD (draft 22, commit 7b485c5)

Two substantial findings.
1. settles-2: the square trick's small wobble comes mostly from the
   small step chosen (3.6 to 11.4 points at bigger steps that still
   converge). → "with the small step we gave it"; the measurements are in
   games/002.md.
2. perceptron-rest-2's "it never settles" goes past 300 measured moves.
   → "it didn't settle".
Minor, taken: tldr's law sat beside the one-at-a-time runs it doesn't
explain; it moves to tldr-2, with the square trick ("Adding up its
nudges, a rule with a fixed step can only come to rest where they
cancel out, and the square trick's cancel at the bottom of a bowl");
numbers-2 cites nesdev-timing for sixty frames a second, as 001 did;
tldr-3 "came within about one in a hundred of it" (not "matched").
Not taken: "Mark One" against "the mark" (fifteen beats apart, said
once).
