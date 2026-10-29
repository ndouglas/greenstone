# Video 001, council round 3

Script reviewed: commit 0603339 (revision 3).

## clarity

Round 3, clarity, legibility and coherence lens.

**Round-2 clarity findings:** all 11 are fixed in this revision.
- 1: "floor" is now the level's bottom, and "on the ground" is used only for the answer.
- 2: "weighs" no longer sits beside "weight".
- 3: the-corner is now "top right: low on the screen".
- 4: the-idea goes total, then zero, then border.
- 5: "cut" is gone from speech.
- 6: castlevania-arc is cut.
- 7: why-weights now says what a weight does.
- 8: smb3 now says "comes out at".
- 9: two-inputs now says "the game's memory".
- 10: the two worked examples are split into example-air and example-ground.
- 11: "frames" is defined, nobody-knows comes before answer-keys, and Castlevania's answer key is named.

I checked the new geometry against the stored weights and it is right. The border leans from upper left to lower right, and "higher needs faster" matches it.

**New findings, most important first:**

1. **land-results → land-mistakes → school → scale-again: the climax trails off into asides.**
   - Problem:
     - the-tilt is where the one idea peaks: what the leaning border means.
     - After it come five lines: how the border was found, its score, its mistakes, a school-algebra aside, and the balance scale again.
     - So the last two things the listener hears before all-of-it are two lines that look back.
     - scale-again is also the line that explains *why* the tilt trades height for speed: the speed block changes pans as Mario rises and falls. Arriving last, it is cut off from the-tilt, the claim it explains.
   - Fix:
     - Put scale-again straight after the-tilt, or right after the-border, as the reason the border leans.
     - Move school next to settings-are-borders or the-border, where "straight line" is first said.
     - That way the chapter ends on land-results and land-mistakes, then goes to all-of-it.

2. **land-results: two 75%s side by side, with no comment.**
   - Problem:
     - "Always answering not yet is right seventy-five percent… A setting on his height alone gets seventy-five percent."
     - Heard aloud, the listener will think they misheard, or that the height setting failed and nobody said so.
   - Fix: say what it means in a clause, such as "a setting on his height alone does no better than that". Then the other two scores land as the improvement.

3. **settings-are-borders: "a single point on one number line" refers to a picture that was never described aloud.**
   - Problem:
     - The number line exists only in simplest-neuron's board ("The thermometer becomes a number line").
     - The narration has only talked about a thermostat setting and a height number. So "one number line" is new in speech here, and the narration has to stand without the board.
   - Fix (either works):
     - Say it in plain words: "its setting was one point along the height numbers, with yes on one side and no on the other".
     - Or plant "number line" in simplest-neuron's narration.

4. **the-idea: "On a picture of its inputs" is a new name for the sheet.**
   - Problem: the whole two-number chapter called it "the sheet". In the closing summary, a new phrase makes the listener wonder whether this is a different picture.
   - Fix: "On the sheet of its inputs", or "Drawn on a sheet, like Mario's".

Nothing else substantial. The lines are sayable aloud, the order now builds from one number to two, and the one idea is clear by the end.

## pedagogy

