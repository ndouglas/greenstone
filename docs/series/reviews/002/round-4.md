# Video 002, council round 4

Script reviewed: commit 4f7dd5b (draft 13), the first council round on
the scrolling question (rounds 1–3 reviewed the landing question, now
replaced). All eight lenses, run in parallel; pacing read draft 13's
storyboard (`pacing.md`). Draft 14 answers them. Each lens's findings in
brief, then the response.

## Found by several lenses

- **"Line" meant two things** (clarity, pedagogy, lay, blind): the
  five-block point where the screen starts to follow Mario, and the
  neuron's border, so the title read as "mistakes move the scroll
  point". Now "the mark" is the game's, introduced in `the-line`; "line"
  and "border" are only the neuron's.
- **`unseen`** (all but pacing): "That's because the neuron can't see
  what the player will press next" claimed a single cause. At spots
  holding both answers, "no" moments still had Right held 59% of the
  window; a third input for presses lifts a straight border only from
  90% to 94%. "When it was right" mixed right yeses (81%) with right
  nos (74%), and aloud put two "rights" in one sentence. Now: "One
  reason is…", "held the Right button", false yes 31% against true yes
  81% (`right_ahead_true_yes`, new), and "the random player", said in
  `the-map` too.
- **`only-these`** (skepticism, cross-examiner, empiricism): "Walking
  left, the screen never moves" is false for the half second asked
  about: 6.5% of those moments saw it move, after he turned round. Now
  "can't move until he turns round, so we leave those moments out".
- **`too-big`** (cross-examiner, empiricism, skepticism): off by one in
  `rules.batch_square_misses` (it recorded the miss before each step,
  so 60× was after 9 steps; after 10 it's 137×); and the mechanism was
  wrong (for the first steps no total crosses its target; the weights
  overshoot the bottom). Fixed in the code with a test; now "each step
  jumps clean over the bottom of the valley and lands higher on the
  other side".
- **`tldr-2`** (clarity, pedagogy, lay, blind, pacing): one 50-word
  sentence. Now three short ones.
- **perceptron-score through batch-2** (lay, pedagogy, pacing, clarity):
  a third explanation of never settling, abstract, in the middle of the
  staircase/bowl argument, and it put chapter 4 three seconds under the
  limit. Moved to the end of the bowl chapter, before `map-outro`, cut
  from six beats to four, with `perceptron-hover` saying the cycle once
  (pulls toward zero; at zero, pushed out again).

## Clarity

1. "line" (above). 2. tldr-2 (above). 3. No single idea at the end: the
Conclusions now open "a learning rule can only stop where its pulls
cancel out", and `this-video` promises "what a rule needs in order to
stop". 4. perceptron-hover and batch-2 sounded opposite: merged into one
cycle. 5. "Step" meant size, move and stair: stairs are now "ledges".
6. "Spot" meant a place on the map and on the sheet: the map's are now
"points"; "total squared miss" kept as a name. 7. `remember` rewritten;
`order` now says "ten different orders". 8. `order`/`dropped` reworded
(see empiricism 3). 9. step-size-2: "the same number of times bigger".
10. `who-moves-it` now calls the troublemakers by name; `guarantee`
"the closer the yes moments come to the no moments". 11. unseen (above).
12. no-shelf cut (see pedagogy 2). 13. square-outro now recalls
`no-fast-way`; `map-intro` is "Here's a picture of why". 14.
perceptron-hover-2 ends "That's why its score kept jumping". 15. Smaller
items taken: rule-intro's mid-sentence bias definition dropped (`learn`
gives it); target's "plus one sits above zero, and minus one below";
target-over "as the last one pulled up"; settles-3 "first, then takes
one small step"; too-big "the step that settled"; sheet "from the
sheet's left edge to its right"; tldr's "even on just". Not taken:
settles-2's "points" (now set against the perceptron's 14, which makes
it clear).

## Pedagogy

1. "Up if short, down if overshot" read wrongly for a "no" moment (a
total of −3 against −1 has overshot, yet is pulled up): the rule is now
by position, "below its target, pulled up; above, down", in
`target-rule`; `settles` (which repeated it) is cut. 2. The shelf
brought in three new ideas and then took them back: `shelf` is one line,
the size-below-zero sign flip and `no-shelf` are gone, and `best-size`
recalls that scaling doesn't move the border. 3. "line"/"right" (above).
4. The pocket version (90–91%) beats the square trick: `remember` now
ends "that's the rule plus a memory; the rule itself still never
settles", and tldr-3 says fewest mistakes "did better here". 5.
perceptron's own measure (above); "stay small" now "stay only a few
nudges from zero". 6. tldr-2 (above). 7. shelf-2's double "most": the
line and its number are gone. 8. No baseline: `search` adds "always
saying yes would be right 51% of the time". 9. `animals` kept: it
mirrors `mark-one` in chapter 1, and the lay viewer called it a welcome
breather. 10. square-why-2 kept as is: true, and short.

## Skepticism

