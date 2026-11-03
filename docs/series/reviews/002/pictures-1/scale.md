# Picture council, video 002: the Scale and intuition lens

Lens question: where a line is about size (how many, how fast, how far),
does the picture make the size *felt*, by comparison with something the
viewer already knows, rather than just stated? Most of my effort is on
**too-many** and **no-fast-way**, which are the beats about size. On the
others I only propose changes where size is part of the idea (a 137x
growth, a 710-moment cliff, a 3-in-100 gap, a sheet of 432,720 borders).
Where my lens has nothing to add, I say so.

---

## 0. The arithmetic (derive every on-screen count from here)

Everything below was checked in Python on 2026-11-02 (scratch scripts:
`/private/tmp/claude-501/-Users-nathan-Projects-ndouglas-greenstone/cf099f6b-6b2a-4507-8759-c42c3f7c29f4/scratchpad/t.py`
and `.../t2.py`). The builder should move these into `exp002.py` so they
land in `numbers.json` and are checked like every other number.

### 0.1 What the search actually did

`search` uses `exp001.best_line`: **360 directions** (one degree apart,
a full turn) x **199 cuts** along each = **71,640 borders**, each scored
against all **10,720** training moments (768 million checks).

Timed on this laptop with the same code on 10,720 moments: **1.48 s**,
or about 48,000 borders a second (4.1 ms per direction). The data values
don't change the cost, but the builder should time it on the real data
inside `exp002.py` and store it (`search_seconds`), and say on screen
"on this laptop" or "on one computer".

### 0.2 What "each extra number multiplies the borders" means, as a count

