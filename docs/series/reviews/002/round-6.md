# Video 002, council round 6

Script reviewed: commit 156a247 (draft 15), with its storyboard
(`pacing.md`). The third and last council round on the scrolling
question. All eight lenses, told not to re-raise what drafts 14–15 had
fixed or declined. Draft 16 answers them.

## Found by several lenses

- **`shrink` cut** (skepticism, empiricism, cross-examiner, clarity).
  The rate / k step "calmed" the perceptron only by freezing it: by
  pass 2 the step is a ten-thousandth of its start, so the border stays
  about where the first pass left it. And "shrinking once a pass
  didn't" names the wrong difference: rate / p² calms it too, and
  rate / √k (every moment) doesn't; what matters is how far the step
  shrinks, not how often. No one-line version survives, so the beat is
  gone, with its code and checks; "with a fixed step" in tldr carries
  the qualifier (round 4's skepticism finding). The measurements are in
  games/002.md.
- **`perceptron-wander` cut** (cross-examiner, skepticism, lay,
  pedagogy, clarity, blind). "That's why its score kept jumping" was a
  cause never tested; rescaling position so its weight sits 8.5 nudges
  out left the swing at 13 points. Without it the beat is a measurement
  with no payoff, and "nudges from zero" was the unit lay viewers
  couldn't feel. `always-wrong` and `who-moves-it` explain the jumping.
- **`perceptron-rest`** (all but empiricism): "zero" meant a dial
  setting where it had always meant the total's dividing line; the
  premise came without a reason; the step "pulls that all lean one way
  can't cancel" was missing, and so was the condition "as long as some
  moment is wrong" (without it, it contradicts guarantee-check). Now two
  beats: each wrong moment "pulls against the weights that got it
  wrong", so every pull leans back toward "where the neuron started,
  with every weight and the bias at zero"; pulls that all lean one way
  can't cancel, "like a tug of war with everyone on one side"; only at
  the start could they, and there they pull out. exp002 now refuses to
  write numbers unless every late nudge leans back (never away). Bishop
  no longer cited for an argument he doesn't make.
- **target-example / target-over** (lay, skepticism, pedagogy): in both
  the perceptron is right and wouldn't move, which is the point; the
  comparison to "a perceptron mistake" hid it. Both now say so, and the
  pull is again "by four fifths of a step times its number" and "two
  and a half times as hard as it pulled the last one up".
- **`sheet`** (lay, clarity, cross-examiner): "step right" broke the
  word rule, and "all the way round by the sheet's right edge" was
  wrong (from the best border it's two-thirds of a turn). Now "the
  sheet is one full turn wide, so going off its right edge brings you
  back on its left".

## Clarity

1. inside: "the square trick's pulls cancel", "set by the squared miss
we gave it". 2. search-2: "one and a half blocks in". 3. step-size:
"after every nudge". 4. perceptron-rest (above). 5. sheet (above). 6.
shrink (cut). 7. perceptron-wander (cut). 8. never-outro: "only just
wrong". 9. The spine's answer stated once before tldr: map-outro now
ends "That's the difference between a rule that stops and one that
doesn't." 10. order reworded. 11. too-big: "step size", "after each of
ten steps". 12. spots: "The game keeps both numbers in small whole
units". 13. settles-2: "fourteen percent".

## Pedagogy

1–2. perceptron-rest (above). 3. The U and the bowl were never tied:
bowl-2 now says it is "the U from before, seen one border at a time".
4. perceptron-wander (cut). 5. staircase-outro (pacing 1). 6.
this-video promised "why" the second rule stops short, which why-short
admits it didn't find: now "how close a second rule… comes to the best
border". 7. target-over (above). 8. "steady" in tldr, "fixed"
elsewhere: now "fixed". 9. spots after numbers-2's speeds: "small whole
units". 10. Chapter 2's number load: minor; not changed.

## Skepticism

1. shrink (above). 2. perceptron-rest (above). 3. target-example /
target-over (above). 4. square-why's "the nudges shrink one number" is
contradicted by too-big: now "small nudges". 5. perceptron-wander (cut).
6. tldr is sound as a necessary condition; "fixed" (pedagogy 8).

## Empiricism

1. shrink (above). 2. The sheet at 300° left a flat sliver on its left
edge (300°–306.5°): now 310°, checked (edge columns under 5% flat). 3.
numbers-2 cites exp002, which doesn't check the speeds: now cites
smbdis, whose entry notes the ram.rs tests (24 and 40 sixteenths);
exp002 names them as WALKING and RUNNING. 4. Literals duplicating exp002
constants: search-2's and short's words are now checked. 5.
perceptron-wander (cut).

## The lay viewer

1. perceptron-rest (above). 2. target-example (above). 3. sheet (above).
4. inside (clarity 1). 5–6. best-size and shelf say what, not why:
kept; best-size gives the reason (doubling doesn't move the border but
changes the miss), and shelf now gives the flat height's (every miss is
one). 7. perceptron-wander (cut). 8. Chapter 2's run of results: shrink
is cut, and `remember` gets 3.0 s. 9. step-size-2's qualifiers: "with a
fixed step, starting from zero" cut. 10. order's coincidence (the run
followed was the outlier) looks picked: it was the first order, chosen
before the results; the board lists all ten. 11. learn: the bias is now
"a fixed amount added on". 12. settles-2 (clarity 13).

