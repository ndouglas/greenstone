# Video 001, council round 2

Script reviewed: commit a012eeb (revision 2).

## clarity

Round 2, clarity, legibility and coherence lens.

**Round-1 clarity findings:** these are fixed: 1, 2, 3, 4, 7, 8, 9, 11, 12, 13, 14 and 16. Three are only partly fixed:
- **Finding 5 (the one idea in plain words):** the wording is now "border", but the closing line brings back a new muddle (finding 4 below).
- **Finding 10 (Castlevania):** Simon is introduced and "dropping" is gone, but the line still has no clear consequence (finding 6 below).
- **Finding 15 (smaller points):** "frame" is still not defined at its first use (finding 11 below).

**New findings, most important first:**

1. **choosing / the-neuron / its-mistakes / smb3: "ground" means two different things.** "On the ground" is the answer category, meaning standing on anything. But "the ground" is also used for the floor of the level. choosing says the setting is "Mario's height number when he's standing on the ground". the-neuron says "higher than the ground". Then its-mistakes says "he's on the ground, but higher up", and smb3 says "the height of the ground". A listener can't tell whether standing on a pipe counts as "on the ground". The round-1 fix to use "on the ground" everywhere caused this. Fix: call the bottom surface "the floor" in choosing, the-neuron, smb3 and their boards. Keep "on the ground" for the answer only.

2. **balance-mario / scale-again: the "weight" pun is now said aloud.** Here is the sentence: "the height block weighs a hundred and fifty, in the no pan, because its weight is negative." It uses "weighs" for a block's heaviness and "weight" for the multiplier, one after the other. scale-again adds "scaled by its weight" and "outweigh". The lay-viewer lens raised this in round 1 (lay viewer 3) and it was never addressed. Fix: on the scale, talk about how heavy a block is ("a block of a hundred and fifty"), and keep "weight" for the multiplier only. Or say once that the scale's heaviness and the neuron's weight are different things.

3. **the-picture: "toward one corner: low down, and falling fast" points the wrong way.** On the sheet, height goes along the bottom and speed goes up the side. "Low down" means low on the screen, which is a big height number, toward the right of the sheet. "Falling fast" is a big positive speed, toward the top. So the corner is the top right, but a listener hears "low down" and pictures the bottom of the sheet. Fix: name the corner in sheet terms ("toward the far, top corner: big height numbers and big falling speeds"), or say "low on the screen".

4. **the-idea: the one-sentence idea is muddled.** "Answers by which side of a straight border the total puts you on." The total is a single number. The border lives on the sheet of inputs, so it is the inputs, not the total, that sit on one side or the other. Also, the script never says that with one input the border is just the single cut. So all-of-it's "the straight border they draw" doesn't clearly cover the Mario height neuron. Fix:
   - In the-idea, say the inputs put you on one side of the border, or that the total is above or below zero. Pick one.
   - Add one clause, in the-border or all-of-it, saying that with one number the border is just the setting, a single cut on the number line.

5. **land-baseline: "cut" is used without ever being introduced.** Before this line the script says "setting". Then "a single cut on his height" arrives with no explanation. That makes four words for related ideas: setting, cut, border, line. Fix: say "a single setting on his height" here, or introduce "cut" once in choosing or the-neuron ("the setting cuts the height numbers in two").

6. **castlevania-arc: wrong section, and an ambiguous key phrase.**
   - It sits under "Back to the first question, in the air or not", but its point is about the second, two-number question. The listener has to make that jump unaided.
   - "No speed number to give a second neuron input" can be heard as "a second neuron". It should mean "a second input for the neuron".
   - "The number its memory map calls his vertical speed" uses "memory map", which is never explained.
   - "Same question, different numbers" doesn't land, because the line hasn't said which question it means.
   
   Fix: open with "The landing question can't be asked the same way here". Say "so the neuron has no speed number to use as its second input". Replace "memory map" with "the published guide to its memory". Or cut the line.

7. **why-weights: the reason for weights doesn't follow.** "To let a neuron read more than one, each number it reads gets a multiplier." The line doesn't say why reading two numbers needs multipliers rather than simply adding them up. Then the first neuron the viewer meets with a weight reads only one number, and has a weight of minus one. Fix: one clause saying the weight is how much each number counts, and which way it pushes, so that different numbers can count for different amounts.

8. **smb3: "also lands on the height of the ground" comes right after the landing section.** In this script, "lands" means Mario touching down, so here it sounds like the neuron or Mario landing. Fix: "also comes out at the height of the floor".

