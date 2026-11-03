# Picture council, lens: Legibility (video 002, 13 flagged beats)

Judged from the 854x480 still sheets, so sizes are given as share of frame and as px at 1080p.
"Phone" means a 1080p frame shown about 6 inches wide: anything under about 28 px tall at 1080p
is a smudge, and contrast under about 4.5:1 against the local background disappears in daylight.

## 0. Things that apply to every beat below

### 0.1 Text

- **Minimum label size: 34 px cap-to-baseline at 1080p** (x-height around 20 px). Most labels in the
  stills are about 7–8 px in an 854 frame, i.e. about 16–18 px at 1080p: half of what a phone needs.
  The spec's "0.42 of Manim's default" is the right size for the spike's sparse frames; on the 3D
  surfaces and charts it is too small. Proposed scale: labels 0.6, the one number the beat is about
  0.9, captions 0.5 (and captions only when there is nothing else on the frame).
- **Text colour for labels: `#E6D5BC` (parchment) at full opacity**, not the dim `#8A7A6A`-ish grey
  used for axis titles and captions ("speed knob, into the picture", "the shelf: every border...").
  The dim grey is about 3:1 on the Ember ground; parchment is about 11:1. Dim is fine only for things
  meant to recede (axis ticks).
- **Never set a label on top of a lit surface without a halo.** Every label that sits over the sheet
  gets a 4 px blurred halo of `#0E0806` at 85% (the type's own shape, dilated), or moves off the
  surface with a 1.2 px leader line to its point. Today "the lowest ledge" (white on gold) and "the
  added-up nudge" (coral on gold) are unreadable.
- **No rotated text.** "slides it ↑" set vertically on the sheet's side is unreadable on a phone.
  Set it horizontal, beside an arrow that points up the sheet.
- **One caption at most, short.** The long top captions ("the shelf: every border here is best at
  size nothing, its squared miss exactly 1") are small, dim, far from what they describe, and
  duplicate the narration. Replace with a 2–4 word label placed on the thing.

### 0.2 Markers

- Points of interest (lowest ledge, square trick, the marker on a ledge) are about 6 px at 1080p
  today. Make them **16 px discs with a 2 px dark ring (`#0E0806`) and a 1.4 px light outer ring**, so
  they read on dark and bright ground alike.
- **The square trick's dot is teal today.** Teal is cool, which in this series means "no / below
  zero"; the square trick's border is neither. Give the two named borders two warm hues that differ
  in lightness as well as hue: lowest ledge (fewest mistakes) **gold `#FFD59A`**, square trick
  **coral `#EE8660`**, both with the dark ring above. (If a hue outside the temperatures is wanted
  for "a thing, not a sign", use white `#FFF4E2`, never a cool.)

### 0.3 The 3D surfaces (the creator's main complaint): colour ramp, light, lines

The problem, from `sheet.py`: height is coloured from gold `#FFD59A` (low) to ash `#3A2A22` (high),
a single-hue *brightness* ramp; then the sun shading multiplies brightness again (SHADOW 0.35, the
sun high and in front). So brightness is carrying two things at once (height and slope), the hue
never changes, and most of the sheet lands in one mid-brown band (`#4A3A2E`–`#6A5440`). Flat ledges
facing the sun (the shelf, the plateau) come out *brighter* than lower curved ground, so "highest"
can look "lightest", which contradicts "low glows".

Proposal: split the jobs.

1. **Height is hue plus lightness (albedo), on a perceptual ramp.** Interpolate in OKLab between
   these stops, t = 0 at the lowest height on screen, 1 at the highest:

   | t    | colour    | name                  | OKLab L (approx) |
   |------|-----------|-----------------------|------------------|
   | 0.00 | `#FFF1CF` | white-gold (best)     | 0.96 |
   | 0.10 | `#FFD59A` | gold                  | 0.89 |
   | 0.25 | `#F2B34A` | amber                 | 0.80 |
   | 0.45 | `#D9784A` | burnt coral           | 0.66 |
   | 0.65 | `#9E4A3E` | brick                 | 0.50 |
   | 0.85 | `#5A2E34` | wine                  | 0.36 |
   | 1.00 | `#33202A` | plum ash (worst)      | 0.28 |

   It stays in one temperature (mistakes aren't signed, so no cool), but the hue walks
   gold → amber → coral → brick → wine, so two heights differ in *colour*, not only brightness, and
   it still obeys "light means good / understood". Use a nonlinear t for the staircase: t =
   (h − hmin)^0.6 / (hmax − hmin)^0.6, which spends more of the ramp near the bottom where the
   interesting ledges are (the lowest ledge at 7.8% and its neighbours must look different).
   Use the same function for the bowl, with its own hmin/hmax (0.37 to 1.0), so the shelf at 1.0 is
   plum ash, the darkest thing on the sheet.
2. **Shape is light, raking and lower.** Move the sun down and to the side: elevation about 30°,
   azimuth across the ledges (for the staircase, roughly perpendicular to the dominant ledge
   direction; for the bowl, from upper left). Range the shade multiplier 0.30 (turned away) to 1.15
   (facing), clipped, so slopes read strongly. Ledge **risers** (the walls) get an extra ×0.55
   (so walls are always darker than treads: the eye reads "step" from dark wall under light tread).
3. **Lip highlights.** Along the top front edge of every riser taller than 0.5% of mistakes, draw a
   1.2 px line in the tread's own colour lightened 35% at 70% opacity. That single line is what makes
   terraces read as terraces at phone size. For risers of 5% or more (the 710-moment drop), 2 px.
4. **Contours.** Hairlines (1.2 px) at equal height steps, drawn on the surface: staircase every 5%
   of mistakes, bowl every 0.05 of squared miss; every fourth one 2 px and labelled at its right end
   ("20%", "40%"…; "0.6", "0.8"…) in 28 px parchment with halo. Colour `#1A0F0C` at 45% on lit
   ground (dark line), which survives video compression better than a light line. On the bowl,
   contours are the single best fix: concentric rings say "bowl" in a still frame, with no motion.
   On the staircase, contour lines coincide with the ledge edges, which is true and shows that the
   ground only changes at edges.
5. **Depth fog.** Blend each cell toward the background `#120A08` by 0% at the near edge to 30% at
   the far edge. Near/far then reads without motion, and the far edge stops competing.
6. **Silhouette.** Outline the sheet's outer edge (top rim and side walls' top edge) with a 1.4 px
   `#E6D5BC` line at 50%, so the sheet separates from the dark ground at every angle.
7. **Kill the moiré.** downhill-2 and staircase-2 show cell-stripe moiré and jagged edges. Render at
   2× and downsample (Lanczos), and on the smooth mesh shade per vertex (Gouraud) instead of per
   cell. ~0.4 s a frame at 1080p is still affordable.
8. **A height key.** A thin vertical bar (24 px wide, 40% of frame height) at the right edge with
   the ramp, a tick per labelled contour, "more mistakes" at the top, "fewer" at the bottom, in
   parchment. Appears in `staircase` with the lift, stays through the act; swaps its labels for
   squared miss in the bowl act.
9. **Floor shadow.** A soft contact shadow under the sheet (ellipse, `#000` at 50%, blur 40 px)
   so it sits on something and its tilt reads.

All of this is renderer work in `sheet.py`, used by every surface beat; I'd build it once and judge it
on staircase-outro and bowl-2 stills before touching the individual beats.

### 0.4 Camera, through this lens only

When the camera moves (Space and motion will propose orbits), legibility asks: keep the beat's one
point of interest in the middle third of the frame for the whole move; orbit no faster than 20° a
second; never let a label cross the point it names; re-place halos per frame (labels track their
points in screen space and stay horizontal).

---

## 1. too-many

**What reads today:** 2,048 tiny dots (about 2 px), dim; a few amber dots light up; the eye goes to the
counter "402 numbers read" at the bottom, which is the brightest, largest, steadiest thing. But the
line is about *borders to try multiplying*, and the counter counts *numbers read*. The eye lands on
the wrong quantity. (Flag for the Idea lens: the number on screen should be the borders, not the
inputs.)

**Proposal:**
- Memory grid filling 85% of frame width (64 × 32 cells, each about 26 px at 1080p), lit bytes as
  14 px amber discs with soft glow; unlit bytes 4 px ash at 25%. Today the grid is ~55% of width.
- Two counters, stacked at the right, both 0.9 scale roman: "numbers read: 2" (parchment, small,
  0.6 scale) and below it, large, "borders to try: …" in gold, which is the number the eye should
  follow. If the true count is astronomically large, show it as a written-out digit string that grows
  in length and runs off the frame, rather than an exponent (a lay viewer can't read 10^600 but can
  read "this number no longer fits on screen"). The digit count must be computed honestly from the
  same grid resolution as the two-number search.
- Start with the map's two axes (from `search-2`) shrinking into the first two lit bytes, so the
  "2" has a picture attached.

## 2. no-fast-way

**What reads today:** a tangle of ~1 px lines at about 15% opacity reads as scratch texture, not as
borders; the clock (the only bright crisp shape) takes the eye. Through this lens the clock is the
problem: the strongest contrast in the frame goes to a symbol.

**Proposal (whatever the Idea lens chooses):**
- Every border drawn must be drawn *on Mario's map*, so it is visibly a border (two sides, dots).
- Exactly one border bright at a time: 2 px `#FFD59A`, with its yes side washed amber at 10% and no
  side blue at 10%; its mistake count beside it, 0.9 scale. Already-tried borders fall back to
  1 px ash at 8% (they accumulate into a haze that says "many").
- The tally of tried / remaining in the corner, gold for tried, ash for remaining; no clock.
- If it shows the search as time (e.g. "at a billion borders a second, this would take…"), the
  duration is a large number in parchment, written out, not a dial.

## 3. exam

**What reads today:** two lists of italic text, small (~16 px at 1080p), the "learned from" list
mid-grey, the exam list slightly warmer; nothing to look at. The eye has nowhere to go.

**Proposal:**
- The six recordings as six tapes (`records.py`), each with a looping thumbnail of its real footage
  (screen.py averaged down, 240 px wide), arranged left (1, 2, 4, 5) and right (3, 6), with a
  hairline gap between the groups. Labels under each tape, 0.6 scale parchment.
- When "never learned from" lands: the training tapes dim to 45% and the exam tapes gain a 2 px gold
  outline and their glow; that's where the eye goes.
- Then "scored it": the moments of recordings 3 and 6 pour out as dots onto Mario's map (hollow
  rings, 8 px, so they read as *different* data from the training's filled dots), the current
  border is laid across, right answers flash gold, wrong ones blue, and a large score (0.9 scale,
  roman, e.g. "86%") counts up beside the map. Real data: test moments and a pass's weights exist.
- Clutter guard: thumbnails stop looping (freeze) when the dots pour out, so only one thing moves.

## 4. downhill-2

**What reads today:** a mid-brown curved sheet with vertical moiré stripes; the bright band runs
along the trough but there's no sense of a valley floor in depth; the coral arrow is horizontal on
screen, so it doesn't look downhill, and its label (coral on gold) is unreadable. "speed knob, into
the picture" is dim and far from the axis.

**Proposal:**
- Apply 0.3 (ramp, raking light, contours every 0.05, fog, supersampling). The 15:1 trough then
  shows as long thin contour ellipses, which is the true shape and is readable even in a still.
- The added-up nudge: an arrow *drawn on the surface* (projected, following the ground), starting at
  a dot on the valley wall and pointing perpendicular to the contours, in `#FFF4E2` 3 px with a 2 px
  dark halo, head 18 px. Its direction is downhill in screen space because the contours make
  downhill visible. Label off-surface: "added-up nudge" 0.6 scale parchment, leader line to the
  arrow's tail.
- Optional, strong: a top-down inset (25% of frame, bottom right) of the same contours with the
  arrow, so the "always perpendicular, always downhill" relation is legible in 2D where perspective
  can't distort it.
- Axis labels horizontal, 0.6 scale, placed at the ends of the sheet's two edges, each with a small
  arrow along its edge: "position knob →" along the near edge, "speed knob ↗" along the side edge.

## 5. balance

**What reads today:** mostly legible. Clean bars, warm pulls right, cool pulls left, "= 0.000" per
knob. Weaknesses: the three rows have equal weight so the eye doesn't know which to read; the
negative numbers sit under the label, separate from their bar; numbers are ~16 px at 1080p; the top
caption is redundant with narration.

**Proposal (small changes; the frame basically works):**
- Make the position row the hero: bar 10 px thick, numbers 0.9 scale; the speed and bias rows
  6 px and 0.6 scale at 60% opacity. The narration's "+0.766 vs −0.766" lives in the hero row.
- Put "−0.766" at the left end of its bar, level with it, mirroring "+0.766" on the right.
- Build each bar from its moments: thousands of hairline segments (one per moment's pull, real data)
  flying in and stacking, warm onto the right, cool onto the left, so "pulls from every moment" is
  visible, then the stack resolves into the solid bar. At the end, the two halves of the hero bar
  slide together and annihilate into a single bright "0.000" at the centre line (gold, 0.9 scale).
- Drop the top caption.

## 6. too-big

**What reads today:** a clean U, but it only fills the left 60% of the frame, ticks are ~14 px, and
the dot is shown mid-flight above the curve (15.6 s still), which reads as a mistake. Nothing on
screen records how high each landing was, so "lands higher up" has to be remembered, not seen.

**Proposal (follows Nate's note):**
- Chart fills 80% of frame width; axis labels 0.6 scale, ticks 0.5.
- Each move in three steps, each held long enough to read (0.5 s each):
  1. A horizontal arrow from the current dot, along the knob axis, length = the real step,
     coral `#EE8660`, 3 px, head 16 px, labelled once (first move only) "the step".
  2. The arrow's tip drops/rises as a dotted 1.4 px plumb line to the curve.
  3. The dot appears there; the previous dot becomes a ring at 50% opacity, fading to 15% over the
     next three moves (a visible trail of landings, alternating sides).
- A short horizontal tick on the y-axis at each landing's height, in gold, staying; the ticks
  climb, so "higher each time" is a column of marks.
- Contrast foil (one move, first): the step that settled, drawn in ash, crossing the bottom and
  landing lower, then fading, so the too-big one is compared against it.
- If the height grows past the chart, rescale the y-axis visibly (ticks slide and relabel), never
  clip; the 137× must be a number on screen at the end, 0.9 scale.

## 7. map-intro and 8. map-axes

**What reads today:** a filled brown rectangle (`~#4A3A2E`) is the largest bright area in the frame,
so it steals the eye from the map for two beats while saying nothing. The map itself is small (30%
of width), the ruler is a plain line with no visible yes/no sides, so "its yes and no sides have
swapped" can't be seen.

**Proposal:**
- map-intro: no filled sheet yet. The map takes 55% of frame width, centred. The word "a sheet of
  borders" can be just the empty *outline* of the sheet (1.2 px parchment at 35%, dashed), slid in at
  the end of the line, smaller than the map, so it reads as "to come", not "look here".
- map-axes: the ruler gets sides. Wash the yes side amber `#F2B34A` at 12% and the no side blue
  `#4F8FB8` at 12%, clipped to the map; small "yes" and "no" tags (0.6 scale, in their colours) ride
  on either side of the ruler near its pivot. At half a turn the washes visibly swap colours. The
  pivot (the map's middle, 2.5 blocks, 1.25) is a 12 px ring.
- The ruler: 2.4 px `#FFF4E2` with a dark halo, so it reads over dense dots.
- If the sheet outline stays on screen in map-axes, give it only a moving cursor: a 1.2 px vertical
  hairline that sweeps across the outline as the ruler turns (wrapping at the edges) and a horizontal
  one that rises as it slides. Nothing inside it until the `sheet` beat. (Continuity lens may prefer
  to bring the sheet in only at `sheet`; through this lens, either is fine as long as nothing large
  and blank sits beside the map.)

## 9. staircase

**What reads today:** the sheet rises seen almost from above; the brightness ramp is the only cue,
so it reads as a soft sinusoidal smear, not raised ground. Rotated side label; labels ~16 px.

**Proposal:**
- Apply 0.3 in full: the ramp, raking light, lip highlights, fog, silhouette, floor shadow, and the
  height key appearing as the ground lifts (its fill sweeping up with the lift).
- The lift itself: colour should arrive *with* height (t ramps 0 → final as height lifts), so flat =
  one colour, raised = coloured; today the flat sheet is already shaded before it rises.
- On "Lower is better": the key's bottom tick pulses gold and the lowest ledge's cells glow; one
  label "lower is better" at the key, not on the surface.
- The map on the left can shrink to 35% and dim to 60% once the sheet starts to rise; the eye
  should be on the sheet.

## 10. staircase-2

**What reads today:** closest view, ledges show only as faint dark ticks; the walls aren't
distinguishable from treads; "the lowest ledge" label is white on gold and illegible; edges jagged.

**Proposal:**
- Raking light from the side across the ledge direction, risers ×0.55 dark, lip highlights: each
  terrace becomes a light band ending in a bright lip over a dark wall.
- A cross-section: a vertical cut through the sheet along one angle row (through the lowest ledge),
  drawn as a 2 px gold profile line on a dark cut face, shown in the foreground. A staircase profile
  is the clearest single picture of "flat, then a sudden drop". Real data: one row of the 720×601
  heights.
- Lowest ledge: outline its whole tread (not just a dot) with a 2 px gold hairline, label off-surface
  with leader: "lowest ledge" 0.6 scale parchment, halo.
- Supersample 2× to remove jagged ledge edges (jaggies look like extra fake steps).

## 11. no-slope

**What reads today:** a tiny marker with tiny arrows; Nate's note is right: the arrows look wider
than the ledge, which says "these arrows leave the ledge", the opposite of the line.

**Proposal:**
- Zoom so the marker's ledge fills at least 30% of frame width. Outline that tread in 2 px gold; dim
  every other cell to 50%.
- Arrows: 6–8 directions, each ending 85% of the way to the tread's edge in its direction (lengths
  differ, computed from the real ledge shape), 2 px `#FFF4E2`, each tipped with a small "0" (0.5
  scale): height change zero. "level, every way" as the single label, off-surface.
- "You find it by falling off": one arrow extends past the edge, the marker slides off the lip and
  drops down the riser; a vertical plumb line on the riser shows the drop with its size ("−0.3%"
  or whatever the real height change is) in gold, 0.9 scale. The arrow that found the drop turns
  gold; the others stay white. The one moment of motion is the one moment of information.
- On "can't walk down… to the lowest ledge": pull back, the lowest ledge's gold outline appears far
  off with the marker's ledge between, several ledges apart.

## 12. staircase-outro

**What reads today:** everything in this frame is 30–60% brown. The two named dots are ~6 px, close
together, the square trick in teal (a cool, wrong by the spec). The sinusoidal band reads as a
glowing smear with a darker smear beside it; there's no sense of ledges or depth.

**Proposal:**
- Apply 0.3 in full; at this wide view, contours every 5% of mistakes are what show the landscape,
  with every fourth labelled.
- Markers per 0.2: lowest ledge gold, square trick coral, 16 px, rings. Labels off-surface with
  leaders, placed on opposite sides (lowest ledge label up-left, square trick down-right) so they
  never collide.
- A dashed 1.4 px line between the two dots, with the true mistake gap written on it ("91% vs 88%"
  style, using the real numbers), so "somewhere off its lowest ledge" has a measured distance.
- On "where does the square trick's slope lead, on this sheet?": everything except the coral dot
  dims to 60%, leading the eye into the bowl act. On "And what about the perceptron?", nothing new
  appears (no marker exists yet): don't add one here unless there's data to place.

## 13. shelf-2

**What reads today:** a curved bowl on the left, a flat lighter-brown block on the right, and a long
dim caption at the top. Because the flat shelf faces the light, it's *brighter* than the bowl's high
walls, so the highest, worst borders look like the "good" ground. The viewer can't tell the shelf is
high, or that it's one height.

**Proposal:**
- Ramp per 0.3 with the bowl's own range: the shelf at 1.0 is plum ash `#33202A`, the darkest ground
  on the sheet; the bowl runs from white-gold at its bottom up to wine at its rim where it meets the
  shelf. The bowl's rim and the shelf now meet in nearly the same colour, which is true (the bowl
  rises *toward* 1.0) and shows the shelf as the plateau at the top.
- A low camera (pitch down to ~25° above the sheet) for this beat, so height is visible as height:
  the shelf's front edge stands as a wall above the bowl's floor.
- On "all those borders sit at the same height": a single horizontal plane-line at height 1.0 sweeps
  across from the height key ("1.0" tick, 0.9 scale) and the shelf's whole edge lights with a 2 px
  gold rim; the bowl dims to 60%. One label on the shelf: "every one: exactly 1" (0.6 scale, halo).
- On "where the ruler mostly faces the wrong way": a small inset of Mario's map (25% of frame,
  top right) with a ruler from a shelf spot, its yes side (amber wash) over mostly blue dots; a gold
  dot on the shelf marks which border it is, joined to the inset by a leader. Real data: any shelf
  (angle, offset), and the 82% figure.
- Remove the top caption.

---

## Lines that are themselves the legibility problem

- `too-many`: the picture is counting "numbers read" because the line's countable is buried; the
  line is fine, but the beat's on-screen number should be the borders (Idea lens).
- `shelf-2`: "Then every total is zero" refers back to the previous beat's "shrinking to nothing";
  on screen, nothing shows a total, so the first clause has no picture to attach to unless the shelf
  inset (above) shows the border's total going to zero. Not a narration change, just a note that the
  picture must carry "total zero → miss exactly one" or the line won't land.
