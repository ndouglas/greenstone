# Adversarial review A (draft 4, commit e28db60)

1. map-outro, tldr-2, never-outro (incorrect): the square trick doesn't settle because each nudge shrinks; at rest most moments still miss by a lot. Their pulls cancel, so their sum shrinks. `settles` is right; the conclusions contradict it. Fix: "its pulls cancel near the bottom; the perceptron's never do".
2. step-size, always-wrong (handwaving): "pass" used before `passes` defines it. Fix: define it earlier.
3. square-why (incorrect): one moment's nudge can raise the total; only on average, or summed, does it shrink.
4. square-intro (incorrect): "often called the square trick": the name is Serrano's, for regression; the field says Widrow–Hoff, LMS, delta rule. Fix: "one textbook calls it".
5. learn, one-at-a-time, rule-wrong, always-wrong, never-outro (unsupported): the 1958 paper is a probabilistic reinforcement model; the explicit error-correction procedure is in Principles of Neurodynamics (1962) and Novikoff. "Starts at zero" needs exp002. Fix citations.
6. perceptron-score (two thoughts, handwaving): four thoughts; at zero weights there's no border, so the picture silently becomes a map of weights. Fix: split; say the picture shows weights.
7. bowl (handwaving): why half is flat isn't explained (best size is zero; every miss is exactly one).
8. square-name (two thoughts, unsupported): three thoughts, two pictures; "the first rule I wrote myself" isn't supported (the repo's three tricks arrived in one commit, for fitting a line). Fix: split; "a version of this rule, for fitting a line, was among the first I wrote".
9. guarantee (two thoughts): theorem and control; split.
10. settles-2 (two thoughts): split.
11. test-question (framing with a claim): cite exp001.
12. too-many (unsupported): 2,048 needs exp001 or a memory source.
13. remember (emphasis): remembering buys consistency (89–90 whatever the order), not a higher top.
14. short (unsupported): "tilted a little" unmeasured (could be a shift). Drop.
15. target (minor): Bishop 4.1.3 is the closed form; the one-moment nudge is 3.1.3.
16. map-outro (cliché): "The choice was never between a good rule and a bad one". Drop.
17. same-scale (minor): say why: so one step size suits both.
Checked: all numbers match numbers.json and exp002.py.

# Adversarial review B (draft 4, commit e28db60)

1. square-why (incorrect): one moment's nudge can raise the total; "on average".
2. short (incorrect, unsupported): "tilted a little" unmeasured; drop.
3. settles (emphasis): pulls balance between moments that fall short and moments that overshoot, not sides of the border.
4. square-name (two thoughts, unsupported, bible): three thoughts, two pictures; "the first rule I wrote myself" unsupported; the bible says Nate stays offstage; bell and food is folklore, Rescorla–Wagner is general conditioning. Fix: cut the personal line; split; "a sound predicting food".
5. step-size, always-wrong (term before definition): "pass"; order's "thirtieth pass" before 30 passes are stated.
6. bowl (skipped step): a ruler that tilts and slides can't "get most moments wrong" unless tilting round flips its yes side; a backwards border's best weights are zero.
7. perceptron-score (handwaving, two thoughts): split; state why hovering near zero makes the score jump.
8. square-intro (incorrect): "often called": Serrano's name; LMS, delta rule, Widrow–Hoff.
9. same-scale (handwaving): say why.
10. guarantee (two thoughts): split.
11. map-outro (cliché).
12. test-question (framing with claims): cite exp001; "within ten frames".
13. rule-wrong, one-at-a-time, learn (unsupported): lean on serrano2021, exp002, rosenblatt1962.
14. remember: Gallant's pocket keeps the longest run of correct answers; ours is a version; "89 to 90".
15. step-size-2: "the same positive amount".
Checked: the numbers match numbers.json and exp002.py.

## Continuity and pacing (draft 4's storyboard)

No section over time (longest 3:19); continuity with 001's `next` and 003 holds.
1. perceptron-score delays the payoff between two-bottoms and why-bowls: move before two-bottoms, or cut; pause 3 if kept.
2. staircase carries four thoughts: split; bowl's "weights that suit that border best" hard to picture.
3. never-outro and square-outro look backward only: end each with the next question (square-outro → "so why not aim straight at the fewest mistakes?").
4. Pauses: 3 s on dropped, who-moves-it, target-example, short; split rule-wrong; step-size-2 pause 1.
5. Digressions: move square-name after short; move or cut codes.
6. this-video promises "what learning does about that": "what learning does instead".
7. tldr-2 omits the staircase; inside needs a lead into next.
8. never-intro: split into the claim and the spots; pause 3.
9. settles-2: "about one point" odd.

## Response (draft 5)

Acted on (both reviews, and the pacing lens, which read draft 4):

- **A1 / B3:** the square trick's pulls *cancel* near the bottom (moments
  that fall short against moments that overshoot); `settles`, `map-outro`
  and `tldr-2` now say that, and the perceptron's pulls "never cancel
  out, even added together" (new check: the batch perceptron moves on
  all 300 steps).
- **A3 / B1:** `square-why`: each nudge shrinks that moment's miss; the
  total shrinks on average.
- **A4 / B8:** "known as the Widrow-Hoff rule, and one textbook calls it
  the square trick".
- **A5 / B13:** the rule's citations lean on Rosenblatt 1962, Serrano and
  Novikoff; `learn` cites the experiment for the zero start; `rule-name`
  keeps 1958 for the dates.
- **A6 / B7:** `perceptron-score` split in two; its board says the picture
  is of weights, not borders; `perceptron-hover` states the link (each
  nudge is about half the weights' size).
- **A7 / B6:** `map-axes` says turning the ruler round swaps its sides;
  `bowl` says why the shelf is flat (best weights zero, every miss one).
- **A8 / B4:** `square-name` split from `animals`; "the first rule I wrote
  myself" replaced by what the repo supports ("one of three I wrote");
  the bell replaced by "food after a sound".
- **A2 / B5:** a pass is defined in chapter 1 (`passes`), with thirty
  passes; the exam half moves to chapter 2's `exam`.
- **A9 / B10 / A10:** `guarantee` and `settles` split.
- **A11 / B12:** `test-question` cites exp001, "within those ten frames".
- **A12:** `too-many` cites exp001.
- **A13 / B14:** `remember` says 89–90% whatever the order.
- **A14 / B2:** "tilted a little" cut.
- **A15:** Bishop cited for 3.1.3 too.
- **A16 / B11:** `map-outro` rewritten without the cliché.
- **A17 / B9:** `same-scale` says why.
- **B15:** "the same positive amount".
- **Pacing:** `staircase` and `never-intro` split; pauses on `dropped`,
  `who-moves-it`, `target-example`, `short`, `two-bottoms`,
  `perceptron-hover`; `rule-wrong` split from `rule-bias`; `codes` cut;
  `square-name` after `how-far`; `never-outro` and `square-outro` end
  with the next question; `this-video` "what learning does instead";
  `tldr-2` names the staircase; `inside` mentions the moments shown.

Not acted on:

- **B4** (cut the personal line; the bible says Nate stays offstage):
  kept in a smaller, supported form for Nate to decide in the interview.
- **Pacing 9** ("about one point"): kept; it's what was measured.
