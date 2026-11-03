# Picture council, video 002: the Honesty lens

Question for every mark on screen: does it come from the data, at its true scale? Could it mislead? Is
anything made up without a label?

I checked the stills, the beat code (`introduction.py`, `chapter_2.py`, `chapter_3.py`, `chapter_4.py`,
`chapter_5.py`, `sheets.py`), `series/sheet.py`, and `lab/film002.py`, and recomputed from
`target/series/002-every-mistake/film.npz` and `maps.npz` (scripts in `scratchpad/hon{1,2,3}.py`). The numbers
below are from those runs.

## Problems that affect several beats (the sheet renderer)

These live in `series/sheet.py` and show up in downhill-2, staircase, staircase-2, no-slope,
staircase-outro and shelf-2. A redesign has to fix them whatever else it does.

1. **The floor isn't zero.** `Sheet.__init__` rescales heights from min to max (`low = h.min()`). The lowest
   ground is drawn at height 0 with a skirt "down to zero", but the lowest ledge is **7.8% wrong (838
   moments)**, the bowl's bottom is a **squared miss of 0.366**, and the valley's bottom is **0.366** out of
   **17.2**. To the eye, the best border looks perfect. Fix: draw the true zero, either as a faint
   reference plane or as a skirt down to it, or add a labelled height scale with the true numbers ("838
   mistakes ... 9,882"). Don't rescale the bottom away.
2. **Zooming silently stretches the height.** `window=` crops and then rescales again. In staircase-2 and
   no-slope the window (393° to 442.5°, offsets −0.22 to 0.77) spans heights 0.078 to 0.442, so its relief
   is **2.3× taller** than the same ground in the full sheet. This is the height-exaggeration risk on the
   staircase. Fix: keep one height scale for the whole chapter. If the zoom needs more relief, say so on
   screen ("heights ×2.3"), or better, grow the height as an animated stretch that the viewer watches
   happen.
3. **The full sheet is resampled, and that drops ledges and the true lowest spot.** `coarse()` draws the
   720×601 map as 180×120 cells by nearest sample. Each angle row has about **333 distinct heights**,
   and each row has at most 120 cells, so about two thirds of the ledges, and every ledge narrower than
   2° × 0.05, vanish or get merged. The true lowest ledge is a **single sample** (1 of 432,720 at
   418°, 0.28). The coarse grid misses it: its minimum is 0.0790, not 0.0782. In staircase and
   staircase-outro, the lit "lowest ledge" region is a neighbour's height, widened. Fix: render
   full-resolution data wherever ledges are the point, or label the full view as smoothed ("drawn
   coarsely; closer below"), and always place markers by `lowest()` on the full map (this is already
   done).
4. **At the sampling we have, the ledges near the bottom are smaller than a sample.** Across the whole
   sheet, 47% (angle) and 41% (offset) of neighbouring samples share a height. In the zoom window
   around the lowest ledge it is only **7.5% and 4.9%**: 2,074 distinct heights in 10,000 samples. Every
   cell is drawn as a flat tile with walls, so the picture *manufactures* ledges exactly one sample wide.
   True flat ledges do exist (the line is right that the count only changes at a crossing), but near the
   bottom they are narrower than 0.5° × 0.01, and the data never resolves them. Fix: for the close views,
   compute the ledges exactly. The ledges are the faces of the arrangement of the 1,313 points' dual
   lines, so (angle, offset) cells can be found exactly, or the window can be resampled at about 20× finer
   resolution. Then draw ledges at their true sizes.
5. **Colour isn't height-linear, and the light mixes slope with value.** `glowing = (1-h)**3` lights only
   the lowest ~20% of heights, so everything above blends into one "ash". Sun shading then darkens
   slopes facing away. A dark patch could mean high or just facing away. That makes the bowl/shelf hard
   to read, and that's partly what Nate saw in staircase-outro. Fix: use a perceptually even ramp tied to
   true height, with a key that shows real values. Keep shading weak or use it only for walls, and add
   contour lines at round true values (for example every 5% of mistakes, every 0.1 of squared miss),
   since contours can't be confused with lighting.
6. **The units across vs into the sheet are arbitrary, and the picture should say so once.** WIDE 8 ×
   DEEP 4.4 × TALL 1.3 is a free choice: angle, offset and height have different units, so no true aspect
   exists. That's honest only if no comparison of steepness across axes is implied. It matters most for
   the valley (below), where both axes are in the *same* units (knob turns, ±1 each) and are still drawn
   8 : 4.4, then foreshortened.

**What any 3D camera move must respect** (rotation and zoom are welcome; Nate asked for them):
- Yaw/orbit is honest: it changes nothing about the data. Choose views that **show, not hide**, the
  narrow lowest ledge and its cliff. Painter's-order occlusion behind a high ridge can hide the bottom from
  most yaws, so at least one view must look straight down at it.
- Keep the height scale fixed while the camera moves. Zooming in must not be a cue to restretch.
- The mistakes sheet has an exact symmetry: turning the ruler half a turn and flipping the offset gives
  `1 − mistakes` (the ridge is the valley upside down). Don't crop or colour so that the two halves read
  differently from what they are.
- A dot standing "on" the ground must sit at that sample's true height, not the coarse cell's. This
  matters once the full-res data is used.

---

## Beat by beat

### too-many
**Current picture.** A real RAM frame from the smb1 recording (the grid is true). Two lit addresses
(`0x0755`, `0x0057`, Mario's x on screen and his speed) are true. The next 400 lit addresses are
**`rng.permutation(2048)` with seed 5: random, unlabelled**, and the counter stops at 402, not 2,048. No
border appears anywhere, though the line is about *borders multiplying*. Nothing is outright false
(it doesn't say these 400 matter), but a viewer will read the lit cells as particular numbers chosen for
a reason.

**Honest redesign must respect.**
- If addresses light up, either light them in address order (a sweep, obviously not a selection) or
  label "any 400 of them". Better still, end at all 2,048.
- The *number of borders* is countable and should be the thing that grows. For points in general
  position, the number of different ways a straight border can split N points using d numbers is at most
  2·Σ_{k=0..d} C(N−1, k) (Cover, 1965). With Mario's **1,313** distinct spots:
  - 1 number: 2,626
  - 2 numbers: **1,722,658**
  - 3 numbers: 7.5×10⁸
  - 4 numbers: 2.5×10¹¹
  - 8: 10²¹; 16: 10³⁷; 64: 10¹¹⁰
  - all 2,048: every one of the 2¹³¹³ ≈ **10³⁹⁵** splits is possible.

  Each extra number multiplies the count by roughly (N − d)/(d + 1), about 437 going from 2 to 3. That is
  the line's "multiplies", literally and with true scale. These are **upper bounds**. Mario's points sit
  on a sixteenths grid with many duplicates, so the true count is lower. Label it "at most". Better: count
  the real number of distinct splits for d = 2 exactly (an O(N²) sweep over the 1,313 spots), and show it
  beside the bound.
- If more numbers mean more distinct moments (N grows too when you read more bytes), compute that from
  the RAM recordings rather than assume it, or hold N fixed and say so.

**Design my lens suggests.** The 2-number map, with the sheet of all 432,720 borders we actually tried
(the real search) shown as a single tile. Then each added address lights in the memory grid, and the
tile count beside it multiplies on a log scale. The tiles become a wall too big to draw, and the camera
pulls back. Each count is real (Cover bound, labelled "at most"), and the first one is the exact count.

### no-fast-way
**Current picture.** 70 lines at **random angles and positions** (`rng` seed 2), with no map and no
axes: not borders on any data. A clock whose hands spin 12 times at a speed with no meaning. Nothing on
screen is from the data. Nate's verdict ("cheap") is also an honesty verdict: it is pure symbol.

**What the line claims, and what a picture must not overstate.** "In general, no known method finds the
border with the fewest mistakes quickly." This is true: minimising disagreements for halfspaces is
NP-hard, even approximately. But **for a fixed number of inputs it is polynomial**. With two numbers, an
exact method can enumerate all the splits (about N^d of them); that's how our search found 91%. The
hardness bites as the *number of numbers* grows. So a picture must not show time exploding with the
number of *moments* at fixed d = 2. It must show time growing with d. It also mustn't imply that nobody
could ever do it ("no *known* method", not "impossible").

**Honest design.** Measure on this machine how long scoring one border on the 10,720 moments takes (for
example X µs), and say "measured". Multiply by the true counts from too-many to get the time to try them
all:
- d = 2 (1.7 million borders): seconds.
- d = 3: minutes to hours.
- d = 4: days.
- Beyond that: longer than the age of the universe.

Show it as a time ruler that has to keep zooming out (second, day, century, age of the universe), with
real, labelled anchors. Any random lines that appear must be real borders on Mario's map (drawn with
`pointmap`), sampled from the sheet, and labelled "a few of them".

### exam
**Current picture.** Text only: "recording 1, 2, 4, 5" and "3, 6". It is true (TRAIN/TEST in exp002) but
shows nothing.

**Honest redesign must respect.**
- The true sizes: **10,720** training moments vs **4,883** exam moments, about a 69/31 split. Draw them
  at those proportions, as dots or as tape lengths, not as equal columns of names.
- If the exam moments are shown on Mario's map, use the test set's own spots (`film["xt"]`, `yt`), which
  overlap the training spots heavily. Don't make them look like new territory: same map, different
  moments. That overlap is the honest point (an exam on the same kind of play, not a different game).
- "After every pass, we scored it": the score per pass is real (`run_scores`). Showing the border from the
  map being scored on the exam set (the exam's dots lighting right or wrong under it) is honest. Don't
  invent a score that isn't the pass's own.
- **Footage caveat:** the frame-exact footage is "one session" (`Recording("smb1")`). Check which
  recording number it is before showing it as the exam. If it's a training recording, it must not be
  labelled "never learned from".

### downhill-2
**Current picture.** The valley over the real 61×61 grid (position and speed knobs, ±1 each around the
resting weights, **bias held**). The arrow's *direction* is the true added-up nudge at (−0.6, +0.53):
(0.98, 0.20) in knob units, mostly across the trough. Its *length* is fixed (0.12 of the sheet), not the
nudge's size.

**What's untrue, stretched or unlabelled.**
- **"The bias held" isn't on screen.** The line says "turn them all at once", but only two of three knobs
  turn. The `downhill` beat labelled "the others held still". This one needs "the bias held still".
- **The single lowest spot is invisible.** Along speed at the bottom, the height runs only 0.366 → 1.12;
  across position it runs 0.366 → 11.5; the corners reach 17.2. Speed's whole rise is **4.5% of the
  picture's height**, and the cubic colour lights the whole floor the same. So the picture reads as a
  flat-bottomed trough, and the *next* line ("that valley has a single lowest spot") is contradicted by
  what's on screen.
- The floor is drawn at 0 while the true bottom is 0.366 (problem 1).
- The knob axes are in the same units (one turn each way), but they're drawn 8 wide by 4.4 deep, then
  foreshortened. So the valley looks wider and shallower one way than it is, and the arrow's on-screen
  angle is not its knob-space angle.
- The arrow's length is arbitrary, and nothing says so.

**Honest redesign must respect.**
- Draw the grid square: equal on-screen length per knob turn on both axes, at least in a top-down view.
- Show the trough's true shape: steep across position, shallow along speed, *with* a single bottom. Use
  contour lines (ellipses about 3.9:1 in axis ratio, from the true curvatures 11.1 vs 0.76), or a
  cross-section along speed at its own labelled scale.
- **The downhill arrow must be drawn perpendicular to the contours, not toward the lowest spot.** In a long
  trough the steepest way down points across the trough, not along it to the bottom. Drawing it aimed at
  the minimum would be a lie, and a common one. This is also the true reason the next chapter's "too big"
  step bounces.
- If arrow length means anything, make it the true step (rate × nudge, in knob units). Otherwise label it
  "direction only".

### balance
**Current picture.** Three rows of up/down bars with numbers +0.766/−0.766, +0.171/−0.171, +0.244/−0.244
and "= 0.000". The numbers match the data.

**What's untrue, stretched or unlabelled.**
- **Each row is scaled to itself** (`top = max(up, -down)`), so position's 0.766 and speed's 0.171 are
  drawn the **same length**. The pulls are in different units (miss × blocks, miss × pixels a frame, miss
  × 1), so there's no common scale. But equal bars invite the reading "equal pulls". Either label "each
  row on its own scale", or (better) drop cross-row bar alignment.
- **"Added up" is mislabelled.** The numbers are means per moment (divided by 10,720). The actual sums
  are **±8,207** (position), **±1,836** (speed) and **±2,618** (bias). Either show the sums or say "per
  moment, on average".
- The board asked for every moment's pull, and the screen shows only two totals per knob. The data exists
  for each moment. On the position knob, **4,032 moments pull up and 6,442 pull down**, and the largest
  single pull is 8.0. Showing them is entirely honest and far more convincing: a tug of war with real
  people on each side.

**Design my lens suggests.** For the position knob, every moment's pull as a thin stick, up-pullers
stacked on the warm side and down-pullers on the cool side, each at its true length. Then the two stacks
slide end to end to show they're equal. This uses the real counts (4,032 vs 6,442: fewer but stronger
pulls up, which is interesting and true). Repeat quickly for speed and bias. Keep the moment counts
visible: the sides balance in total pull, not in number of moments.

### too-big
**Current picture.** The real U (position knob alone, others at rest). The ball's landing spots are true
one-knob jumps at twice the step (0.2): the factor per jump is −1.22 (|1 − 0.2·11.1|), so −0.12 → +0.146 →
−0.179 → … → −0.59.

**What's untrue, stretched or unlabelled.**
- **The arc through the air is invented.** `UP * 1.2 * sin(πt)` lifts the ball above the curve, and on
  this chart "up" means "more miss". Mid-hop the ball claims miss values that never happened: the knob
  just changes, and the miss is the curve's value where it lands. Nate's "too literal" and the honesty
  problem are the same thing.
- **The starting point (−0.12 from the bottom) is chosen**, not from any run. The real 2× run starts all
  three knobs at zero. "One knob alone, twice the step" is labelled; "starting nearby" isn't.
- This is a different experiment from the next line's 137× (3 knobs, from zero, 10 moves). It must not
  carry or imply that number. In the 3-knob run the growth comes from the steepest direction (mostly
  position, eigenvector (−0.94, −0.21, −0.27)), so the one-knob picture is a fair simplification, but
  only if labelled.
- In the 3-knob run the miss barely moves at first (1.000 → 1.005 → 1.07 → 1.27 → …). It doesn't jump
  higher dramatically from the first move. Don't animate it as if it did.

**Honest redesign (Nate's arrow version is also the honest one).**
- The step is horizontal: an arrow along the knob axis, from the current setting, of length exactly rate ×
  added-up nudge (in knob units, same axis scale as the chart). Then the landing point is read off the
  curve with a vertical drop or rise. Nothing travels through miss values that weren't visited.
- Show the settled step's arrow too (rate 0.1): from −0.12 it lands at +0.013, crossing the bottom but
  lower (factor −0.11). That's the board's own note, and it's true in this data.
- Label "one knob alone, from near the bottom". The start can be any spot, so say it was picked.

### map-intro
**Current picture.** A real map with the real best border, beside a **blank rectangle labelled
"borders"**. It's blank because `flat=1.0` paints every cell one colour. Nothing false is said, but a sheet
labelled "borders" that shows none is an empty promise (Nate: "it looks like it should show something").

**Honest redesign must respect.** If the sheet shows anything, it must be real borders at their real
spots: for example a grid of tiny rulers, each the actual border at that sheet spot (angle, offset), drawn
on a thumbnail of Mario's map. Put the true axis ranges on the sheet edges once it has axes: 310° to 670°
(one full turn) across, −3 to +3 blocks of slide up. Don't put heights or colour on the sheet until
staircase introduces them.

### map-axes
**Current picture.** The ruler turns about the real middle (2.5 blocks, 1.25 px/frame) through the real
angles 420° → 600° → 420°, then slides ±0.8.

**What's hidden.** **The yes and no sides aren't drawn**, so at 600° the ruler looks *identical* to 420°
(compare the stills at 1.6 s and 7.9 s), though it's the opposite border. The narration says "half way
round, its yes and no sides have swapped". The picture hides the very thing the line names, and
suggests the sheet repeats every half turn. That would be false: the sheet's right half is the left half
turned inside out.

**Honest redesign must respect.**
- Mark the yes side (a warm wash, or tick hairlines on the yes side) and let the viewer watch it swap.
  The points it calls right and wrong could relight live, which is real.
- The map's two axes are in different units (blocks and px/frame) and drawn at different scales. "Angle"
  here is in data units, so equal steps across the sheet aren't equal on-screen turns. Fine, but don't
  claim on-screen degrees.
- The blank sheet beside it: same as map-intro. A marker tracking the ruler's actual (angle, offset) on
  the sheet would make the sheet mean something already (this is what `sheet` does next; it could start
  here).

### staircase
**Current picture.** The real mistakes map rises from flat to full, static camera at pitch 1.0,
"height: mistakes".

**What's untrue, stretched or unlabelled.** Problems 1, 3 and 5 all apply.
- No height scale; the floor (7.8%) is drawn as zero.
- The surface is resampled from 720×601 to 180×120, and most ledges merge.
- The cubic colour makes everything above about 25% one dark ash, so the high side's shape is hidden.

**Honest redesign must respect.**
- A real height scale, and a true zero reference.
- One fixed height scale that later beats keep.
- An orbit (yaw is fine) including a top-down pass showing the exact symmetry (ridge = valley flipped).
- Colour by true value with a key.
- Mark one known border (the best one, or the marker from `sheet`) so the viewer can see a spot rise to
  its true height ("838 mistakes").

### staircase-2
**Current picture.** A zoom around the lowest ledge at full resolution (100×100 window), glow on the
lowest cell, "the lowest ledge".

**What's untrue, stretched or unlabelled.**
- **Height silently stretched 2.3×** relative to the full sheet (problem 2).
- **The "lowest ledge" is one sample** (0.5° × 0.01). Its glow is one cell, and its real width is unknown
  (smaller than or equal to the grid). The board's "narrow lowest ledge with a cliff along one side" may
  be true but is *not shown by this data*.
- The terraces drawn here are mostly **artefacts of tile drawing**: 93–95% of neighbouring samples differ,
  so each tile is its own "ledge" (problem 4). The line ("perfectly flat between crossings ... a staircase
  of flat ledges") is true mathematically, but this picture doesn't demonstrate it. It shows a grid of
  tiles.

**Honest redesign must respect.** Recompute the window exactly or finely enough that real ledges are
several pixels wide, and show the true lowest ledge's real shape and size. Keep the chapter's height
scale, or show the stretch happening with a label. If any drop is called out, use its true size in
moments: in this data, drops between neighbouring samples are mostly small (**median about 7 moments**)
with rare huge ones (**up to about 1,645**, more than the 710-moment start point, because a single sample
step can cross several crowded points). Don't label a sample-step drop as "the 710" unless it is that
crossing alone.

**Design my lens suggests.** A cross-section: a slice through the lowest ledge along one line of the sheet
(fixed angle, offset varying), drawn as a true step chart in moments. It's flat, then jumps, and every
step is exactly one crossing. Then the slice lifts back into the 3D view. This shows "flat between
crossings" with exactness a 3D render can't.

### no-slope
**Current picture.** The same zoomed window; a marker at (0.3, 0.62) of the window with 8 grey arrows,
"level, every way".

**What's untrue.**
- **The marker isn't on a ledge wider than one sample.** Its height (13.5%) is shared by **no**
  neighbouring sample in any direction. At this data's resolution the spot is level in no direction.
- The arrows are about 0.55 units long (about 5–6 cells across), drawn flat on the screen, not on the
  ground. Every one of them runs over several drops. The picture's claim "level, every way" is false as
  drawn. **This is exactly Nate's note "the arrows look wider than the flat step"**: he spotted a real
  untruth, not just a layout issue.
- The heights are stretched 2.3× here too.

**Honest redesign must respect.**
- Choose the marker's spot from exact (or finely resampled) ledges, on one that's demonstrably flat in
  all directions over the arrows' length.
- Draw the arrows *on the ground*, projected, and no longer than the ledge (or stopping at its edge, where
  a drop appears).
- If the walker "falls off" to find down, the drop must be a real one, at its true height in moments.
- "Around the best border we found": put the true lowest ledge marker in view, and don't make it look
  closer than it is.

### staircase-outro
**Current picture.** The full coarse sheet, lowest ledge (warm) and the square trick's resting border
(cool).

**What's untrue, stretched or unlabelled.**
- Problems 1, 3 and 5. Nate's complaint (contrast, can't tell the landscape) comes partly from the cubic
  colour plus sun shading: high ground, ridges and shadowed slopes all go the same brown.
- The two dots are correctly placed by full-res `lowest()` and `spot(REST)`. But the coarse surface under
  them isn't their true height, so they may float or sink.

**Honest redesign must respect.**
- The square trick's spot is at the same sheet place as the bowl's bottom (434.5°, 0.35). It's honest to
  show that the next chapter's bowl bottom lands *here*, on a ledge that is not the lowest.
- If the narration's "about three in a hundred short" is shown, use the same set as the line: check
  whether it's training mistakes (89.9% vs 92.2% at the lowest ledge) or exam scores (88% vs 91%), and
  label it.
- An orbit needs the fixed height scale and a true-value colour key.

### shelf-2
**Current picture.** The bowl sheet (smooth mode), shelf cells tinted with the warm glow, plus a caption.

**What's untrue, stretched or unlabelled.**
- **Colour contradicts height.** The shelf is the *highest* ground (exactly 1.0, the maximum), but the
  `glow` tint paints it a light warm tan. By the visual language, warm and light read as good or low. Its
  bowl neighbour's high rim is drawn dark ash at the same height. A viewer reads the shelf as a lower,
  brighter plain: the opposite of the truth. This is probably why Nate couldn't tell what it shows.
- The floor of the box is the bowl's 0.366, drawn as zero (problem 1).
- Smooth mode averages corner heights, so the bowl/shelf seam is softened. That's fine here because the
  true join is continuous: the bowl's rim reaches 1.0 at the shelf's edge. **Don't draw a cliff
  there.** The stills' dark seam reads like a step, and it's lighting, not data.
- The shelf is **50.0%** of the sheet (the brief says about 52%; `numbers.json` says 50%). It's a band of
  angles from about 486.5° to 657.5°, plus 35 partial angle rows, so the edge is nearly but not exactly
  straight. Draw it as computed, not as a ruled rectangle.

**Honest redesign must respect.**
- Mark the shelf by an outline or hatching that doesn't change its height colour, and keep it at the top
  of the height scale. If anything glows, it should be the bowl's bottom.
- Show that the shelf is the half of the sheet "where the ruler faces the wrong way": map-axes's half-turn
  swap made visible on the sheet. A small map thumbnail of one shelf border with its yes side marked
  ties it back (and **82%** of shelf borders get most moments wrong; "mostly, not always" is the true
  claim).
- Show the height-1 level as a labelled line ("squared miss 1: every moment misses by exactly one"), so
  the shelf is visibly *at* it.

---

## Lines that are themselves a problem (not changed; flagged)

- **downhill-2:** "Turn them all at once and it's a valley." The valley turns two of three knobs, with the
  bias held. Either the line or an on-screen label must say so. The label is enough.
- **no-fast-way:** True as written ("in general", "no known"). Note for any picture or caption: it's
  polynomial for a fixed small number of inputs (that's how the two-number search worked). Don't let the
  picture imply that two numbers were already hard.
- **staircase-2 / no-slope:** The lines ("perfectly flat between crossings"; "standing on a flat ledge")
  are true, but the current data can't show it near the bottom: the ledges there are smaller than a
  sample. The fix is better data (exact ledges), not a new line.
- **balance:** "add up to exactly nothing". The data's net is 0.000 to three places after 2,000 moves:
  near zero, not exactly. That's fine for the line (it describes the lowest spot itself), but the screen
  should say "= 0.000", not "= 0".
