# Video 001, adversarial reviews, thirteenth pass

Script reviewed: commit 60d59f8. Both reviewers found the same single
substantial problem, introduced by the twelfth pass's fix; everything else
was minor. Fixed.

| Finding (reviewer) | Change |
|---|---|
| testing-end: "a second number won't fix that for a neuron like this one" is broader than anything tested (only falling speed was; the game's own in-the-air byte obviously would, which is 003's "cheating byte") and states a result with no reason (1, 2; substantial) | The clause is cut: "So it goes wrong because all it knows about is the floor. But other questions need two numbers to answer well…"; its board shows only the new question and the empty slot |
| one-dot-4b seems to contradict why-zero ("any fixed number would work") (2) | "Here, zero matters. Any fixed number gives a neuron the same answers, but only a test against zero survives this tripling…"; board "below the 100 mark" |
| every-session-games doesn't parse by ear (1, 2) | "choosing the setting on any one session by itself gives the same setting every time" |
| nobody-knows-3 "is how we'd check" overstates BlueDot ("one path"); board "a neuron's lit dots" (1, 2) | "is one way we'd check"; "the network's lit dots" |
| depth's board: the ruler's reading doesn't match where it points; two marks (2) | One convention: a ruler whose zero starts 32 dots below the top, so its reading at Mario's feet is the game's number |
| castlevania-2's board prints 176 and 192 unexplained (2) | Draws the setting where it is, a little above where Simon usually stands, without the numbers |
| why-zero-b's board slides a "> 100" the formula never showed (1) | Shows the formula with "> 100" first |
| frames-3: a freeze isn't "the game moving him" (1) | "takes Mario out of the player's hands" |
| recipe-end gave away what draw-intro teases; draw-intro's "the first step" (2) | recipe-end ends "Next, we draw it as a line."; draw-intro without "the first step" |
| modern-2 two thoughts (2); found-intro's dangling modifier (2) | The textbook clause cut; "it lets us check" |
