# Video 001, adversarial review 2

Script reviewed: commit 95a99b2 (revision 4). Findings, ranked:

1. height / the-sheet / the-corner / land-in-words (emphasis, handwaving): "height number" runs opposite to its name, and the sheet's vertical axis puts falling at the top; two inversions the viewer must translate every time. Fix: rename to match the direction (e.g. "how far down he is"); flip the sheet so up means rising, or say outright that up means falling.
2. recipe / formula / bias-is-setting / the-border (unsupported): Goodfellow ch. 6 has no yes/no threshold unit and no single-unit border; ch. 1 §1.2.1 describes the McCulloch–Pitts neuron as a linear model tested for sign; Rosenblatt 1958 is listed but cited by no line.
3. simplest-neuron / one-measurement (incorrect; framing with a claim): McCulloch–Pitts inputs are on/off; "give it a single measurement and it's a thermostat" is a wrong factual claim under a framing tag. Fix: later threshold units and perceptrons take ordinary numbers; cite them; drop framing.
4. the-tilt / school / bias-is-setting / our-numbers (missing steps): the landing rule is given in thermostat form, never as a bias of −152; the depth weight changes from −1 to +1 unexplained; "the bias is the setting" holds only because the weight is −1. Fix: 1 × height + 5 × speed − 152 > 0; say why the weight is now positive; show −h + 176 > 0 is h < 176.
5. balance / balance-mario (minor): the weight's sign vs. the block's sign. Fix: "minus 1 times 150 is minus 150".
6. how-found (minor): fitting by search vs. "letting a neuron find its own border is a later video" needs the difference said; the bible's pass-7 note says "placed by eye". Fix: one clause; update the bible.
7. frames (minor): cite NESdev timing (NTSC is about 60.1).
8. never-see (minor): a checkable claim (exp001's check_inputs enforces it); cite exp001, not framing.
9. land-results (minor): "because Mario is usually still in the air" is muddled; say most in-air moments are more than a sixth of a second from landing.
10. the-corner (minor): the board would light raised landings that aren't in the top right; light only floor landings, or tint the raised ones.
Side note: the games doc's "false alarms are mostly fast falls landing just after the window" is wrong (142 of 222 land 21–59 frames later).
Checked and fine: 94%/176/55%, 88% at 112/128/144, 91%/75%, the other games, Data Crystal, BlueDot, Hendrycks; no clichés.

## Responses to both adversarial reviews (revision 5)

- "Depth" (how far down the screen, bigger is lower) and "falling speed" (positive when falling) replace "height number" and "speed"; the sheet reads naturally (top right: deep and falling fast) with no inversions to explain. (R1 6, R2 1)
- The landing neuron in recipe form: 1 × depth + 5 × falling speed − 152 > 0, weights 1 and 5, bias −152, "rounded to whole numbers"; a line on why the depth weight is now plus one; the worked example uses the bias; "the border drops one step of falling speed for every 5 steps of depth". (R1 2, R2 4)
- land-mistakes uses the whole-number border's figure (85%), counted as moments before a landing. (R1 4)
- balance-mario: "minus one times 150 is minus 150, so a block of 150 goes in the no pan". (R1 1, R2 5)
- Citations: the threshold unit, the recipe, the formula, the bias and the border cite Goodfellow ch. 1 §1.2.1 and Rosenblatt 1958; ch. 6 only for modern; nobody-knows adds ch. 1; frames cites NESdev timing ("about sixty"); never-see cites exp001. (R1 3, 5, 8; R2 2, 7, 8)
- simplest-neuron is now "the simplest kind of artificial neuron, called a threshold unit"; the one-measurement framing line is merged into it and cited. (R1 3, R2 3)
- our-numbers shows −depth + 176 > 0 is the same test as depth < 176. (R2 4)
- baseline: "new play, though on the same stretch of the same level". (R1 7)
- how-found says the difference from learning (a program tries borders from outside; a neuron adjusting its own weights after each mistake is a later video); the bible's pass-7 note updated. (R2 6)
- land-results: "most moments in the air are more than a sixth of a second from landing". (R2 9)
- the-corner's board tints raised landings separately. (R2 10)
- Games doc: the false-alarm sentence corrected; the 85% added. (R2 side note)
