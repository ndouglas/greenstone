# The idea: proposals for 002's flagged beats

Lens: what is the one thing each line says, and what picture would let a
viewer get it with the sound off? Does the picture show the idea, or a
symbol of it?

Beat lengths below are read off the still sheets' timestamps (approximate);
second marks are relative to the beat's start and should be re-timed to
the recorded narration.

A thread runs through several proposals and is worth deciding once:
**the tile sheet.** "Every spot on the sheet is one whole border" is the
hardest idea in the chapter, and the current picture asks the viewer to
take it on trust (a blank brown rectangle with a dot). If the sheet is
first shown *literally* as a sheet of tiny Mario maps, each with its own
ruler, and only later shrinks until each tile is a spot, the idea is
seen, not told. The same tiles then serve `too-many`/`no-fast-way`
(borders as things you have to try one by one), `map-intro`/`map-axes`
(rows turn, columns slide), `staircase` (a tile rises by its mistakes)
and `shelf-2` (shrunk borders all look the same). One new piece:
`tilesheet.py`, a grid of miniature `pointmap` renders (dots as 1-2 px
specks, the border as a hairline with faint warm/cool halves), able to
zoom from a grid of tiles to the flat `sheet.py` heightfield by
cross-fading each tile into its cell's colour.

---

## 1. too-many (~18 s)

**The idea.** Each extra number the neuron reads doesn't *add* borders to
try; it *multiplies* them: the whole set you had is copied once for every
setting of the new knob. With 2,048 numbers that is beyond counting.

**What's there.** The memory grid lighting bytes, with "N numbers read".
That shows "more numbers", not "multiplied borders". A counter of inputs
is a symbol; the borders never appear.

**Proposal A (ranked first): the copying picture.**
- 0-3 s ("Trying borders like that only works easily with two numbers"):
  from `search-2`'s map, the border sweep of `search` is laid out as a
  tile sheet: a grid of tiny maps, e.g. 24 turns × 16 slides = 384
  tiles, each a real border on the real dots, its wrong dots faintly
  ringed. The best tile (the border just shown) glows. Caption:
  *two numbers: a sheet of borders*.