With two numbers, a border's direction is one dial (the angle), turned a
degree at a time through a full turn: 360. With three numbers, the
direction needs a second dial; turned the same way, a degree at a time,
through half a turn (that's all it needs): 180 settings. Every extra
number adds one more such dial. So **the same search, extended the
obvious way, tries**

    borders(n numbers) = 71,640 x 180^(n - 2)

That's an exact count of a procedure (what a grid search at the
search's own fineness would try), not a claim about how many
*different* borders exist. Honesty caveat for the builder: an evenly
spaced set of directions one degree apart is smaller (in 3 numbers about
41,253 directions rather than 64,800), so if the Honesty lens wants a
gentler multiplier, every ladder below slides by about one rung; the
conclusions don't change. A floor that is beyond argument is in 0.5.

| numbers read | borders tried | time at the search's speed | one 1 mm sand grain per border |
|---|---|---|---|
| 2 (the search) | 71,640 | 1.5 s | 72 ml: a handful |
| 3 | 12.9 million | 4.5 minutes | 12.9 litres: a bucket |
| 4 | 2.32 billion | 13 hours | 2.3 m^3: a cube 1.3 m a side |
| 5 | 418 billion | 100 days | 418 m^3: about a house |
| 6 | 7.5 x 10^13 | 49 years | 75,000 m^3: 30 Olympic pools |
| 7 | 1.35 x 10^16 | 8,900 years (longer than writing has existed) | 13.5 million m^3: 5 Great Pyramids |
| 8 | 2.4 x 10^18 | 1.6 million years | 2.4 km^3: a third of Loch Ness |
| 9 | 4.4 x 10^20 | 290 million years | 440 km^3: about Lake Erie (484) |
| 10 | 7.9 x 10^22 | 52 billion years: 3.8 x the age of the universe | 79,000 km^3: 3 Lake Baikals |
| 12 | 2.6 x 10^27 | | twice all Earth's oceans (1.34 billion km^3) |
| 14 | 8.3 x 10^31 | | 77 Earths |
| 16 | 2.7 x 10^36 | | 1.9 Suns |
| **40** | 2.7 x 10^89 | | **overfills the observable universe** (about 3.6 x 10^80 m^3 = 3.6 x 10^89 mm^3) |
| 2,048 | 71,640 x 180^2046 | | a number **4,620 digits long** |

Digits: log10(71,640) + 2046 x log10(180) = 4.855 + 4,614.3 = 4,619.2,
so 4,620 digits. Reference volumes (public figures, to put in
`citations.toml`): Olympic pool 2,500 m^3; Great Pyramid ~2.6 million
m^3; Loch Ness ~7.4 km^3; Lake Erie ~484 km^3; Lake Baikal ~23,600 km^3;
oceans ~1.335 x 10^9 km^3; Earth 1.083 x 10^12 km^3; Sun 1.41 x 10^18
km^3; observable universe ~3.6 x 10^80 m^3; age of universe 13.8
billion years.

**Headline facts for the viewer:**
- At **ten numbers** (half a percent of the console's memory) the
  search we did would outlast the universe almost four times over.
- At **forty numbers** (two percent of the memory) one grain of sand
  per border would overfill the observable universe.

### 0.3 The genuinely different borders, and why "no fast way"

Many grid borders split the moments identically. The number of
*genuinely different* ways a straight border can split N points in n
numbers is at most 2 x (C(N-1,0) + C(N-1,1) + ... + C(N-1,n)) (Cover,
1965; exact when no three points line up, which ours do, so it's a
ceiling). With Mario's **1,313** distinct map points:

| numbers | at most this many different splits | x per extra number |
|---|---|---|
| 2 | 1,722,658 | |
| 3 | 752,803,298 | x437 |
| 4 | 2.47 x 10^11 | x327 |
| 5 | 6.45 x 10^13 | x262 |

The exact methods that are known, at best, step through splits like
these: their work multiplies with each number (roughly N^n). Finding the
fewest-mistakes border in general is NP-hard (Johnson and Preparata 1978,
"densest hemisphere"; Hoffgen, Simon and Van Horn 1995), which is what
"no known method finds it quickly" means. The builder can compute the
exact count for 2 numbers from the 1,313 points (a rotating sweep), so
the first rung is exact, not a ceiling.

### 0.4 What "quickly" looks like: the square trick

One added-up move of the square trick costs 10,720 x (n + 1) multiply-adds:
it grows by *adding* one row per number, not by multiplying. With all
2,048 numbers, 2,000 moves are about 4.4 x 10^10 multiply-adds: seconds
on a laptop. The builder should **time it for real**: the frame-exact
footage session has all 2,048 RAM bytes every frame, so the square trick
can literally be run on the console's whole memory (or, failing that, on
random 2,048-wide inputs of 10,720 rows; the cost is the same). Store
`square_seconds_2` and `square_seconds_2048`.

### 0.5 The floor, if the Honesty lens wants it

Give each of the 2,048 knobs only two settings, for or against, all the
same strength: 2^2048 different directions, a **617-digit number**. At
the search's real 4.1 ms per direction, that crude search passes one
second at 8 numbers, a day at 25, a year at 33, a lifetime (80 years) at
40, the pyramids (4,500 years) at 45, and **the age of the universe at
67 numbers**. This is the bound nobody can argue with, and 617 digits fit
on one screen (25 lines of 25), legibly.

---

## too-many (the main event; about 17 s)

> "Trying borders like that only works easily with two numbers; each
> extra number multiplies the borders to try. A neuron reading more of
> the two thousand and forty-eight numbers in the console's memory would
> have far too many borders to try."

Current picture: the memory grid with cells lighting and a "N numbers
read" counter. It shows *how many numbers*, a symbol of input size; it
never shows *how many borders*, which is what the line is about. The
count goes up linearly (2, 55, 204, 402) while the thing it describes
multiplies, so it actually under-sells the line.

### Proposal A (ranked first): "One grain per border", a powers-of-ten pull-back

The idea: make every border the search tried a visible thing, a grain,
and let each extra number multiply the pile while the camera pulls back
past objects the viewer knows. The memory grid stays as the input
counter along the bottom, so the 2,048 is always in view.

- **0.0-1.5 s** ("Trying borders like that..."): the last search
  borders on the map (carried over from `search`) collapse into a
  single bright grain each. The real 71,640 tries pour into a small
  heap: render all 71,640 dots, a 268 x 268 block of 1 px dots on the
  right half, filling in at the search's real order (direction by
  direction, each column of 199 cuts lighting together). Label: *the
  71,640 borders the search tried*. Beside it, at true scale, a cupped
  hand silhouette: *one grain of sand each: a handful*.
