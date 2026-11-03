# Picture council: Continuity lens (video 002, 13 flagged beats)

## What the lens sees in the current render

Almost every beat opens with `away(scene)`: everything fades out and the next
picture fades in on an empty ground. The video has strong nouns (Mario's
map, the knobs, the U, the sheet), but the cuts between beats mean they never
*become* each other. The viewer is shown a U, then a separate valley, then
bar charts, then the U again, and has to work out alone that these are one
landscape seen four ways. In chapter 4, the sheet is born blank, then
`staircase-2` cuts to a new close-up, `no-slope` re-cuts the same close-up,
and `staircase-outro` cuts back out to a different camera. Through this lens,
most of the flagged complaints ("what is the blank sheet for", "static",
"not sure what's illustrated") come from pictures that arrive without a
parent. A picture someone watched being *made from* the last one doesn't
need explaining.

Below: a shared vocabulary, then each beat with ranked proposals. Times are
fractions of the beat's voice (`T`), with rough seconds from the current
stills in brackets.

---

## The shared vocabulary (applies to every beat below)

1. **The map** is always the same object, `PointMap`, at one of two
   sizes: *full* (centre, about 7.6 wide) and *inset* (`small_map`, left,
   5.4 wide). It never fades out and back in between beats that both use
   it. It moves between the two sizes with a transform (0.6 s). In
   chapters 4 and 5 the inset map stays on screen (dimmed to 40% when
   nothing on it moves) for every sheet beat, so the viewer can always look
   across to see which border a spot means.
2. **The sheet** is one `SheetPicture` with one continuous camera from
   `map-intro` to the end of chapter 5. It is never rebuilt with a new
   camera between beats. Zooms are camera moves (distance and window
   change smoothly), not cuts. Its two axes keep their labels whenever it
   is seen from far enough away to read them.
3. **The knobs** (`Knobs`, three dials) are the weights' only face. They
   sit top-left in every beat where the weights change (learn, downhill,
   downhill-2, balance, too-big). When a beat says "the weights", the dials
   are what moves.
4. **One landscape for the square trick**, seen three ways: the **U** (one
   knob turned: a slice), the **valley** (two knobs: the surface the U is a
   slice of), and later the **bowl** (every border at its best size). Each
   of these is introduced *out of* the last one, never beside it.
5. **Pulls and moves**: thin arrows are each moment's own pull (warm if it
   turns the knob up, cool if down, per the visual language); one thick
   warm-white arrow is their sum, the added-up nudge; a move is that arrow
   times the step, drawn along the knob's axis. The same three marks mean
   the same three things in downhill-2, balance and too-big.
