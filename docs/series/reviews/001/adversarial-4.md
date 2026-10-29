# Video 001, adversarial reviews, fourth pass

Script reviewed: commit 4974f9d (revision 7).

## Reviewer 1

**Video 001, fourth adversarial pass (script at commit 4974f9d, revision 7)**

The earlier fixes hold. I reproduced these from the seed-3 recording:
- **Landing border:** the whole-number border scores 90.7%. The single cut on depth alone scores 74.8%, against a baseline of 74.5%, so "no better" is fair. The cut on falling speed alone scores 84.0%.
- **Worked examples:** at depth 140±5, 87% of moments with a falling speed of 4 land within ten frames (281 moments), and 8% with a falling speed of −2.
- **Missed landings:** all 532 land at depths 112, 128 and 144 (454, which is 85%) or at 176 (78). These are block and pipe heights, with no enemy stomps among them.
- **Recordings:** all play stays in world 1-1, with Mario's furthest position at 1,438 of its roughly 3,300 dots. The random player matches "mostly running right and often jumping".

Two of revision 7's own changes brought in new problems (findings 1 and 2). Ranked:

1. **depth** (incorrect in emphasis, introduced by the units fix; the nesdev-oam citation doesn't support it).
   - **Not where he is:** $00CE is not where Mario is on a screen that runs from 0 to 240. In the recording's sprite table, small Mario's feet are at $00CE + 32 (about 208 on the floor, with the byte at 176). His head is 16 dots below the value. Small Mario is 98% of frames.
   - **Wrong range:** the bottom of the screen is about 208 on this scale, not 240. The byte also goes past 240, up to 255, when he falls into a pit.
   - **How it misleads:** a viewer who hears "zero to about two hundred and forty" puts 176 at three-quarters of the way down the screen, 32 dots above where the floor is drawn.
   - **The boards inherit the offset:**
     - the depth board's "marker follows Mario";
     - the-neuron's "setting drawn … at the floor";
     - its-mistakes' "setting line is below his feet". On the 144 pipe his feet are at screen 176, so a line drawn at 176 sits at his feet, not below them.
   - **Fix:** "counted in dots down from the top of the screen, measured from a point just above his head", and drop "to about two hundred and forty". Cite exp001 for the offset; the OAM page only covers sprite rows. Every board should draw depth d at screen row d + 32.

2. **school** (incorrect; introduced by the last fix).
   - **No such crossing on the sheet:** "school's b, where the line crosses the side, is … about thirty" describes a crossing the viewer never sees. In-air depths start around 55 (1st percentile) and falling speed only runs from −5 to +5. The border reaches falling speed 30 only at depth 0, far above the sheet, and the sheet's side is not at depth 0.
   - **Steepness looks wrong:** "steepness minus one fifth" will look nothing like a gentle −1/5. The two axes have very different scales (about 200 depth across, 10 speed up), so the drawn border looks steep.
   - **Fix:** "where the line would cross depth zero, far off the top of our sheet". Also add a clause: the steepness is counted in the sheet's own steps (one step of speed per five of depth), so it can look steeper than a school graph.

3. **Bible vs script** (out of order). The bible's 001 "Plants for later" lists three plants the script doesn't contain:
   - "shuffled inputs change nothing";
   - "weights over pixels as a picture (007)";
   - "the widest-gap border (margin)".

   Only "distance from the border" is planted, in the modern line. Fix: plant them in the script, or move them out of 001's entry.

4. **how-found** (handwaving, low). "Kept the one that matched that answer most often" has no clear antecedent: the landing answer was last mentioned in new-question, six lines earlier. Fix: "that matched the look-ahead answer most often".

5. **harder** (factual claim marked framing, low). "Networks built from huge numbers of them are much harder to see into" is a claim. Fix: cite bluedot2024.

6. **castlevania** (uncited, low). "Simon, a monster hunter with a whip" isn't in the Data Crystal RAM map, which is the only game source cited. Fix: cite a game source, or accept it as on-screen common knowledge.

7. **two-inputs** (emphasis, low). $009F holds only whole dots per frame; the fraction is in a separate byte. "A falling speed of 4" really means 4 to 5 dots per frame. Fix: "in whole dots per frame".

No clichés or filler. Nothing else out of order against the bible.

Files read:
- /Users/nathan/Projects/ndouglas/greenstone/tools/series/videos/001-one-neuron/script.toml
- /Users/nathan/Projects/ndouglas/greenstone/tools/series/videos/001-one-neuron/numbers.json
- /Users/nathan/Projects/ndouglas/greenstone/tools/series/videos/001-one-neuron/citations.toml
- /Users/nathan/Projects/ndouglas/greenstone/tools/series/lab/exp001.py
- /Users/nathan/Projects/ndouglas/greenstone/src/nes/gym/series/record.rs
- /Users/nathan/Projects/ndouglas/greenstone/docs/series/games/001.md
- /Users/nathan/Projects/ndouglas/greenstone/docs/series/bible.md

## Reviewer 2

**Video 001, fourth adversarial pass (script at commit 4974f9d, revision 7)**

**What I checked from the third pass**
- All the third pass's findings are fixed as its responses describe. The script now:
  - defines weight in simplest-neuron and no longer says "with a weight of one";
  - gives the thermostat reason in our-numbers;
  - names the bias of minus 152 again, with the reason scaling doesn't move it, and cites Bishop;
  - says in new-question that the recording, not the game, supplies the landing answer;
  - says the school rearrangement aloud;
  - sizes the balance blocks;
  - gives depth and falling speed units, with exp001 cited on depth.
- **Arithmetic:** 5.03 rounds to 5 and 151.9 rounds to 152. Both worked examples come out at 8 and minus 22. The school line's 30.4 is right.
- **Recording claims:** the random player and restart-on-death claims match `src/nes/gym/series/record.rs`. Castlevania's state 07 ("dropping") never occurs in the recordings, so describing its answer key as jumping or knocked back is accurate for this data.

Nothing badly misleading is left. One medium finding came in with the last fixes; the rest are minor.

1. **school** (incorrect, medium; introduced by the revision 7 fix). "School's b, where the line crosses the side, is 152 divided by 5, about thirty" points the viewer at something they can't see on the sheet.
   - School's b is where the border reaches depth zero, at a falling speed of 30.
   - On the real sheet, depths run from about 41 to 217 and falling speeds only from minus 5 to plus 5.
   - Inside the sheet, the border runs from depth 127 at the top to depth 177 at the bottom. It never reaches the side, and 30 is six times higher than the sheet's top.
   - A viewer who looks at the side of the sheet for "about thirty" won't find it.
   - Fix: "school's b, where the line would cross depth zero if we stretched the sheet far to the left and far up, is 152 divided by 5, about thirty: off our sheet, and a different number from the neuron's bias." Or drop school's b and keep only m, which the sheet does show.

2. **new-question** (incorrect in detail, low). "We look ten frames ahead and see whether the in-the-air number says he's landed" sounds like a check of the tenth frame only.
   - `landing()` in exp001 asks whether he is on the ground at any of the next ten frames.
   - Checking only the tenth frame gives a different label on about 2% of in-the-air moments: he lands and jumps again inside the window.
   - The phrase "our recording does [store that answer]" also overstates it. The answer is worked out from the recording, not stored in it.
   - Fix: "The game doesn't store that answer, but we can work it out from the recording: we check the next ten frames and see whether the in-the-air number says he's landed in any of them."

3. **why-plus** (handwaving, low). "With the weight plus, the setting goes in as a minus bias" states a sign rule without its reason.
   - The viewer can take away "a bias's sign is always the opposite of the weight's". That only holds here because both settings are positive depths.
   - The actual step: moving the setting to the other side of the comparison flips its sign. Depth plus 5 times falling speed above 152 is the same test as depth plus 5 times falling speed, minus 152, above zero. The same step gave plus 176 in our-numbers.
   - Fix: "Moving the 152 to the other side of the comparison turns it into minus 152, the bias, just as moving 176 across turned it into plus 176 before."

4. **the-tilt / why-plus, all-of-it / the-idea** (filler, low).
   - The-tilt gives the landing rule twice in one line: once as "weights of one and 5, a bias of minus 152", then again in full. Why-plus then says it a third time.
   - All-of-it and the-idea are back-to-back summaries of the same content: weight, bias, border.
   - Fix: in the-tilt, keep the "If one times the depth…" sentence and cut the list before it. Merge all-of-it into the-idea, keeping "we can see all of it, and we know exactly what it does", which the bible requires.

5. **modern** (not supported by its citation, low). "For years the standard choice" claims a length of time and hints that the standard has since been replaced.
   - Goodfellow 2016 (chapter 6) only says the rectified linear unit is "the default recommendation" in modern networks. It supports neither the span of years nor any later change.
   - Fix: "in one common kind, long recommended as the default", or cite a source for the later shift.

6. **how-found** (unclear wording, low). "Kept the one that matched that answer most often": the last mention of "the answer" is in new-question, four lines earlier, so "that answer" has nothing near it to point back to.
   - Fix: "kept the one that most often matched whether he really landed within ten frames".

Nothing is out of order against the bible, I found no clichés, and every number reproduces from exp001 and the seed-3 recording.

## Responses (revision 8)

- depth: "counted in dots from near the top"; the 0–240 range dropped; the games doc records the offset (feet at `$00CE` + 32) so animations draw depth d at screen row d + 32. (R1 1)
- school: m counted in the sheet's own steps; school's b "would be where the line crosses depth zero, far off our sheet". (R1 2, R2 1)
- new-question: "we can work it out from the recording … in any of them". (R2 2)
- why-plus gives the reason for the sign: moving 152 across the comparison, as 176 was before. (R2 3)
- the-tilt says the rule once; all-of-it merged into the-idea, which keeps "we can see all of it, and we know exactly what it does". (R2 4)
- modern: "long recommended as the default". (R2 5)
- how-found: "most often matched whether he really landed within the next ten frames". (R1 4, R2 6)
- harder cites BlueDot; two-inputs says "whole dots per frame". (R1 5, 7)
- Bible: plants not in the script moved to "ideas not used in 001". (R1 3)
Not acted on: Castlevania's "a monster hunter with a whip" (R1 6) is on-screen common knowledge about the game, not a claim the RAM map needs to support.
