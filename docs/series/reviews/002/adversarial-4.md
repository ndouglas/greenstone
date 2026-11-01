# Adversarial review G (draft 7, commit 8ff4884)

All numbers match; source hedges match 002.md; no clichés.
1. shelf / bowl-2 (incorrect): the flat shelf is where the square trick starts (all weights zero), so by no-slope's logic it'd be stuck; it isn't, because in the weights themselves the squared miss is a true bowl; the shelf is an artefact of the sheet's best-size-never-below-zero choice. bowl-2's board puts the rule's steps on the sheet; it walks the weights.
2. perceptron-score (incorrect, unsupported): Bishop 4.1.7: the rule isn't guaranteed to reduce its total error at each step; each nudge is a step down the measure for that one moment. "Every weight" should include the bias.
3. no-slope: "a rule that follows a slope" used before any rule is described that way.
4. who-moves-it (emphasis): 61% of late nudges come from moments the best border gets right; troublemakers are over-represented, not the drivers.
5. target-example: "two and a half times as hard" only for the same numbers.
6. settles: shrinking pulls don't by themselves guarantee a balance point; the summed pull changes smoothly from one way to the other, so it passes through zero.
7. shelf: Bishop doesn't support it; only exp002.
8. both-matter: two thoughts; split.
9. rule-bias: three thoughts; split off "Either way".
10. tldr: "depending only on the order" overstates; "even just the order".

# Adversarial review H (draft 7, commit 8ff4884)

All numbers match (1-point wobble, 39%, 73–90%, 2.5×, ~96% shelf); no clichés; no framing claims.
1. perceptron-score (emphasis): "it shrinks a measure of its own": it starts at the measure's lowest point; Bishop 4.1.7: not guaranteed to reduce the total at each stage. "Each nudge is a step down… for the one moment that caused it".
2. no-slope (order, term): "a rule that follows a slope" used before linking nudges to going downhill (bowl-2 is a chapter later). Add a beat before.
3. step-size: "makes exactly the same mistakes" needs step-size-2's reason first: swap them.
4. map-outro (emphasis): it does stop when a line splits the moments: "on Mario's moments".
5. tldr-2: only the added-up version settles.
Minor: tldr-3 uses {search} for the lowest step (should be {stairs_bottom}); rule-bias two thoughts; square-why two thoughts (move the name); bounded "proved early on" → "later".

## Response (draft 8)

All acted on:

- **G1:** new beat `no-shelf`: the shelf belongs to the sheet's picture;
  the rule moves the weights, where the total squared miss slopes
  everywhere but its bottom, including at zero; `bowl-2`'s board no
  longer puts the rule's steps on the sheet; `shelf` cites only exp002
  (G7).
- **G2 / H1:** `perceptron-score`: each nudge is a step down its own
  measure for the one moment that caused it; "every weight and the bias".
- **G3 / H2:** new beat `downhill` in chapter 3 defines following a slope
  (the square trick's added-up nudges point downhill on its total);
  `no-slope` calls back to it.
- **G4:** `who-moves-it`: "far more than their share", "moving most".
- **G5:** "for the same numbers".
- **G6:** `settles`: the summed pull changes smoothly from one way to the
  other, so it passes through zero.
- **G8 / G9 / H minor:** `both-matter` split (`the-map`); `rule-bias` split
  (`rule-why`); `square-why` loses the name to `square-name`.
- **G10:** "depending even on just the order".
- **H3:** `step-size-2`'s reason now comes first.
- **H4:** `map-outro` "on Mario's moments".
- **H5:** `tldr-2` "with its nudges added up before each step".
- **H minor:** `tldr-3` uses the lowest step's own number; `bounded`
  "later".
