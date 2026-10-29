# Video 001, council round 6 (the last of three for the re-cut)

Script reviewed: commit 88fc706 (128 beats). The six lenses that had
substantial findings in round 5 (blind listener and empiricism had none).
Skepticism: nothing substantial. The two substantial findings left were
both introduced by round 5's edits, and both are fixed.

## Substantial

- **the-tilt-4** (clarity, pacing; pedagogy as polish): "it's still right
  91%" assumed a score never given and spoiled chapter 8's reveal. Now
  "Rounding the rest then moved the border only a hair, and barely changed
  its answers."
- **The scale's promise** (lay viewer, pedagogy, cross-examiner):
  found-end and weigh-intro promised "no minus signs", then the example
  said "minus" four times. The scale removes taking away, not minus
  numbers: weigh-intro "Sums that take numbers away are hard to follow by
  ear. Here's a way … where every number is only ever added, to one pan
  or the other"; found-end "a way to work it by only adding"; weigh-end
  "weighed instead of summed, only ever adding".

## Polish applied

- testing-end: "goes wrong because all it knows about is the floor"
  (echoes land-mistakes-3); "Other questions need two numbers to answer
  well".
- other-games now says every session picks the same setting in each game,
  which testing-end-1 had claimed beyond what the chapter showed;
  testing-end-1 gives "on play it wasn't chosen on" as its reason.
- why-zero: "take the 100 off the total, by taking it off the bias"; pause
  3.0. why-zero-3: "Computers do the same", now with an "at least" test
  matching the neuron's "above": the `cmp #10` / `bcs` at $801B, checking a
  digit of the top score (bytes checked in the ROM; mass:werk's carry flag
  in the citation; games/001.md updated).
- why-plus-2: "the reason we saw when we asked why zero" (two 100s had
  competed).
- recipe-end "by the same plus number"; our-numbers split into two
  sentences; the-map-3 "where his falling speed is below zero"
  ("level" means a game level elsewhere); settings-are-borders-2 "a flat
  one"; bias-slides-2 "Making one weight bigger compared with the other";
  land-balance names the earlier example ("Take Mario at depth 140 again,
  as in the last chapter"); balance-3 calls back why-zero-2 (the bias is
  the weight on the input that's always one); tldr-2 pause 3.0 like
  tldr-3; the-tilt-3's explicit default pause removed.

## Code

The unused `smb1_land_balance_yes_pan` (160, the falling example's yes
pan) is gone from exp001, so it can't be set against `no_pan` (162, the
rising example's) by mistake. Test first.

## Not acted on

- two-end's "straight border" for three or more inputs is a flat one
  (skepticism, cross-examiner: harmless for a lay viewer).
- one-dot-4b: scaling the 100 too would also work (skepticism: context
  rules it out).
- The lesser two-thought beats pacing listed (depth, choosing, school-4,
  settings-are-borders-2, balance-3): acceptable at their length; split
  if the animation needs it.
