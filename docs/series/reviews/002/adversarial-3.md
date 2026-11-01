# Adversarial reviews E and F (draft 6, commit fd160dc)

Both: every number matches numbers.json and exp002.py; no clichés.

E1 / F4. perceptron-score: "how far its wrong moments sit on the wrong side" sounds like distance from the border, which scaling doesn't change (step-size-2); what shrinks is how far the totals are past zero.
E2 / F1. settles: "landed pull one way, not-landed the other" contradicts target-example (a landed total of 3 pulls down), and the perceptron also has two opposing teams, so it explains nothing; the real reason is that each pull shrinks smoothly as its total nears its target, so a balance point exists; they cancel *at* the lowest point, not near it.
E3 / F3. no-slope, bowl-2, why-bowls: "a rule that follows a slope" never connected: say the square trick's added-up nudges step downhill on its total squared miss.
E4 / F5. bowl: two thoughts; why the best weights are nothing (the size can't go negative without swapping the yes side, so a backwards border does least badly by switching off).
E5 / F2. why-short → how-far restates the gap. E: say no single cause was found. F: squared misses weight deep wrong moments (Bishop 4.1.3).
E6 / F7. my-version: offstage rule; cut or move to the description.
E7. bounded board: a tight ring, not a wide one.
E8. remember: second thought; cut "but that's a patch".
E9 / F lesser. tldr-2: three thoughts; split.
F6. square-name: "usually called" overstates; LMS and the delta rule are as common.
F lesser: perceptron-hover four steps; target-example two cases (optional); sheet: the turn wraps round.
E minor: rule-name: the rule is from the 1962 book.

## Response (draft 7)

- **E1 / F4:** `perceptron-score` measures how far wrong totals are past
  zero, and says shrinking the weights shrinks the totals while the
  border stays put.
- **E2 / F1:** `settles` gives the real reason: each pull shrinks smoothly
  as its total nears its target, so there's a balance point, where the
  pulls add to exactly nothing. (Draft 6's "landed against not landed"
  came from review D; both partitions cancel at the bottom, so neither is
  wrong, but neither explains settling.)
- **E3 / F3:** `bowl-2`: the square trick's added-up nudges are small
  steps down its total squared miss.
- **E4 / F5:** `bowl` split into the height (each border at its best
  size) and `shelf` (the size can't go below zero without swapping the
  yes side, so a backwards border switches off).
- **E5:** `how-far` says no single cause was found. (F2's "deep wrong
  moments outweigh" isn't tested, and logistic, which damps them, lands
  at 88% too, so it isn't used.)
- **E6 / F7:** `my-version` cut, with its citation; Nate can restore it.
- **E7:** `bounded`'s ring no longer wide. **E8:** `remember` trimmed.
- **E9:** `tldr-2` split (`tldr-3`).
- **F6:** "the Widrow-Hoff rule, or least mean squares".
- **F lesser:** `perceptron-hover` split; `sheet` says the turn wraps
  round. `target-example` kept as one contrast.
- **E minor:** `rule-name`: perceptron in 1958, this rule by 1962.
- **Length:** chapter 4 grew past the limit's comfort (estimated 4:39),
  so it's split into "A staircase" and "A bowl", with a new
  `staircase-outro`.