9. **two-inputs: "a second number from his memory".** Mario has no memory. The opening was careful to say the numbers are in the game's memory. Fix: "from the game's memory".

10. **our-numbers: too many numbers in one breath.** It packs about eight spoken numbers into one line, including "one hundred and seventy-six" three times, the second worked example and its minus four. It's sayable, but it's the densest line in the script. Fix: split it into two lines, one per example, so each answer gets a pause.

11. **Minor points:**
    - **recording:** "frame" is never defined at its first use; "sixty frames a second" assumes the viewer knows the word. Saying "sixty pictures a second" once would fix it. The first sentence is also one long chain of five clauses; it would read better split in two.
    - **answer-keys:** "things that learn" and "a learning machine" come before the script has said what is being studied (AI, artificial neurons, in nobody-knows). Swapping the two lines, or calling it "the systems we'll study", would make the order flow.
    - **castlevania:** it never says that the jumping/falling/knocked-back number is that game's answer key.

## pedagogy

Pedagogy lens, round 2 (script tools/series/videos/001-one-neuron/script.toml)

**Round-1 pedagogy findings, checked:**
- **Fixed:**
  - 1: a real tilted line now beats either input alone, and the two-cuts material has moved to 004.
  - 2: place value is now "the adding-up half".
  - 4: negative blocks go in the no pan, and our own neuron is worked on the scale.
  - 5: there are two spoken worked examples, and the bias is said to take over the setting's job.
  - 6: one account of how the setting was chosen.
  - 8: "why games" is answered.
  - 9: the doubters line now names its argument.
- **Partly fixed:**
  - 3: y = mx + b is now hedged, but it is still claimed rather than shown (finding 7 below).
  - 9: the "why a straight line" intuition is still missing (finding 1).

**Findings, most important first:**

1. **the-border, the-idea, all-of-it: the link at the centre of the one idea is never made.**
   - Nothing says why "total above zero" turns into "one side of a straight line on the sheet". The border is never described as the place where the total comes out exactly zero.
   - So the viewer is asked to accept that the border is straight, not shown it.
   - the-idea then mixes the two levels. "Which side of a straight border the total puts you on" treats the total, a single number, as a point on the sheet. The bible says it the right way round: yes or no by which side of zero the total lands, and with two inputs that is a line.
   - Fix:
     - In the-border, add one sentence: "the border is every point where the total comes out exactly zero; on one side it's above zero, on the other below."
     - Add one sentence of intuition for why it's straight, using the trade-off already in the-tilt.
     - Reword the-idea to: total against zero first, then the border on the sheet.

2. **land-baseline, smb3, castlevania, all-of-it: "cut" appears without being introduced, and the first neuron is never called a border.**
   - Until land-baseline the spoken word is always "setting". Then "a single cut on his height" arrives without explanation, and smb3 and castlevania go back to "setting".
   - all-of-it says "the straight border they draw", but the in-the-air neuron the viewer spent half the video on drew no line. Nobody says that a one-input setting is the same idea: a border on a number line instead of on a sheet.
   - Fix:
     - Pick one word and use it throughout.
     - When the sheet appears, call back: "our first neuron's setting was a border too, a single point on one number line; with two numbers, the border becomes a line."

3. **other-games, smb3, castlevania, castlevania-arc: the order drops back to an easier idea after the climax.**
   - The tilted two-input line is the peak of the one idea. Straight after it, the script returns to one-input in-the-air neurons in two more games, then a six-input aside.
   - The last thing the viewer worked on before the wrap-up is not the idea being wrapped up.
   - Fix: move the other games to straight after its-mistakes, which is still the one-input chapter. The arc then climbs: one input across three games, then weights, then two inputs and the border, then the ending.

4. **land-line: the landing line's weights arrive with no account of where they came from.**
   - The script took care to say how the height setting was chosen: every value tried on the first two sessions, the best kept. For the two-input neuron it just says "each with its own weight, the neuron gets 91%".
   - A viewer may assume the machine learned them. Learning is 003's idea, and leaving it vague is the handwave the bible's 001 entry warns against.
   - The code searched directions and positions on the first two sessions (best_line in tools/series/lab/exp001.py).
   - Fix: one clause such as "we tried many tilts and positions on the first two sessions and kept the best". Optionally say that letting the neuron find them itself is a later video.

