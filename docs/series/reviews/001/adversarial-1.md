# Video 001, adversarial review 1

Script reviewed: commit 95a99b2 (revision 4). Findings, ranked:

1. balance-mario vs balance (incorrect): balance says a block's own sign picks the pan; balance-mario says the weight's sign does. Matters when the speed input is negative. Fix: "150 times minus one makes minus 150, so the block goes in the no pan, at a size of 150."
2. the-tilt / school / land-example (handwaving): the landing neuron is given in thermostat form ("more than 152"), never as weights 1 and 5 and bias −152; "in whole numbers" hides a rounding; "5 against 1" invites reading the slope as 5 (on the sheet it's −1/5). Fix: "1 × height + 5 × speed − 152 > 0: weights 1 and 5, bias −152", say it's rounded, and give the drop (one step of speed per five of height).
3. simplest-neuron → one-measurement (incorrect; framing with a claim): McCulloch–Pitts inputs are all-or-none, so "give it a single measurement and it's a thermostat" isn't true of their neuron. Fix: "the simplest kind", cite Goodfellow ch. 1 §1.2.1 (which describes the McCulloch–Pitts neuron as a weighted sum tested for sign), or call it a linear threshold unit.
4. land-mistakes (incorrect / unsupported): the 90% is for the unrounded border; for the rounded one shown, 454 of 532 missed frames, 85%. And it counts frames, not landings (242 landings on seed 3). Fix: compute for the rounded border; "of the moments before a landing that it gets wrong, about 85%".
5. recipe / bias-is-setting / the-border / why-weights (unsupported): Goodfellow ch. 6 has w⊤x + b, weights, biases and ReLU, but no yes/no threshold, no bias-as-setting, no straight border. rosenblatt1958 is listed but cited by no line. Fix: cite Rosenblatt 1958 and/or Goodfellow ch. 1 §1.2.1 for the threshold and border lines; keep ch. 6 for modern.
6. height (emphasis): "height number" runs the opposite way to its name, forcing repeated "bigger means lower". Fix: a name that goes the right way ("distance from the top", "down-the-screen number").
7. baseline (emphasis): "play it has never seen" overstates: same player, same opening stretch. Fix: "new play, on the same stretch of the same level".
8. Citation gaps: nobody-knows ("built from enormous numbers of artificial neurons" needs Goodfellow, not BlueDot or 1943); frames ("sixty pictures a second" needs NESdev frame timing).
Checked and fine: the answer key's transitions, 8 seconds, the floors, Castlevania, the worked arithmetic, doubters, modern's ReLU; no clichés; nothing out of order.
