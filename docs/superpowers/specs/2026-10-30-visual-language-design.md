# The series' visual language, and 001's pieces

## Why

The series studies something dark and mysterious, how a network thinks,
and its job is to shine a light into it and show what's there: its
beauty, variety and versatility, empirically but with intuition and
empathy (Nate: "Rainbow in the Dark", the prism on *The Dark Side of the
Moon*). The look has to carry that, stay legible to a lay viewer, and
keep the games themselves honest. Nate chose it by reacting to mockups
and to a moving prototype (the "Ember" spike, 2026-10-30,
`target/series/spike/sketches/ember_neuron_v2.mp4`, thirteen passes).

This document fixes the look for every video, and designs the reusable
pieces 001 needs. Building 001's individual animations comes after, as
sketches (the bible's step 5).

## Goals

- One look across the series, written down so every animation is built
  in it.
- A small library of reusable pieces (the neuron, the game screen and
  its overlays, the graph, the map, the balance scale, and the rest of
  001's vocabulary) in `tools/series/`, each with tests, each tried as a
  sketch before any beat uses it.
- Game footage that is exact: true pixels, exact positions, frame-exact
  overlays.

## Non-goals

- Animating 001's 133 beats; that's the next step, built on these pieces.
- Changing the script; anything the pieces suggest goes to Nate's
  interview.
- Nate's own narration (the beat recorder is separate).

## Anti-goals

- Primer or 3Blue1Brown pastiche; Flump's cartoon faces; synthwave
  cliché; "everything 8-bit".
- Clutter: more than one picture per beat (the bible's rule).
- Anything drawn over the game that disagrees with the game, even by a
  frame or a pixel.

## The look (decided)

**Ember.** A deep, nearly black ground with a faint warm glow toward
where the action is (a radial gradient, `#24100A` to `#030303`). Light
means understanding: what the video has explained gives light; what it
hasn't stays dim (unexplained neurons are dim coals).

**Colour: temperature is sign; hue is identity.**
- Warm (amber `#F2B34A`, coral `#EE8660`, gold `#FFD59A`, rose, crimson)
  is above zero: pushes up, excites, says yes.
- Cool (blue `#4F8FB8`, light blue `#7FB7E0`, teal, violet) is below
  zero: pushes down, inhibits, says no.
- The dark is zero.
- Within a temperature, hue tells things apart; where paths combine,
  their colours blend (gradients). A concept needn't keep one colour
  across videos (Nate: consistency of association isn't critical); the
  meaning of warm and cool is what stays fixed. The orange-against-blue
  contrast is the film-poster complementary pair, and colour-blind safe.
- This replaces the bible's "one colour per concept in every video" and
  `style.py`'s Okabe-Ito palette.

**Type.** A serif, italic for labels and words, roman for numbers:
Charter (its free release, Bitstream Charter, so renders are
reproducible on any machine). Negative numbers use a true minus (−).
Formulas are set in the same serif, not Manim's default maths font.
Sizes as in the spike: labels about 0.42 and verdicts 0.5 of Manim's
default text scale, at 1080p.

**Lines.** Thin and precise: hairlines of about 1.2–1.4 px at 1080p for
paths, axes and rulers; nothing heavy.

**Light.**
- A continuous value (a neuron's total, an activation) is a smooth
  radial glow, one gradient image, never stacked discs (they band):
  warm and brighter the further above zero, cool and brighter the
  further below, nothing at zero; its intensity eases toward the value
  (about a ninth of a second) rather than snapping.
- One light scale per video: a total's brightness is a fixed function of
  its size, the same for every neuron and beat in the video, easing
  smoothly toward full rather than clipping (my call, after the builder
  check). So when a neuron's numbers are tripled, its light really is
  brighter, because its total really is three times bigger, while the
  line and the answer stay put: what changes is how much, not where or
  whether.
- A binary answer is crisp: a ring its line runs into, filled with a
  warm dot for yes, empty and cool for no. No glow. (001's "the light
  becomes a dial" beat is the moment the crisp dot becomes a glow.)

**Motion.** Instruments arrive one at a time, each drawn or faded in
over about half a second to a second, after the thing they measure;
numbers are live from the moment they appear (they never fade in stale
and jump); nothing snaps that represents a continuous quantity.

## The game screen (decided)

The game is the thing studied, so it stays exactly itself; our
instruments are made of light around and over it.

- **Exact pixels.** Footage is scaled by a whole number of screen
  pixels per NES pixel wherever possible (3× at 1080p: 768 × 720),
  placed on whole-pixel boundaries, nearest-neighbour. Below 1× (a
  screen among many), it is averaged down smoothly instead.
- **Original colours.** No dimming.
- **An old TV, in effects that survive resizing and compression:**
  rounded corners (about 26 screen px at 3×); a glow that is a real blur
  of the rounded screen, in the clip's own average colour, spilling into
  the dark and fading up with the picture; the same light blooming over
  the screen's own rim so no dark border forms; edges darkening softly
  toward the corners. All scale with the screen.
- **Scanlines** only at 3× or more, faint (the last screen row of each
  NES line, about 12% dark), automatic, with a flag to force them off
  (or on where they fit). Below 3× they can't be drawn without aliasing.
- **Arrival.** The game arrives from black by a cover fading off it (not
  Manim's FadeIn, which animated a copy that played and then restarted).
- **Frame-exact data.** Footage comes only from recordings whose RAM was
  saved on every frame alongside every picture (`series_record
  --footage`), never from takes sampled every few frames and
  interpolated (jump starts and landings go wrong). Overlays read RAM
  one frame behind the picture: in Super Mario Bros. the screen shows
  the position the game computed on the previous frame (measured: the
  sprite matches RAM one frame back on 99% of moving frames, and never
  the same frame). The lag is measured per game, by a check against the
  sprite, not assumed.

## The neuron, drawn (decided)

- A hairline ring; its running total inside, in roman; its glow (the
  total) behind it.
- Each input arrives as a path from the thing it measures: a mark on the
  game screen, a square of the memory grid, or (when it measures
  nothing on screen) a bare number. A bias that isn't a place on the
  screen is a number with no path. (My calls, after the builder check.)
- Falling speed, 001's second input, is a mark on the screen too: an
  arrow at Mario, its length his speed, warm falling and cool rising.
- In 001's first neuron the input is a line on the game screen: depth's ruler under Mario's feet (the
  bottom edge of his lowest pixel row, depth + 33 rows from the top),
  running the width of the screen and leaving it at the right edge.
- A path is warm as it leaves the game (a number) and takes the
  temperature of what it contributes: depth, weighted by −1, cools as it
  approaches the neuron.
- **The setting is on the screen too:** a fixed gold line at the floor's
  depth (176), whose path into the neuron is the bias. The gap between
  the two lines *is* the total, and is shaded: warm while Mario is above
  the setting, gone when he stands on it, cool below it.
- **Weights other than −1** (Nate, 2026-10-30): the total is weight ×
  (depth − setting). The setting line never moves when the weights
  change; the weight sets how much each pixel of the gap counts, shown
  as the band burning brighter on the video's one light scale (my call:
  brightness, not a denser ruler).
  The bias is then the setting translated into the neuron's units, −w ×
  setting: computed, not seen. 001's tripling beat shows exactly this:
  everything × 3, the line doesn't move, no answer changes.
- The answer: the ring and dot, with its verdict beneath it in italics,
  warm ("in the air") or cool ("on the ground").

## 001's pieces

Each is a reusable piece in `tools/series/`, tried alone as a sketch
before beats use it. From 001's boards:

1. **Ground and style:** the Ember backdrop, the colour rule, the type,
   glow images, live text (numbers that are right from the moment they
   appear).
2. **The game screen:** exact footage and its TV treatment, with frame-
   exact RAM access and the per-game lag check; overlays (ruler, setting
   line, gap, labels) that follow the game.
3. **The neuron:** inputs as paths, ring, total, glow, answer ring;
   one or two inputs (001's second neuron reads depth and falling speed).
4. **The formula:** w·x + b > 0 typeset in the serif, able to fill in
   numbers and rearrange (the "> 100 slides into the bias" beats). A
   symbol is neutral parchment until a number fills it; a filled term
   takes its sign's temperature, and terms blend where they combine. The
   boards that still name the old colours ("w in orange, x in blue, b in
   purple") are updated to match; they're production notes, so no
   narration changes. (My call.)
5. **The graph:** hairline axes, the line burning warm above zero,
   dark at the setting, cool below; the setting as a small mark; the
   "in the air" side faintly warm, "on the ground" faintly cool; the
   tripling beat (line steepens and pivots about the setting).
6. **The map:** one dot per moment, mostly dim ash, outcomes lit in
   their own hues; the border a hairline; sides faintly warm and cool;
   the border sliding and leaning as the bias and weights change.
7. **The balance scale:** warm blocks in the yes pan, cool in the no
   pan, a beam running warm to cool, the verdict in the colour of the
   side it tips to.
8. **The memory grid:** the NES's 2,048 bytes as a grid of numbers,
   changing as the game plays; squares that light and label.
9. **Records and scores:** the three tapes (session recordings) and
   their strips; bars and scorecards (accuracy against the baseline).
10. **Title screens and credits,** redone in this look (the chapters
    design's drifting wall of a section's own pictures, in Ember). The
    wall's tiles stay stand-ins until beats are rendered; real stills,
    taken from rendered beats in a two-pass build, come later. (My call.)

Smaller one-off pictures (the thermostat, the NES chip and its
instruction, the network of dots, the magnifying glass, the dial) are
built as their beats come, in the same look.

## Left to the builder

- The pieces' APIs and file layout, following `scene.py`'s and
  `footage.py`'s existing style.
- How fonts are installed for reproducible renders (Bitstream Charter in
  the repo or fetched at setup), within its licence.
- Exact easing curves and timings, within "about half a second to a
  second" and "about a ninth of a second for light".
- Tests: everything measurable (integer scaling and pixel alignment,
  overlay frame-lag against the sprite, colour-by-sign, live text) is
  tested; looks are judged by sketch and contact sheet, and by Nate.

## Process

The pieces go through a plan and its reviews like any code. Each piece
gets a sketch Nate sees before beats are built on it. The bible's
"Visual language" section and `style.py` are rewritten to match this
document.
