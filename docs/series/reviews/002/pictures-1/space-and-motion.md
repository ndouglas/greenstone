# Picture council, video 002: Space and motion

Lens: is the picture a scene the camera explores, or a diagram held still? Does the viewer get enough angles, depth and
movement to know a surface's shape? Does every motion mean something?

Timings are estimated from the line's length (about 2.5 words a second) and the still sheets; the builder should snap
each cue to the recorded beat's word timings.

What the stills show, through this lens, in one paragraph: every sheet beat uses one camera (yaw 0, pitch ~0.9 rad,
looking down the offset axis), held still, and the staircase is drawn at 180 x 120 cells, coarser than its own steps.
So the viewer never sees the ground from a second angle, never sees a ledge as a ledge (they read as faint stripes on
a smooth slope), and can't tell high from low except by a colour ramp that has little contrast in the middle. The
shelf reads as a flat brown rectangle beside a lit dish, and nothing says it's the highest ground. The non-sheet beats
(too-many, no-fast-way, exam, balance) are diagrams held still; too-big has motion, but the motion is the ball, and the
step that causes it is never drawn.

---

## Part 1. What the renderer needs (sheet.py), shared by all the sheet beats

In order of how much they buy:

1. **A free camera: yaw, look-at target, and distance (orbit and zoom).** Replace "camera behind the near edge,
   looking at the centre" with `Camera(yaw, pitch, distance, target=(x, y, z), focal)`. Rotate the world about z by
   yaw around the target, then apply the existing pitch projection. Zooming is distance (dolly) or focal (lens); use
   dolly for "going closer", focal never (it flattens depth). A look-at target lets the camera fly to a spot (the
   lowest ledge, the square trick's border) and orbit it.
   - Painter's order must become general once yaw isn't 0: sort cells by the camera-space depth of their centre, far
     first; for each cell paint the (up to two) walls whose outward normal faces the camera (dot with the
     view direction < 0), then the top. That replaces the hand-written "far rows first, edges in toward the middle"
     order, which is only right at yaw 0. Vectorise the sort (one `argsort` per frame); cost stays near 0.2-0.3 s.
2. **Ledge-edge hairlines (contours for the staircase) and iso-lines (for the bowl).** For the flat-ledge mode, draw
   a thin dark line (or a thin light rim, see 3) along every top-face edge where the neighbour's height differs: the
   staircase's step edges, drawn exactly where they are. For the smooth mode, iso-lines every fixed step of squared
   miss (0.05), hairline, slightly darker than the ground. These lines are what lets a viewer read shape from a
   single still: terraces as nested rings, the bowl as nested ellipses.
3. **A stronger height ramp plus banding.** Keep "lowest ground glows gold, highest is dark ash" (light means
   understanding: the good borders are lit), but make the ramp perceptually even from bottom to top (gold `#FFD59A` to
   amber to a deep rust to ash), not `(1-h)^3`, which leaves the whole middle the same dull brown. Add hypsometric
   bands (the colour steps every 5 points of "share wrong" for the staircase; every 0.05 of squared miss for the bowl)
   so equal-height ground reads as equal at a glance. Keep shading, but weaken it on top faces (shading 0.6 to 1.0,
   not 0.35 to 1.0) so a flat high shelf facing the sun can't look brighter than low ground; give the walls (risers)
   their own darker value so every drop reads as a dark band.
4. **A raking light that moves with the camera.** The sun is fixed in world space now; for close-ups of ledges, the
   builder needs a low sun (elevation ~20 degrees) from the side, so every riser casts a visible shadow band. Add a
   simple cast-shadow pass for the flat-ledge mode: a cell is in shadow if, marching toward the sun over the height
   map, something is higher (a few dozen samples per cell, numpy, once per light direction, cached). This is the
   single thing that makes terraces look like terraces.
5. **Depth fog.** Blend each face toward the background `#030303` by its camera distance (10-30%), so far ground
   recedes. Gives depth on a still frame and makes the orbit legible.
6. **Native-resolution windows.** The sheet is 720 x 601 samples; the staircase has ~330 distinct heights per angle
   row. At 180 x 120 cells a ledge is often narrower than a cell, so the drawing smooths the staircase into a slope
   (Honesty will flag this too). For close-ups, crop a window and draw it at one cell per sample (e.g. 80 angles x 100
   offsets = 8,000 cells, cheaper than now). For the whole sheet, either accept coarse cells but sample each cell's
   height at its centre (as now: still flat ledges, just fewer), or switch to a rasterising renderer (moderngl
   offscreen, or a numpy z-buffer) which handles 432,720 cells at full resolution easily. Recommendation: keep PIL
   for windows and coarse overviews; consider moderngl only if Nate wants the full-resolution overview orbit.
7. **Cross-sections (slices).** A vertical cutting plane at a chosen angle (or offset): everything on the camera's
   side of it is not drawn, the cut face is filled in a flat darker tone, and the profile along the cut is drawn as a
   bright hairline on top. Plus a matching 2D profile panel (chart.py) of the same line at full sample resolution. The
   slice is the most direct way to show "flat ledges, sudden drops" and "flat at exactly 1.0". The plane should be
   movable (animate its position), so the profile sweeps.
8. **3D markers that sit in the scene.** A marker is a small bead on the ground with a hairline stem down to the
   floor (z = 0) and a soft shadow; drawn into the same depth sort so ground in front of it hides it. A floating 2D dot
   (as now) loses its place when the camera moves.
9. **Per-cell lift and a height ruler.** `lift` as an array (cells can rise at different times: a wave, or "these
   spots join the shelf"). A 3D vertical ruler at one corner of the slab with real ticks (10%, 50%, 90% wrong; or
   squared miss 0.4, 0.7, 1.0), drawn in the scene so it turns with the camera: it gives the height axis units and
   tells a window's vertical exaggeration honestly ("height x4 here" as a label when a window is renormalised).
10. **Optional: the ring.** The angle axis wraps. A late option, not needed for any flagged beat: bend the sheet into
    a band around a cylinder for one shot so the wrap is seen, not told. Mentioned only so the camera API doesn't
    preclude it.

A useful fact for every staircase shot: turning the ruler half way round and sliding it to the opposite side swaps
yes and no, so mistakes(angle + 180, -offset) = 1 - mistakes(angle, offset). The staircase's high ridge is the low
valley turned upside down and moved half a sheet across. The camera can show this (a half-orbit that lines the ridge
up where the valley was); it's why the ground has exactly one big valley and one big ridge.

---

## Part 2. The beats

### too-many (about 17 s)

"Trying borders like that only works easily with two numbers; each extra number multiplies the borders to try. A
neuron reading more of the two thousand and forty-eight numbers in the console's memory would have far too many
borders to try."

Currently: the memory grid with bytes lighting up and a counter "N numbers read". Held still; the multiplying is a
count, not a space. The idea "each extra number is another axis, and the borders multiply" is spatial, so show axes.

**A (recommended): the map grows a third axis, then the camera pulls back into the memory.**
- 0-4 s ("Trying borders... with two numbers"): Mario's map (pointmap), flat, face on, as it ended in search-2. The
  ruler sweeps through a fan of real borders from the search (a few hundred hairlines at once, faint, every one a real
  angle and offset from the 720 x 601 grid), a counter beside the map: "432,720 borders tried" (720 x 601, the true
  size of the search).
- 4-9 s ("each extra number multiplies the borders to try"): a third axis grows out of the map, toward the viewer. The
  camera orbits 35 degrees and pitches up, so the map becomes the floor of a box. The dots lift to their true values
  of a real third number from RAM (pick one that varies: Mario's vertical position, or the frame timer byte; it
  doesn't matter which, it's an honest example and is labelled "a third number: [name]"). The ruler (a line on the
  floor) becomes a pane of glass standing in the box, and now it tilts as well as turns: the fan of ghost lines
  becomes a fan of ghost panes, tilting through a second angle. The counter multiplies to the true count at the same
  fineness (each new direction resolved as finely as the old one: about 432,720 x 360). The orbit is the meaning:
  the viewer sees one more direction for the border to lean in.
- 9-17 s ("A neuron reading more of the two thousand and forty-eight numbers..."): the box can't grow a fourth axis
  we can see, so the camera pulls back out of the box, and the box shrinks to the three lit bytes in the memory grid
  (memory.py), the two bytes the neuron already reads plus the third. More bytes light (as now), and the counter,
  set as a long number in roman, gains about 2.6 digits per byte; at 2,048 bytes it has thousands of digits, so the
  counter becomes a line of digits running off the right of the screen, and the camera pans along it, accelerating,
  never reaching the end before the cut. That pan is the "far too many" in a motion: the number is too long to read
  past.
- Data: the search grid; one extra RAM byte per moment (the recordings have every byte every frame); the count
  formula agreed with Honesty (say: 601 offsets x 720^(k-1)/2^(k-2), or simply "x about 360 for each number at this
  fineness"; it must be stated on screen once).
- Build: medium. Needs a small 3D point/plane projection for the box (can reuse sheet.py's Camera once it has yaw);
  the digit pan is easy.

**B: digit pan only**, over the existing memory grid (the last 8 s of A). Cheaper, keeps continuity; loses the axis.

### no-fast-way (about 6 s)

"And in general, no known method finds the border with the fewest mistakes quickly."

Currently a crowd of random lines and a clock: symbols, not the problem.

**A (recommended): the search's light sweeps the sheet, then the camera pulls back through a stack of sheets.**
- 0-2 s: the sheet of every border from the search, seen from above, flat, its cells lit by mistakes (the real
  data; this is only a look ahead at the sheet as a coloured grid, no heights, so it doesn't steal map-intro's
  reveal; if Continuity objects, use a plain grid of cells instead). A scanning bar crosses it, lighting cells as
  they're scored, at the true rate of the real search on the machine that ran it (measured; label "trying every
  border: 2 numbers, [x] seconds").
- 2-6 s ("no known method ... quickly"): the camera pulls back; the sheet is revealed as one slice in a stack (the
  third number's extra direction), each slice needing its own sweep; the scanning light shrinks to a speck. A second
  pull back: the stack is one block in a row of blocks (the fourth number), receding into fog. A time readout keeps
  pace in true numbers: seconds, then hours, then years. The pull-back speed is constant in log scale, so each extra
  number costs the same zoom-out: the motion is the multiplying.
- Honesty note for the builder: the line is about every known method, not only trying every border. The best known
  exact methods for this problem still slow down by a large factor with each extra number (roughly the moments
  count raised to the number of numbers), so the picture's "each number multiplies the cost" is true in spirit; the
  times shown must be labelled as "trying every border", not as the best possible.
- Data: the search timing (measure it), the sheet. Build: medium (2D grid, a stack drawn in the same projection).

**B:** cut the clock; keep too-many's digit pan running and slow it to a crawl under the line. Cheap, but weaker.

### exam (about 8 s)

"After every pass, we scored it on recordings it never learned from, like an exam it hasn't seen."

Currently: two text lists. Through this lens: show the two sets as places, and the border moving between them.

**A (recommended): two maps stacked like glass layers, the border passing down through them.**
- 0-2.5 s ("After every pass,"): Mario's map of the training moments (recordings 1, 2, 4, 5), as pointmap draws it.
  The camera tilts back 30 degrees so the map is a pane seen at an angle. Beneath it, a second pane fades up: the
  same axes, the moments from recordings 3 and 6 only, dim (unseen). Each pane labelled with its recordings, small.
- 2.5-6 s ("we scored it on recordings it never learned from"): the perceptron's border after pass 1 (the real
  weights) is drawn on the upper pane; then it drops straight down, as a line, onto the lower pane. There the exam
  dots light: right ones lit warm or cool as they are, wrong ones ringed; the score appears beside the lower pane
  (the real exam score for pass 1). Pass 2, pass 3: the border on the top pane jumps to its next real position, drops
  again, the lower pane re-lights, the score changes. Each drop takes ~0.6 s; the rhythm is "after every pass".
- 6-8 s ("like an exam it hasn't seen"): the camera eases closer to the lower pane; the upper one dims. Hold on the
  scored exam, which leads into one-run's score line.
- Why: the viewer sees that the exam is different dots, the same kind of map, and that the border is carried over
  unchanged and scored there. "Never learned from" is the vertical gap: nothing flows up.
- Data: test-set moments by point (exists via pointmap), perceptron weights and exam score per pass (exist). If
  footage of recordings 3 and 6 doesn't exist (the brief says one session has frame-exact footage), don't show game
  frames for them.
- Build: medium; two pointmaps with a shared perspective transform (or a skew transform in Manim).

**B:** side-by-side maps, border copied across with a slide. Easier, flatter, still shows the data.

### downhill-2 (about 12 s)

"Turn them all at once and it's a valley. Added up over all the moments, the square trick's nudges always point
downhill in it. That's what we'll call following a slope."

Currently: a smooth trough seen square-on down its length, lit evenly; the arrow is a flat 2D overlay with an
illegible label. The viewer can't see that the U is a slice of it, or which way is down.

**A (recommended): the U is the valley's cut face; the camera orbits to reveal what's behind it, then looks down on
contours with arrows crossing them.**
- 0-1 s: start exactly where downhill ended: the U, as a 2D curve (position knob across, total squared miss up).
- 1-5 s ("Turn them all at once and it's a valley"): the U becomes the front cut face of the 61 x 61 valley (cut at
  the speed weight's resting value): the surface extrudes back from the curve along the speed knob axis, while the
  camera orbits ~40 degrees of yaw and pitches up to ~35 degrees, so the U stays visible at the front as a lit
  profile and the trough runs away from us at an angle. A slow orbit (another 20 degrees) continues under the line
  so the viewer gets parallax on the trough's two walls. Axis labels in the scene: "position knob", "speed knob" on
  the floor edges, "total squared miss" on a 3D ruler at the corner. Label "the bias held still" small (see the
  note on the line, below).
- 5-9 s ("Added up over all the moments, the square trick's nudges always point downhill in it"): iso-lines fade onto
  the surface (long thin ellipses: the trough is ~15x steeper across position than along speed). Then arrows, one at
  a time at ~8 places on the surface, each the real added-up nudge at that setting (the negative gradient of the
  total squared miss, bias held), lying on the surface, length true to size (scaled by one constant). Each is drawn
  from its tail and the tail-to-tip drop is shown by the tip sitting on a lower iso-line. They cross the iso-lines
  at right angles, and in this long trough they point mostly across it, toward its floor, not along it at the bottom:
  that's true and worth seeing.
- 9-12 s ("That's what we'll call following a slope"): the camera rises to nearly overhead (pitch ~75 degrees), so
  the picture becomes contour map + arrows, the clearest reading of "points downhill". One arrow chain: start at one
  setting, follow the arrows (true steps at the settled step size, using the real gradient), a hairline path that
  zig-zags down the trough. If that path disagrees with the real 2,000-move run (which also moves the bias), don't
  call it the run; label it "following the slope".
- Data: the 61 x 61 grid; gradient computed from the moments at the grid points (or by finite differences of the
  grid; better analytically). Build: medium-hard (needs the sheet.py camera with yaw, iso-lines, 3D arrows on the
  surface).
- Note on the line, separately: "Turn them all at once" says all three knobs; the valley turns two (the bias is held).
  Either say "turn two at once" or keep the small on-screen "bias held still". Not a picture problem.

**B:** keep the current camera, but add iso-lines, a raking light and a top-down end. Easier, much less shape.

### balance (about 14 s)

"That valley has a single lowest spot. There, the pulls are still there, but they balance: the ones pulling up and
the ones pulling down add up to exactly nothing, like a tug of war at a standstill."

Currently: three bar pairs with numbers, held still. True but abstract; the pulls don't come from anywhere, and
nothing is pulled.

**A (recommended): fly to the bottom; the pulls fly in from Mario's map and line up as a rope.**
- 0-3 s ("That valley has a single lowest spot"): continue downhill-2's camera; dolly down to the valley's lowest
  point; a bead (3D marker) settles there. Camera ends low and close, looking along the position knob's direction, so
  "up" (more weight) is right and "down" is left on screen.
- 3-8 s ("There, the pulls are still there"): the valley dims to a hairline floor. From the bottom, a horizontal rope
  runs left and right (the position knob's axis). Mario's map fades in small, below. From its dots, pulls fly up to
  the bead, each point's pull together (miss x position, summed over the moments at that point; 1,313 points, real),
  as short arrows, warm ones pointing right (pulling the weight up), cool ones left. Biggest first, so the first few
  are readable; then a rush of the rest.
- 8-12 s ("but they balance: ... add up to exactly nothing"): the arrows slide head to tail along the rope: warm
  ones chain to the right, reaching +0.766; cool ones chain left, reaching -0.766. The two chain ends are the two
  teams. The knot at the centre does not move.
- 12-14 s ("like a tug of war at a standstill"): the rope tightens (a slight straightening, both chains pulling
  outward), and the knot stays exactly at the bead. "= 0.000" in roman at the knot. The speed knob and bias ropes,
  smaller, below the first (+/-0.171, +/-0.244), also at a standstill.
- Why: the viewer sees the pulls are real and many, come from the moments, are big each, and still sum to nothing.
  It also sets up settles-2 ("each moment's own pull isn't" nothing): any single arrow, alone, would move the knot.
- Data: per-point pulls at the resting weights (exists per moment). Build: medium (2D, with one camera move borrowed
  from downhill-2).

**B:** keep the bars, but grow them as head-to-tail chains of real per-point pulls instead of solid bars.

### too-big (about 20 s)

"Here, the size of the step decides whether it gets there at all. The targets stay at plus and minus one, so doubling
the step can't just double everything, the way it did for the perceptron. Too big, and each move overshoots the
bottom of the U so far that it lands higher up the other side."

Nate's note: draw the step as an arrow, then show where the arrow takes the knob. Through this lens, the motion
should be cause then effect, each time: slope, arrow, landing.

**A (recommended): tangent, step arrow along the knob axis, drop to the curve; the camera pulls back as it grows.**
- 0-5 s ("the size of the step decides whether it gets there at all"): the U (position knob alone), a hairline
  vertical at its bottom (labelled "the bottom"). The ball at its starting knob value.
- 5-11 s ("The targets stay at plus and minus one, so doubling the step can't just double everything"): one move
  with the step that settled: the slope at the ball shown as a short tangent hairline; from the ball's knob value, an
  arrow along the knob axis (just under the curve, on a track), length = step x slope (true); a dotted vertical from
  the arrow's tip up to the curve; the ball slides along the arrow and rises to its landing: lower, past the bottom.
  Then the same start, with the step doubled: a second arrow twice as long, drawn over the first (the first stays,
  dashed, inside it). The curve does not change (the targets stay put): only the arrow doubles. That shows "can't
  just double everything".
- 11-20 s ("Too big, and each move overshoots ... lands higher up the other side"): the real 10 moves (one knob, twice
  the step). Each move: tangent, arrow (coral, along the axis, crossing the bottom line), dotted drop, ball to the
  curve. Each landing leaves a small numbered tick; the arrows get longer each time because the slope grows. Once a
  landing is about to leave the frame, the camera pulls back (both axes, uniform zoom, axis ticks re-labelling), so
  the U shrinks to a narrow notch while the arrows keep spanning more of the screen. The pull-back is the growth: the
  viewer feels the picture having to make room. End on a wide frame: the U tiny at the bottom, the last landing high
  on one wall, the arrows alternating sides.
- Data: the one-knob too-big run (the current beat seems to have it: "one knob alone, twice the step that settled");
  the settled step for the comparison. If the 137x is the 3-knob run, the on-screen growth must be the one-knob
  run's own number. Build: easy-medium (chart.py plus arrows and an animated axis range).

### map-intro (about 4 s)

"Here's a picture of what that would take: a sheet of borders."

Currently: the map beside a blank brown rectangle labelled "borders"; it sits blank through two beats and looks like
it should show something.

**A (recommended): pull back from one map to a sheet made of maps.**
- 0-1.5 s ("Here's a picture of what that would take:"): Mario's map, alone, centred, with the best border on it.
- 1.5-4 s ("a sheet of borders"): the camera pulls back; the map is one tile in a grid of small copies of the map
  (say 24 across x 9 up), each with its own border drawn: across the grid, the ruler turns (one full turn over the
  grid's width); up the grid, it slides. Every tile is a real border (angle and offset from the sheet's grid). The
  grid is the sheet, literally: a sheet of borders. Nothing is explained yet; the motion only says "there are many,
  laid out in order". The original tile stays outlined.
- Then map-axes zooms back in to the one tile (below), and `sheet` (not flagged) pulls back out again and lets the
  tiles shrink into spots: the flat sheet, with every spot one whole border. That makes the sheet mean what it is
  before it ever rises.
- Data: none new. Build: easy-medium (many small pointmaps; draw them as dot images rendered once, plus ruler lines).

**B:** no sheet at all in map-intro and map-axes; map only; the sheet appears in `sheet`. Simple; loses the "sheet
of borders" image the line promises.

### map-axes (about 18 s)

"Picture a border on Mario's map as a ruler laid across it, with a yes side and a no side. You can turn the ruler
around the middle of the map, and half way round, its yes and no sides have swapped; or you can slide it nearer to or
further from the middle."

Currently: the ruler turns and slides on the map (good), with the blank rectangle beside it.

**A (recommended): zoom into the one tile; the ruler is a physical thing with sides; then return.**
- 0-1.5 s: camera dollies from map-intro's grid into the outlined tile until it fills the screen (the other tiles
  slide off the edges, so the viewer knows this is one spot of that sheet).
- 1.5-6 s ("a ruler laid across it, with a yes side and a no side"): the border becomes a ruler: the hairline with
  short tick marks, a faint warm wash on its yes side and a faint cool wash on its no side (the washes cover the whole
  map, so the sides read as regions, not labels). The middle of the map (2.5 blocks, 1.25) marked with a small pivot.
- 6-12 s ("turn the ruler around the middle..., half way round, its yes and no sides have swapped"): the ruler turns
  smoothly about the pivot, washes turning with it. A small angle dial in the corner (knobs.py style) sweeps. At half
  a turn, pause 0.5 s: the ruler lies on the same line as at the start, but the warm wash is now on the other side.
  Ghost the starting ruler (with its washes, faint) for that pause so the swap is seen as "same line, sides
  swapped".
- 12-18 s ("or you can slide it nearer to or further from the middle"): the ruler slides along its normal: a short
  hairline from the pivot to the ruler, perpendicular, shows the slide distance growing and shrinking (a second dial,
  or a little ruler). Out, back, in.
- Ends on the ruler where it began, ready for `sheet` to pull back.
- Build: easy (pointmap + polygon washes).

### staircase (about 10 s)

"Imagine the sheet as ground, with each spot raised to a height: how many mistakes that border makes. Lower is
better."

Currently: the sheet beside the map, square on; the lift happens at one fixed camera, so it reads as a shading change
(Nate: static; rotate, zoom). This is the first time the sheet is 3D: the viewer must be shown that it's 3D.

**A (recommended): tip the sheet up into a slab, raise a few pillars with their borders, then the whole ground, under
an orbit.**
- 0-3 s ("Imagine the sheet as ground,"): start from `sheet`'s end (sheet face on, flat, the best border's spot).
  The map slides away. The camera tilts from straight down (pitch 90 deg) to pitch ~40 deg while yawing ~-30 deg
  over 3 s: the sheet becomes a floor seen in perspective, with fog toward the back. Still flat.
- 3-7 s ("with each spot raised to a height: how many mistakes that border makes"): four chosen spots first (the
  best border, the start-point border from big-drops later is fine too, one wrong-facing border, one middling):
  each rises as a narrow pillar to its true height, with a small floating map inset beside it showing its border and
  its "share wrong" number (roman). The 3D height ruler appears at the slab's corner as the first pillar rises (ticks
  10%, 50%, 90% wrong). Then the rest of the ground rises in a wave that sweeps across the sheet in the turning
  direction (the ruler turning), cells reaching their true heights. The camera keeps orbiting (yaw -30 to +20 deg)
  through the rise, so the rising is seen with parallax: depth is understood from motion.
- 7-10 s ("Lower is better"): the ground's low places light gold (the new ramp), high places fall to ash; a short
  downward arrow on the height ruler, "fewer mistakes". The camera, still orbiting, dips its pitch to ~30 deg so the
  big ridge and the valley read in silhouette against the dark.
- Renderer: yaw, target, fog, ramp, height ruler, per-cell lift, 3D markers. Data: exists.
- Build: medium once the renderer has yaw.

### staircase-2 (about 12 s)

"The height only changes when the border crosses a moment, so between those crossings it's perfectly flat. The
ground is a staircase of flat ledges."

Currently: a cropped window held still at the same angle, ledges as faint stripes. The line is about why it's flat,
which is a link between the map and the ground: show the link, then show the ledges from where they look like ledges.

**A (recommended): a slice through the ground, walked in step with the ruler sliding on the map; then a low raking
fly-over.**
- 0-3 s: camera dollies from staircase's overview down toward the lowest ledge (418 deg, offset 0.28), and the drawing
  switches to a native-resolution window (each cell one sample; label the vertical scale, e.g. "height x5 here").
  A cutting plane appears at the angle 418 deg, running up the sheet (the slide direction), hiding the ground in front
  of it: its cut face shows the profile as a lit hairline. The camera looks at the cut face from ~30 deg off square.
- 3-8 s ("The height only changes when the border crosses a moment, so between those crossings it's perfectly flat"):
  Mario's map inset (top left) shows the ruler at 418 deg. A bead walks along the profile line on the cut face as the
  ruler slides on the map, in lockstep. While the ruler moves between dots, the bead moves level; each time the ruler
  touches a dot (the dot flashes, and its count shows: "+1", "+3", "+710" is too big for this window, avoid it), the
  bead steps down or up by exactly that many moments. A 2D profile panel under the inset draws the same staircase as it
  goes (chart.py, full resolution). Slow at first (each flat stretch, each step visible), then faster.
- 8-12 s ("The ground is a staircase of flat ledges"): the cut plane fades; the camera lowers to pitch ~18 deg with a
  low raking sun from the side, cast shadows on; it flies slowly along the ledges (dolly sideways, ~1 window width over
  4 s), so the risers pass in front of each other like a hillside of rice terraces. Ledge-edge hairlines on. The
  lowest ledge marked with a 3D bead.
- Why: the cause (crossing a dot) and the effect (a step, flat in between) are on screen at the same time; then the
  viewer sees the terraces from the angle where terraces are obvious.
- Renderer: slices, native windows, raking light with shadows, ledge edges, yaw, target. Data: exists (mistakes at
  every sample; for the walk, the dots the ruler crosses at angle 418 as offset changes).
- Build: hard (the slice and lockstep walk), but every part is reusable for no-slope and shelf-2.

**B:** skip the walk; just do the slice (static profile panel + cut face) and the low raking fly-over. Medium.

### no-slope (about 19 s)

"Standing on a flat ledge, every direction looks the same. Only a drop tells you which way is down, and you find it by
falling off. So a rule that follows a slope, like the square trick, can't walk down this staircase to the lowest
ledge, around the best border we found."

Currently: a bead with arrows that reach past its ledge (Nate: confusing), one held camera. Through this lens: put the
viewer on the ledge, turn around, then step off.

**A (recommended): stand on one ledge, look all the way around, step off, rise up.**
- 0-2 s: camera descends to one ledge a few steps above the lowest one (a real ledge, chosen so it's big enough to
  stand on at native resolution). The ledge's outline lit as a hairline (the connected set of cells at its height:
  compute by flood fill), the rest of the ground dimmer. A bead stands on it.
- 2-7 s ("Standing on a flat ledge, every direction looks the same"): the camera orbits a full 360 deg around the bead
  at a low pitch (~15 deg) and close distance. Eight short feeler arrows from the bead, each shorter than the
  distance to the ledge's edge in that direction (computed, to fix Nate's note), each with a tiny "0" (the slope there).
  The orbit is what the line says: the view from every side is the same, level.