- **1.5-3.0 s** ("...only works easily with two numbers;"): bottom
  strip: the console memory grid (MemoryGrid, 64 x 32), two cells lit
  ($0755 position, $0057 speed), counter *2 numbers read*. A small
  clock-free readout: *1.5 seconds on this laptop* (the measured
  time, see 0.1).
- **3.0-6.5 s** ("each extra number multiplies the borders to try."):
  a third memory cell lights. The heap is copied 180 times, the copies
  fanning out in a row and stacking into a block (show the 180 really:
  the eye can count rows of 10); the camera pulls back x5.6 (the cube
  root of 180, since volume grows by 180) and the hand shrinks to a
  speck beside a bucket. Labels: *x180*, *12.9 million: a bucket*.
  Fourth cell: x180 again, pull back again, *a cube taller than you*.
  The rhythm "light a cell, x180, pull back, name the thing" is set
  three times slowly so the viewer learns it.
- **6.5-11 s** ("A neuron reading more of the two thousand and
  forty-eight numbers..."): the rhythm speeds up, one cell every ~0.4 s,
  reference silhouettes at true relative size sliding through the
  frame as the camera recedes (house, 30 Olympic pools, 5 pyramids,
  Loch Ness, Lake Erie, Earth's oceans as a sphere of water beside the
  Earth, the Earth, the Sun). Each silhouette fades as the next arrives
  so only one reference is ever on screen. A small italic running
  caption keeps the time at the search's speed: *4.5 minutes, 13
  hours, 100 days, 49 years, 8,900 years...*; at the tenth lit cell the
  caption reads *longer than the universe has existed* and stays.
- **11-14 s** ("...would have far too many..."): at the **fortieth**
  lit cell, the pile is drawn as a sphere the size of the observable
  universe, and it overfills it (its edge passes the frame). Caption:
  *40 numbers read: more grains than fit in the observable universe*.
  The memory grid shows 40 of 2,048 cells lit: the viewer sees how
  little of the memory that is (2%).
- **14-17 s** ("...borders to try."): the remaining 2,008 cells flood on
  in a wave across the grid (about 2 s), and the whole frame dissolves
  into the digits of the actual count, 71,640 x 180^2046, 4,620 digits
  in small Charter numerals, scrolling upward as a wall. Caption:
  *all 2,048: a number 4,620 digits long*. (Optional: the 617-digit
  floor from 0.5 instead; it fits one screen without scrolling.)

Why it teaches better: "multiplies" is shown as copies of a pile, not a
counter; the size is pinned to objects everyone has handled (a handful
of sand, a bucket, a pool, a lake, the Earth), and the punch comes from
the gap between how few memory cells are lit and how absurd the pile
already is. Nothing is invented: the first heap is the real 71,640, each
multiplier is the derivation in 0.2, every reference volume is at true
relative scale and cited.

Data: 0.1's real search order and time; the ladder in 0.2; reference
volumes; the exact integer for the digit wall (Python big ints:
`sys.set_int_max_str_digits(0)`).

Build: hard but bounded. A log-scale camera (one zoom tracker, every
object drawn at size x/zoom, culled when under a pixel or over the
frame); silhouettes as simple SVGs (hand, bucket, person for scale,
house, pool, pyramid, lake outline, spheres for Earth/Sun/universe); a
dot-block renderer for the first two rungs (71,640 and 12.9 million
can't both be drawn as dots, so from rung 2 on the block is a solid with
a fine dot texture, which is honest because the camera is too far back
to resolve grains). The memory strip is the existing MemoryGrid. About
2-3 days.

Risk: lots happening for a lay viewer. Mitigation: one reference object
at a time; only three labels ever on screen (the count, the object, the
cells lit).

### Proposal B: the time ladder, on the memory grid

Same rhythm but the unit is time, not volume: keep the current memory
grid (it already exists and Nate's seen it), and beside it a vertical
time ribbon that climbs as each cell lights, with tick labels a viewer
knows (*a minute, an hour, a day, a year, a lifetime, since the
pyramids, since the dinosaurs, the age of the universe*), the camera
riding up the ribbon. Cell 3 lands on 4.5 minutes, cell 6 on 49 years,
cell 7 past written history, cell 10 past the universe. Then the
remaining cells flood, and the ribbon runs off the top of the screen.

Easier (1 day: a ribbon with log ticks and a camera tilt), and very
readable, but I'd save time for no-fast-way (below), so the two beats
don't feel the same; if no-fast-way gets A-style instead, swap.

### Proposal C: the sheet multiplies (if the sheet is introduced earlier)

If continuity moves the "sheet of borders" earlier, too-many could
multiply *the sheet*: the 2D sheet becomes a stack of 180 sheets (a
block) for 3 numbers, a row of 180 blocks for 4, and so on, the camera
retreating. Elegant, but the sheet isn't introduced until map-intro, 20
minutes later, so it would show the sheet before it's explained. Not
recommended here; worth a callback in map-intro (see there).

### On the line itself (not to change, just to flag)

"Multiplies the borders to try" is right. "Only works easily with two
numbers" is soft: with 3 numbers the same search takes 4.5 minutes,
which is still easy. If Nate wants the picture and words to agree
exactly, "only works easily with a few numbers" would; otherwise the
picture should not linger on rung 3 as if it were already hopeless (its
caption should read calmly: *4.5 minutes*).

---

## no-fast-way (about 7 s)

> "And in general, no known method finds the border with the fewest
> mistakes quickly."

Current: a clock beside a tangle of random lines. A symbol of "slow",
and the lines are made up, which breaks the "nothing that isn't true"
rule. Nate: cheap.

The line's size claim is subtler than too-many's: not "brute force is
slow" (already shown), but that *even the clever methods* grow by
multiplying, while "quickly" means growing by adding. The scale picture
should put the two kinds of growth side by side, in time, on the same
axis, with a quick method that the viewer is about to meet: the square
trick.

### Proposal A (ranked first): the race up the time ribbon

- **0.0-1.0 s** ("And in general,"): too-many's final frame (the lit
  memory grid) slides down to become the floor; above it, a vertical
  ribbon of time with familiar ticks (*a second, a minute, an hour, a
  day, a year, a lifetime, all of written history, the age of the
  universe*), log-spaced but labelled only in words. Two markers sit at
  the bottom: warm *the square trick* and cool *the best known exact
  method* (cool because it's the no; warm is the one that works, which
  is the next beat's hero).
- **1.0-5.0 s** ("no known method finds the border with the fewest
  mistakes"): a counter of numbers read climbs, 2, 3, 4, 5, ... 2,048,
  memory cells lighting along the floor. At each count, both markers
  rise to their real time:
  - the square trick creeps: from 2 to 2,048 numbers it rises only from
    a blink to *a few seconds* (measured, 0.4), barely leaving the
    floor. Its trail is a gentle straight ramp.
  - the exact method leaps: its marker multiplies at each step (x437,
    x327, x262 in splits, 0.3; or the x180 search time, 0.2), passing
    *an hour* at 4 numbers, *a lifetime* around 6, *written history* at
    7, *the age of the universe* at 10, and leaves the top of the frame
    while the counter has barely started. The camera tries to follow it
    up for half a second, gives up, and returns to the floor.
- **5.0-7.0 s** ("...quickly."): with the counter still running to
  2,048, the square trick's marker arrives at the top of its short ramp
  and gives a small warm pulse: *seconds*. The exact method's trail is
  a vertical line out of frame, labelled where it left: *past the age of
  the universe at 10 numbers*. Hold.

Why: "quickly" becomes something you can see, a ramp versus a cliff, and
it hands off to `learn` ("So this time, the neuron finds its own
border"): the warm marker is the method the video is about to use. Every
height is a measured or derived time.

Data: `search_seconds` (0.1), the ladder (0.2 or 0.3 at the measured
rate), `square_seconds` measured at several n up to 2,048 (0.4; ideally
on the footage session's real RAM). Which exact method to time: the
honest choice is "trying every different split" (0.3) at the search's
measured per-border speed; label it *the best known exact methods* only
if the Honesty lens accepts that they are N^n in the worst case
(they are, by the NP-hardness results; I'd say *known exact methods*
and leave out "best").

Build: moderate (1 day): a log ribbon, two markers with trails, a
camera tilt, the memory floor reused.

Caution: the square trick doesn't find the fewest-mistakes border; it
finds its own (the video says so later). Its marker must be labelled
*the square trick: quick, but aims at something else* or simply *a rule
that learns*, so the picture doesn't claim it solves the same problem.
If that is too much to say in 7 s, use Proposal B.

### Proposal B: the splits, counted honestly, on the map

Stay on Mario's map. Show what an exact method would actually have to
step through: every genuinely different way a straight border can split
the 1,313 points. With 2 numbers: at most 1,722,658 (0.3; exact count
computable). Draw them as hairlines sweeping the map at the real rate
(48,000 a second: a blur that fills the map with light in about 36 s;
shown 30x sped up, labelled *30x*). Then a third axis rises off the map
(speed and position joined by a third number, drawn as depth), and the
count line reads *x437: 753 million*; the sweep that took a second now
needs 7 minutes at the same speed-up; fourth axis, *x327*, 1.5 days...
The blur stops being able to keep up. Truthful and on the established
map, but harder to read than A, and "in general" is less felt.

### Proposal C: the 617-digit floor

One still screen: the exact integer 2^2048, all 617 digits, set as a
block. *Even if every number could only count for or against: this many
borders.* Beneath, *at the search's speed: 10^596 lifetimes of the
universe*. Simple, unarguable, but it's a stated number, not a felt
one; best used as the closing frame of too-many A rather than as its own
beat.

---

## exam

> "After every pass, we scored it on recordings it never learned from,
> like an exam it hasn't seen."

Current: two lists of recording names. Nate asks to show the data.

Scale proposal: make the sizes of the two sets visible and comparable
in a unit everyone knows, play time.
- **0-2 s**: the six recordings as six horizontal film strips (thumbnails
  from the frame-exact footage where it exists; otherwise a strip of the
  recording's own moments as ticks), each strip's length proportional to
  its number of moments. Four are warm-lit and stack on the left as
  *learned from*; recordings 3 and 6 slide right as *the exam*.
- **2-5 s**: each pile is labelled in moments and in seconds of play:
  *10,720 moments: about 3 minutes of play* and *4,883 moments: about 81
  seconds* (moments / 60.1 frames a second, if each moment is one frame;
  the builder must confirm `scroll_moments` keeps one moment per frame;
  if not, drop the seconds and keep the counts). The exam strip is
  visibly a bit under half the length of the training stack.
- **5-8 s** ("after every pass, we scored it"): the exam's 4,883 moments
  pour onto Mario's map as their points; the perceptron's border after
  pass 1 is drawn, each exam point flashes warm if right, cool if wrong,
  and a score bar fills to the real pass-1 exam score; then pass 2's
  border, re-score; three passes quickly, setting up `one-run`'s jumping
  score.

Data: per-recording moment counts, exam points, the per-pass weights and
exam scores (exist). Build: easy-moderate (pointmap + records.py).

---

## downhill-2

> "Turn them all at once and it's a valley. Added up over all the
> moments, the square trick's nudges always point downhill in it..."

Through my lens, two sizes matter: *how many* nudges are added up, and
*how lopsided* the valley is (position counts ~15x more than speed).
- The added-up arrow should be shown to be made of 10,720 tiny arrows:
  at a point on the valley, each moment's nudge (miss x its numbers)
  appears as a faint hairline arrow, warm and cool pointing every which
  way; they slide tip to tail into a chain that wanders and ends at one
  bright arrow pointing downhill. Run it at accelerating speed (first
  20 one by one, the rest in a 2 s rush) with a counter *10,720 nudges*.
  Viewers feel that the downhill direction is a consensus of thousands,
  which is exactly what `balance` and `settles-2` need next.
- Draw the valley at its true aspect so the trough's narrowness is real:
  a long trench steep across position, shallow along speed. Label the
  ratio once, felt: a ruler showing that a turn of the position knob
  that climbs the wall would, on the speed knob, barely lift off the
  floor (draw both moves as the same-length arrow on the floor, show
  each height).

Data: per-moment pulls at a chosen non-bottom point (cheap to compute
from the data and weights), the 61 x 61 valley. Build: moderate.

---

## balance

> "That valley has a single lowest spot. There, the pulls are still
> there, but they balance... a tug of war at a standstill."

Current: three bars, each +x and -x, "= 0.000". It states the sum but
not the size of the crowd or of each pull, which `settles-2` relies on
("each moment's own pull isn't" nothing).

Scale proposal: a real tug of war on the position knob only (the other
knobs as a small footnote).
- The rope: each moment pulling up is a warm sliver on the right, each
  pulling down a cool sliver on the left, laid end to end along the
  rope, each sliver's length its real pull. Counts at each end: *N
  moments pulling up*, *M pulling down* (compute; they needn't be
  equal). The two sides grow to exactly +0.766 and -0.766 and the knot
  stays on the centre mark.
- Then the scale beat: zoom onto one big single pull (the largest), and
  compare its length to the net (zero): *one moment alone: 0.0xx; all
  together: 0.000*. This sets up `settles-2` without saying it.

Data: per-moment pulls at the resting weights (exist). Build: moderate
(sorting, a sliver strip, a camera zoom).

---

## too-big

> "...Too big, and each move overshoots the bottom of the U so far that
> it lands higher up the other side."
> (next: "...the total squared miss grew after each of 10 moves, to 137
> times where it started.")

Nate's note (arrow showing the size of the change, overlaid to show the
overshoot) is already the right idea through my lens too: it shows the
step's *size* against the U's *width*. My additions:
- Show the settled step first, to scale, as a short warm arrow on the
  U, then double it: the arrow visibly doubles length (copy laid end to
  end on itself, so "twice" is seen). Then place the doubled arrow's
  tail at the ball; its tip lands past the bottom, the ball moves there,
  the arrow from the new spot is longer (the slope is steeper), and so
  on. Each arrow lengthening is the felt reason it runs away.
- The 137x needs a camera: as the misses grow, the camera pulls back
  (y-axis rescaling with a smooth zoom, with the axis numbers updating)
  so that by move 10 the original U, which filled the screen, is a tiny
  notch at the bottom, and a ghost of the starting height is a hairline
  near the floor. A bar on the side, *total squared miss*, grows to 137
  copies of its first value (stacked unit bars, countable in tens).
- Check: the current chart is "one knob alone". The 137x and the 10
  moves are from all three knobs together. The builder must use either
  the real one-knob iterates at twice the step (if they diverge) or the
  cross-section along the steepest direction, and the readout must
  match the shown run (an Honesty point, but it decides the scale).

Data: the 10 too-big moves (exist); one-knob iterates if used (cheap).
Build: moderate (arrow chains, camera zoom on a chart).

---

## map-intro

> "Here's a picture of what that would take: a sheet of borders."

Nate: the blank sheet looks like it should show something. Through my
lens: it should show how *many* borders it holds, and that each is a
whole border.
- **0-1 s**: Mario's map with its best border, as now.
- **1-4 s**: beside it, the sheet arrives as a mosaic of tiny maps:
  start with a 12 x 10 grid of thumbnails, each a miniature of Mario's
  map with its own ruler (turned by column, slid by row; real
  geometry). Then the grid subdivides twice (24 x 20, 48 x 40) as the
  camera pulls back, the thumbnails shrinking to specks, until it reads
  as a fine texture. Label: *720 turns x 601 slides: 432,720 borders,
  one per spot*. (That is the real sheet resolution from `exp002.py`.)
- The sheet is never blank, and the viewer has already understood "every
  spot is one whole border" before `sheet` says it.

Data: geometry only. Build: moderate (thumbnail renderer; a few hundred
tiny maps; beyond that a texture).

## map-axes

Through my lens, a small change: as the ruler turns and slides on the
map, a bright dot moves across the mosaic sheet to the thumbnail that
*is* that border (the mosaic from map-intro), and the thumbnail under
the dot pulses. The turn moves it right (half a turn = halfway across:
show the yes/no swap as the thumbnail's warm side flipping), the slide
moves it up. This keeps the sheet's size (432,720) in view while the
idea is taught one border at a time. Build: easy once the mosaic exists.

---

## staircase

> "Imagine the sheet as ground, with each spot raised to a height: how
> many mistakes that border makes. Lower is better."

Lens point: the heights are counts of mistakes; let the viewer read the
height in moments, not a share. As the sheet rises, a single vertical
ruler at the camera's focus reads in moments (*0 to 10,720 moments
wrong*), with the lowest ledge at 7.8% = about 840 moments and the top
at 92% = about 9,860. (Use exact counts from the maps file: share x
10,720.) A tiny Mario-map inset beside the ruler shows the border under
the focus, its wrong moments lit, so "height = mistakes" is literally
counted. The camera should orbit (Space and motion's job); I'd add only
that the orbit should end low, near ground level, so the heights read
as heights. Build: yaw for sheet.py plus an inset.

## staircase-2

> "The height only changes when the border crosses a moment... a
> staircase of flat ledges."

Lens point: the size of a single step. Most steps are one moment, 1 in
10,720 (0.009%), invisible at whole-sheet scale, so a viewer can't
believe the ground is a staircase. Proposal: a zoom into one angle row
around the lowest ledge, x100 then x1,000 in height, until single
1-moment steps are visible as real treads and risers; a counter beside
reads *one moment* at each riser; the border on the map inset sweeps a
single dot across its line at each riser. Then zoom back out, so the
viewer knows the smooth-looking slope is made of ~330 such steps per row
(the real distinct-heights count). Data: one row of the mistakes map
(exists). Build: moderate (vertical exaggeration must be labelled:
*heights x1,000*).

---

## no-slope

> "Standing on a flat ledge, every direction looks the same..."

Nate: arrows look wider than the flat step. That's a scale error in the
picture: the marker's "look around" arrows must be smaller than the
ledge it stands on, at true scale. Proposal: zoom until the marker's
ledge fills most of the frame (its true outline in angle and offset),
draw the eight looking arrows to reach no further than 80% of the way to
the ledge's nearest edge; all show *0 change*. Then the marker walks
one way until it reaches the edge, and falls a real riser (size labelled
in moments). Then pull back, at speed, to the whole staircase, the
marker's ledge shrinking to one of thousands, the lowest ledge (418
degrees, offset 0.28) lit far off: the viewer feels how many ledges lie
between. Data: the ledge region around the chosen spot (flood fill of
equal height in the maps file). Build: moderate.

## staircase-outro

> "So from a rule that settles about three in a hundred short..."

Lens point: "three in a hundred" should be a felt gap on the ground.
At the end of an orbit, the camera settles at ledge level between the
two markers: the square trick's border (bowl bottom) and the lowest
ledge. A vertical bracket between their heights, labelled in moments
(the training mistakes at the bowl bottom minus at the lowest ledge,
from the maps file) and in exam terms (*about 3 in 100 of the exam's
moments: about 146 more mistakes on 81 seconds of play*, using 91% -
88% = 3 points of 4,883). Colour and orbit are other lenses'; I'd ask
that the height scale be stretched only with a label.

---

## shelf-2

> "Then every total is zero, every moment misses by exactly one, and all
> those borders sit at the same height: a high, flat shelf..."

Lens point: two sizes, the shelf's extent (about 52% of the sheet) and
its height relative to the bowl (1.0 against a bottom of 0.37).
- Start on one border on the shelf, shown on the Mario map, with its
  size dial. Turn the size down to zero: every moment's miss bar (a
  strip of 10,720 hairlines) goes to exactly 1 together, the total bar
  stops at 1.0. That is "every moment misses by exactly one", counted.
- Then flood: from that spot, the shelf fills in across the sheet at
  its real extent; a running tally *borders on the shelf: N of 432,720
  (52%)*. A height ruler beside it marks 1.0 for the shelf and 0.37 for
  the bowl's bottom, so "high" is a ratio the viewer sees (the shelf is
  nearly three times the bowl's lowest height).
Data: shelf mask and heights (exist). Build: moderate.

---

## Summary of what I'd prototype first

1. too-many, Proposal A (grains, pulled back past known objects, memory
   cells as the counter; universe overfilled at 40 numbers; 4,620-digit
   wall at 2,048).
2. no-fast-way, Proposal A (ramp vs cliff on a time ribbon; the square
   trick seconds, the search past the universe at 10 numbers), labelled
   so the square trick isn't claimed to solve the same problem.
3. too-big with a zooming chart so 137x is felt.
4. staircase-2 / no-slope zooms to single 1-moment steps and true-size
   look-around arrows.
5. map-intro mosaic of 432,720 borders.
