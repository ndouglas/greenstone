# Video 002, council round 5

Script reviewed: commit dcc7c79 (draft 14), with draft 14's storyboard
(`pacing.md`). All eight lenses, told not to re-raise what draft 14 had
fixed (`round-4.md`). Draft 15 answers them. Each lens's findings in
brief, then the response.

## Found by several lenses

- **perceptron-score / perceptron-hover** (all but empiricism and the
  cross-examiner): "lowest at zero, where no mistake costs anything",
  then "at zero… every moment where the screen moved is wrong"; a slope
  the staircase chapter had just said couldn't be followed. Skepticism
  also measured that the weights never come near zero (never within
  about 9 nudges; 58% of late nudges push them further out), so "pulled
  back, pushed out again" is a cycle that doesn't happen. Now one beat,
  `perceptron-rest`, argues where a rest could be: added up, the pulls
  point partly back toward zero, so zero is the only place they could
  cancel, and at zero they push outward. The perceptron's own measure
  is gone.
- **perceptron-hover-2's "only a few nudges"** (empiricism,
  cross-examiner, skepticism): the weights are about 15 nudges out; only
  position is about 3. Now `perceptron-wander`: "about fifteen nudges
  from zero, but the weight on position only about three"; "35%" is now
  "about a third" (pedagogy: one in three); "tilt" is "how much position
  counts against speed" (blind).
- **batch's 83–86%** (empiricism, cross-examiner): only its last 100 of
  300 steps (49–88% over all). Now "over its last hundred steps".
- **Walking and running** (empiricism, cross-examiner, skepticism): the
  example's 2 pixels a frame is faster than the walking top just given;
  "mostly running right" isn't what SMB calls running (B is held about
  20% of frames). `numbers-2` (new, with the frame aside moved out of
  `numbers`) gives walking's 1.5 and running's 2.5, with B; `the-map`
  says "mostly heading right"; the example is now 4 blocks, 1 pixel a
  frame.
- **search-2** (pedagogy, clarity, skepticism, cross-examiner): "each
  pixel a frame counts for one and a half blocks more" flips direction,
  mixes units, and understates 1.61. Now its two ends: "standing still,
  it waits until he's about four blocks in; at full walking speed, it
  says yes from about one and a half" (3.98 and 1.56, both now computed
  and checked in exp002 to fit the words).

## Clarity

1. this-video's "it" named two rules: now "the neuron's rule" and "a
second rule, which does stop". 2. perceptron-score (above). 3. `inside`
ended on a third idea: now restates where each rule's pulls cancel. 4.
settles-2's "fourteen" came from nowhere: now "over its last ten passes…
the perceptron's typically swung by fourteen". 5. "Point" meant a map
place and a score share: scores are "percent" now. 6. spots' "whole
steps": now "whole units". 7. tldr's stacked opener: "On Mario's
moments, which no border can split, the perceptron's pulls never do."
8. target-over split into short sentences. 9. "the Right button… rightly
said yes": now "correctly". 10. map-outro rewritten (pacing 4). 11.
step-size-2: "doubling the step doubles every weight"; best-size:
"doubling them all doesn't move it". 12. search-2 (above). 13. dropped:
"A tiny change to the moments moves it too." 14. learn's appositive:
reordered. 15. Minor items taken where they touched lines changed here.

## Pedagogy

1. search-2 (above). 2. perceptron-score (above). 3. "Ruler" meant the
inputs' scale and a border: spots now uses "a clock that only shows
whole minutes". 4. best-size now says one border comes in many sizes.
5. balance now calls back the contrast: each pull shrinks as its total
nears its target, "which is what the perceptron's pulls never do". 6.
"point" (clarity 5). 7. too-big's "valley": now "the U". 8. "35%"
(above). 9. The shrinking-step caveat in an outro (pacing 5).

## Skepticism