- 7-12 s ("Only a drop tells you which way is down, and you find it by falling off"): the bead rolls (constant
  speed, slow) in one direction to the ledge's edge; the camera follows behind it; at the edge it drops to the ledge
  below (a short fall, real height), the mistakes readout ticks down by that ledge's step. Then a second try, from the
  same start, in a direction where the neighbouring ledge is higher: the bead reaches that edge and can't go on (it
  meets a wall; a small "+k mistakes" if it climbed). The point: the direction was only known after moving.
- 12-19 s ("So a rule that follows a slope... can't walk down this staircase to the lowest ledge, around the best
  border we found"): back to the bead's starting ledge; a compass rose over it with no needle ("slope: 0 in every
  direction"). The camera rises and pulls back to an oblique view of the whole neighbourhood: the bead's ledge, the
  terraces between, the lowest ledge glowing with the best border's bead on it. Hold.
- Renderer: yaw/orbit, target, ledge outline (flood fill), markers, raking light. Data: exists.
- Build: medium-hard.

### staircase-outro (about 17 s)

"So from a rule that settles about three in a hundred short, we've come to the thing we actually want, the count of
mistakes, and found a staircase no slope-following rule can walk down. So where does the square trick's slope lead,
on this sheet? And what about the perceptron?"

Currently: the whole staircase at one camera, low contrast (Nate: can't tell what the landscape looks like). This is
the beat to give the full shape: a fly-around with the improved look.

**A (recommended): a slow orbit and a swoop, with the new ramp, banding, ledge lines and fog.**
- 0-5 s ("So from a rule that settles about three in a hundred short,"): overview of the whole sheet, pitch ~50 deg,
  yaw -40 deg. Two beads in the scene: the lowest ledge (gold) and the square trick's border (cool), each on a stem
  so it's anchored in 3D. Their heights on the 3D ruler, ticked: about 3 points apart (91% vs 88% right, i.e. 9% vs
  12% wrong).
- 5-10 s ("we've come to the thing we actually want, the count of mistakes, and found a staircase..."): the camera
  orbits 80 deg around the sheet's centre, slowly, while dropping its pitch to ~30 deg, so the valley's two walls and
  the big ridge are seen from the side and from the end. During the orbit the ridge passes behind the valley; the
  half-turn symmetry (the ridge is the valley flipped) becomes visible as the camera lines them up. Optional label
  at that moment: none, the motion is enough.
- 10-14 s ("no slope-following rule can walk down"): swoop: the camera dives along the valley's floor toward the
  lowest ledge at a low pitch (~15 deg), raking light, terraces passing.
- 14-17 s ("So where does the square trick's slope lead, on this sheet? And what about the perceptron?"): pull up and
  back to a 3/4 view; the square trick's bead pulses once. The perceptron is not shown (that's the next sections).
