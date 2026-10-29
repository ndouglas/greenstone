# Video 001, adversarial reviews, third pass

Script reviewed: commit 24097d9 (revision 6).

## Reviewer 1

**Video 001, third adversarial pass (script at commit 24097d9, revision 6)**

The earlier fixes hold:
- The figures reproduce: 962 of 1,089 mistakes is 88%; the whole-number border scores 91%.
- The 88% mistakes really are Mario on pipes and blocks. They sit at depths 112, 128 and 144, at the pipe and block positions in 1-1, all with player state 8 (none are deaths).
- The-corner holds. Floor landings have median depth 155 and falling speed 4; raised landings have median depth 114.
- Falling speed runs from −5 to +5, so zero is the middle of the sheet.

Two of revision 6's own changes brought in new problems (findings 1 and 2). Ranked:

1. **simplest-neuron** (incorrect; introduced by the latest fix). "Given just one number, with a weight of one, it's a thermostat" is backwards.
   - Under the recipe's rule (yes when the total is above the setting, or above zero), a weight of one says yes *above* the setting. That is an air conditioner.
   - The thermostat says yes (heat on) *below* its setting. Matching it takes a weight of minus one. That is exactly what our-numbers does later, which now seems to contradict this line.
   - Fix: drop "with a weight of one" ("Given just one number, it works like a thermostat"). Then in our-numbers say: "The weight is negative because, like the thermostat, the yes answer is below the setting: smaller depths."

2. **the-tilt / bias-is-setting / school** (handwaving; a regression from the last revision). Revision 6 cut "weights of one and five, and a bias of minus 152".
   - The landing neuron's bias is now never named. School's "changing the bias slides the border" points at a number the viewer was never told is the bias.
   - Bias-is-setting and our-numbers (bias 176 = setting 176) teach that the bias *is* the setting. Here the setting is 152 and the bias is −152, and the flip is never explained.
   - Fix: restore "weights of one and five, and a bias of minus 152". Add one clause to why-plus: "and with the weight plus, the setting goes in as a minus bias".

3. **new-question / how-found / land-results** (handwaving; contradicts answer-keys). Answer-keys says "the game's own memory holds the right answer to check it against".
   - No byte in the game's memory holds "will he land in the next ten frames?". exp001 works it out by looking ten frames ahead in the recording at the in-the-air number (`landing()`).
   - The script skips this step, yet how-found says borders were scored against "the answer key".
   - Fix, in new-question: "The game doesn't store that, but the recording does: we look ten frames ahead and check whether the in-the-air number says he's landed."

4. **why-weights** (handwaving). "Those weights are the new part" refers back to simplest-neuron, 13 lines and a whole Mario section earlier. The Mario neuron was stated as "depth less than 176", with no weight. A listener has nothing for "those" to point to.
   - Fix: "Back to the weights from the start: they're the new part…" or "What makes a neuron more than a thermostat is its weights."

5. **school** (handwaving for the stated audience). "Work out our border for the falling speed" asks someone who hasn't done algebra since school to rearrange 1·depth + 5·speed − 152 = 0 into speed = −depth/5 + 30.4 in their head.
   - Fix: say it outright: "Along the border, the falling speed is 152 minus the depth, divided by five: so its steepness is minus one fifth…". Or drop "work out" and rest on the steady-trade line already given.

6. **the-tilt** (unsupported, minor). "Multiplying every weight and the bias by the same positive amount doesn't move the border" cites only exp001, which doesn't state it, and gives no reason.
   - Fix: add the one-clause reason ("a total of zero stays zero, and a positive total stays positive") and cite bishop2006 alongside.

7. **new-question** (emphasis, low). "A question that needs two numbers" overstates it: falling speed alone gets 84% against a 75% baseline.
   - Fix: "a question where two numbers do better than one".

Bible side note (not the script): 001's "Plants for later" still lists "scaling the weights doesn't move the border (003)", but the-tilt now says it out loud. Move it to "Said out loud".

No clichés or filler. Nothing out of order against the bible.

Files read: `/Users/nathan/Projects/ndouglas/greenstone/tools/series/videos/001-one-neuron/script.toml`, `/Users/nathan/Projects/ndouglas/greenstone/tools/series/lab/exp001.py`, `/Users/nathan/Projects/ndouglas/greenstone/docs/series/bible.md`

## Reviewer 2

**Third adversarial pass on video 001: findings, most misleading first**

All the earlier findings hold. The bias citations (Bishop §4.1.1 and Goodfellow ch. 6), "picture a simple thermostat", the scaling clause, the "modern" hedge, the exp001 citations and the 85% are all in place. I reproduced these numbers from the seed-3 recording:
- **Accuracy and examples:** the landing border's 91%, its 85% of misses before raised landings (454 of 532), and both worked examples (87% and 8%).
- **The 88%:** in the "is he in the air?" test, 962 of 1,089 mistakes are Mario standing on something at depth 112, 128 or 144: pipes and blocks.
- **The answer key:** it has no 1→2 transition, so "two means falling after walking off a ledge" is accurate.

Two findings below come from the latest fixes: #1 from the 1b-5 fix, and #2 is made worse by it.