5. **land-line, before the-picture and the-border: the result comes before the thing it scores.**
   - 91% is announced before the viewer has seen what the neuron does: the sheet, the border and the tilt.
   - Fix: picture, then border, then tilt, then the 91% against the three bars.

6. **balance-mario, scale-again: "weight" means two things.**
   - Round-1 lay-viewer 3 raised this pun, and it remains. "The height block weighs 150, in the no pan, because its weight is negative" uses "weighs" for the block's size and "weight" for the multiplier in the same sentence.
   - scale-again's "outweigh the bias" quietly assumes the bias sits in the no pan. That was never said for the landing neuron, whose bias is negative, and the speed block switches pans while Mario rises.
   - Fix:
     - Give the block's size its own word, such as "heaviness", or say "its weight, the multiplier, is minus one".
     - In scale-again, say the bias block sits in the no pan.

7. **school: the y = mx + b link is still claimed rather than shown, though it is now hedged.**
   - A viewer who half-remembers school algebra is told the letters don't match but not what m and b meant: steepness, and where the line crosses the side.
   - Fix: half a sentence, "in school, m was the steepness and b where the line crossed the side", followed by the existing mapping.

8. **place-value: the bias is never said aloud.**
   - "Add a bias and the comparison with zero" does not give the bias. Only the board shows minus 1,000.
   - This is the one place the viewer could reuse the bias-as-setting rule on their own.
   - Fix: say "a bias of minus a thousand, in the no pan". That also brings the scale back in speech.

9. **castlevania-arc: an aside that also borrows 002's idea.**
   - "Same question, different numbers" is about how games store facts, which is encoding, an idea the bible's curriculum gives to 002.
   - "No speed number to give a second neuron input" is also unclear, because the two-input question was landing, not in-the-air.
   - Fix: either cut the line, or cut it to one sentence that points forward ("how a game stores a fact matters; that's the next video").

10. **Minor:**
    - smb3: "also lands on the height of the ground" comes right after the landing section, so "lands" reads as the landing question. Use "settles at" or similar.
    - the-picture: "Here's what that neuron looks like" goes on to describe the data, not the neuron. Use "here's what that question looks like", and leave "the neuron" for the-border.

No other later-video ideas are borrowed:
- The modern line plants the bible's "distance from the line as confidence" plant (paid off with sigmoid) without teaching it.
- Choosing a setting by search does not teach learning.
- Using the answer key only to check the neuron does not take 008's probing.

## skepticism

**Skepticism, round 2 (script at revision 2)**

Round-1 skepticism findings 1–6 and 8–12 are fixed. I checked the numbers against the recordings and they hold up:
- **SMB1 mistakes:** 1,089 test errors. 962 of them (88%) are on-ground frames at heights 112, 128 or 144.
- **SMB1 restarts:** the recorder restarts on the frame of death, so dying frames are about one per death.
- **SMB3 dying flag:** `$058C` is set on one frame per death.
- **Castlevania:** `$03C4` is zero on every frame, and `$0488` counts 0 to 39 during state 01.

Hendrycks, BlueDot and the Data Crystal entries for `$001D`, `$009F`, `$07DD`–`$07E2`, `$03C4`, `$0488` and `$046C` say what the lines claim.

Findings, most important first:

1. **memory: round-1 finding 7 was marked fixed but isn't in the script.** The text still says the NES "runs its games out of a small working memory: two thousand and forty-eight numbers". The response says "main working memory" was adopted, but the line doesn't contain it. An NES expert will object: there is also video RAM, sprite memory, palette RAM and cartridge RAM, and SMB3 (used in this video) has 8 KB of cartridge RAM. Fix: "its main working memory holds 2,048 numbers", or "the memory we'll read".

2. **simplest-neuron / recipe: McCulloch and Pitts is cited for things their paper doesn't contain.** Their 1943 unit takes several all-or-nothing (0/1) inputs, counts the active ones against a threshold, and lets any inhibitory input veto. It does not read one graded number like a height. It has no multiplying weights and no bias. So recipe's citation of mcculloch1943 for "multiply it by its weight … add one more number, called the bias" is wrong. simplest-neuron's "takes in a number, compares it with a setting" only fits if "number" means a count of yes/no inputs. Fix: cite McCulloch–Pitts only for "a threshold unit that answers yes or no" and drop it from recipe, which Goodfellow alone covers. Or say "one of the oldest kinds", built on McCulloch and Pitts's threshold.

