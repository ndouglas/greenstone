# Video 001, adversarial review 2, second pass

Script reviewed: commit 4609e7a (revision 5). Earlier fixes hold except the citations.

1. the-tilt / how-found (handwaving): "rounded to whole numbers" hides a scaling step (the found weights 0.033, 0.168, −5.08 were divided by 0.033 first); the viewer never learns that multiplying every weight and the bias by the same positive number leaves the border where it is. Fix: say so.
2. recipe / formula / bias-is-setting / the-border (unsupported, introduced by the last revision): Goodfellow ch. 1 §1.2.1 (as the reviewer knows it; the page didn't render) has f(x, w) = Σ xᵢwᵢ tested for sign, no bias, nothing about a straight border; Rosenblatt 1958 has a threshold θ, no bias, no geometry; ch. 6 has W⊤x + b. Fix: cite ch. 6 for the bias; cite a source that states a linear threshold unit's boundary is a line (e.g. Bishop 2006 §4.1.1, or Minsky & Papert 1969), or exp001 plus a geometry source.
3. two-inputs: the sign convention isn't stated by Data Crystal; the recordings confirm it (92% of frames; the rest zero at the top of the jump). Fix: cite exp001.
4. Minor framing claims: thermostat (real ones switch around a band; "a simple thermostat" or cite); settings-are-borders (follows from what's shown; cite or accept as derived).
5. memory (low): "each between 0 and 255" and "changing many times a second" are true but not in the NESdev page.
Bible (not the script): 001's entry still says "placed by eye" / "hand-placed line tested on a recording it has never seen"; the table row says "from two numbers"; "Introduces: place value" contradicts the move to 002; "Said out loud: the only neuron we'll understand completely" isn't said.
Checked and fine: every number and the arithmetic, depth's direction, the answer key, restarts, the floors, Castlevania, why-plus, steady-trade, school, modern; no clichés.
