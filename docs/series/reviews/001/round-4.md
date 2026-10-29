# Video 001, council round 4 (the re-cut into sections and beats)

Script reviewed: commit 3fd4662 (115 beats). All eight lenses, including
the new continuity and pacing lens. Revised script: 122 beats.

## What every lens or most lenses found

- **The second-number promise** (pedagogy, clarity, cross-examiner,
  pacing, lay viewer): testing-end, recipe-intro and draw-end implied a
  second number would fix the pipe mistakes; it doesn't (a straight line
  on depth and speed scores no better, 001.md), and chapter 5 silently
  changes question. Fixed: testing-end says so ("giving it his falling
  speed as well doesn't help a straight line here. Other questions do
  need two numbers"), now checked by exp001's `check_claims`
  (`best_line`); two-intro introduces a new question.
- **The zero beats** (all lenses): why-zero-2 credited zero with what any
  constant allows and said the neuron "adjusts"; why-zero-3 offered the
  6502's cost as a reason neurons use zero, with code on screen and a
  wrong address ($8022 for the compare at $8025). Not cut, because Nate
  asked for both ideas; rewritten honestly. why-zero-2: "with zero,
  nothing is left over: the bias can be treated as just one more weight".
  why-zero-3: the processor tests "is it 165?" by subtracting 165 and
  checking what's left, the same move as folding 100 into the bias (CMP is
  a subtraction, mass:werk); one labelled instruction (cmp #$a5 at $8025),
  no wall of code. The true dependence on zero is now said where it
  matters, one-dot-4: scaling keeps every answer "only because the test is
  against zero". For Nate's interview.
- **"Every neuron"** (skepticism, cross-examiner): contradicted by
  `modern`. Now "every neuron of this kind" / "neurons like ours".
- **The TL;DR** (skepticism, cross-examiner, empiricism): no baselines,
  "wherever" where it's "mostly". Fixed in tldr-2 and tldr-3.
- **Castlevania** (skepticism, empiricism): "it only knows about the
  floor" is Mario's pattern (Castlevania: 0%). Now "in Super Mario Bros.".

## Continuity and pacing, and the rest

- Chapter 2 reordered: the evidence (fluke check, steady player, other
  games) together, its mistakes last, leading into the conclusion.
- Weights left out of chapter 1 (its neuron has none); introduced in the
  recipe.
- Chapter 3 reordered: recipe, formula, our numbers, the examples, then
  "so the setting is where the total is zero", then why zero.
- The balance scale is motivated ("sums with minus signs are hard to
  follow") and used on the rising, minus-sign example (new
  `smb1_land_balance_no_pan` from exp001), which the blind listener and
  lay viewer couldn't follow by ear.
- Splits: the-tilt (the formula now said aloud), school-2, one-dot-2,
  not-a-fluke, steady-player, frames, simplest-neuron, our-numbers-2,
  settings-are-borders, land-results; baseline's "the score to beat" moved
  to the beat that gives it.
- Intros and conclusions: found-intro states a problem and names the
  scale; draw-intro names the need (rounding without changing answers);
  recipe-end says what the drawing shows; found-end and two-end tie back.
- The Introduction opens on the problem (nobody-knows) and folds the
  memory fact into "old games are a good place to study neurons".
- the-corner-2 narrates the block landings the later mistakes rely on;
  the-map-3 says the orientation in Mario's terms; two-inputs says
  plus/minus like the rest; "them" given its noun; answer-keys-2 "holds,
  or lets us work out"; two-end-1 "the best of the borders it tried";
  draw-end "by the same plus number"; choosing says the third session is
  kept aside; modern-2's textbook clause moved to the end; `next`: "how
  its weights can be found" (the neuron isn't the actor).
- Pauses set per the pacing lens: 0.5–1.0 s after bridges, 3.0 s after
  payoffs (result, formula, example-air, steady-trade-2, land-example,
  land-balance-2, land-results-4, tldr-3).

## Not acted on

- depth-2 (all who mentioned it said cut or explain): its narration is
  cut; the 32-dot offset stays on screen in `depth`'s board, since Nate
  asked for it to be shown directly.
- "A sixth of a second" and "five-minute" stay literals (fixed by
  LAND_WITHIN = 10 and the recordings' length; noted for a later check).
- Moving the balance scale to chapter 4 (pacing): kept in chapter 6,
  where it now carries the minus-sign example.
