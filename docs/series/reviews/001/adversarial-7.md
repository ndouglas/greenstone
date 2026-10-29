# Video 001, adversarial reviews, seventh pass

Script reviewed: commit a25e399. Reviewer 1: nothing that would badly
mislead; five smaller findings. Reviewer 2: eight findings. Fixed in the
next commit, together with Nate's two number-load decisions.

| Finding (reviewer) | Disposition |
|---|---|
| land-mistakes: "as with the first neuron" compares a share of missed landings (85%) with a share of all mistakes (88%); 29% of its mistakes are false alarms (2) | Now the share of all mistakes (60%, `smb1_land_raised_share_of_all_mistakes`), and says most of the rest are "lands soon" too early |
| bias-slides pairs the first neuron's graph with the landing neuron's map as if one neuron; "changing a weight changes the lean" clashes with the-tilt's scaling (2) | Names each neuron; "changing one weight on its own"; board shows the panels in turn, not linked |
| one-dot: "the bias is three times bigger" points at nothing on the graph (2) | school: "starts at 176, the bias"; one-dot: "starts three times higher and is three times steeper" |
| steady-player not supported by games/001.md, which said all play is 1-1's opening; flagpole frames (2) | games/001.md records the rule teacher's 1-1 clear, 4,060 frames in 1-2, floor also 176, and the flagpole state |
| why-weights credits weights with reading several numbers (2) | "What makes a neuron more than a thermostat is that it can read several numbers at once, and its weights let each one count…" |
| the-idea: factual claims under framing; triplet cadence (2) | Cites bishop2006 and goodfellow2016; "That's all there is inside it, and we can see every part." |
| modern: "long recommended" beyond the source (2) | "which a standard textbook recommends as the default" |
| the-map: "below the middle" leans on the board (2) | "below the level where his falling speed is zero" |
| why-plus: "more than 152" never says what is (1) | Says it: depth plus 5 times the falling speed |
| depth: "a little above his head" understates (the point is a Mario-height up); board contradicts its rule; offset untested (1) | "a point well above his head: his feet are always 32 dots lower"; board fixed; new test `smb1_small_marios_feet_are_32_dots_below_his_depth` (failed first on parked offscreen sprites, then passed) |
| "dot" is both the unit and the map's marks (1) | The map has points; dots stay the unit |
| one-dot's "any neuron" fails for neurons that pass on a number; the-tilt's scaling needs "plus" (1) | "any neuron of this yes-or-no kind"; the-tilt "by the same plus number" |
| balance: a block "as big as" minus 150 (1) | "as big as the number, leaving off any minus sign" |
| Number load (Nate's decision) | (a) SMB3 and Castlevania baselines become "well above always guessing"; depth-alone's 75% becomes "no better than guessing". (c) The balance scale moves before the graph. |
