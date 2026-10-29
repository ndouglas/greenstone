# Video 001, adversarial reviews, fifth pass

Script reviewed: commit 8b583d6 (revision 8). Substantial findings only.

## Reviewer 1

Fifth adversarial pass on video 001, script at revision 8 (commit 8b583d6). The fourth pass's fixes are all in the script:
- **depth:** the 0 to 240 range is gone.
- **school:** the steepness is counted in the sheet's own steps, and school's b is "far off our sheet".
- **new-question:** it checks "any of" the next 10 frames.
- **the-tilt:** it says the rule once.
- **the-idea:** all-of-it is merged into it.
- **modern:** "long recommended as the default".
- **how-found:** its antecedent is now clear.
- **harder:** it cites BlueDot.
- **two-inputs:** "whole dots per frame".

I re-checked the arithmetic by hand: 140 + 20 − 152 = 8, 140 − 10 − 152 = −22, and (152 − depth)/5 gives m = −1/5. The figures match numbers.json and the games doc. I did not re-run the lab.

Two findings, ranked:

1. **why-plus** (incorrect; introduced by the revision 8 fix for R2-3).
   - **What's wrong:** the line says "moving that 152 to the other side of the comparison turns it into minus 152, just as moving 176 across turned it into plus 176 before." That second half is false.
     - our-numbers turned "depth less than 176" into "minus the depth, plus 176, is above zero". What moved across there was the depth, which is why the depth picked up the minus. The 176 kept its plus sign.
     - If you move the 176 across "depth < 176", you get "depth − 176 < 0", which is minus 176. Getting back to plus 176 means turning the whole test round, which is the step that gave the weight of minus one.
   - **How it misleads:** the line states a rule ("moving a number across makes it minus") and then gives an example that seems to break it. A viewer who tries the rule on 176 gets minus 176, while the script says plus 176. That is the same sign confusion the fix was meant to clear up.
   - **Fix:** drop the "just as" clause, or replace it with the true parallel: "Before, 'depth less than 176' became '176 minus the depth is above zero': there the depth moved across and got the minus, so the 176 stayed plus. Here the 152 is what moves, so it's the 152 that gets the minus."

2. **depth, board** (low; a revision 8 fix that the board didn't pick up).
   - **What's wrong:** the board still says "A ruler down the left of the screen, zero at the top".
   - The revision 8 response and the games doc say depth d should be drawn at screen row d + 32. The `$00CE` value is about 32 dots above Mario's feet: 176 on the floor puts his feet at about row 208.
   - **How it misleads:** a ruler with zero at the top edge would put the 176 mark 32 dots above the floor. That is the same visual error the fourth pass's first finding was about, and the narration's "from near the top" would contradict it.
   - **Fix:** "a ruler down the left, its zero just below the top of the screen (screen row 32), so the marker at Mario's feet reads his depth". Apply the same offset to the-neuron and its-mistakes when they're animated.

Nothing else substantial. No clichés, no uncited claims marked framing, and nothing out of order against the bible.

Files read:
- /Users/nathan/Projects/ndouglas/greenstone/tools/series/videos/001-one-neuron/script.toml
- /Users/nathan/Projects/ndouglas/greenstone/tools/series/videos/001-one-neuron/numbers.json
- /Users/nathan/Projects/ndouglas/greenstone/tools/series/videos/001-one-neuron/citations.toml
- /Users/nathan/Projects/ndouglas/greenstone/tools/series/lab/exp001.py
- /Users/nathan/Projects/ndouglas/greenstone/docs/series/games/001.md
- /Users/nathan/Projects/ndouglas/greenstone/docs/series/bible.md
- /Users/nathan/Projects/ndouglas/greenstone/docs/series/reviews/001/ (adversarial-1.md, adversarial-2.md, adversarial-1b.md, adversarial-2b.md, adversarial-3.md, adversarial-4.md)

## Reviewer 2

nothing substantial

I re-ran exp001 on the seed-3 recording to check the fourth pass's fixes and the claims that rest on data. Everything I checked holds:

- **its-mistakes:** the depth-alone neuron makes 1,089 mistakes. 962 of them (88%) are Mario standing at depths 112, 128 or 144, which are block and pipe tops. Every one of those frames has the normal player state, so none of them are death animations.
- **the-corner:** across the three recordings, the moments before a floor landing sit at depths 137 to 173 (5th to 95th percentile) and falling speeds 1 to 4. The moments before landings on raised things sit near depth 114, toward the middle of the sheet.
- **land-example:** at depth 140 (±5), 88% of moments with a falling speed of 4 land within ten frames (845 moments), and 6% with a speed of −2 (120 moments).
- **land-mistakes:** "All this border knows about is the floor" holds. The whole-number border catches 95% of landings onto the floor but only 7% of landings onto raised things. 85% of its missed landings come before landings onto blocks or pipes.
- **school:** "far off our sheet" is right. In-air depths run from about 58 to 174 (1st to 99th percentile) and falling speeds from −5 to +5, while school's b sits at depth 0 and speed about 30.
- **Fourth-pass fixes:** these are all in place: depth "from near the top", new-question's "in any of them", the reason for the sign in why-plus, "long recommended as the default" in modern, the clearer reference in how-found, BlueDot cited on harder, and the plants moved out of the bible's 001 entry.
- **Sources:** BlueDot says "AI researchers currently have very little understanding of what is happening inside state-of-the-art models", which supports nobody-knows and harder.
- **Bible:** I found no lines out of order against it, and no clichés or filler.

## Responses (revision 9)

- why-plus: the false "just as moving 176 across turned it into plus 176" clause is removed; the line keeps the true step (moving 152 across the comparison makes it minus 152). (R1 1)
- depth board: the ruler is drawn with its zero 32 dots below the top of the screen, so the marker at Mario reads his depth; the same offset applies to the-neuron and its-mistakes. (R1 2)

Reviewer 2 found nothing substantial; with these two fixes, the adversarial stage is closed.
