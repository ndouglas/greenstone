# Adversarial review C (draft 5, commit db87d0d)

All numbers match numbers.json and exp002.py; the bowl claim matches maps.npz.
1. tldr-2 (incorrect): the staircase is why we don't aim at mistakes, not why the square trick stops short; it stops short because its total bottoms out elsewhere.
2. rule-bias (emphasis): "a little" contradicts perceptron-hover; guarantee board: it stops on *a* splitting border.
3. square-intro board: perceptron nudges scale with the moment's numbers, so "same size" is wrong: "doesn't depend on how far over".
4. no-slope / why-bowls: "no rule can walk it" too strong ("no rule that follows a slope"); never says the square trick's nudges are steps down the bowl; the bowl's lowest point is where the rule comes to rest (checked).
5. square-why: the name's origin cites sources that don't use the name; cite serrano2021 or cut.
6. perceptron-hover: "never cancel" rests on an unshown batch experiment, and "never" on 300 steps; a second thought. map-outro repeats it.
7. target-example: "the weights move up" is wrong when a number is below average; "the total is pulled up".
8. bowl: "the best weights are nothing at all" for ~96% of such borders; two pictures: split.
9. this-video: "never quite settles" undersells; drop "quite".
10. remember: ours is a version of Gallant's pocket (his keeps the longest run of correct answers).
11. Two thoughts: square-intro, target, map-axes, square-name (also "Nate stays offstage").
12. step-size: the step doesn't matter for this rule only.
13. map-outro: "Each rule pays for what it gets" is filler.
Nothing out of order against the bible.

# Adversarial review D (draft 5, commit db87d0d)

All numbers match; least squares re-run.
1. settles (incorrect): the tug of war is yes-moments against no-moments falling short (summed pulls about (501, 2484, 3094) against (233, −2083, −3557)); overshooting moments are only no-moments and pull a small leftover (−734, −401, 463). Fix: "yes-moments that fall short pull one way, no-moments the other".
2. tldr-2 (wrong cause): the bowl's bottom isn't the lowest step; the staircase is why we don't aim at mistakes.
3. bowl (overstated): "nearly every border that gets most moments wrong".
4. perceptron-score / hover (handwaving): why it doesn't settle at zero: there it says no to everything, so every yes-moment pushes it away, at full size.
5. perceptron-hover (two thoughts): the batch claim needs its own beat saying what was run.
6. square-why (unsupported): the name needs serrano2021.
7. square-name (two thoughts, bible): history and Nate's repo; "Nate stays offstage".
8. target-example: weights move "toward the target" (a negative number moves its weight down); map-axes: a half turn swaps the sides, not a full one.
Lesser: perceptron-score's board is a loss landscape over weights (006's); remember: a version of Gallant's pocket; map-outro aphorism.

## Response (draft 6)

All acted on except where noted:

- **C1 / D2:** `tldr-2` gives the right cause: learning aims at the bowl
  because the staircase has no slope, and the bowl's bottom isn't the
  lowest step.
- **D1:** `settles`: landed moments and not-landed moments pull against
  each other, each toward its own target.
- **C2:** "a little" cut; `guarantee`'s board stops on *a* splitting border.
- **C3:** `square-intro`: the perceptron's nudges are sized only by the
  moment's numbers, not by how far over it is; its board drawn that way.
- **C4:** `no-slope` "no rule that follows a slope"; new `bowl-2` says the
  square trick comes to rest at the bowl's bottom (checked in the run).
- **C5 / D6:** the name cites Serrano; the usual name moves to
  `square-name`.
- **C6 / D5:** the batch perceptron gets its own beat, `batch`, saying
  what was run (300 steps, moving on every one, mostly 73–90%).
- **D4:** `perceptron-hover` says why it can't rest at zero.
- **D lesser (006's landscape):** `perceptron-score`'s board shows dials,
  not a landscape over weights.
- **C7 / D8:** `target-example` moves weights "pulling the total up";
  `map-axes` half a turn swaps the sides.
- **C8 / D3:** "nearly every border"; `bowl` split into shelf and bowl.
- **C9:** "never settles down".
- **C10 / D lesser:** "a version of an idea called the pocket algorithm".
- **C11:** `target` split from `target-rule`; `map-axes` from `sheet`;
  `square-name` from `my-version`.
- **C12:** "for this rule".
- **C13 / D lesser:** the aphorism cut from `map-outro`.

Not acted on:

- **C11 / D7** (cut Nate's repo line; the bible says he stays offstage):
  now its own beat, `my-version`, for Nate to keep or cut.