1. only-these (above). 2. "Never settles" also comes from the fixed
step: a step shrinking as rate / k calms it (median swing 0.16 points).
step-size-2 now says "with a fixed step", and `never-outro` adds that a
shrinking step would calm it, "a schedule we'd impose, not the rule";
measured in exp002 and checked (under a point), citing Robbins and
Monro. 3. unseen (above). 4. too-big (above). 5. batch-2's "adding them
up doesn't help": `batch` now says it calms it (83–86%) but it still
moved on every step. 6. who-moves-it: "moment for moment, far more than
their share". 7. one-run: now "as high as 90%, and on its very last
pass, down to 67%". 8. tldr-3 "did better here". 9. "Every border" is a
grid: kept "as last time", 001's wording; the 720 × 601 sheet agrees.

## Empiricism

1. unseen (above). 2. The player was never said to be a random program:
`the-map` now says so. 3. The run shown is the outlier (its 67% the
lowest of ten orders): `order` now says all but one ended between 82%
and 90% (`perceptron_second_lowest`, new) and that the run we watched
was the one that fell to 67%, checked in the run; tldr likewise. 4.
batch (above). 5. too-big (above). 6. Baseline (pedagogy 8). 7.
only-these (above). 8. shelf-2's 83% depends on the sheet's extent (95%,
82%, 68%): no longer said; noted in games/002.md. 9. The 91% vs 88% gap
holds on every split (1.6–3.1 points): noted in games/002.md. 10. The
control's gap measure is search's, near zero by construction:
`guarantee-check` drops "the two sides come very close" (the colon tied
it to the 100 passes, which the bound doesn't explain); noted. 11.
rule-wrong's (4, 2) isn't a recorded moment: the board labels it an
example. 12. Counts are of training moments: minor, not changed.

## The lay viewer

1. perceptron-score → batch-2 (above). 2. "line" (above). 3. unseen
(above). 4. The shelf (pedagogy 2). 5. square-why/downhill: `downhill`
now says the slice aloud ("Turn just one dial… a U"); "on average"
dropped from square-why. 6. target-example's fractions: "times its
number" kept (it is the rule), "it's below by four fifths, so each
weight moves up…"; target-over now names its surprise ("corrected for
being too right"). 7. step-size-2 (clarity 9). 8. "step" (clarity 5).
9. tldr-2 (above). 10. spots: "like a ruler with no in-between marks".
11. "thirty passes" then "100": "Given as many passes as it needed".
12. remember (clarity 7). 13. settles-2 now "against the perceptron's
14". 14. `numbers` adds "a frame is a sixtieth of a second, and walking
tops out at one and a half pixels a frame" (the ram.rs test); `search`
says "recordings it wasn't chosen from".

## The blind listener

1. "line" (above). 2. The listener never heard what the best border
does: new `search-2`, "it says yes once he's past about four blocks,
and each pixel a frame of speed counts for about one and a half blocks
more" (blocks + 1.61 × speed > 3.98). 3. unseen (above). 4. "spot"
(clarity 6). 5. Two grounds: `downhill` says the U; `no-shelf` is cut.
6. square-intro's "over": now "past the border on the wrong side". 7.
tldr-2 (above). 8. batch-2's direction in dial-space: said as dials in
`perceptron-hover`. 9. step-size-2 (clarity 9).

## The cross-examiner

1. unseen (above). 2. too-big (above). 3. only-these (above). 4. The
game keeps sub-pixel parts the two bytes don't show: `spots` now says
"the very same two numbers"; noted. 5. guarantee-check (empiricism 10).
6. rule-wrong (empiricism 11). 7. dropped: "a different 30 each time.
In ten tries…". 8. `next`'s cheating byte is 003's plan in the bible.

## Continuity and pacing

1. Chapter 4 (4:27): the perceptron block moved into chapter 5 (above).
2. `staircase-outro` now ends "Where, and why there?", and `bowl` opens
"To see where the square trick rests…". Title "A bowl" kept. 3. `learn`
was said twice, 100 s apart: merged into one `learn`, after
`no-fast-way`; `the-line` opens "This time, we need a new question".
4. this-video (clarity 3). 5. never-intro now asks "So what does the
rule do, when something is always wrong?". 6. 3.0 s pauses on
rule-wrong, target-rule, spots, too-big and batch (shelf-2 is gone). 7.
1.0 s on map-intro and rule-bias. 8. tldr-2 (above). 9. who-moves-it
moved to after always-wrong; `dropped` kept. 10. `settles` cut, its one
new idea in `balance`. 11. `short` names the square trick. 12.
big-drops kept: its picture (the standing start) is the one place the
crowding becomes concrete; first to cut if needed. 13. map-outro now
zooms out from the staircase to the bowl first.

## Code changes

- `rules.batch_square_misses` returns the miss before the first step
  and after each (31 values for 30 steps); test updated.
- `rules.train(shrink=True)`: the step at the k-th moment visited is
  rate / k; tested against a hand-rolled loop.
- exp002: `right_ahead_true_yes`, `{rule}_second_lowest`,
  `perceptron_shrinking_swing_points`; claims refused unless the run
  shown ends on its lowest pass and lowest of all orders, and unless a
  shrinking step calms it under a point; both tested.