3. **land-line: the script never says how the landing neuron's weights were chosen, or that 91% was tested on unseen data.** For the first neuron, the script explains that the setting was tuned on two sessions and tested on the third. For this neuron, "each with its own weight" leaves the viewer free to assume we placed the line by hand or tuned it on the test. In fact the code tries 360 tilts × 199 positions on sessions 1–2 and scores the best on session 3. Fix: one clause, e.g. "we tried many tilts and positions on the first two sessions and kept the best; on the third it's right 91% of the time."

4. **the-tilt: "The higher Mario is, the faster he has to be falling to land in time" states the line's rule as a fact about Mario.** The fitted line (0.0335·height + 0.1683·speed − 5.084 > 0) needs:

   | Height number | Speed needed for "lands soon" |
   |---|---|
   | 160 | more than −1.6 |
   | 140 | more than 2.3 |
   | 120 | more than 6.3 |

   Speed never goes above 5 (fastest fall, per Data Crystal and the data), so above a height of about 128 (three blocks over the floor) the neuron says "not yet" at any speed. It also can't foresee landings on blocks and pipes. Real fall paths are curves, and the line is the best straight approximation. Fix: "The neuron's border says: the higher Mario is, the faster he has to be falling…"

5. **castlevania-arc: "So there's no speed number" is stronger than the evidence.** What we observed is that the byte the map lists as vertical speed (`$03C4`) was zero on every frame. As round 1 noted, a byte that is always zero also looks exactly like a wrong address. Fix: "we found no speed number", or "the speed the map lists never moves". The step counter (`$0488`) itself could also serve as a second input, so "no … second neuron input" could become "no speed to give it".

6. **modern (minor): "a number that says how far past the border the input is".** Strictly, w·x + b is proportional to that distance, not equal to it. A ReLU outputs zero on the "no" side and sigmoid-type units saturate. Given "close cousins", this is defensible. An optional fix is "a number that grows the further past the border the input is", which matches the board.

7. **recording (minor):** "play the first level" is broader than the data. The player never got past about 1,440 pixels into the level (the first ~6 screens of 1-1, out of about 3,300 pixels). The games doc says "the opening of the first level". Fix: "the start of the first level".

Not a finding:
- **smb3:** "lands on the height of the ground, 128 in that game's numbers" is supported. 14,511 of 18,733 ground frames are at 128, and speed is non-zero on only 1% of ground frames now that deaths are cut out.

## empiricism

**Empiricism, round 2: script 001 (one neuron)**

**Round 1 is fixed.** I reran the lab read-only and every number in `numbers.json` reproduces. Each round-1 finding is resolved:
- **SMB3 deaths:** `$058C` dying frames equal the restart count exactly in each session: 68, 76 and 75.
- **Castlevania knockback:** state 05 is now counted as in the air.
- **its-mistakes:** reproduces as 962 of 1,089 errors, all at heights 112, 128 or 144.
- **Castlevania speed slot and jump counter:** `$03C4` is 0 on all 54,000 frames. `$0488` counts up through each jump in all 400 jumps.
- **choosing:** the search is now described as it was done, and the cut chosen on the third session alone would also be 176.
- **"Every straight line"** is gone.
- **recording:** the random player is now disclosed.

New or remaining findings, most important first:

1. **the-tilt / the-picture / land-line: the landing border only knows the floor, and the script presents it as a fact about how Mario lands.**
   - On the third session, 453 of the line's 505 missed landings (90%) are landings on a block, pipe or platform (heights 112, 128 and 144). Of the 490 moments just before such a landing, it catches only 37. Of the 2,061 moments just before any landing, it misses 505 (25%).
   - "The higher Mario is, the faster he has to be falling to land in time" holds only for landing on the floor. Mario high up over a block lands soon without falling fast.
   - "Gather toward one corner: low down" has the same problem: landings on raised ground aren't in that corner.
   - This is the same failure the script reports honestly for the first neuron (its-mistakes), but here it goes unstated.
   - **Fix:** say "higher above the floor" / "to land on the floor in time". Add a line about the mistakes, and have exp001 output the share (about 90% of misses are landings on something raised).

2. **land-baseline / land-line / smb3 / castlevania: the script doesn't say where these numbers were chosen and tested.** The two-number line was found by search on sessions 1–2 and scored on session 3. The SMB3 and Castlevania cuts were chosen the same way. The script states this only for the first neuron. A viewer could think the 91% border was placed by hand, or that it was scored on the data it was fitted to.
   - The SMB3 and Castlevania recordings are never described either. SMB3 restarts about every 4 seconds, so its data is only the first few seconds of its first level.
   - **Fix:** add one clause each ("found the same way, on the first two sessions, and tested on the third"), plus a clause on how the other games were recorded.