- 3-8 s ("each extra number multiplies the borders to try"): a third
  number is added (a real byte from the memory grid flies in, e.g.
  Mario's vertical speed). The sheet doesn't grow a row; it is *copied*:
  the whole 384-tile sheet duplicates into a stack, 16 sheets deep, one
  per setting of the new knob (the stack's depth labelled with the new
  knob's name). Fourth number: the camera pulls back, the stack shrinks
  to one block and that block is copied 16 times in a row. Fifth: the
  row is copied into a square of blocks. Each copy is a visible
  replication of the entire previous picture, so "multiplies" is seen.
  Running count beneath, roman: 384 → 6,144 → 98,304 → 1,572,864.
- 8-18 s ("A neuron reading more of the 2,048 numbers…"): the memory
  grid (existing `memory.py`) sits beside the receding wall. Each byte
  that lights triggers another copy-and-shrink; after about eight, the
  tiles are smaller than a pixel and the wall is a haze; the camera
  can't pull back any further. The count stops being a number you can
  read and becomes its digits: it lengthens by about 1.2 digits per
  byte, running off the screen's edge by the time a few hundred bytes
  are lit. (The digits are the true count at 16 settings per knob.)
- Why it teaches better: "multiply" is shown as copying the whole
  picture, which is what multiplication *is*; the viewer sees borders,
  not inputs.
- Data: real borders over the real map for the first sheet (exists);
  the copies beyond the first sheet can be honest *repeats in layout*
  but must be labelled with what they are, since we won't render 3-D
  borders on a 3-D map for every tile. The resolution (16 or 24 per
  knob) is a choice and must be stated on screen: *16 settings a knob*.
  For the third number to be a real map axis we need a third RAM byte
  per moment (the frame-exact footage has it; check the training
  recordings store full RAM).
- Build: medium. Tile renderer + a recursive zoom/copy with nearest-
  neighbour shrink; the digit run is text.

**Proposal B (cheaper): keep the memory grid, give it borders.** Beside
the existing grid, put one tiny map with its border; every lit byte
copies the map row ×16. Weaker: the grid still dominates, and the copies
are a counter in disguise.

---

## 2. no-fast-way (~7 s)

**The idea.** There's no shortcut: no one knows a way to be sure you have
the border with the fewest mistakes without, in effect, checking an
enormous number of them. The picture should show *checking* and how
little of the space checking covers, not a clock.

**What's there.** A tangle of random lines and a spinning clock: both
symbols (Nate: "cheap"). The lines aren't real borders.

**Proposal A: the searchlight on the wall.** Continue from the end of
`too-many` (no cut).
- 0-2 s ("And in general,"): the camera dives back into the very first
  sheet of 384 tiles. A warm glint walks tile to tile in scan order, each
  tile it visits briefly showing its mistake count, and a ring marking
  *best so far* hops around (real: the running minimum over the real
  tiles; it jumps unpredictably, because nothing about a tile tells you
  where the best one is). The sheet is finished in ~1.5 s; a small
  roman figure gives our real search time for two numbers (measure it).
- 2-7 s ("no known method finds the border with the fewest mistakes
  quickly"): pull back through the copies of `too-many`. The checked
  part stays lit; everything unchecked stays dark. One level back, the
  lit sheet is one block of sixteen; then one of 256; then a single lit
  speck in a dark wall; then not even a pixel. The darkness is the
  problem's size; the speck is how much "trying them" gets done.
- Why: it shows the work and its hopelessness directly; the hopping
  best-so-far ring quietly shows why there's no shortcut (the answer
  isn't signposted).
- Data: real mistakes per tile for the first sheet; real search
  timing. Beyond the first sheet, the lit fraction is arithmetic, true
  at the stated resolution.
- Build: medium, reusing `too-many`'s piece; the hopping ring is a cumsum.

**Line note (not a change request).** "No known method" is a claim about
all methods, which no picture of our data can show; the picture can only
show the method we know (try them) and its cost. That's honest as long
as nothing on screen implies we've proved more.

---

## 3. exam (~8.5 s)

**The idea.** Two different piles of real play: the neuron learns from
one and is scored, after every pass, on the other, which it never
touches. The exam is real moments, not a word.

**What's there.** Two lists of recording names. Text only (Nate: show the
data).

**Proposal A: two maps, one sealed.**
- 0-2 s ("After every pass,"): the record tapes (`records.py`) slide in,
  four on the left, two on the right. Each tape's moments pour off it as
  dots onto its own Mario map: left map the training moments
  (recordings 1, 2, 4, 5; 10,720 moments), right map the exam (3 and 6;
  4,883). The right map sits behind a faint glass edge; nothing crosses.
  Ideally each tape shows a few frames of its real footage as it plays
  (only where frame-exact footage exists; otherwise the tape is a strip).
- 2-5 s ("we scored it on recordings it never learned from"): on the
  left, the border twitches as nudges hit (real nudges of pass k,
  sped up; the dot that caused each flashes). On the right, nothing
  moves. At the pass's end, the border is *copied* across the glass onto
  the exam map; the exam's wrong dots ring, and a score bar fills to the
  real exam score for that pass.
- 5-8.5 s ("like an exam it hasn't seen"): two or three more passes,
  each one a burst of twitches on the left and one copy-and-score on the
  right. The score bar jumps between them, which hands straight to
  `one-run` ("its score kept jumping").
- Why: the viewer sees that the exam is different moments (the dot
  clouds differ), that learning only ever touches the left, and that
  scoring is "lay the border on the other pile and count".
- Data: exists (per-nudge weights and pass, per-pass exam scores, test
  moments).
- Build: low-medium: two `pointmap`s, `records.py` tapes, existing
  nudge animation.

---

## 4. downhill-2 (~12 s)

**The idea.** Two ideas: (a) the U is one slice of a valley; turning all
the knobs at once gives the whole valley; (b) each moment's nudge points
wherever, but *added up* they point downhill, every time.

**What's there.** A lit surface with a small arrow and labels, held
still. The U doesn't become the valley; the arrow isn't seen to be a sum;
"always" isn't shown.

**Proposal A.**
- 0-4 s ("Turn them all at once and it's a valley."): `downhill`'s 2-D U
  chart tilts back into 3-D, standing on the floor as one slice (at the
  resting speed knob). Then neighbouring slices rise beside it, each the
  real U for another speed-knob setting (rows of the 61×61 grid), front
  to back, until they close into the valley surface. Contour lines on
  the floor. The camera yaws ~30° so the trough's shape (steep across
  position, shallow along speed) reads.
- 4-9 s ("Added up over all the moments, the square trick's nudges
  always point downhill in it."): a marker on the valley wall. From it, a
  spray of faint per-moment nudge arrows (a sample of a few hundred,
  drawn to a common scale; real: −miss × (position, speed) for each
  moment). They point every which way, some uphill. They then slide
  tip-to-tail into one chain; its end is the added-up nudge, drawn bold,
  crossing the contour lines toward lower ones. Repeat at two more
  spots, faster ("always").
- 9-12 s ("That's what we'll call following a slope."): from the third
  spot, the marker follows its summed arrow step after step (each arrow
  recomputed at its new spot) down to the trough floor. Path drawn as a
  hairline. Stop short of the bottom (that's `balance`'s moment).
- Why: the valley is built from the U the viewer already understands;
  the arrow is seen to be a sum; "always" is three spots, not one.
- Data: the 61×61 grid (exists); per-moment misses at arbitrary
  (position, speed) with bias held: cheap to compute. Honesty: the
  steps here are with the bias held, so label the floor *bias held
  still* (the real 2,000-move path also moves the bias).
- Build: medium. `sheet.py` needs yaw and contours (listed as addable).

---

## 5. balance (~15 s)

**The idea.** At the bottom the pulls haven't gone; they cancel. Not
"zero pull" but "equal pull both ways".

**What's there.** Three bar pairs with ±0.766 etc. Accurate, but a table
of numbers; nothing shows that thousands of real moments are pulling.

**Proposal A (ranked first): the chain that closes.** Continuous with
downhill-2's arrows.
- 0-3 s ("That valley has a single lowest spot."): the marker reaches
  the valley bottom; its summed arrow shrinks to a point.
- 3-8 s ("There, the pulls are still there,"): the point bursts back into
  the spray of per-moment arrows (all 10,720, thin, warm for pulls that
  turn the position knob up, cool for down). Big, plainly not nothing.
- 8-15 s ("but they balance… like a tug of war at a standstill"): the
  arrows line up tip-to-tail: first all the warm ones, a long chain
  marching out; then the cool ones, marching back; the chain ends
  exactly where it began. A closed loop, the gap at the join zero. Then
  the loop is pulled straight into a rope with the knob's needle at its
  middle: warm length on one side, cool on the other, equal (+0.766 and
  −0.766 written at the ends), the needle still.
- Why: "add up to exactly nothing" is *seen* as a walk that returns home;
  the tug of war is the same data, not a cartoon.
- Data: per-moment pulls at the resting weights (exists).
- Build: medium. Ordering arrows (warm first, then cool, each sorted by
  angle) makes the loop a clean lens shape.

**Proposal B: pulls from the map.** Mario's map at the resting border;
every dot sends a hairline to the position-knob dial, warm clockwise,
cool anticlockwise, thickness = its pull; the strands gather into two
bundles that are laid side by side, equal. Teaches *where* the pulls come
from (dots on either side of the border), which `settles-2` then needs
("each moment's own pull isn't [nothing]"). Could follow A in the same
beat if the line is long enough; otherwise B is a good opening for
`settles-2`.

---

## 6. too-big (~20 s)

**The idea.** A move is the slope times the step size. Too big a step
and the move, laid from where you stand, carries you past the bottom and
up the other side to a *higher* point than you started; the next move is
bigger again.

**What's there.** A ball hopping on the U (mid-hop it floats off the
curve). Nate: too literal; he asks for an arrow for the size of the
change, overlaid to show the overshoot. Agreed: the arrow is the idea.

**Proposal A (Nate's, made concrete).**
- 0-5 s ("Here, the size of the step decides whether it gets there at
  all."): the U with a marker on its left wall. A dashed horizontal at
  the marker's height ("where it starts"). The slope there drawn as a
  short tangent hairline.
- 5-10 s ("…doubling the step can't just double everything…"): below
  the curve, on the knob axis, the move appears as an arrow starting
  under the marker, length = step × slope, for the step that settled
  (faint, gold). Then the doubled step's arrow (twice as long, bright).
  Ghost the settled one's landing: up from its tip to the curve, lower
  than the dashed line.
- 10-20 s ("Too big, and each move overshoots…"): the bright arrow's tip
  is past the bottom. A vertical rises from the tip to the curve: the
  new spot, *above* the dashed line (the gap shaded rose: worse). New
  dashed line, new tangent (steeper), new arrow, longer, pointing back
  the other way; it overshoots further. Three or four rounds, the camera
  pulling back as the arrows outgrow the frame. The arrows stay on the
  axis as a record: alternating direction, each longer, a visual
  definition of "growing overshoot".
- Why: it separates the two things a lay viewer conflates (how far the
  knob moves vs. how high the miss is) and shows the cause: the arrow's
  length, not a bounce.
- Data: the one-knob U exists. The too-big run moves all three knobs, so
  for the one-knob picture compute a one-knob run at 2× the settled step
  and check that it does diverge; if it doesn't, the U must be a slice
  along the real run's direction, labelled so. (Honesty lens should
  confirm.)
- Build: low-medium on `chart.py`.

**Line note.** The clause "the targets stay at plus and minus one, so
doubling the step can't just double everything, the way it did for the
perceptron" has no picture of its own here and is a callback the viewer
must hold in their head. A half-second inset of the perceptron's border
doubled-and-unmoved (from the earlier beat) would carry it; otherwise
consider whether the line needs it.

---

## 7. map-intro (~5 s) and 8. map-axes (~18 s)

**The idea.** map-intro: we're about to see every border at once, as a
sheet. map-axes: a border is a ruler with two motions, turn and slide;
half a turn swaps yes and no.

**What's there.** The map, and a blank brown panel labelled "borders"
for both beats (Nate: looks like it should show something). The panel
promises content and delivers none; the ruler is a plain line whose yes
side isn't visible.

**Proposal A (ranked first): the sheet of borders is literal.**
- map-intro, 0-5 s ("a sheet of borders"): the map's border copies
  itself into the panel as a grid of tiny maps (say 24 turns × 9
  slides), each its own border on the same specks. The panel is never
  blank: it *is* borders. No axes labels yet.
- map-axes:
  - 0-5 s ("a ruler laid across it, with a yes side and a no side"): on
    the big map the border becomes a ruler: a hairline with tick marks,
    the yes side washed faintly warm, the no side faintly cool (the
    same wash in every tile).
  - 5-12 s ("turn the ruler around the middle… half way round, its yes
    and no sides have swapped"): the ruler turns about the marked middle
    (2.5, 1.25). A cursor walks along the matching *row* of tiles in
    step. At half a turn, the big map's wash swaps, and the viewer can
    see, along the row, that the tiles' washes swap halfway across: the
    right half of the row is the left half with colours flipped.
  - 12-18 s ("or you can slide it…"): the ruler slides; the cursor walks
    up a *column*. Row and column stay highlighted as a cross.
- Then `sheet` zooms out until each tile is a spot, cross-fading into
  the flat sheet: "every spot is one whole border" has already been seen.
- Data: exists. Build: medium (`tilesheet.py`), reused below.

**Proposal B (cheap fix).** Don't show the sheet until `sheet`; in
map-intro, show only the map with its ruler copying into a fan of faint
rulers ("borders"). Solves the blank panel but loses the payoff.

---

## 9. staircase (~10 s)

**The idea.** Height = how many mistakes that border makes; lower is
better. Each spot's height *means* a count of wrong dots.

**What's there.** A lifting surface, seen from one fixed angle in low
contrast; the map beside it doesn't connect to the height.

**Proposal A.**
- 0-4 s ("Imagine the sheet as ground, with each spot raised to a
  height: how many mistakes that border makes."): three chosen spots
  first. For each, the big map shows its border with its wrong dots
  ringed; the spot rises as a thin column to that height, its count
  beside it (e.g. 82 in 100 for a backwards one, 8 in 100 near the
  best). Then the whole sheet lifts, the camera swinging from overhead
  to a low three-quarter view (yaw ~35°, pitch dropping) so height
  becomes visible as height.
- 4-10 s ("Lower is better."): a marker walks a real path across the
  ground from a high spot to the lowest ledge, the map's ruler moving in
  step and its ringed wrong dots thinning out as the marker descends.
  "Lower" and "fewer wrong dots" happen together on screen.
- Why: height is tied to something countable on the map; the motion is
  meaningful (walking = changing the border).
- Data: exists. Build: medium; yaw for `sheet.py`.

---

## 10. staircase-2 (~12 s)

**The idea.** The count only changes at the instant the ruler crosses a
dot; between dots nothing changes, so the ground is flat ledges and
sudden drops.

**What's there.** A static terrain where the ledges are too fine to see,
and nothing shows *why* they're flat.

**Proposal A: one row first, then the ground.**
- 0-6 s ("The height only changes when the border crosses a moment, so
  between those crossings it's perfectly flat."): pick one angle row
  through the lowest ledge. Beside the map, its height profile is drawn
  live as the ruler slides across the map: a flat line while the ruler
  sweeps empty map; a sudden step at each dot it crosses (the dot
  flashes; the step's size is that point's count). Slow-motion at one
  crossing. The profile is exact: a staircase, not a curve.
- 6-12 s ("The ground is a staircase of flat ledges."): that profile
  becomes one edge of the 3-D ground; the camera tracks low along it
  into a close crop near the lowest ledge, with raking light so each
  ledge is a lit tread and each drop a dark riser, and orbits slowly.
  End on the narrow lowest ledge and its cliff.
- Why: the cause of flatness (no dot crossed = no change) is seen before
  the terrain, so the terrain reads as a consequence.
- Data: exists. Build: medium (profile chart is `chart.py` with a step
  line; close crop already supported).
- Sets up `big-drops`: the 710-moment point will make a visible giant
  step in the same profile.

---

## 11. no-slope (~19 s)

**The idea.** A slope-following rule decides where to go by the tilt
under its feet. On a ledge the tilt is zero every way, so it gets no
direction; the only clue is a drop, which you only find by stepping off,
and it might be up.

**What's there.** A marker with radial arrows on a distant terrain;
arrows longer than the ledge (Nate), and little motion.

**Proposal A.**
- 0-6 s ("Standing on a flat ledge, every direction looks the same."):
  close-up on one real ledge, cropped so the ledge fills a third of the
  frame. The marker sends out short feelers (shorter than the ledge, by
  construction from the ledge's real extent), eight directions, each
  reading the height change: 0, 0, 0… A tiny level tick on each. The
  slope rule's own arrow is drawn: it has no length; a dot.
- 6-11 s ("Only a drop tells you which way is down, and you find it by
  falling off."): the marker wanders (it has no direction to prefer)
  until it crosses an edge. First crossing is upward: it rises a riser,
  count up (rose). Back. Another edge: it drops, count down (warm-gold
  for better? Legibility to decide; the point is up vs down). Both real
  heights.
- 11-19 s ("So a rule that follows a slope… can't walk down this
  staircase to the lowest ledge"): pull out and orbit: the marker's
  ledge, the many ledges between it and the lit lowest ledge. The slope
  rule's dot-arrow stays a dot the whole way.
- Why: "no slope" becomes "the arrow has zero length", and the
  stepping-off is shown as a gamble, not a direction.
- Data: exists (heightfield). Build: medium.

**Alternative B (simpler, 1-D):** reuse staircase-2's profile; draw the
tangent under a marker on each tread: always horizontal. Very clear, but
loses "every direction".

---

## 12. staircase-outro (~17 s)

**The idea.** Recap: the square trick settles about 3 in 100 short of
the best, and on this ground that is a higher ledge; and a staircase
can't be walked by slope. Then two open questions.

**What's there.** A tabletop seen from one angle, low contrast; two dots
close together, so "short" doesn't register.

**Proposal A.**
- 0-6 s ("from a rule that settles about three in a hundred short"):
  orbit the ground, colour-ramped by height. The lowest ledge is lit;
  the square trick's border is a marker on its own ledge. A vertical
  hairline from its ledge down to the lowest ledge's height, labelled
  "3 in 100": the shortfall as a height.
- 6-11 s ("…the count of mistakes, and found a staircase no
  slope-following rule can walk down"): camera dips to ledge level
  between them: the treads in between seen edge-on, a flight of steps.
- 11-17 s ("So where does the square trick's slope lead… And what about
  the perceptron?"): the camera rises back over the whole sheet and
  holds, the square-trick marker pulsing once on "slope"; nothing new
  appears (the bowl is next; showing it early breaks Nate's rule).
- Data: needs the square trick's *training* mistake height at its
  border against the lowest ledge's (check that "3 in 100" is the same
  quantity the ground shows; numbers.json's 88%/91% may be exam
  scores). Build: low on top of the yaw/colour-ramp work.

---

## 13. shelf-2 (~15 s)

**The idea.** Shrink any border to nothing and it stops being that
border: every moment gets a total of zero, misses by exactly one, and so
all of those borders are *the same neuron* at the same height. They fill
the side of the sheet where the ruler faces the wrong way.

**What's there.** A bowl and a flat slab, side by side, with a caption.
It shows the result without the reason (Nate: unsure what's
illustrated).

**Proposal A.**
- 0-6 s ("Then every total is zero, every moment misses by exactly
  one"): four tiles from the shelf side, different angles. Each border
  shrinks to nothing: its warm/cool wash fades to dark, its dots all
  turn the neutral ash of "zero", and its miss bar reads 1.00. The four
  tiles end identical. Hold a beat on the four identical blanks.
- 6-10 s ("and all those borders sit at the same height: a high, flat
  shelf"): on the sheet, those four spots rise to height 1 together;
  then the rest of the size-zero spots rise to join them, the shelf
  spreading as one level plateau. Camera low, so its flatness reads
  against the bowl's curve.
- 10-15 s ("filling the side of the sheet where the ruler mostly faces
  the wrong way"): a cursor crosses the shelf; the big map's ruler turns
  in step, its yes side lying over the no dots. Crossing back onto the
  bowl, the ruler faces the right way.
- Why: the reason for "same height" (they're all the same blank neuron)
  is shown, not captioned.
- Data: exists (best size per spot; spots at size zero). Build: medium,
  reusing the tile sheet.
