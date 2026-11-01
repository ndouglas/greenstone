# Adversarial review K (draft 9, commit 5861891)

Numbers match numbers.json and exp002.py.
1. batch (incorrect, unsupported): "skip right over the point where the
   pulls would balance" says a balance point exists; there is none.
   The measure is lowest at zero, where every landing pushes out
   (perceptron-hover), and the spine says its pulls never cancel.
2. learn / rule-wrong (emphasis): "a little" and "a small step", then
   step-size-2 says the size doesn't matter, and perceptron-hover-2
   says each nudge is about half the weights' size.
3. downhill, perceptron-score, perceptron-zero (out of order): ground
   over the weights and nudges pointing downhill is 006's idea in the
   bible.
4. shelf (skipped step): why a mostly-wrong border shrinks to nothing;
   and the script's direction (mostly wrong → flat) is derived, not
   measured (exp002 measures flat → mostly wrong).
5. how-far (minor): restates the gap; say plainly we didn't find why.
6. Two thoughts: rule-name, batch, square-why; settles borderline.
7. perceptron-zero (low confidence): couldn't reach Bishop 4.1.7.

# Adversarial review L (draft 9, commit 5861891)

Numbers match; claims about 001 match 001.md; no clichés.
1. batch (incorrect, unsupported): the balance point conflicts with
   perceptron-zero and map-outro; nothing in exp002 tests "skips over".
2. downhill, settles, no-slope, bowl to bowl-2 (out of order, partly):
   006's landscape; either drop the weight-space ground or change the
   bible so 006 calls back to 002.
3. map-intro (term clash): "a map of borders" blurs the bible's map
   (the inputs, one dot per moment); call it a sheet from the start.
4. how-far: "both add up squared misses" doesn't fit the version that
   leaves out moments past their target.
5. animals (out of place): sits in the middle of "close, but not quite";
   move it after square-name.
6. too-many: the 2,048 numbers of memory is a hardware fact; cite the
   NES memory map, not exp001.
7. rule-name: two thoughts; split.
8. learn: two thoughts and two pictures; split at "Ours starts".
9. square-outro: "the total we gave it, not the one we wanted" reuses
   "total" (the neuron's sum) for a count of mistakes.

## Response (draft 10)

- **K1 / L1:** `batch` keeps the result; new beat `batch-2` gives the
  reason: every wrong moment's pull points partly back toward zero (its
  nudge has a negative product with the current weights and bias,
  since the moment's total is on the wrong side), so on Mario's
  moments, where something is always wrong, the pulls never cancel; and
  at zero every landing pushes out. New test
  `test_every_perceptron_nudge_points_partly_back_toward_zero`.
- **K2:** `learn` drops "a little"; `rule-wrong` "a fixed step";
  `rule-bias` "one step"; `step-size-2` "How big should the step be?".
  The square trick's "small step" stays: there, size matters.
- **K3 / L2:** kept, as after review I3; the bible's 006 row now says it
  calls back to 002's ground over one neuron's weights.
- **K4:** `shelf` gives the step (any size pushes most totals away from
  their targets) and quotes the measured direction: new number
  `most_wrong_flat` (96%) in exp002 and 002.md.
- **K5 / L4:** `how-far` "we didn't find why. All we can say is…", and
  the second version "leaving out moments past their target".
- **K6 / L7 / L8:** `rule-name` split (`mark-one`); `learn` split
  (`learn-2`); `square-why` split (`square-why-2`); `batch` split.
- **L3:** `map-intro` "a new picture: a sheet of borders".
- **L5:** `animals` moved after `square-name`.
- **L6:** `too-many` cites `nesdev-memory` (as 001 does).
- **L9:** `square-outro` "chosen by the squared misses we gave it, not by
  the count of mistakes we wanted".

Not acted on:

- **K6** `settles`: one thought (why it stops), as the reviewer allowed.
- **K7:** Bishop 4.1.7 defines the measure as a sum over the wrong
  moments of how far each total is past zero; it is never negative and
  is zero with every weight at zero, so the claim follows from the
  definition. To check against the page in the interview.
