# Adversarial review M (draft 10, commit aaec22d)

Numbers match numbers.json and exp002.py; no clichés; 006 in order now.
1. rule-intro (incorrect): "the bias, its built-in lean" gives 001's word
   for the border's tilt (set by the weights) to the bias, which slides it.
2. tldr-2 (emphasis): "settles because its pulls shrink as they near
   their targets"; at rest most moments are still far from them; what
   stops it is that the pulls balance (as `settles` says).
3. rule-bias (incorrect): "if he hadn't landed, everything would turn
   down" holds only for that same moment; the board shows any dot.
4. shelf: the real test is whether the totals, added up, point away from
   their targets, not "most totals"; two thoughts.
5. batch (scope dropped): the score range is over the last 100 steps.
6. settles, rule-bias board, target-example: two thoughts each.

# Adversarial review N (draft 10, commit aaec22d)

Numbers match; citations hold where checked; no clichés.
1. rule-intro (incorrect): "lean", as M1 (001's `bias-slides`).
2. how-far: "their best borders" collides with "the best border"
   (search's); "neither counts mistakes" reads as an explanation.
3. batch-2: skips that the pulls all share a direction, so can't add to
   nothing; its last sentence repeats `perceptron-hover`.
4. target: no word on where the border went: it still says yes above
   zero.
5. bounded (emphasis): Minsky–Papert and Block–Levin proved the weights
   stay bounded; "never settle" comes from the mistakes, not from them.
6. shelf: the "so" claims more than it can (hence 96%, not 100%); three
   thoughts.
7. square-outro board: "one swinging" but neither border swings.
8. settles-3: comes to rest only with a small enough step.
9. rule-bias: two pictures.
10. map-axes: "the middle" undefined.

## Response (draft 11)

- **M1 / N1:** `rule-intro` "the bias, a fixed amount that slides its
  border".
- **M2:** `tldr-2` "its added-up pull fades smoothly to nothing at one
  spot, the bottom of its bowl, where the moments' pulls balance".
- **M3 / M6 / N9:** `rule-bias` split; new `rule-flip` "For that same
  moment…", on the same dot.
- **M4 / N6:** `shelf` split (`shelf-2`): "any size adds more miss than
  it takes away" (true exactly when the best size in `rules.landscape`
  is zero), "for nearly all… {most_wrong_flat} of them".
- **M5:** `batch` "over its last hundred steps".
- **M6:** `settles` split (`balance`); `target-example` split
  (`target-over`).
- **N2:** `how-far` "their lowest points"; it already says we didn't
  find why.
- **N3:** `batch-2` "the pulls all share that direction, and added up,
  they can't come to nothing"; the repeated zero sentence cut.
- **N4:** `target` "The neuron still says yes above zero, and the
  targets sit one on either side of it."
- **N5:** `bounded` "stay within limits. With mistakes that never run
  out, they wander inside them".
- **N7:** `square-outro` board: two still borders, a gap between them.
- **N8:** `settles-3` "a small enough step".
- **N10:** `map-axes` "the middle, where depth and speed are both at
  their averages".