1. never-outro's shrinking step held for one schedule only (rate / k);
once a pass, or by √k or √p, it still swings 12–14 points. Now its own
beat, `shrink`, after `remember`: "shrink the step after every moment,
in step with how many moments it has seen… Shrinking it only once a
pass didn't." exp002 now runs the once-a-pass schedule and refuses to
write numbers unless it still swings (over 5 points; it's 12). The note
that Robbins–Monro promises only a drift toward zero is in games/002.md.
2. The perceptron's cycle (above). 3. search-2 (above). 4. tldr's
"a learning rule can only stop where its pulls cancel" is contradicted
by the shrinking step and by the per-moment square trick: now "with a
steady step, a learning rule can only come to rest where its added-up
pulls cancel out". 5. "mostly running right" (above).

## Empiricism

1. Walking and running (above). 2. perceptron-hover-2 (above). 3. batch
(above). 4. search-2's literals unchecked: now computed and checked. 5.
too-big is right, but the settled step also crosses the bottom along the
steepest way, landing lower: noted on the board and in games/002.md. 6.
A stale `wrong_nudges_times_share` in numbers.json (the file was written
before that code was removed): regenerated. 7. Optional: unseen at the
mixed points (yes 88%, no 53%). Not taken; the line compares like with
like on the border's yes side.

## The lay viewer

1. The sheet had nothing concrete: `sheet` now starts from "the best
border we found" (step right, it turns; step up, it slides), and the
half-turn swap is "the same line, but saying yes where it said no". 2.
perceptron-score (above). 3. target-example's arithmetic: now "four
fifths as far as a perceptron mistake would move it, for the same
step"; target-over "twice as far as a perceptron mistake would". 4.
settles-2 (clarity 4). 5. Number fatigue in chapter 2: `dropped` now
says "more than ten thousand" rather than the exact count. 6.
step-size-2 had no reason up front: now opens "You might think a bigger
step means faster learning." 7. shelf (blind 2). 8. "step" (taken where
lines changed). 9. search-2 (above). 10. "35%" (above). 11.
why-short/how-far's "beyond its target, on its own side": kept; it is
the plainest exact wording found. 12. tldr (clarity 7).

## The blind listener

1. "tilt" (above). 2. Why the shelf is flat and where: `shelf` now says
every total is zero and every moment misses by exactly one, "filling
one side of the sheet"; exp002's sheet now starts at 300°, so the bowl
is one piece on the left and the shelf, 120°–300°, fills the right. 3.
"dial" said first in downhill: now "Change just one weight". 4. `next`'s
"the line": kept; it's 003's title, and the neuron's line. 5. The dots'
layout: search-2 now says where the yes side starts. 6. big-drops now
places the point: "two and a half blocks in and not moving sideways"
(cross-examiner: 28% of those moments are in the air, so not
"standing"). 7. order's "the run we just watched": now "the one we
followed". 8. "point" (clarity 5). 9. "the mark" against "Mark One":
minor, fifteen beats apart; kept.

## The cross-examiner

1. batch (above). 2. The sheet's layout (blind 2). 3. Walking and
running (above). 4. hover-2 (above). 5. big-drops (blind 6). 6. order:
"ten different orders… the one we followed among them". 7.
who-moves-it's 43% is a median over ten orders: now "in a typical run's
last ten passes… about". 8. search-2 (above). 9. how-far's "close":
optional, not taken; `short` gives the gap in score and in answers.

## Continuity and pacing

1. staircase-outro asked "where" after `short` had answered it: now a
zoom-out ending "So what is the square trick following, and why does it
stop where it does?". 2. map-intro promised "why" no fast way is known;
the staircase shows why slope-following fails: now "a picture of what
that would take", and square-outro no longer restates `no-fast-way`.
3. why-bowls sounded like an ending: its last clause is cut. 4.
map-outro repeated the TL;DR: now a short zoom-out. 5. never-outro
added a new fact: moved to `shrink`. 6. square-intro didn't connect to
"what would a rule need to stop?": now "so its pulls can shrink as it
gets closer". 7. never-intro's question waited 45 s for its answer: it
now opens `always-wrong`. 8. 3.0 s after `sheet` (and after the two
perceptron beats); numbers split into `numbers` and `numbers-2`. 9.
tldr-2/tldr-3 said 88% twice: tldr-3 now "whose bottom falls short of
the lowest ledge's 91%". 10. Chapter 3's margin: draft 15's storyboard put
it at 4:00 (about 4:36 at a slower reading), so `animals` is cut, the
one beat the argument doesn't use (Rescorla–Wagner stays in the
sources): 3:48. 11. Minor ordering: taken
where it touched the lines above.

## Code changes

- `rules.train(shrink=...)` takes "moment" (rate / k) or "pass"
  (rate / p); both tested against a hand-rolled loop.
- exp002: `perceptron_pass_shrinking_swing_points`,
  `search_cut_walking_blocks`; claims refused unless once-a-pass
  shrinking still swings, and unless the best border fits search-2's
  words; both tested. The sheet's angles now run 300°–660°.