## The blind listener

1. Where the square trick's border sits was board-only: `short` now
says it leans on speed: "standing still, it never says yes before the
mark; at full walking speed, it says yes almost from the left edge"
(5.69 and 0.33 blocks, checked in exp002). 2. perceptron-rest's "zero"
(above). 3. shelf: now "the side of the sheet where the ruler has
turned round to face the wrong way". 4. perceptron-wander (cut). 5.
the-line: five blocks is now "about a third of the way across". 6.
staircase-2's cliff: board-only and not needed by the argument; kept.
7. map-intro: fine as is.

## The cross-examiner

1. perceptron-wander (cut). 2. shrink (cut). 3. sheet (above). 4.
perceptron-rest: the claim holds (cosines −0.98 to −0.04); now checked
in exp002. 5. dropped: minor; not changed.

## Continuity and pacing

1. Chapter 5's perceptron half wasn't announced: staircase-outro now
asks "what is the square trick following instead, and why does the
perceptron have nowhere to rest at all?", and `bowl` opens "To see
both". 2. "The lowest ledge" was first spoken in two-bottoms: no-slope
now ends "to the lowest ledge, where the best border we found sits". 3.
never-intro now states the chapter's problem: "With the troublemakers
back, no border of any shape could get all of Mario's moments right, so
the rule always has a mistake to react to." 4. perceptron-rest is now
two beats. 5. downhill's 3.0 s: given, then taken back for chapter 3's margin
(its first build was 3:59); `square-why-2` folded into `square-why`. 6. bowl-2 tied to the U (pedagogy
3). 7. The Introduction reaches the video's subject about 2:50 in:
kept; the new question needs its setup, and the Introduction is under
the limit even read slower. 8. remember gets 3.0 s. 9.
step-size-2/step-size say one claim twice: kept; the second is the
check. 10. Chapter 4's title covers its first beats: kept ("Close, but
not quite" is the chapter's finding). 11. map-outro now says why it
mattered. 12. inside repeating tldr: it now restates where each rule's
pulls cancel, the spine.

## Code changes

- `rules.train` loses its `shrink` option (and its tests).
- exp002: no shrinking runs; `batch_square_cut_blocks`,
  `batch_square_cut_walking_blocks`; `WALKING`, `RUNNING`; the sheet
  starts at 310°. Claims refused unless every late perceptron nudge
  leans back toward zero, unless the sheet's edges are sloped, and
  unless both borders fit the words search-2 and short use; tested.
