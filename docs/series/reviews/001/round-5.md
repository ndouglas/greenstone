# Video 001, council round 5

Script reviewed: commit 6501294 (122 beats). All eight lenses. Blind
listener and empiricism: nothing substantial (empiricism reproduced
numbers.json byte for byte and checked every new claim on the
recordings). Revised script: 128 beats, ten sections.

## Substantial, and what changed

- **testing-end** (all six other lenses): "his falling speed as well
  doesn't help a straight line here" used falling speed and lines before
  either is introduced, and was heard as "speed doesn't help with the pipe
  mistakes", which is false (the "speed isn't zero" rule scores 97%; it
  takes two cuts, video 004). The bridge no longer makes the claim: "So it
  goes wrong where it only knows about the floor. Other questions need two
  numbers from the start". The praise beat (testing-end-1) now closes the
  evidence, before the mistakes, and gives the real reason it isn't luck
  (every session picks the same setting).
- **The zero beats** (clarity, pedagogy, lay viewer, pacing): "the same
  trick" pointed two beats back; "nothing is left over" didn't say from
  what. Now why-zero, why-zero-3, why-zero-2: why-zero works a case
  (130 above 100 is 30 above zero); why-zero-3 "Computers move a number
  across in the same way"; why-zero-2 "the test has no number of its own
  … since one times the bias is just the bias". `next` pays it off
  ("its own weights, bias included").
- **one-dot-4** (clarity, cross-examiner, skepticism): "only because the
  test is against zero" asserted, and three times zero omitted. Now says
  zero stays zero, and one-dot-4b shows it: against 100, a total of 50 (a
  no) tripled is 150 (a yes).
- **Rounding** (pedagogy, cross-examiner, skepticism): draw-intro and
  the-tilt-2 said rounding leaves answers unchanged; it shifts the border a
  hair (raised-landing misses 90% vs 85%). draw-intro: "messy numbers into
  simple ones while barely changing its answers"; the-tilt-2 says the
  search's numbers were long decimals and only scaling is free; the-tilt-4
  "Rounding the rest then moved the border only a hair: it's still right
  91%".
- **the-tilt order** (blind listener, pacing): the formula is finished
  ("If that total is above zero…") before the rounding aside.
- **found-end** (cross-examiner): the border's rule stated as Mario's;
  now "to land on the floor soon".
- **two-end** (clarity, cross-examiner): "the whole of what one neuron
  like this can do" read as a limit on inputs; now "However many numbers
  it reads, one neuron like this can only split them with a straight
  border."
- **next** (cross-examiner): promised weights "can be found" by learning,
  which this data contradicts (the rule never settles); now "how it can
  try to find its own weights … and why, on these games, that never quite
  settles".
- **land-balance** (pacing): split in two. With the-tilt-4 added,
  chapter 6 would have run near 4:00, so the balance scale became its own
  chapter, "Weighing it" (intro: sums with minus signs are hard to follow
  by ear; conclusion: worked or weighed, both say not yet).

## Polish applied

memory "try neurons out"; recipe-2 "instead of a setting, the test is
against zero"; our-numbers-2 "makes the total zero at"; its-mistakes-2
"There, he's on the ground"; steady-player split; the-neuron pause 3.0;
school-2 pause back to the default; recipe-end "multiply all its numbers
by the same amount"; the-corner-2 "on the map, at smaller depths";
settings-are-borders-2 gives its reason; settings-are-borders-3 "the
two-number version of the first neuron's setting" (no graph/map blur);
bias-slides-2 "one weight against the other"; how-found names Mario;
the-map-3 "level"; steady-trade-2 names each five; modern "often with a
bias".

## Code

exp001's `check_claims` now also guards the-corner-2 (raised landings at
median depth 114, floor 155) and land-results-3 (among falling moments,
depth alone 84% against 57.5% for always "not yet"; overall it's level).

## Not acted on

- why-zero-3 is an equality test ("is it 165?") while the neuron's is
  "above" (skepticism, cross-examiner: accurate as spoken; noted for the
  interview).
- The "32 dots" stays on the depth board (Nate asked for it shown).