3. **castlevania-arc: "So there's no speed number to give a second neuron input" overreaches.** Only one address (`$03C4`, the one the map names) was checked. The jump counter `$0488`, which we did verify, could itself serve as a second input.
   - **Fix:** "so we have no speed number to give it" or "we found no speed number".
   - Related, in the castlevania line: "a number says whether he's … falling" comes from the RAM map. State 07 ("dropping") appears on 0 of 54,000 frames, so falling was never seen in our recordings. Say "the map says", or drop "falling".

4. **castlevania (board) contradicts the data.** The board says "the cut at Simon's feet", but the cut (176) isn't where Simon usually stands.
   - In the third session's ground frames, Simon stands at 192 on 6,702 frames, at 176 on 4,978 and at 196 on 368. The cut sits at the higher of two ground levels.
   - All 1,454 Castlevania test errors are in-the-air frames at height 176 or lower on the screen (the low part of jumps). That is the reverse of Mario's mistakes.
   - Also, the smb3 line ("also lands on the height of the ground") invites the viewer to assume the same for Castlevania.
   - **Fix:** correct the board. Optionally say that this cut misses the bottom of Simon's jumps.

5. **recording / baseline: the player isn't just random.** It is weighted: about 75% of its button choices include Right and 45% include A. The code comment says it was "tuned so a few minutes of play jump often but not always". So the 45/55 air/ground split, and the "score to beat", come from that tuning, not from how Mario is normally played.
   - "Play the first level" also overstates: with restarts about every 8 seconds, all the data comes from the level's opening stretch.
   - **Fix:** "random buttons, mostly running right and jumping" and "the start of the first level".

6. **Minor: the cited write-up is out of date in two places.**
   - `docs/series/games/001.md` still says Castlevania's "airborne means `$046C` is 01 or 07". The code and the correction note use 01, 05 and 07. That page is what the `exp001` citation points to.
   - SMB3's `$00D8` takes the value 6 on about 0.8% of frames (fast falls just after being in the air). The key counts these as "in the air", which is probably right, but the write-up documents only the value 1.
   - **Fix:** correct the doc and note value 6.

Files: `/Users/nathan/Projects/ndouglas/greenstone/tools/series/videos/001-one-neuron/script.toml`, `/Users/nathan/Projects/ndouglas/greenstone/tools/series/lab/exp001.py`, `/Users/nathan/Projects/ndouglas/greenstone/docs/series/games/001.md`

## lay

Round 2, lay-viewer lens: findings ranked by importance.

**Round-1 findings that are now fixed (not repeated below):** the picture is anchored, the weights have a reason, the bias is tied to the old setting, there are worked examples, the scale handles negative numbers, place value is now "the adding-up half", the sign is explained again, Simon is introduced, the "by hand" contradiction is gone, doubters is paid off, and "artificial neurons" / "networks" are clearer. The first third and the Mario test still work well.

1. **the-picture: "low down, and falling fast" points to the wrong place on the paper.** I've just been told height goes "along the bottom" and speed "up the side". So when I hear "one corner: low down", I picture the bottom of the paper. But low down in the game means a big height number, which is the far right. Falling fast is a big positive speed, which is the top. The corner is actually top-right, and I'm looking bottom-left, so the border that follows makes no sense to me. **Fix:** describe the corner in terms of the paper ("toward the far right and the top: big height numbers, meaning low on the screen, and big falling speeds"), or say "low on the screen" so it isn't heard as low on the page.

2. **balance / balance-mario: the word "weight" means two things, and a negative block "weighs 150".** "The height block weighs a hundred and fifty, in the no pan, because its weight is negative." In one sentence, "weighs" means heaviness and "weight" means the multiplier. On top of that, balance just said a block is "input times its weight", which here is minus 150, yet now it weighs plus 150. I'd have to stop and untangle this, and I'd feel dumb doing it. **Fix:**
   - Say once that on the scale only how big a block is matters, and its sign picks the pan.
   - Avoid "weighs" next to "weight". For example: "the height block is a hundred and fifty, and its minus sign sends it to the no pan."

3. **land-baseline: "a single cut" is a term I've never heard.** All through the Mario section, the narration said "setting". "Cut" exists only on the board, so by ear it arrives undefined, right when three percentages are coming at me. **Fix:** say "a single setting on his height" / "on his speed", or define "cut" once back at choosing.

