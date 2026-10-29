# Video 001, adversarial review 1, second pass

Script reviewed: commit 4609e7a (revision 5). Every earlier finding fixed in the script; all numbers reproduce; the worked examples are realistic (87% of moments at depth 135–145 with falling speed 4 land within ten frames; 8% with −2).

1. the-tilt (emphasis, handwaving): the search found 0.033, 0.168, −5.08; exp001 divides by the depth weight, then rounds; the viewer is never told scaling every number by the same positive amount leaves the border unchanged. Fix: "written with whole numbers…", plus that clause.
2. recipe / formula / bias-is-setting (unsupported, introduced by the last fix): ch. 1 has no bias term (searched); Rosenblatt uses thresholds; ch. 6 has w⊤x + b. Fix: add ch. 6; the threshold–bias equivalence is shown by our-numbers.
3. school (handwaving, emphasis): b is used for the intercept right after b was the bias; for our border m = −1/5 and the crossing point is 152/5; "the letters don't match one for one" is a deferral. Fix: say it with our numbers, or drop the letter b.
4. the-border (unsupported): neither ch. 1 nor Rosenblatt says a single unit's boundary is a line. Fix: cite a source that does (Minsky & Papert 1969, or a textbook on decision boundaries), or show it from the algebra.
5. simplest-neuron / why-weights (emphasis, unsupported): the cited neuron is weighted from the start; "threshold unit" isn't the sources' name. Fix: "takes in numbers, weighs each one, adds them up…".
6. thermostat (framing with a claim): real thermostats have hysteresis; the bible lists a thermostat citation to find. Fix: cite, or "picture a simple thermostat".
7. modern (emphasis): today's large models mostly use GELU or gated variants (SwiGLU), not zero on the other side. Fix: "a common kind, for years the standard choice".
Bible side note: the 001 entry still has "placed by eye", "hand-placed line…never seen", and place value.
No clichés; nothing out of order.

## Responses to both second passes (revision 6)

- the-tilt: "written with whole numbers"; multiplying every weight and the bias by the same positive amount doesn't move the border, so they were scaled to make the depth's weight one, then rounded. (1b-1, 2b-1)
- Bishop 2006 §4.1.1 (y = wᵀx + w₀, w₀ the bias, the decision boundary a hyperplane) added for recipe, formula, bias-is-setting, the-border and settings-are-borders; Goodfellow ch. 6 restored for the bias. (1b-2, 1b-4, 2b-2)
- school: says m was steepness; our border's steepness is minus one fifth; changing the bias slides it "the way changing b did in school, though the two b's aren't the same number". (1b-3)
- simplest-neuron gives the neuron weights from the start ("gives each one a weight"); "threshold unit" dropped; why-weights says weights are the new part. (1b-5)
- thermostat: "picture a simple thermostat". (1b-6, 2b-4)
- modern: "in one common kind, for years the standard choice". (1b-7)
- two-inputs and memory also cite exp001. (2b-3, 2b-5)
- The bible's 001 entry rewritten to match the script (no "by eye", no place value, what's said out loud, what moved out), and the curriculum row. (1b side note, 2b bible)
