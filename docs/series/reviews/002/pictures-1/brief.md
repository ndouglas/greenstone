# Picture council brief: video 002, beats Nate flagged

Video 002, *Every mistake moves the line*. Numbers in {braces} are filled from the experiment (values below).

## Nate's general notes
- He understands best from pictures; text alone confuses him. The pictures must carry the ideas.
- 'Even if it's a really complex animation': cost is not the constraint; clarity and truth are.
- Other already-fixed notes: transitions must not show things early; dials aligned; title walls slower.

## The beats (each with the line before and after for context)

### too-many
- (neighbour) `search-2` [Introduction]: "The best straight border is a compromise: it gets some moments wrong to get more right. It says yes from about four blocks in when he's standing still, and from about one and a half at full walking speed."
- **THIS BEAT** `too-many` [Introduction]: "Trying borders like that only works easily with two numbers; each extra number multiplies the borders to try. A neuron reading more of the two thousand and forty-eight numbers in the console's memory would have far too many borders to try."
  - Board (the original brief for its picture): The map's two axes multiply into a crowd of axes; the sweeping borders blur into too many to draw
- (neighbour) `no-fast-way` [Introduction]: "And in general, no known method finds the border with the fewest mistakes quickly."
  - Nate: Not really feeling the visualization.
  - Stills of the current render (6 moments across the beat): `/private/tmp/claude-501/-Users-nathan-Projects-ndouglas-greenstone/cf099f6b-6b2a-4507-8759-c42c3f7c29f4/scratchpad/check/beat-too-many.png`

### no-fast-way
- (neighbour) `too-many` [Introduction]: "Trying borders like that only works easily with two numbers; each extra number multiplies the borders to try. A neuron reading more of the two thousand and forty-eight numbers in the console's memory would have far too many borders to try."
- **THIS BEAT** `no-fast-way` [Introduction]: "And in general, no known method finds the border with the fewest mistakes quickly."
  - Board (the original brief for its picture): A clock spinning beside the crowd of borders
- (neighbour) `learn` [Introduction]: "So this time, the neuron finds its own border. It starts with its weights, how much each number counts, and its bias, one more number added on, all at zero, and moves them every time it makes a mistake."
  - Nate: It's not actually a valuable visualization, just a cheap one. Let's figure out something that actually depicts the scope and scale of the problem here, even if it's a really complex animation.
  - Stills of the current render (6 moments across the beat): `/private/tmp/claude-501/-Users-nathan-Projects-ndouglas-greenstone/cf099f6b-6b2a-4507-8759-c42c3f7c29f4/scratchpad/check/beat-no-fast-way.png`

### exam
- (neighbour) `bounded` [It never settles]: "Later, mathematicians proved that the weights stay within limits. With mistakes that never run out, they wander inside them, and never settle."
- **THIS BEAT** `exam` [It never settles]: "After every pass, we scored it on recordings it never learned from, like an exam it hasn't seen."
  - Board (the original brief for its picture): Separate recordings wait; after each pass, the border is scored on them
- (neighbour) `one-run` [It never settles]: "Over its last ten passes, its score kept jumping, as high as {perceptron_one_run_highest}, and on its very last pass, down to {perceptron_one_run_lowest}."
  - Nate: Maybe we can make this visualization more interesting as well? If we actually showed the different data, for instance.
  - Stills of the current render (6 moments across the beat): `/private/tmp/claude-501/-Users-nathan-Projects-ndouglas-greenstone/cf099f6b-6b2a-4507-8759-c42c3f7c29f4/scratchpad/check/beat-exam.png`

### downhill-2
- (neighbour) `downhill` [How wrong was it?]: "Hold every weight still but one, and turn that one: the total squared miss falls, bottoms out, and rises again, a U."
- **THIS BEAT** `downhill-2` [How wrong was it?]: "Turn them all at once and it's a valley. Added up over all the moments, the square trick's nudges always point downhill in it. That's what we'll call following a slope."
  - Board (the original brief for its picture): The U widening into a valley over every setting; the added-up nudge an arrow pointing down it
- (neighbour) `balance` [How wrong was it?]: "That valley has a single lowest spot. There, the pulls are still there, but they balance: the ones pulling up and the ones pulling down add up to exactly nothing, like a tug of war at a standstill."
  - Nate: I think this could be a better visualization.
  - Stills of the current render (6 moments across the beat): `/private/tmp/claude-501/-Users-nathan-Projects-ndouglas-greenstone/cf099f6b-6b2a-4507-8759-c42c3f7c29f4/scratchpad/check/beat-downhill-2.png`