4. **castlevania-arc: still feels like trivia, and it's hard to follow.**
   - "The number its memory map calls his vertical speed": "memory map" is new jargon.
   - "No speed number to give a second neuron input" can be heard as "a second neuron".
   - "Same question, different numbers" doesn't tell me why I should care.

   **Fix:**
   - Replace "memory map" with "the published guide to the game's memory".
   - Reword to "no speed number to give the neuron as a second input".
   - Give a one-clause payoff ("so in this game, the landing neuron wouldn't work as it is"), or cut the line.

5. **Pacing after the climax (other-games, smb3, castlevania, castlevania-arc, score, place-value).** The tilted border is the high point. After it, the script steps back to the easier first question in two more games, then a Castlevania memory detail, then a new score example, before the ending. That's six lines of stepping back, and this is where I'd drift off. **Fix:** tighten this stretch. Possible moves:
   - Fold smb3 and castlevania into one sentence of results.
   - Cut or shrink castlevania-arc.
   - Move the other games before the landing question, so the tilted border comes just before the close.

6. **two-inputs: "a second number from his memory".** That sounds like Mario has a memory. It's a small slip, but it jars after the script worked hard to set up "the game's memory". **Fix:** "from the game's memory".

7. **smb3: "also lands on the height of the ground".** Coming right after a section about Mario landing, I hear this as Mario landing, not the setting ending up at ground level. **Fix:** "its best setting also turns out to be the ground's height, 128 in that game's numbers".

Nothing else substantial. our-numbers is dense by ear, but the arithmetic (176 − 150, 176 − 180) is easy enough, and the worked examples were the right call.

## blind

**Blind listener, round 2**