1. **simplest-neuron: incorrect (introduced by the last fix).** "Given just one number, with a weight of one, it's a thermostat" contradicts the script's own rule. Under "yes if the total is above zero" (recipe), a weight of +1 says yes *above* the setting. That is the opposite of the thermostat, which turns the heat on below it. The thermostat-shaped neuron is the Mario one, and our-numbers says it needs a weight of minus one for exactly this reason. A viewer who remembers this line will find our-numbers contradicting it.
   - Fix: drop "with a weight of one", or say "with a weight of minus one", which our-numbers then pays off.

2. **simplest-neuron → why-weights: handwaving and order.** "Gives each one a weight" uses "weight" with no meaning. It isn't defined until why-weights, 14 lines later. That line opens "Those weights are the new part" right after the Castlevania line, so "those" has no antecedent the viewer can still recall. The whole Mario section runs with an undefined term in the air.
   - Fix: define it where it first appears ("a multiplier saying how much each number counts"), or keep simplest-neuron weight-free and reopen why-weights with "The simplest neuron also gives each input a weight…".

3. **school: handwaving.** "Work out our border for the falling speed" asks a viewer who hasn't done algebra since school to rearrange 1·depth + 5·speed − 152 = 0, and never shows how. There are also two letter clashes:
   - The recipe used x for "each input". Here school's x is depth and its y is falling speed, both inputs. Only the b clash is acknowledged.
   - "The two b's aren't the same number" still doesn't say what school's b is (152 ÷ 5, about 30). 1b-3 asked for it to be said with our numbers.
   - Fix: say it outright: "falling speed equals minus one fifth of depth, plus 152 fifths: m is minus one fifth, and school's b is 152 fifths, about 30". Name which input plays y and which plays x.

4. **why-plus / the-tilt: handwaving.** bias-is-setting teaches "the bias takes over the setting's job". our-numbers then gives bias = +176 for a floor at 176. The landing neuron's bias is −152, and nothing says why the sign flipped. why-plus explains only the weight. The script never says "the bias is minus 152" either; the viewer has to pull it out of "minus 152".
   - Fix: add one clause to why-plus: "and with the weight flipped, the bias flips too: minus 152, so the total passes zero when depth plus five times speed passes 152."

5. **the-tilt / how-found: handwaving and order.**
   - "Multiplying every weight and the bias by the same positive amount doesn't move the border at all" is stated with no reason. It's a one-clause step: every total gets multiplied too, so it's above zero in exactly the same places (doubling land-example's 8 gives 16, still yes).
   - "Here is the border we found" comes five lines before how-found says how it was found.
   - Fix: add the clause, and move how-found to just before the-tilt.

6. **balance / balance-mario: emphasis.** The neuron's "weight" and the scale's "heavier" collide. In "Each input times its weight makes a block… yes when the yes pan is heavier", the heavy thing is |w·x|, not w. A lay viewer can easily read "the pan with the bigger weights wins".
   - Fix: give the blocks a size ("a block as big as that number; the pan holding more wins"), or say outright that a block's heaviness is the product, not the weight.

7. **depth / two-inputs / land-example: unexplained units.** Depth 150, 176 and 140, and falling speed 4 and −2, are never given units. The viewer can't tell whether 176 is near the bottom of the screen or what "a falling speed of 4" means. That weakens the "five steps of depth per step of speed" trade in steady-trade.
   - Fix: say once "counted in dots on the screen, from 0 at the top to about 240 at the bottom" and "dots per frame".

8. **depth: citation (low).** "Counted from the top; bigger means lower" isn't stated by Data Crystal ("Player y pos on screen"). NESdev OAM covers sprite bytes, not the game's own position variable $00CE. The recordings show the direction: 176 on the floor, smaller in jumps.
   - Fix: add exp001.

Bible (not the script): curriculum row 001 still lists "slope and intercept". The script no longer uses "intercept"; it should once #3 names school's b as 152 fifths, otherwise drop "intercept" from the row.

No clichés, nothing else out of order against the bible, and the numbers all match their sources.

## Responses (revision 7)

- simplest-neuron: weights defined where they first appear; "with a weight of one" dropped (it was backwards: a weight of plus one says yes above the setting); our-numbers says the weight is negative because, like the thermostat, its yes is below the setting. (R1 1, R2 1)
- why-weights no longer leans on "those weights". (R1 4, R2 2)
- the-tilt names the weights (1 and 5) and the bias (−152) again, with the reason scaling doesn't move the border (every total is multiplied too); cites Bishop. how-found moved before it. (R1 2, 6; R2 4–5)
- why-plus: "with the weight plus, the setting goes in as a minus bias". (R1 2, R2 4)
- new-question: the game doesn't store the landing answer; we look ten frames ahead in the recording; "a question where two numbers do better than one". (R1 3, 7)
- school: depth plays x, falling speed plays y; the rearrangement is said aloud; m is minus one fifth, school's b is 152 ÷ 5, about 30, a different number from the neuron's bias. (R1 5, R2 3)
- balance: "a block as big as the number"; "the yes pan holds more". (R2 6)
- Units: depth in dots, zero to about 240; falling speed in dots per frame; depth cites exp001. (R2 7–8)
- Bible: scaling moved to "said out loud"; the curriculum row keeps "slope and intercept", now both named. (R1 and R2 bible notes)