- Renderer: everything from Part 1 items 1-5, 8, 9. Data: exists. Build: medium once the renderer is done.

### shelf-2 (about 15 s)

"Then every total is zero, every moment misses by exactly one, and all those borders sit at the same height: a high,
flat shelf, filling the side of the sheet where the ruler mostly faces the wrong way."

Currently: a lit dish on the left, a flat lighter-brown block on the right, one camera. The block looks lower and
brighter than the dish, the reverse of the truth; nothing shows it's at exactly one height, or that it's high.

**A (recommended): slice through the ground across all angles, then sweep the slice; the shelf cells settle into one
plane.**
- 0-4 s ("Then every total is zero, every moment misses by exactly one,"): from shelf's end, the camera swings low to
  the side (pitch ~10 deg, looking along the offset axis, the angle axis across the screen). The wrong-facing borders'
  cells are raised a little off their true height (their pre-shrink ghost, or just faded) then fall into place: a
  translucent hairline plane at height 1.0 (labelled "every moment misses by 1" on the 3D ruler) appears, and the
  shelf cells settle into it, all at once, in perfect level. The bowl part is already in place and lower. Banding
  colours: the shelf in the darkest ash band (it's the highest ground), the bowl glowing down to gold at its bottom.
- 4-10 s ("and all those borders sit at the same height: a high, flat shelf"): a cutting plane at constant offset
  (through the bowl's bottom, offset 0.35) across all angles: the cut face shows the profile: from the shelf's
  flat line at exactly 1.0, it falls into the bowl's curve to 0.37 and rises back to 1.0. The profile panel below
  draws the same. Then the plane sweeps up through the offsets (3 s): the bowl's curve in the profile changes, the
  shelf's line stays dead flat at 1.0 every time. That's "the same height" shown by motion.
- 10-15 s ("filling the side of the sheet where the ruler mostly faces the wrong way"): the slice fades; the camera
  rises to pitch ~55 deg and orbits 60 deg, so the shelf reads as a high mesa occupying one side of the sheet, the bowl
  sunk into the other. Tiny map insets at two points on the shelf (optional), each showing the ruler facing the wrong
  way (its yes side on the no dots).
- Renderer: slices with a sweeping position, per-cell lift, ramp/banding with weak top shading (so the flat top can't
  look brighter than low ground), yaw, height ruler. Data: exists.
- Build: medium.

---

## Ranked summary (strongest first, by what this lens buys)

1. Renderer: yaw/orbit with look-at, ledge-edge and iso-lines, a perceptual height ramp with bands, raking light with
   cast shadows, fog, native-resolution windows, slices, 3D markers, a height ruler. Every sheet beat depends on it.
2. staircase-2: a slice walked in lockstep with the ruler crossing dots on the map, then a low raking fly-over.
3. shelf-2 and staircase-outro: a sweeping slice that keeps the shelf at exactly 1.0; a full orbit and swoop of the
   staircase with real contrast.
4. map-intro / map-axes: the sheet of borders as a literal grid of small maps; pull back to it, zoom into one tile.
5. too-big: tangent, step arrow along the axis, drop to the curve; the camera pulls back as the overshoot grows.
6. downhill-2 / balance: the U as the valley's cut face under an orbit, iso-lines and true gradient arrows; then a
   rope of real per-point pulls at the bottom.
7. too-many / no-fast-way: the map grows a third axis; a pull-back through stacked sheets at log-constant speed.
8. exam: two stacked glass maps; the border drops from training to exam and the exam dots light.

## Lines, separately

- downhill-2: "Turn them all at once and it's a valley" turns three knobs; the valley shows two (bias held). Suggest
  "Turn two at once" or keep "bias held still" on screen.
- no-fast-way: fine as written; the picture must label any times as "trying every border", since the line claims
  something about every known method.