**Round-1 check:**
- Fixed: 1 and 2 (the two-cuts / no-help material moved to 004), 4 (our-numbers now has spoken worked examples), 5 (the-neuron says "less than … meaning he's higher than the ground"), 7 (other-games give the neuron's score first and always-guess second), 8 (school says the match is loose and maps tilt and shift in words), 9 (gone).
- Fixed in part: 3. The sheet is now anchored, but the corner and side within it are not; see finding 1.
- Not fixed: 6. "Cut" and "line/border" are still never tied together, and "cut" now turns up unintroduced; see finding 2.

**Findings, most important first:**

1. **the-picture / the-border / the-tilt: the listener can't place the corner, the side or the slant.**
   - **What's wrong:**
     - "Gather toward one corner: low down, and falling fast" says "low down" about Mario on screen. On the sheet he described a second earlier, a listener hears "low down" as the bottom of the sheet. In fact a low Mario has a big height number, so he sits to the right. Falling fast means a positive speed, so the dot sits near the top. The corner is top-right, and the words point to the bottom.
     - Nothing says that rising moments sit below the middle of the sheet and falling ones above it. Most listeners will put "going up" at the top.
     - the-border says "on one side of it … lands soon; on the other, not yet" but never says which side.
     - the-tilt gives the trade-off but not which way the line slants across the sheet.
   - **The result:** the line meant to carry the video's one idea can't be pictured.
   - **Fix:**
     - Name the sheet's directions in sheet terms: bigger height numbers (lower on screen) further right; falling near the top, rising near the bottom, zero speed across the middle.
     - Put the landing moments in the top-right corner, without "low down".
     - Say the "lands soon" side is the top-right.
     - Say the border runs from high on the left down to low on the right.

2. **land-baseline: "a single cut" is jargon the ear hasn't been given, and it isn't linked to the border.**
   - **What's wrong:**
     - Until this line the script says "setting", and castlevania goes back to "one setting". "Cut" appears for the first time here, unexplained.
     - A listener isn't told that a one-number neuron draws a border on this same sheet. A height-only setting is a straight up-and-down line, and a speed-only setting is a level line across. Knowing that is what makes "tilted" mean something in the-tilt.
   - **Fix:**
     - Use "setting" (or define "cut" once, as the setting).
     - In the-picture or the-border, add one clause: a height-only neuron would draw a straight up-and-down border, a speed-only one a level border, and using both lets the border lean.

3. **scale-again: the pans aren't named, so the listener can't rebuild the scale.**
   - **What's wrong:**
     - balance and balance-mario set up yes and no pans with sign rules. Here "together they outweigh the bias" leaves the pans to the screen.
     - The landing neuron's weights are positive and its bias is negative, so both input blocks go in the yes pan and the bias in the no pan.
     - When Mario is rising, his speed is negative, so the speed block moves to the no pan. That goes unmentioned.
   - **Fix:** say both blocks normally go in the yes pan and the bias sits in the no pan. Add a clause that while Mario is rising, the speed block switches to the no pan.

4. **the-idea: "which side of a straight border the total puts you on" mixes two pictures.**
   - **What's wrong:**
     - The total is one number, compared with zero.
     - The border lives on the sheet of inputs.
     - A listener who built the sheet can't put "the total" on it.
   - **Fix:** "which side of a straight border its inputs fall on" (the total's sign tells you which).

5. **place-value (minor): the bias's value and pan are only on the board.**
   - **What's wrong:** "Add a bias" doesn't say what bias answers "more than a thousand points".
   - **Fix:** "a bias of minus a thousand, in the no pan". This also brings the scale back in speech.

6. **smb3 (minor, by ear): "also lands on the height of the ground" comes straight after the landing question.**
   - **What's wrong:** a listener can hear "lands" as Mario landing.
   - **Fix:** "also comes out at the height of the ground".

## cross

Round 2, cross-examiner lens. Ranked findings below.

**Round-1 items now fixed:** 1, 2 (except point 4 below), 4, 5, 7, 8, 9, 11, 13 and 14. Items 3, 6 and 12 are only partly fixed; what's left of them is folded into findings 2, 3 and 5.

1. **land-line (and the-picture, the-border).** The script never says where the landing neuron's two weights and bias came from, or what data its 91% was measured on.
   - For the first neuron, the script says we tried every setting on sessions 1–2 and tested on session 3. Here it says nothing, so a viewer could assume the weights were set by hand, or that the neuron learned them. Neither is true: `best_line` tried 360 directions times 199 cuts on sessions 1–2, then tested on session 3 (90.7%).
   - The rounded weights in numbers.json (`smb1_land_weights` = 0.0335, 0.1683, −5.084) score only 88.9% on session 3, not 91%. Height and speed are whole numbers, and the found border runs exactly through many data points, so rounding flips those frames. Any border drawn from numbers.json would not match the spoken figure.
   - Fix: add one clause ("we tried hundreds of thousands of tilted borders on the first two sessions, kept the best, and tested it on the third"). Store the weights unrounded, or move the border off the data points.

2. **recording / baseline / land-baseline.** "Pressing random buttons" undersells how the player was built. The `HumanLike` player in record.rs picks from a weighted list: Right is in about 75% of picks and A is in at least 35%. The code comment says it was "tuned so a few minutes of play jump often but not always".
   - So the 55% and 75% scores to beat are partly set by that tuning, not by Mario.
   - "Play the first level" also overstates: all play stays in the opening of 1-1.
   - Fix: "a program that mostly runs right and often jumps, picking its buttons at random", and "the start of the first level".

3. **castlevania-arc.** "The number its memory map calls his vertical speed never changed" is now defensible. The sentences around it are not.
   - "Instead of a speed" and "there's no speed number to give a second neuron input" claim the game stores no speed at all. The evidence is one address (`$03C4`) staying zero.
   - "Step by step, up to 39" is also stronger than the evidence. exp001 measures only the maximum of `$0488` while jumping, not that it counts one step at a time.
   - Fix: "we found no speed number to give it". Either measure that the counter goes up by one each step, or just say "a counter that reaches 39".

4. **modern.** Two words carry more than the source supports.
   - "The same weighted sum *and bias*": many of today's large models leave out biases entirely. Goodfellow et al. (2016) comes before those models.
   - "How far past the border": a ReLU passes on the total itself. That total is proportional to the distance, not the distance, and it is zero for everything on the "no" side.
   - Fix: "a weighted sum, often with a bias", and "pass on the total itself, or zero if it's on the no side".

5. **simplest-neuron / recipe / why-weights.** The citations and the causal claim don't hold up.
   - simplest-neuron cites McCulloch and Pitts for a unit that "takes in a number" and compares it with a setting. Their units take several on/off inputs, not one graded number.
   - recipe cites mcculloch1943 for the weights-and-bias recipe. That paper has neither graded weights nor a bias.
   - why-weights says weights are what let a neuron read more than one number. Adding inputs already does that; weights set how much each one counts, and with what sign.
   - Fix: "adds up its inputs and compares the total with a setting". Drop mcculloch1943 from recipe. In why-weights, say "and decides how much each one counts".

6. **scale-again.** "A landing is coming when together they outweigh the bias" breaks the pan rule set up in balance.
   - While Mario rises, the speed block is negative, so by that rule it sits in the "no" pan with the bias, not "together" with the height block.
   - "Outweigh the bias" also depends on the bias being negative (−5.08), which is never said.
   - Fix: "a landing is coming when the yes pan is heavier", or state the signs.

7. **choosing vs its-mistakes.** "Ground" means two things. In choosing, 176 is "exactly Mario's height number when he's standing on the ground", which is the floor. But the answer key's "on the ground" includes pipe tops at 112–144, and its-mistakes says "he's on the ground, but higher up".
   - Fix: say "the floor" wherever the script means height 176.

8. **two-inputs.** "A second number from his memory" makes Mario the owner of the memory. Fix: "from the game's memory".

9. **Minor.**
   - **harder:** "networks of thousands of them" contradicts nobody-knows' "enormous numbers". Today's systems have billions.
   - **castlevania (board note):** the board's "the cut at Simon's feet" isn't right. The cut is 176, but Simon normally stands at 176–196 (median 192), so the cut sits at the bottom of his standing range, not at his usual feet. It also matches SMB1's 176 only by coincidence.

I checked these against tools/series/videos/001-one-neuron/numbers.json, tools/series/lab/exp001.py, src/nes/gym/series/record.rs and the recordings in target/series/001-one-neuron/data/smb1/.

## Responses (revision 3)

- memory: "main working memory" is now actually in the line (it had been marked fixed in round 1 but wasn't; my error). (skepticism 1)
- McCulloch–Pitts is cited only for "adds up what it's given, compares with a setting, yes or no" ("one of the oldest kinds"); dropped from recipe, which Goodfellow covers. why-weights says weights set how much each number counts and which way it pushes. (skepticism 2, cross 5, clarity 7)
- One word for one-input borders: "setting"; "cut" is gone from speech. A new line ties settings to borders (a point on one number line; up-and-down or level borders on the sheet; reading both, it leans). "Floor" for the level's bottom; "on the ground" only for the answer. (clarity 1, 5; pedagogy 2; lay 3; blind 2; cross 7)
- The other games moved into the one-input chapter, before weights; castlevania-arc cut (encoding is 002's idea; kept in the games doc). The climax is now the leaning border, just before the close. (pedagogy 3, 9; lay 4–5; clarity 6)
- The sheet is described in sheet terms (height along the bottom, bigger to the right; speed up the side, falling above the middle, rising below); the landing moments are "top right"; the lands-soon side is named; the border's lean is given (upper left to lower right). (blind 1, lay 1, clarity 3)
- the-border says the border is where the total is exactly zero; the-idea goes total → zero → border. (pedagogy 1, clarity 4, blind 4)
- The landing border's origin and test are stated (many leaning borders tried on sessions 1–2, the best kept, tested on 3), and results come after the picture. The tilt is stated as what the border says, "to land on the floor in time", and a new number reports that 90% of its misses are landings on something raised. Weights are stored unrounded. (pedagogy 4–5, skepticism 3–4, empiricism 1–2, cross 1)
- The scale: only a block's size matters, its sign picks the pan; no "weighs" beside "weight"; scale-again names the pans and the speed block's switch. (clarity 2, pedagogy 6, lay 2, blind 3, cross 6)
- The recording: "picking buttons at random, mostly running right and often jumping"; "the start of the first level"; "frames" defined. (skepticism 7, empiricism 5, cross 2, clarity 11)
- school says what m and b were in school. (pedagogy 7)
- modern: "usually with a bias"; "a number that grows the further past the border, and zero on the other side". harder: "huge numbers of them". (skepticism 6, cross 4, 9)
- "the game's memory" (not his); smb3's "comes out at the height of that game's floor"; Castlevania's key is named as its answer key, without "falling". (clarity 8–9, 11; lay 6–7; blind 6; empiricism 3)
- Place value moved to 002 (how games store numbers), which also brings 001 under 12 minutes. (pedagogy 8, blind 5)
- The ground example is now Mario on the floor (176 gives zero, not above zero); the old "180, sinking into a pit" example was wrong (a Mario sinking into a pit is in the air).
- Games doc: Castlevania's key 01/05/07 (07 never seen); SMB3's value 6; `$058C` documented. (empiricism 6)

Not acted on:
- Castlevania board ("the setting at Simon's feet"): reworded to "the setting across the screen"; where Simon stands relative to it is an animation detail. (empiricism 4, cross 9)
