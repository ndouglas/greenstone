# Picture council 1, merged (video 002)

Nate flagged 13 beats after watching the first full draft (2026-11-02):
too-many, no-fast-way, exam, downhill-2, balance, too-big, map-intro,
map-axes, staircase, staircase-2, no-slope, staircase-outro, shelf-2.
Six lenses proposed designs (files beside this one). This is the merge:
what gets built, in what order. Nate chooses by watching prototypes.

## First: what the current pictures get wrong (Honesty, with Legibility)

These are errors, fixed whatever the design:

- **The sheet's floor isn't zero.** Heights are rescaled so the lowest
  ground sits at zero: the best border (838 mistakes, 7.8%) and the
  bowl's bottom (squared miss 0.366) look perfect. Draw the true zero
  and a height key with real values.
- **Zooming restretches the height** (staircase-2 and no-slope are 2.3×
  taller than the full sheet). One height scale per chapter.
- **The coarse grid invents and loses ledges.** Near the bottom, the
  real ledges are narrower than a sample (only 5–8% of neighbouring
  samples share a height), so drawing each sample as a flat tile makes
  terraces one sample wide; no-slope's marker has no level neighbour,
  so "level, every way" is false as drawn. Close views need the ledges
  computed finely (per angle, sort the moments once; every offset's
  count is then a lookup, so a 20× finer window is cheap).
- **Colour mixes height with lighting**, so the shelf (the highest,
  worst ground) faces the light and looks best, and the landscape reads
  as one brown. Height becomes a perceptually even ramp (gold low to
  plum-ash high), lighting only shapes walls and slopes, contours at
  round true values.
- **map-axes never marks yes and no**, so the half-turn swap is
  invisible.
- **balance shows per-moment averages labelled "added up"** (the real
  sums on the position knob are ±8,207; they still cancel).
- **Unlabelled made-up marks:** too-many's 400 random memory cells,
  no-fast-way's 70 random lines and clock, too-big's ball arc (mid-hop
  it shows misses that never happened).
- **The square trick's marker is teal** (cool means no); it becomes
  coral.
- Labels and markers are about half the size a phone needs.

## The designs

**A sheet made of real borders** (The idea, Continuity, Space and
motion, Scale, all independently). The sheet of borders is first shown
as what it is: a grid of tiny Mario maps, each with one real border,
turned across a row (yes and no sides washed warm and cool, so the swap
at half a turn shows) and slid down a column. map-intro deals it out
from the map's own border; in map-axes the tile matching the ruler
lights as it turns and slides; in sheet the tiles melt into the
continuous sheet. It's the same object the staircase then raises.

**too-many and no-fast-way: the size of the search, felt** (Scale,
The idea, Honesty). The search tried 71,640 borders (360 directions ×
199 cuts, checked) in about 1.5 seconds. Each extra number adds a dial,
and the same search, extended the obvious way, tries 180 times as many:
71,640 × 180^(n − 2), labelled as that procedure's count. too-many: the
sheet of tiles copies itself into a stack, a row of stacks, a block,
as memory cells light one at a time (the real bytes, in address order),
until a tile is smaller than a pixel; then one grain of sand per border,
with the camera pulling back past a bucket (3 numbers), a house (5),
pyramids (7), lakes (9), Earth (14), the Sun (16), to the observable
universe overfilled at 40 numbers (2% of the memory). no-fast-way: the
same counts as time on a ribbon of familiar spans (a minute, a
lifetime, written history, the age of the universe, passed at 10
numbers, 3.8 times over), honest that two numbers were easy (the
picture starts there) and that what explodes is adding numbers. A
fallback count beyond argument: the number of genuinely different
splits of Mario's 1,313 points (Cover, 1965), 1.72 million at two
numbers, ×437, ×327, ×262 per number (checked).

**exam: the data itself** (Scale, Continuity, Legibility). The six
recordings as tapes; 1, 2, 4 and 5 pour their moments into the
learning map, 3 and 6 into a second, exam map behind glass. After a
pass, the border is copied across, the exam's wrong moments ring, and
the score becomes the first point of one-run's chart.

**downhill-2, balance, too-big: one landscape** (Continuity, Space and
motion, The idea). The U tilts into 3D and the speed knob sweeps it out
slice by slice into the valley (both axes the same units; on screen:
"bias held still"), with contour lines. Real per-moment nudges, a spray
of arrows, add tip to tail into one arrow that crosses the contours
downhill, at three spots ("always"). balance returns to the bottom:
every moment's real pull chains tip to tail and ends where it began,
then straightens into a tug of war whose two ropes are the real sums
(±8,207 on the position knob), net 0.000. too-big (Nate's design):
back on the U, the move is an arrow along the knob axis, step × the
added-up nudge, then a plumb line to the curve, landing higher on the
far side than it started (a dashed line at the starting height), a
trail of landings, the miss's ticks climbing the axis; the settled
step's arrow ghosted beside it for comparison. Check first that the
one-knob run at twice the step diverges (rate × mean of position² is
2.22 > 2, so it should).

**The staircase sequence: one flight** (Space and motion, Continuity,
Legibility, Honesty). On the new renderer (free orbit camera, true
heights and key, even colour ramp, contours, ledge lips, fog, 2×
supersampling): staircase rises while the camera orbits; staircase-2
cuts a slice through the lowest ledge and a bead walks its profile in
step with the ruler on an inset map (level between dots, a step of
exactly that dot's moments at a crossing), then a low fly-over at fine
resolution; no-slope stands a marker on a real ledge with feelers
shorter than the ledge, each reading 0, then steps off (once up, once
down); staircase-outro orbits out over the whole sheet, lowest ledge
gold and the square trick coral, the gap labelled, ending on bowl's
camera so bowl changes only the heights. shelf-2: a low camera; the
shelf a dark high plane at exactly 1; a slice sweeps across showing the
bowl's profile changing while the shelf's stays flat; an inset ruler
facing the wrong way shrinks to nothing.

## Lines flagged (not changed; Nate decides)

- **downhill-2:** "Turn them all at once" (three knobs); the valley
  turns two, the bias held. An on-screen label is enough (two lenses).
- **too-many:** "only works easily with two numbers": three numbers
  take 4.5 minutes, which some would call easy. Soft, not false.
- **target-over and why-short:** "by plenty" (Nate: idiomatic).
  Proposed: "with room to spare".
- **balance:** "add up to exactly nothing": the data's net is 0.000 to
  three places; the screen says "= 0.000".

## Order of work

1. The renderer: true heights and key, one scale per chapter, fine
   ledges for close views, even colour ramp, contours, orbit camera.
2. The tiled sheet piece.
3. Prototypes, rendered and shown to Nate: the staircase sequence;
   too-many and no-fast-way; downhill-2 to too-big.
4. exam, shelf-2, and the rest of the honesty fixes (labels, sizes,
   coral marker).
