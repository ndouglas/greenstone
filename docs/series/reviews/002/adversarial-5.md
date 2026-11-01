# Adversarial review I (draft 8, commit 60e4a09)

All numbers match; source claims match 002.md; no clichés.
1. spots (incorrect, handwaving): whole numbers are a property of the two bytes we read, not the game (it keeps sub-pixel bytes too); and rounding isn't why one spot both landed and didn't: what's under him (blocks, pipes, pits, enemies to bounce off; 001.md) isn't in either number.
2. learn: "its own bias, which is to say its own border" parses as "the bias is the border".
3. downhill / bowl / no-shelf: downhill on what? Three surfaces (the total over weights, the border sheet, the weights again); 006's idea. Fix: drop it or draw the weight-space slope and say 006 goes further.
4. tldr-3 / why-bowls (emphasis): learning aims at a measure with a slope, usually not a single-bottomed bowl: "here, that's a bowl".
5. settles: "changes smoothly from one way to the other, so it passes through zero" is a one-dimensional argument; with three dials the reason is that the total squared miss has one lowest point.
6. perceptron-score: two thoughts; split.
7. rule-name (minor): motor-turned knobs (potentiometers), not dials.

# Adversarial review J (draft 8, commit 60e4a09)

Numbers match; no clichés.
1. spots (incorrect): SMB keeps a fractional part of vertical speed in another byte; mixed spots come from what the two bytes don't carry, not rounding.
2. downhill (handwaving, 006): no ground pictured; draw the miss over a dial, or move it.
3. settles: the one-dimensional argument; the bowl shape is the reason; the perceptron's added-up pull jumps when a moment crosses the border, so it can skip over zero: say it.
4. square-why: why square the misses (both directions count; big misses count more).
5. perceptron-score: two thoughts; board shows the second.
6. bowl: two thoughts; split.
7. step-size-2: the step-size claim needs exp002.
8. test-question: "wouldn't help anyone play" is unsupported opinion; three thoughts.
9. too-many: two thoughts; why-bowls repeats Höffgen.
Minor: guarantee-check board hard-codes "three passes"; tldr-2 "shrink and cancel".

## Response (draft 9)

- **I1 / J1:** `spots` says the two numbers *we read* are whole numbers;
  new beat `unseen` gives the real reason the same spot can go both ways:
  the neuron can't see everything that matters, such as what's under
  Mario (001's landing mistakes: raised landings, bounces, pits).
- **I2:** `learn` "weights and bias, which together set its border".
- **I3 / J2:** `downhill` pictures the ground (the total squared miss over
  the weights and bias, drawn as a U through one dial). Kept to "slope",
  as the bible's 002 entry has it.
- **I4:** `why-bowls` and `tldr-3`: "a measure with a slope; here, a bowl".
- **I5 / J3:** `settles` rests on the ground's single lowest point, not the
  one-dimensional argument; `batch` says why the perceptron can skip the
  balance point (its added-up pull jumps when a moment crosses).
- **I6 / J5:** `perceptron-score` split (`perceptron-zero`).
- **I7:** "knobs turned by electric motors".
- **J4:** `square-why` says why the misses are squared.
- **J6:** `bowl` split (`best-size`).
- **J7:** `step-size-2` cites exp002.
- **J9:** `too-many` split (`no-fast-way`).
- **J minor:** the control's board uses its number; `tldr-2` "cancel".
- **Length:** chapter 3 split at the square trick's resting point: "How
  wrong was it?" and "Close, but not quite" (new `rest-outro`).

Not acted on:

- **J8** (cut "knowing the answer wouldn't help anyone play"): Nate asked
  for that note (2026-11-01).