1. **Lines the-tilt, how-found, school and scale-again: the main idea's climax is the only neuron in the video with no numbers, so it still isn't shown why the border is straight.** Round 2's finding 1 was only partly fixed. the-border now says the border is where the total is zero, but "straight" is still just stated. The in-the-air neuron got a weight, a bias, two worked sums and a scale with sized blocks. The landing neuron gets none of these. As a result, school's "its steepness comes from how the two weights compare" has no two weights to compare, and scale-again is a scale with no block sizes.
   - **Fix:** the found border has a clean whole-number form: height number + 5 × speed, lands soon when the total is more than 152. That is a weight of 1 on height, 5 on speed, and a bias of −152. I re-ran it on session 3 with the exp001 data: 90.7%, the same 91% as the found border (90.7% is the found border's session-3 score from round 2). Say it, and work one dot through it (for example, a height number of 140 falling at 4 gives 160, more than 152, so lands soon). Then use the 5 three times:
     - In the-tilt: one step of speed always makes up for five steps of height, at the same rate everywhere on the sheet. That fixed rate is why the border is straight.
     - In school: the 5 against the 1 is "how the weights compare".
     - In scale-again: the blocks get real sizes.

2. **Lines modern, harder, the-idea: the summary comes right after a line saying today's neurons don't answer yes or no.** the-idea begins "One neuron ... says yes if the total is above zero". The viewer has just heard that a common modern neuron "passes on a number" instead of a bare yes or no, so they can't tell which neuron the summary describes. modern also brings in a new mechanism, the bend that is 005's idea, in the second-to-last stretch. The point of finding 1 in round 2's pedagogy section was to make the last thing the viewer works on the idea being wrapped up.
   - **Fix:** put the-idea straight after all-of-it, then modern and harder as the outro. Or open the-idea with "The simplest neuron" or "This neuron". Keep modern brief, as a plant, not as something taught.

3. **Lines choosing and baseline: why we test on a session the setting wasn't chosen on is never said.** "In the third session, the one the setting wasn't chosen on" states the method but not the reason. A lay viewer won't know why holding a session back matters, so the 94%, and every later "tested the same way", reads as a ritual. This is the video's one behavioural test, which the bible plans to show but not name.
   - **Fix:** one clause in baseline: "so we're testing it on play it has never seen, where it can't just have memorized the answers."

4. **Line land-results: "a setting on his height alone gets 75%" equals the always-"not yet" score, and nothing says so.** Heard back to back, the two 75%s sound like a repeat or a mistake.
   - **Fix:** "no better than always guessing".

5. **Lines doubters and harder (minor): the opening's promise is never called back.** doubters ends "To weigh it, you need to know what one piece is." The close never says "now you know what one piece is". The bible's spoken line, "the only neuron in the series we'll understand completely", has been softened into all-of-it and harder.
   - **Fix:** half a sentence in harder that closes both.

All other round-2 pedagogy findings are fixed: the setting is now described as a border, there is one word ("setting"), the other games moved before weights, where the landing border came from is stated, results come after the picture, the scale no longer puns on "weight", school says what m and b were, and place value moved to 002. Nothing else borrows a later video's idea: 004's two cuts and 002's encoding are both gone.

I checked the whole-number rule with a scratch script, /private/tmp/claude-501/-Users-nathan-Projects-ndouglas-greenstone/cf099f6b-6b2a-4507-8759-c42c3f7c29f4/scratchpad/chk.py. Nothing in the repo was changed.

## skepticism

**Skepticism, round 3 (script at commit 0603339)**

**Round-2 skepticism findings: all seven are fixed.**
- **1, memory:** the line now says "main working memory".
- **2, McCulloch and Pitts:** dropped from recipe. simplest-neuron now says "one of the oldest kinds".
- **3, how the landing border was found:** how-found says how it was searched for, and land-results says it was tested on the third session.
- **4, the-tilt:** the claim is now framed as what the border says ("In words, it says…", "to land on the floor in time"). The landing mistakes are reported in land-mistakes.
- **5, castlevania-arc:** cut.
- **6, modern:** now "grows the further past the border… zero on the other side", "usually with a bias".
- **7, recording:** now "the start of the first level".

The round-2 cross-examiner's rounding problem is also fixed: the stored unrounded weights give 90.7% on session 3, which rounds to the 91% the script says.

**Numbers rechecked.** I reran exp001 with its output sent to the scratchpad. Every value in numbers.json reproduces with no differences.
- **its-mistakes:** 962 of the 1,089 errors (88%) are frames where Mario is standing at height 112, 128 or 144. The rest are in-air frames at height 176 or more, mostly falls into pits.
- **land-mistakes:** 453 of the 505 missed moments (90%) come before landings onto something raised. Counted per landing the claim is stronger, not weaker. Of 208 landings, 34 of the 49 raised landings are never flagged at any moment, and none of the 159 floor landings goes completely unflagged.
- **the-tilt:** the border's lean (upper left to lower right) and the "top right" side for "lands soon" match the weights.
- **scale-again:** the pan placements match the signs: both weights are positive and the bias is negative.
- **choosing:** "every possible setting" is effectively true. Only the values that occur are tried, but any setting between them scores the same.

**Nothing substantial.** Two minor points remain. Neither changes a claim the viewer hears.

1. **The cited source contradicts how-found and choosing.** The exp001 citation points to `docs/series/games/001.md` and `tools/series/lab/exp001.py`.
   - **What's wrong:**
     - The exp001.py docstring says "one neuron, placed by hand".
     - The games doc's note for 003 calls the result "the best hand-placed cut", and its "(84% in Super Mario Bros.)" reads as the cut's score, but 84% is Rosenblatt's rule.
     - The doc's opening still says the neuron reads "two *other* numbers", while the script's in-the-air neuron reads one.
     - An expert who follows the citation finds "by hand" where the script says the setting came from a search.
   - **Fix:** in the docstring and doc, say the setting was found by search on sessions 1–2. Attach 84% to Rosenblatt's rule clearly. Say the in-the-air neuron reads one number.

2. **simplest-neuron: "Given just one number, it's a thermostat" is still tied to mcculloch1943.**
   - **What's wrong:** a McCulloch–Pitts unit's inputs are all-or-none (0 or 1), so with one input it can't act as a thermostat on a graded reading like temperature or height. The "adds up, compares with a setting, yes or no" part is now accurate. Only this last clause still stretches past the source.
   - **Fix:** keep it but make clear it describes the threshold idea, not their unit. For example: "Give that kind of neuron a single measurement, and it's a thermostat." Or credit the graded-input form to goodfellow2016.

The files I checked are `/Users/nathan/Projects/ndouglas/greenstone/tools/series/videos/001-one-neuron/script.toml`, `/Users/nathan/Projects/ndouglas/greenstone/tools/series/videos/001-one-neuron/numbers.json`, `/Users/nathan/Projects/ndouglas/greenstone/tools/series/lab/exp001.py` and `/Users/nathan/Projects/ndouglas/greenstone/docs/series/games/001.md`.

## empiricism

Empiricism, round 3: script 001 (one neuron)

**Round 2 is fixed.** I reran exp001 read-only, writing its output to /dev/null. All 102 entries in numbers.json reproduce exactly. Each round-2 empiricism finding is resolved:
- **Landing border (round-2 finding 1):** the-tilt now says "to land on the floor in time", and land-mistakes reports the 90% figure. That figure reproduces: 453 of 505 misses.
- **Where things were chosen and tested (finding 2):** how-found describes the search on sessions 1–2 and the test on session 3. other-games says the other games were "chosen and tested the same way".
- **Castlevania (findings 3 and 4):** castlevania-arc is cut, "falling" is gone, and the board is reworded.
- **Recording (finding 5):** the player is described as weighted, and the data as coming from the start of the first level.
- **Games doc (finding 6):** it now gives 01/05/07 for Castlevania, value 6 for SMB3, and `$058C`.

Other checks held:
- **Stable cuts:** the chosen cut is also the best cut on every single session, in every game: 176, 128 and 176. Its accuracy varies by at most 2.6 points between sessions.
- **Answer key:** SMB1's `$001D` enters state 2 only from state 0, which supports "walking off a ledge".
- **Floor height:** Mario's ground heights are only 176, 144, 128 and 112. He is big on 310 of 54,000 frames, which doesn't change the floor height.
- **Balance-scale signs:** the signs in scale-again match the fitted weights.

New findings, most important first:

1. **land-mistakes: "Its mistakes have the same cause as before" covers only some of the mistakes.**
   - On session 3 the border makes 749 mistakes: 505 missed landings and 244 false alarms.
   - Landings onto something raised account for 453 of the 749 (60%), not all of them.
   - The false alarms are mostly Mario falling at speed 4 and landing 11 to 30 frames later, just outside the ten-frame window. 82 of them come within 60 frames of a death.
   - The 90% the line quotes is correct, but only for misses. The opening clause claims more than that.
   - **Fix:** "Most of the landings it misses (90%) are onto something raised…". Drop "Its mistakes have the same cause", or say "most of its mistakes".

2. **the-corner: "The moments just before a landing gather toward the top right" is true only for landings on the floor.**
   - Of the 2,061 moments before a landing on session 3, 1,571 are floor landings. These average height 155 and speed +3.5, and the border catches 97% of them.
   - The other 490 are landings onto something raised. These average height 117 and speed +0.9, so they are not in that corner, and the border catches only 8% of them.
   - land-mistakes later explains the misses. Still, the board shows "real dots", and as written it would light dots that are not in the corner.
   - **Fix:** "just before a landing on the floor". Or let the board show the raised-landing dots off in the middle.

3. **Minor: the cited write-up and code still describe the setting as placed by hand.**
   - exp001.py's docstring says "one neuron, placed by hand". docs/series/games/001.md line 146 says "the best hand-placed cut". The script, correctly, says the setting was found by trying every value.
   - The games doc, which is what the exp001 citation links to, also leaves out the 90% landing-misses figure that land-mistakes cites. It exists only in numbers.json.
   - **Fix:** change both phrases to "chosen by search on sessions 1–2", and add the 90% figure to the games doc's landing table.

Files:
- /Users/nathan/Projects/ndouglas/greenstone/tools/series/videos/001-one-neuron/script.toml
- /Users/nathan/Projects/ndouglas/greenstone/tools/series/lab/exp001.py
- /Users/nathan/Projects/ndouglas/greenstone/docs/series/games/001.md

## lay

Lay-viewer lens, round 3. All seven of my round-2 findings are fixed:

- **Fixed:**
  - The corner on the sheet is now "top right" and described in the sheet's own directions.
  - "Weighs" no longer sits next to "weight", and "only a block's size matters, its sign picks the pan" is said out loud.
  - "Cut" is gone from the narration.
  - castlevania-arc is cut, and the other games now come before the weights, so the leaning border is the high point just before the close.
  - It's now "the game's memory", not "his memory".
  - SMB3's setting "comes out at the height of that game's floor".
- **Still works:** the opening third, the thermostat, the padlocked answer key, the worked examples, and the mistakes on top of pipes.

New findings, most important first:

1. **the-sheet: "falling above it, rising below it" sounds backwards, and nothing says why.**
   - **What's wrong:** By ear, I'm told falling sits at the top and rising at the bottom, which is the opposite of what my gut expects. The line also says bigger height numbers to the right mean lower on the screen. That's two upside-down ideas in one breath. The previous line said the speed number is positive while falling, but this line never joins that to "bigger numbers higher up the side". So "falling above" sounds like a slip, and I lose the sheet just before the border, which is the high point of the video.
   - **Fix:** Add one clause: "bigger speed numbers higher up, and since falling counts as positive, the falling moments sit above the middle." Or give the speed axis the same "bigger means…" wording the height axis gets.

2. **land-results: "a setting on his height alone gets seventy-five percent" goes by without comment.**
   - **What's wrong:** That's the same score as always saying "not yet", which I heard one sentence earlier. As a listener I either miss it, or I notice it and wonder whether I misheard or whether height is useless. Nobody tells me. It's also four percentages in a row, the densest stretch for the ear.
   - **Fix:** Add a clause: "no better than always guessing". Then say speed alone does better, and both together do best of all.

3. **scale-again: the bias moves pans with no reason given, and the blocks aren't the plain numbers any more.**
   - **What's wrong:**
     - In balance-mario the bias block was 176 in the yes pan. Now "the landing neuron's bias sits in the no pan", and I'm not told why. The rule is that its sign picks the pan, so the reason is that this bias is negative, but the line never says so.
     - In balance-mario the height block was simply Mario's height number, 150. Here the weights are small fractions, so the height block is no longer the height number. A listener will assume it still is.
   - **Fix:**
     - Add "this neuron's bias is a minus number, so it sits in the no pan".
     - Add a half-clause saying each block is the number times its own weight, so the blocks here are small.

4. **answer-keys (minor): "when we ask an artificial neuron a question" comes before I know what a neuron does.** Asking a neuron a question doesn't mean anything to me at that point, but it gets cleared up within a minute by the thermostat and "is Mario in the air?". It only needs a fix if one is cheap, for example "when we use an artificial neuron to answer a question about a game".

Nothing else substantial. I never felt talked down to or made to feel dumb. The y = mx + b line is clearly optional ("if you remember"), and the closing lines land.

## blind

**Blind listener, round 3**

**Earlier findings in this lens:**
- Round 1, item 6 (tying "cut" to "line"): fixed. "Cut" is gone, and settings-are-borders now links a setting to a border.
- Round 2, item 1 (where the corner and sides are, and which way the border slants): fixed. the-sheet gives the directions in sheet terms, the-corner says top right, the-border names the top-right side as "lands soon", and the-tilt says the border runs from upper left to lower right. I checked the slant against `smb1_land_weights`: both input weights are positive, so a bigger height number needs a smaller speed to stay on the border. The slant and the "lands soon" side are both correct.
- Round 2, item 2 ("cut" as jargon, and the border tied to one-number neurons): fixed by settings-are-borders.
- Round 2, item 3 (the scale's pans not named): fixed. scale-again names the pans and says the speed block switches pans.
- Round 2, item 4 (the-idea mixing the total and the border): fixed. The total and zero are in the first sentence, the border on the picture in the second.
- Round 2, items 5 and 6: the place-value example moved to 002, and the smb3 line now says "comes out at".

**New findings:**

1. **the-tilt (minor): "lower" and "higher" switch between the sheet and the screen in consecutive sentences.**
   - **What's wrong:**
     - "Leans from the upper left down to the lower right" uses sheet directions.
     - The next sentences, "the higher above the floor Mario is…" and "A bit lower can make up for a bit slower", use screen directions.
     - A listener still holding the sheet can hear "a bit lower" as lower on the sheet. On the sheet, lower already means slower, so the sentence sounds circular, and the trade-off it is meant to state is lost.
     - the-corner has a milder version of this: "top right of the sheet: low on the screen". It is labelled, so it works, but it is the same collision.
   - **Fix:** avoid "lower" and "higher" for Mario's place on the screen right after sheet directions. For example: "the further above the floor Mario is, the faster he has to be falling… A bit closer to the floor can make up for a bit slower."

Nothing else substantial. Wherever a picture is planned on the board, the narration either describes it or can be followed without it.

## cross

Round 3 review from the cross-examiner lens: two substantial findings and one minor one. I checked each against the session-3 test data with the stored, unrounded landing weights.

**Round 2 follow-up.** My round-2 findings 1, 2, 3, 4, 6, 7, 8 and 9 are fixed:
- 1 is fixed: the landing border's search and test are stated, and the weights are stored unrounded. They reproduce 90.7%.
- 3 is fixed by cutting castlevania-arc.

Finding 5 is fixed in recipe and why-weights. What remains of it in simplest-neuron is finding 3 below.

**New findings, most important first:**

1. **land-mistakes: "Its mistakes have the same cause as before" claims more than its number shows.**
   - The 90% figure counts only one kind of mistake: missed landings, where the neuron said "not yet" and Mario landed. It leaves out false alarms, where it said "lands soon" and he didn't.
   - On session 3 the border makes 749 mistakes: 505 misses and 244 false alarms. So about a third of its mistakes aren't covered by the number.
   - Misses onto something raised (90% of 505, about 455) are about 61% of all mistakes, not "its mistakes".
   - The false alarms have a different cause. About 190 of the 244 are at falling speed 4. A few dozen are below the floor (height over 176), where Mario is falling into a pit.
   - Fix: say "most of the landings it misses (90%) are onto something raised". Drop "its mistakes have the same cause", or report the share of all mistakes (about 6 in 10). Exp001 should compute that share if it's spoken.

2. **land-results against the-corner and the-tilt: "height alone gets 75%" is the same as always answering "not yet" (75%), and the script never says so.**
   - A sharp viewer will ask: if height alone does no better than doing nothing, how can "low on the screen" be half of what marks a coming landing (the-corner), and why does the border lean with height (the-tilt)?
   - The data has an answer. Moments just after take-off are also low on the screen, but rising. So height says something only together with speed. Landing frames have median height 149 and speed +4; "not yet" frames have median 128 and speed −2.
   - Unsaid, the listener hears two lines that seem to contradict each other.
   - Fix: one clause in land-results, such as "height alone does no better than always saying 'not yet', because Mario is low just after he jumps too; it helps only alongside his speed."

3. **simplest-neuron (minor, left over from round 2's item 5): "Given just one number, it's a thermostat" sits under the McCulloch and Pitts (1943) citation.**
   - Their units take on/off inputs, not a graded number like a temperature or a height.
   - So the thermostat comparison is the narrator's extension, but the citation makes it read as theirs.
   - Fix: say the 1943 units took on/off signals and that later versions take any number. Or mark the thermostat sentence as framing, not cited.

Everything else I checked holds up against exp001.py, numbers.json and the games doc:
- choosing, its-mistakes, the other games and the baselines;
- the signs and pans in scale-again;
- the lean in the-tilt (the border runs from about height 127 at speed +5 to height 177 at speed −5);
- the modern line, and the recording description. Castlevania's random player uses its own button mix, which is close enough to "the same way".

## Responses (revision 4)

- The landing border in whole numbers (height + 5 × speed > 152, 91%, measured in exp001), with a worked example, and the reason it's straight ("one step of speed always makes up for 5 steps of height, at the same rate everywhere"); school uses the 5 against 1. (pedagogy 1)
- Order: school next to the tilt; the chapter ends on results and mistakes; the-idea follows all-of-it, and modern and harder are the outro. scale-again cut (the whole-number form now carries the tilt; trims the video to about 12 minutes). (clarity 1, pedagogy 2)
- land-results: height alone "does no better than that". (clarity 2, pedagogy 4, lay 2, cross 2)
- settings-are-borders: "one point along the height numbers, with yes on one side and no on the other". (clarity 3)
- the-idea: "on the sheet of its inputs". (clarity 4)
- baseline says why we test on the third session. (pedagogy 3)
- harder opens "Now you know what one piece is", closing the opening's promise. (pedagogy 5)
- land-mistakes: "Most of the landings it misses, 90%, …"; the games doc adds that these are 60% of all its mistakes, and what the false alarms are. (empiricism 1, cross 1)
- the-corner: "just before a landing on the floor"; described in sheet terms. (empiricism 2, blind 1)
- the-sheet: "bigger numbers higher up; falling counts as positive, so falling moments sit above the middle". (lay 1)
- land-in-words: "further above the floor", "a bit closer to the floor". (blind 1)
- simplest-neuron cites McCulloch–Pitts for adding signals and comparing with a setting; the thermostat comparison is a separate framing line. (skepticism 2, cross 3)
- answer-keys: "when we use a neuron to answer a question about a game". (lay 4)
- exp001's docstring and the games doc no longer say "by hand"; the 84% is attributed to Rosenblatt's rule; the doc says the in-the-air neuron reads one number. (skepticism 1, empiricism 3)

Not acted on: lay 3 (block sizes on scale-again) is moot now that scale-again is cut.