### balance
- (neighbour) `downhill-2` [How wrong was it?]: "Turn them all at once and it's a valley. Added up over all the moments, the square trick's nudges always point downhill in it. That's what we'll call following a slope."
- **THIS BEAT** `balance` [How wrong was it?]: "That valley has a single lowest spot. There, the pulls are still there, but they balance: the ones pulling up and the ones pulling down add up to exactly nothing, like a tug of war at a standstill."
  - Board (the original brief for its picture): Arrows from every moment, pulling up and pulling down; at the bottom of the valley, they add to nothing
- (neighbour) `settles-2` [How wrong was it?]: "Only the sum is nothing; each moment's own pull isn't. So if it learns one moment at a time, with the small step we gave it, its score still moves around over its last ten passes, by about {square_swing_points} in a hundred; the perceptron's typically by {perceptron_swing_points}."
  - Nate: I think this could be improved.
  - Stills of the current render (6 moments across the beat): `/private/tmp/claude-501/-Users-nathan-Projects-ndouglas-greenstone/cf099f6b-6b2a-4507-8759-c42c3f7c29f4/scratchpad/check/beat-balance.png`

### too-big
- (neighbour) `settles-3` [How wrong was it?]: "If it adds up every moment's nudge first, then makes small moves, it comes to rest."
- **THIS BEAT** `too-big` [How wrong was it?]: "Here, the size of the step decides whether it gets there at all. The targets stay at plus and minus one, so doubling the step can't just double everything, the way it did for the perceptron. Too big, and each move overshoots the bottom of the U so far that it lands higher up the other side."
  - Board (the original brief for its picture): A ball on the U, each jump crossing the bottom and landing higher up the other side. (The step that settled also crosses the bottom along the steepest way, but lands lower each time: don't draw it as a smooth roll.)
- (neighbour) `too-big-2` [How wrong was it?]: "With {too_big_step} times the step size that settled, the total squared miss grew after each of {too_big_steps} moves, to {too_big_growth} times where it started."
  - Nate: The 'jump' is too literal. Better: an arrow that indicates the magnitude of the change, then overlay it to show how that plus the current position overshoots the bottom and goes up the other side.
  - Stills of the current render (6 moments across the beat): `/private/tmp/claude-501/-Users-nathan-Projects-ndouglas-greenstone/cf099f6b-6b2a-4507-8759-c42c3f7c29f4/scratchpad/check/beat-too-big.png`

### map-intro
- (neighbour) `square-outro` [Close, but not quite]: "So the square trick, adding up its nudges, settles in one place, but that place was chosen by the squared misses we gave it, not by the count of mistakes we wanted. So why not aim straight at the fewest mistakes?"
- **THIS BEAT** `map-intro` [Close, but not quite]: "Here's a picture of what that would take: a sheet of borders."
  - Board (the original brief for its picture): Mario's map, with its border; beside it, a blank sheet labelled 'borders'
- (neighbour) `map-axes` [Close, but not quite]: "Picture a border on Mario's map as a ruler laid across it, with a yes side and a no side. You can turn the ruler around the middle of the map, and half way round, its yes and no sides have swapped; or you can slide it nearer to or further from the middle."
  - Nate: It's unclear what the blank sheet is supposed to show. It looks like it should show something, but it's blank for these two beats.
  - Stills of the current render (6 moments across the beat): `/private/tmp/claude-501/-Users-nathan-Projects-ndouglas-greenstone/cf099f6b-6b2a-4507-8759-c42c3f7c29f4/scratchpad/check/beat-map-intro.png`

### map-axes
- (neighbour) `map-intro` [Close, but not quite]: "Here's a picture of what that would take: a sheet of borders."
- **THIS BEAT** `map-axes` [Close, but not quite]: "Picture a border on Mario's map as a ruler laid across it, with a yes side and a no side. You can turn the ruler around the middle of the map, and half way round, its yes and no sides have swapped; or you can slide it nearer to or further from the middle."
  - Board (the original brief for its picture): A ruler-like border on Mario's map turning, its yes side swapping at half a turn, and sliding
- (neighbour) `sheet` [Close, but not quite]: "Now picture a flat sheet where every spot is one whole border. Moving right across the sheet turns the ruler; the sheet is one full turn wide, so going off its right edge brings you back on its left. Moving up slides it. The best border we found is one spot on it."
  - Nate: (same note as map-intro: the blank sheet)
  - Stills of the current render (6 moments across the beat): `/private/tmp/claude-501/-Users-nathan-Projects-ndouglas-greenstone/cf099f6b-6b2a-4507-8759-c42c3f7c29f4/scratchpad/check/beat-map-axes.png`

### staircase
- (neighbour) `sheet` [Close, but not quite]: "Now picture a flat sheet where every spot is one whole border. Moving right across the sheet turns the ruler; the sheet is one full turn wide, so going off its right edge brings you back on its left. Moving up slides it. The best border we found is one spot on it."
- **THIS BEAT** `staircase` [Close, but not quite]: "Imagine the sheet as ground, with each spot raised to a height: how many mistakes that border makes. Lower is better."
  - Board (the original brief for its picture): The border sheet rising into ground, each spot lifted by its border's mistakes
- (neighbour) `staircase-2` [Close, but not quite]: "The height only changes when the border crosses a moment, so between those crossings it's perfectly flat. The ground is a staircase of flat ledges."
  - Nate: The static display doesn't make this clear; rotate it, zoom, make it more dynamic.
  - Stills of the current render (6 moments across the beat): `/private/tmp/claude-501/-Users-nathan-Projects-ndouglas-greenstone/cf099f6b-6b2a-4507-8759-c42c3f7c29f4/scratchpad/check/beat-staircase.png`

### staircase-2
- (neighbour) `staircase` [Close, but not quite]: "Imagine the sheet as ground, with each spot raised to a height: how many mistakes that border makes. Lower is better."
- **THIS BEAT** `staircase-2` [Close, but not quite]: "The height only changes when the border crosses a moment, so between those crossings it's perfectly flat. The ground is a staircase of flat ledges."
  - Board (the original brief for its picture): Terraces: flat ledges, sudden drops, a narrow lowest ledge with a cliff along one side
- (neighbour) `big-drops` [Close, but not quite]: "And because so many moments share the same position and speed, some of the drops are big. {spot_biggest_change} moments share one point on Mario's map, two and a half blocks in and not moving sideways, where each life starts, so a border crossing it changes its count by all of them at once."
  - Nate: (same note as staircase)
  - Stills of the current render (6 moments across the beat): `/private/tmp/claude-501/-Users-nathan-Projects-ndouglas-greenstone/cf099f6b-6b2a-4507-8759-c42c3f7c29f4/scratchpad/check/beat-staircase-2.png`

### no-slope
- (neighbour) `big-drops` [Close, but not quite]: "And because so many moments share the same position and speed, some of the drops are big. {spot_biggest_change} moments share one point on Mario's map, two and a half blocks in and not moving sideways, where each life starts, so a border crossing it changes its count by all of them at once."
- **THIS BEAT** `no-slope` [Close, but not quite]: "Standing on a flat ledge, every direction looks the same. Only a drop tells you which way is down, and you find it by falling off. So a rule that follows a slope, like the square trick, can't walk down this staircase to the lowest ledge, around the best border we found."
  - Board (the original brief for its picture): A marker on a ledge, looking every way; all directions level
- (neighbour) `staircase-outro` [Close, but not quite]: "So from a rule that settles about three in a hundred short, we've come to the thing we actually want, the count of mistakes, and found a staircase no slope-following rule can walk down. So where does the square trick's slope lead, on this sheet? And what about the perceptron?"
  - Nate: The point is becoming clearer, but it could be more dynamic. The arrows look wider than the flat step, which may confuse.
  - Stills of the current render (6 moments across the beat): `/private/tmp/claude-501/-Users-nathan-Projects-ndouglas-greenstone/cf099f6b-6b2a-4507-8759-c42c3f7c29f4/scratchpad/check/beat-no-slope.png`

### staircase-outro
- (neighbour) `no-slope` [Close, but not quite]: "Standing on a flat ledge, every direction looks the same. Only a drop tells you which way is down, and you find it by falling off. So a rule that follows a slope, like the square trick, can't walk down this staircase to the lowest ledge, around the best border we found."
- **THIS BEAT** `staircase-outro` [Close, but not quite]: "So from a rule that settles about three in a hundred short, we've come to the thing we actually want, the count of mistakes, and found a staircase no slope-following rule can walk down. So where does the square trick's slope lead, on this sheet? And what about the perceptron?"
  - Board (the original brief for its picture): The staircase, and the square trick's border, still, somewhere off its lowest ledge
- (neighbour) `bowl` [A bowl]: "To see where the square trick's slope leads, give the border sheet a different height: each border's total squared miss."
  - Nate: Contrast and dimensionality aren't sufficient; it's hard to tell what the landscape looks like. Needs better colouring and rotation/zooming/other movement to explore the surface.
  - Stills of the current render (6 moments across the beat): `/private/tmp/claude-501/-Users-nathan-Projects-ndouglas-greenstone/cf099f6b-6b2a-4507-8759-c42c3f7c29f4/scratchpad/check/beat-staircase-outro.png`

### shelf-2
- (neighbour) `shelf` [A bowl]: "Some borders do best by shrinking to nothing: any size above zero makes their misses bigger. Mostly, though not always, they're borders that get most moments wrong."
- **THIS BEAT** `shelf-2` [A bowl]: "Then every total is zero, every moment misses by exactly one, and all those borders sit at the same height: a high, flat shelf, filling the side of the sheet where the ruler mostly faces the wrong way."
  - Board (the original brief for its picture): Their spots on the sheet joining a high flat shelf, on the side where the ruler faces the wrong way
- (neighbour) `bowl-2` [A bowl]: "Everywhere else, the ground curves smoothly down into one bowl, with one lowest spot, and that's the border where the square trick comes to rest."
  - Nate: Not entirely sure what's being illustrated here.
  - Stills of the current render (6 moments across the beat): `/private/tmp/claude-501/-Users-nathan-Projects-ndouglas-greenstone/cf099f6b-6b2a-4507-8759-c42c3f7c29f4/scratchpad/check/beat-shelf-2.png`

Neighbouring stills also exist for context: beat-learn, beat-downhill, beat-sheet, beat-big-drops, beat-shelf, beat-bowl-2 (same folder).

## Numbers (numbers.json)
- line_blocks: 5
- search: 91%
- scrolls_soon_rate: 51%
- spots: 1313
- spots_with_both: 232
- train_moments: 10720
- test_moments: 4883
- batch_square: 88%
- search_cut_blocks: 3.98
- search_cut_walking_blocks: 1.56
- batch_square_cut_blocks: 5.69
- too_big_step: 2
- too_big_steps: 10
- too_big_growth: 137
- stairs_bottom: 91%
- bowl_bottom: 88%
- spot_biggest_change: 710
- map_middle: [2.5, 1.25]
- bowl_flat: 50%
- flat_getting_most_wrong: 82%

## What data exists (all real; computed by the experiment code)

- Mario's moments (training set): position in blocks (0 to 5, in sixteenths), rightward speed in pixels a frame (0 to 2.5, in sixteenths), and whether the screen moved within 30 frames. 1,313 distinct points on the map; many moments share a point. Test set: separate recordings (3 and 6) used to score; training recordings 1, 2, 4, 5. There is frame-exact game footage (picture + RAM every frame) of one session, so any moment can be shown on the real game screen.
- The perceptron's run: every nudge (which moment caused it, the weights after it, which pass), ten shuffled orders' weights and exam scores per pass, the pocket version, runs with 30 moments left out.
- The square trick, added up: its weights after each of 2,000 moves, the total squared miss after each; with twice the step (too big): 10 moves, miss growing to 137× the start.
- The U: total squared miss as the position weight alone turns (others held at the square trick's resting weights), and the same for the speed weight and the bias. The valley: total squared miss over a 61×61 grid of (position weight, speed weight) with the bias held. Note: position counts ~15× more than speed, so the valley is a long trough, steep across position, shallow along speed.
- At the resting weights, every moment's pull on each knob (miss × its number); added up they cancel to 0.000 (pull up totals ≈ +0.766 on the position knob, matched by −0.766 down).
- The sheet of every border: 720 angles (one full turn, from 310° to 670°) × 601 offsets (−3 to 3, the ruler slid from the middle of the map at (2.5 blocks, 1.25)). Two heights: mistakes (share of training moments wrong, 0.078 to 0.92: a staircase of flat ledges, about 330 distinct heights per angle row) and the total squared miss at each border's best size (0.37 to 1.0; the borders facing the wrong way are best at size zero and form a flat shelf at exactly 1.0 covering ~52% of the sheet; the rest is one smooth bowl). Lowest ledge at 418°, offset 0.28; bowl bottom at 434.5°, offset 0.35.
- The start point: 710 moments at (2.5 blocks, speed 0), all "screen didn't move". A border crossing it changes its mistake count by 710 at once.

## Pieces that exist (tools/series/)

- pointmap.py: Mario's map by point (dots sized and lit by count, warm/cool, split where both happened), border from (w, b), animate along a path, fade/ring points.
- sheet.py: a heightfield renderer (PIL polygons, painter's order; flat-ledge mode with walls, or smooth lit mesh) with a perspective camera (pitch, distance; no yaw yet), lift from flat to full height, cropping a window, glow on cells. ~0.2 s a frame at 1080p, so camera moves are affordable. Yaw/orbit, contour lines, better colour ramps, fog, cross-sections could be added.
- chart.py (line charts), knobs.py (dials and a total bar with targets), screen.py (exact game footage with overlays), memory.py (the console's 2,048 bytes as a live grid), records.py (score bars, tapes), bordermap.py, props.py (console, controller), ember.py (glow, type).
- Visual language: docs/superpowers/specs/2026-10-30-visual-language-design.md (read it: Ember, warm = yes/above zero, cool = no/below, hairlines, light means understanding).
- The script: tools/series/videos/002-every-mistake/script.toml. The current beat code: tools/series/videos/002-every-mistake/beats/*.py.