6. **Transitions**: `away()` is kept only where the subject really
   changes (a new chapter's title wall, the footage). Elsewhere a beat's
   first step is a transform of what's already on screen. When a cut is
   unavoidable, it's a match cut: the outgoing object's position, size and
   colour are where the incoming one appears.

---

## too-many (Introduction) [~17 s]

The line: two numbers make trying borders easy; each extra number multiplies
the borders to try; 2,048 numbers would be far too many.

The current picture (the memory grid lighting up) has no parent: it appears
after a map and the viewer has to guess that the lit squares relate to the
map. It also never shows *borders*, which is the line's noun.

**Proposal 1 (strongest): the search's borders become a sheet of tiles, and
each new number multiplies the sheet.**

- `search` (not flagged) already flicks through 14 tried borders on the
  map. Small change there: each tried border leaves a ghost (opacity 0.12)
  instead of vanishing, so `search-2` ends on the map with its best border
  and a faint fan of tried ones.
- 0 to 0.25 T: the map shrinks to a tile (about 1.2 wide) top-left, keeping
  its best border. Out of it, copies deal out to the right and down into a
  grid of small tiles, each one the same map (dots as a faint texture) with
  a different border: across the row the ruler turns; down the column it
  slides. This is honest and real: tiles are drawn from the actual sheet
  grid, subsampled (for example 24 turns × 12 slides shown, labelled "every
  border we tried: {432,720}", from 720 × 601). Words: "two numbers".
- 0.25 to 0.45 T ("each extra number multiplies"): to the left of the tile
  grid, the memory grid fades in, small, with two cells lit; thin hairlines
  run from those two cells to the tile's two axes, so the map's axes are
  seen to *be* those two numbers. A third cell lights. The tile grid
  tilts back into a block, copies of itself stacking into depth (a third
  direction of turning), and the count multiplies (shown as the
  multiplication, not just the result: "432,720 × 720").
- 0.45 to 1.0 T ("two thousand and forty-eight"): more cells light,
  accelerating (the existing lighting animation). Each time a cell lights,
  the block's edge of tiles recedes further: the camera pulls back so the
  block becomes a speck in a growing lattice that runs off every edge of
  the frame. The count becomes a power ("720 multiplied by itself 400
  times") once it can't be written. End frame: memory grid bright, the
  lattice of borders filling the whole background as a fine texture.
- Why better for continuity: the borders the viewer just watched being
  tried *are* the crowd. The memory grid gets a parent (the map's axes).
- Data: the sheet (720 × 601) for tiles; the search's tries; memory cell
  addresses (0x0755, 0x0057 already used). The multiplier per extra number
  needs an honest definition (same angular fineness per added direction);
  the Honesty lens should rule on it.
- Cost: medium-high. Tile thumbnails: pre-render one dot texture, draw
  borders per tile (cheap). The block-and-lattice pull-back is 2D fakery
  of depth (scaled copies), fine for a texture that's labelled as a count.

**Proposal 2 (cheaper): keep the memory grid, give it a parent.** Open on
the `search-2` map; its two axis labels detach and fly to the two memory
cells they're read from (the cells light as they land). Then the existing
lighting, with a thin border line flickering through each newly lit cell
to say "another way for the border to turn". Weaker: still no crowd of
borders.

## no-fast-way (Introduction) [~7 s]

The line: no known method finds the fewest-mistakes border quickly.

The clock and random lines are symbols with no parent (the lines aren't
the borders from the beat before; they're random).

**Proposal 1 (strongest): the lattice from too-many is searched, and the
search can't keep up.** Continue the very same lattice (no `away`).

- 0 to 0.2 T: a single warm cursor (a lit tile) starts at one corner of the
  lattice and steps through tiles at a speed the narration gives time to
  read: it visits a row, scoring each tile (a tiny score number under it).
- 0.2 to 0.8 T: the cursor speeds to a blur; the camera keeps pulling back
  as before, and the region it has covered stays a visible patch that
  shrinks relative to the frame: the faster it goes, the smaller its patch
  looks, because the lattice is growing faster than it's covered. (This
  shows "not quickly"; the line says *no known method*, not just "trying
  them all", so add words: "trying them all: the only sure way known".)
- 0.8 to 1.0 T: everything dims except one tile, near the centre, which is
  pulled forward and enlarged: a tile whose map has **no border** at all.
  That tile grows into the full map of `learn`, where the neuron starts
  with every weight at zero (no border), and the knobs fade in reading
  0.00. Match cut into `learn` with no fade.
- Data: same as too-many. The "scored" numbers on visited tiles are real
  mistake counts for those tiles (2-number case) at the start.
- Cost: medium, if too-many's lattice exists; the push-in to the empty
  tile is a transform.
- Honesty note for the builder: "no known method quickly" is a claim about
  *all* methods; the picture shows one method (exhaustive search). Words on
  screen should say so; the Idea lens may have a better picture of the
  claim itself.

**Proposal 2:** if the lattice is not built, at least make the crowd the
search's real tried borders on Mario's map (not random lines), multiplying
on the map, and drop the clock. Push in on the map to start `learn`.

## exam (It never settles) [~8 s]

The line: after every pass, scored on recordings it never learned from.

Current: two lists of words. Nate asks for the actual data.

**Proposal 1 (strongest): the map splits in two.**

- Neighbour before (`bounded`) is a weights-path chart; neighbour after
  (`one-run`) is a map plus a score chart. So this beat should end looking
  like `one-run`'s opening.
- 0 to 0.15 T: `bounded`'s path dims; its last point (the weights after
  pass 30) stays lit, then becomes a border drawn on a full map that fades
  in under it (training dots).
- 0.15 to 0.5 T: six tapes (records.py) sit above the map labelled by
  recording number. Tapes 1, 2, 4, 5 tip their dots into the map (the dots
  pulse as their tape glows: these are the moments it learned from). Tapes
  3 and 6 slide right; a second map, the exam map, fades in beside the
  first, *empty*.
- 0.5 to 0.85 T ("after every pass, we scored it"): tapes 3 and 6 pour
  their 4,883 moments onto the exam map. The border from the training map
  slides across onto the exam map (the same line, copied, hairline from one
  to the other). Wrong moments get a ring; a score number counts up under
  the exam map to that pass's real exam score.
- 0.85 to 1.0 T ("an exam it hasn't seen"): the training map dims to an
  inset at left; the exam map and its score move to the positions where
  `one-run` puts its map and chart, and the score becomes the chart's
  first point. `one-run` then starts without `away`.
- Small change requested in `one-run`: its map should be the exam map
  (test dots), since its chart is exam scores. Then the border jumping
  there visibly gets different exam dots wrong each pass.
- Data: test-set moments by point and label; the run's weights per pass;
  exam scores per pass. Optional: the game footage could play on each tape
  (a few frames), tying back to the intro's footage.
- Cost: medium. Two PointMaps, tapes exist, a ring mask exists.

## downhill-2, balance, too-big: one picture (How wrong was it?)

These, plus `downhill`, should be one landscape the camera keeps returning
to. The order is downhill (U) → downhill-2 (valley) → balance (bottom) →
settles-2 (score chart) → settles-3 (map) → too-big (U) → too-big-2 (chart).
The thread through the two interruptions is the **knobs**, top-left,
which never leave between downhill and too-big-2 (in settles-3 they read
the border's weights as it moves, as they did in `learn`).

### downhill-2 [~11.5 s]

The line: turn them all at once and it's a valley; the added-up nudges
always point downhill; that's following a slope.

Current: a valley that appears from nothing, unexplored, with an arrow
pointing roughly along the camera. Nothing shows it's the U's big sibling.

**Proposal 1 (strongest): the U is pulled into a valley.**

- Start on `downhill`'s last frame: the U chart, the ball, the knobs with
  "the others held still".
- 0 to 0.3 T ("turn them all at once"): "the others held still" fades. The
  chart tilts back in 3D (camera pitch from 0 to about 0.85, with a slow
  yaw) so the U is now a curve standing on a floor. The speed knob starts
  turning; as it turns, the U slides along a new axis into depth, and
  leaves behind it the surface it sweeps: the valley is built slice by
  slice out of the U the viewer just watched being drawn. Then the speed
  knob turns back the other way and the surface fills towards the viewer.
  Axis labels move with it: "position knob" stays on the axis it was on;
  "speed knob" is new, on the new axis.
- 0.3 to 0.5 T: camera orbits a quarter turn so the trough's shape is felt
  (steep across position, shallow along speed: say so on screen? no, let
  the orbit show it). The U slice stays drawn on the surface as a warm
  line so it's still findable.
- 0.5 to 0.85 T ("the square trick's nudges always point downhill"): at
  about 6 spots on the slopes, a thick added-up-nudge arrow is drawn
  lying on the surface (projected, so its length is the true direction in
  knob space), each pointing downhill. Then the ball from the U, still on
  its slice, gets its own arrow, and takes a few real added-up moves
  (bias held, computed in the 2D valley) down to the trough, the knobs
  turning as it goes.
- 0.85 to 1.0 T ("following a slope"): words "following a slope" beside
  the ball's trail.
- Ends on the ball resting at the lowest spot. That is `balance`'s start.
- Data: the 61 × 61 valley (exists); the U (exists, and is the valley's
  row at the resting speed weight: check they agree); added-up nudges at
  chosen spots (computed in the beat already); a 2D descent path with the
  bias held (needs computing, a few lines).
- Cost: high. sheet.py needs yaw and to draw a curve and arrows on the
  surface (it can project points already: `sheet.at`). The "slice sweeps
  out" is a lift animation masked by column (render columns up to k).

### balance [~14 s]

The line: one lowest spot; the pulls are still there but balance; tug of
war at a standstill.

Current: bars on an empty screen: correct numbers, but no place. The
viewer can't see that this is "the bottom of the valley".

**Proposal 1 (strongest): zoom to the bottom of the same valley, and
let the pulls be the rope.**

- 0 to 0.2 T: continue from downhill-2: camera rotates back to face the U
  head-on (pitch back to 0) as it pushes in on the bottom, so the valley
  becomes the U again, the ball at its bottom, now at the resting weights.
  The knobs read the resting weights.
- 0.2 to 0.6 T ("the pulls are still there"): from the ball, every
  moment's pull on the position knob springs out as a thin horizontal
  arrow along the U's position axis: warm to the right (turn up), cool
  to the left (turn down). There are 10,720 moments; draw them as a
  sorted sheaf (each arrow's length its real pull, stacked one above
  another, so the sheaf has the real shape) and fade it to the two summed
  bars: warm +0.766 right, cool −0.766 left. They are the same length, and
  meet at the ball: a rope held still.
- 0.6 to 0.85 T ("add up to exactly nothing"): the two bars pull, tremble
  (tiny, not a real motion, so keep it under 2 px or skip it), and their
  sum, the thick added-up-nudge arrow from downhill-2, is drawn at the
  ball with length zero: a dot and "= 0.000".
- 0.85 to 1.0 T: the same pair of bars appears small beside each of the
  other two dials (speed: ±0.171, bias: ±0.244), so all three knobs show a
  standstill. The current bar rows survive as these three small ropes.
- Bridge to settles-2 ("only the sum is nothing; each moment's own pull
  isn't"): the sheaf of single pulls flickers back for a beat: one
  moment's arrow alone is not zero. settles-2 can start from there.
- Data: per-moment pulls at rest (exist); the U at the resting weights.
- Cost: medium. 2D chart overlay; the sheaf is a few thousand lines,
  rasterise them.

### too-big [~19 s]

The line: step size decides whether it gets there; targets fixed, so
doubling isn't harmless; too big overshoots the bottom of the U and lands
higher up the other side.

Nate: the jump is too literal; an arrow showing the size of the change,
laid on the position to show the overshoot.

**Proposal 1 (strongest): the U returns with the arrows it already
taught.**

- Entering from settles-3 (map with the border settling, knobs reading it):
  the map shrinks to an inset; the knobs stay; the position dial is
  picked out and the U fades in from it (the way downhill drew it), with
  the ball placed at the real start point (resting − 0.12), the other
  knobs held. "one knob alone" appears where "the others held still" was
  in downhill: same words place, same meaning.
- 0 to 0.35 T ("the size of the step decides"): at the ball, the thick
  added-up-nudge arrow from downhill-2 (pointing downhill). Under it, on
  the position axis, the **move**: an arrow starting at the ball's knob
  value, length = step × nudge. First show the step that settled: a
  dimmer arrow that ends short of the far side, a dashed hairline up from
  its tip to the curve, a ghost dot lower than the start. Then the
  arrow doubles in length, in place (this is "doubling the step"),
  reaching past the bottom.
- 0.35 to 0.9 T ("too big, and each move overshoots"): the ball moves
  along the axis to the arrow's tip (straight, along the floor, no arc),
  then a vertical hairline rises to the curve: it lands higher than where
  it started (both heights marked on the y axis with a tick, so higher is
  read, not guessed). The next arrow is drawn from there: it's longer,
  because the slope is steeper. Repeat for the real moves, each arrow
  longer, ticks climbing. Keep each spent arrow as a faint ghost so the
  growing lengths are visible side by side.
- 0.9 to 1.0 T: the stack of landing ticks on the y axis is lifted out and
  laid on its side, becoming the first points of `too-big-2`'s chart of
  total squared miss per move. (Honesty flag: `too-big-2`'s numbers are
  all three knobs moved together, not one knob alone; if so, don't carry
  ticks across, carry only the axis label and say "all three knobs" on
  entry.)
- Data: `jumps()` already computes the real one-knob moves; the settled
  step's moves the same way with 1× rate.
- Cost: low-medium. All 2D on the existing chart.

**Why one picture**: the viewer has now seen one U, swept into a valley,
descended, stood at the bottom with balanced ropes, and returned to the
same U where the arrow they learned as "downhill" becomes the move that
overshoots. The arrow's meaning never changes.

## map-intro and map-axes (Close, but not quite)

The line pair: "a picture of what that would take: a sheet of borders" /
the ruler on the map, turned and slid.

The blank sheet looks like a slide that failed to load: nothing on screen
says what a "sheet of borders" is, and it stays blank for two beats.

**Proposal 1 (strongest): the sheet is born as tiles of real borders, the
same tiles as too-many.**

- If too-many's tile grid is built, this is a callback the viewer already
  holds: "the borders we tried" laid out turn × slide.
- map-intro [~4.6 s]:
  - 0 to 0.3 T: from square-outro (map, the two still borders), the
    square trick's border dims; the map moves to the inset position, keeping
    the best border.
  - 0.3 to 1.0 T ("a sheet of borders"): a copy of the map, with its border,
    shrinks off it and lands on the right as one tile. Around it, tiles
    deal out row by row into a rectangle (say 16 turns × 9 slides), each a
    miniature of Mario's map with a different real border and its yes side
    faintly warm. The label "borders" sits over them. No axis labels yet.
    It is now obviously a sheet *made of* borders.
- map-axes [~17 s]: the line is about the ruler on the map; the sheet
  should help without being explained early.
  - 0 to 0.05 T: tiles dim to 30%; the inset map grows a little toward
    centre-left. Middle dot appears (exists).
  - Turning (exists): as the ruler turns, the tile in the current row whose
    border matches is lit (full opacity, a hairline frame), so a lit square
    walks along a row. At half a turn, the viewer sees on the map the yes
    side swapping, and on the lit tile its warm tint jumping to the other
    side. The sheet isn't named; the viewer just sees the lit tile move
    *right* as the ruler turns.
  - Sliding (exists): the lit tile walks up and down a column.
  - This does show the sheet's axes one beat early, but only as a
    correspondence (no labels, no claim); sheet then names what the viewer
    has already watched. If that breaks "don't show things early", the
    alternative is: keep the tiles at 30% and unlit through map-axes, and
    only light the matching tile in `sheet`.
- sheet [not flagged, but it must change]: 0 to 0.15 T, the tiles melt into
  a continuous sheet: each tile shrinks to its centre spot, and the gaps fill
  in; the lit tile becomes the moving spot. Then the existing animation
  (spot moves, ruler moves) with the axis labels. Off the right edge and
  back on the left is clearer if, at that moment, the first column of tiles
  briefly reappears as ghosts past the right edge (same borders as the
  left column).
- Data: the sheet's angle/offset grid gives every tile's border; tiles use
  `PointMap`'s dot layer downsampled.
- Cost: medium. ~150 tiles of a dot texture plus a line each; one
  "melt" transform (tiles scaling to dots and a sheet image fading in under
  them).

**Proposal 2 (cheaper):** in map-intro, the map's border is copied as a
single spot onto the sheet ("this border is one spot"); and in map-axes the
sheet is moved off-screen (or dimmed to nearly nothing) while the ruler is
explored, returning in `sheet`. Removes the blank, but loses the "made of
borders" picture.

## staircase, staircase-2, no-slope, staircase-outro: one continuous flight

These four (with big-drops between) are one ground. They're now four
separate pictures with three different cameras.

### staircase [~9 s]

The line: the sheet as ground, each spot raised by its mistakes.

**Proposal 1:** from sheet's last frame (map inset left, flat sheet right,
"the best border we found" spot):

- 0 to 0.15 T: the inset map dims to 40% but stays; the sheet slides
  toward centre and grows.
- 0.15 to 0.7 T ("raised to a height"): before the whole sheet lifts, lift
  one spot: the best-border spot rises on a hairline to its height while a
  counter reads its mistakes (7.8%), and the matching ruler on the inset
  map lights the wrong dots. Then a second spot (a bad border: the ruler
  facing the wrong way, ~90%) rises high. Then everything rises (the
  existing lift), camera yawing slowly (about 30°) as it rises and pitching
  down from top view to about 0.8, so the ground is seen to be ground.
- 0.7 to 1.0 T ("lower is better"): a colour ramp lands on the heights
  (warm low? no: per the language warm is yes/above zero. Use light for
  low/good, i.e. lightness as understanding, dark high: the Legibility
  lens should choose). The best spot glows at the low end.
- Cost: needs yaw (Nate's note asks for it everywhere in this run).

### staircase-2 [~11 s]

The line: height only changes when the border crosses a moment; flat
between; a staircase of ledges.

**Proposal 1:** no cut. The camera flies from staircase's view down into
the window around the lowest ledge (interpolate window and distance over
2 s), the best-border spot leading the eye in.

- 0.3 to 0.7 T ("only changes when the border crosses a moment"): a marker
  walks along one ledge; on the inset map (brought back to full opacity) its
  ruler moves but passes between dots, the mistake count unchanged. Then the
  marker steps off the ledge's edge: on the map the ruler touches a dot,
  the count ticks, the marker drops by exactly that. Match: one ledge
  edge = one point on the map.
- 0.7 to 1.0 T: camera orbits around the lowest ledge (narrow, with its
  cliff side) so the terraces read in depth.
- big-drops (not flagged) then **keeps** this: the map grows to full size,
  the close sheet shrinks to an inset at right, and the curve of all
  borders passing through the start point (2.5, 0) is drawn on the sheet:
  it's the tallest cliff. As big-drops' border crosses the 710-moment point,
  a marker on the sheet steps over that cliff and falls 710. Same picture,
  roles swapped.

### no-slope [~18 s]

The line: on a ledge every direction looks the same; you find a drop by
falling off; a slope-follower can't walk down to the lowest ledge.

Nate: more dynamic; the arrows are wider than the ledge.

**Proposal 1:** return from big-drops by swapping the insets back (sheet big,
map inset). Same camera as the end of staircase-2.

- 0 to 0.4 T ("every direction looks the same"): replace the eight arrows
  with the marker *trying* directions: short test steps, each smaller than
  the ledge, each returning, while a small height readout beside it stays
  fixed. The inset map's ruler wiggles by the same amounts and crosses no
  dots. Arrows, if kept, must be drawn on the ledge's surface in the
  sheet's projection and clipped to the ledge.
- 0.4 to 0.6 T ("you find it by falling off"): one test step goes past the
  edge: the marker drops; readout changes; a ring on the map where the
  ruler crossed a dot.
- 0.6 to 1.0 T ("can't walk down… to the lowest ledge"): camera rises and
  pulls back along the staircase toward the lowest ledge, which glows,
  many ledges away from the marker. This pull-back is the first half of
  staircase-outro's camera move.

### staircase-outro [~16 s]

The line: the chapter summary; the square trick's border off the lowest
ledge; where does its slope lead on this sheet?

Nate: contrast and dimensionality aren't enough; colour and move.

**Proposal 1:** continue the pull-back from no-slope out to the whole
sheet (no `away`): a slow full orbit (about 120° of yaw over the beat, pitch
easing between 0.7 and 1.0), so the staircase's long trough and ridges are
seen from several sides.

- 0.3 T: the lowest ledge spot (warm, carried from staircase) lit.
- 0.55 T ("the square trick"): a cool spot lands on the sheet off the
  lowest ledge; on the inset map, the square trick's border appears beside
  the best border (the pair from square-outro: a callback that closes the
  chapter at the picture it opened from).
- 0.8 to 1.0 T ("where does the square trick's slope lead, on this sheet?"):
  camera settles on the exact view `bowl` will use; the heights start (very
  faintly) to breathe, as a cue. Then `bowl` morphs the heights from
  mistakes into squared miss on the same sheet, same camera, no `away`: the
  chapter boundary is a height change, not a new picture. (If the chapter
  title wall must sit between them, then bowl opens on this exact frame.)

## shelf-2 (A bowl) [~14.5 s]

The line: at size zero every total is zero, every miss is one, these
borders sit at one height: a high flat shelf on the side where the ruler
faces the wrong way.

Current: a sheet that cuts in, already finished, half of it a flat tan
block. Nothing links it to `shelf`'s one shrinking border.

**Proposal 1 (strongest): the shrinking border from `shelf` lands on the
sheet, and the shelf grows from it.**

- `shelf` ends on: inset map with a wrong-way border; size chart with the
  dot at size 0, squared miss 1.
- 0 to 0.2 T: the size chart's dot (height 1) lifts off and flies to the
  sheet, which fades in (same camera as bowl/bowl-2) in its *pre-shelf*
  state: flat at floor level, empty. The dot lands at its own border's
  spot, at height 1 (on a hairline post), labelled "1".
- 0.2 to 0.7 T ("all those borders sit at the same height"): neighbouring
  borders that are also best at size zero rise to height 1 one wave at a
  time, spreading from that spot (a flood fill ordered by distance), so
  the shelf visibly *forms* as one border after another joins it. On the
  inset map, the ruler sweeps through the borders the wave is reaching:
  it's facing the wrong way, its yes side over mostly cool dots.
- 0.7 to 1.0 T ("filling the side where the ruler mostly faces the wrong
  way"): camera yaws to show the shelf's edge as a cliff over the
  not-yet-risen side (still at floor). Then `bowl-2` raises the remaining
  side into the bowl (bowl-2 is unflagged; its job becomes "the rest
  rises, curving down into one bottom", same sheet, no `away`).
- Data: the sheet of squared miss and its best-size mask (the ~52% shelf);
  `shelf`'s chosen border's spot.
- Cost: medium. A lift with a per-cell delay (lift is already per frame;
  give it a per-cell start time).

---

## Lines that are themselves the problem (narration not changed)

- **downhill-2**: "Turn them all at once and it's a valley." The valley holds
  the bias still and turns two knobs. With "all", the picture is not quite
  what's said. Either "turn two at once" or put "the bias held" on screen
  (the knobs show it: the bias dial doesn't move).
- **map-intro**: "a sheet of borders" names a thing the viewer hasn't seen
  yet and the very next line is about the map, not the sheet. Fine if
  the tiles carry it; if the sheet must stay unexplained until `sheet`,
  the line asks for a picture it can't honestly give yet.
- **too-big**: comes after `settles-3`, which is on the map, so "the U" in
  "overshoots the bottom of the U" refers back three beats. The returning
  U (with the knobs as thread) handles it; no line change needed if it
  does.
- **no-fast-way**: "no known method" is a claim about every method, but any
  search picture shows one method. Not a line problem, but the picture
  should carry words saying which method is shown.

## Build order (if prototyping one thing first)

1. The continuous sheet camera with yaw (unblocks staircase through
   shelf-2, and is what Nate asked for most).
2. The U → valley → U sequence (downhill-2, balance, too-big), mostly 2D
   plus one 3D tilt.
3. Tiles (too-many, map-intro, map-axes, sheet), which are one piece of
   code used in four beats.
