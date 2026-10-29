# Video 001, adversarial reviews, sixth pass

Script reviewed: commit 43245d3 (after Nate's interview). Both reviewers
found substantial problems, nearly all in the interview's new lines, and
largely the same ones. Dispositions below; fixed in the next commit.

| Finding (reviewers) | Disposition |
|---|---|
| why-zero: "a bias 100 lower" heard backwards (1, 2) | Reworded: "Asking whether the total is above 100 is the same as taking 100 off the bias and asking whether it's above zero." |
| why-zero: "the bias carries the whole setting" is only true when the weight is −1; contradicts one-dot (1, 2) | "there's no separate setting to keep: it's folded into the bias" |
| one-dot: "all the answer depends on" is false (×−10 flips every answer); "any neuron" unargued; the border-doesn't-move conclusion lost from the-tilt (1, 2) | one-dot gives the reason (every total ×3; plus stays plus, minus stays minus) and says the line still slopes down; the-tilt says scaling doesn't move the border on the map |
| one-dot: "the first graph" implies the map is a graph; it interrupts the map section (2, minor in 1) | school names "the graph"; one-dot moved to just after school, before the map exists |
| ×10 clashes with the 10-frame window (2); "lit dot" clashes with the map's dots (1) | ×3; a pin, not a dot |
| bias-slides invites "bias = the border's school b" (1, 2); board shows a weight changing the lean, unnarrated (2) | Rewritten through the graph: the bias moves the line, so the crossing slides; on the map the border slides keeping its lean; a weight changes the lean (now narrated) |
| recording's "everything we recorded comes from the start of the first level" false once the steady player is in (1, 2) | "everything it recorded" (the random program); steady-player says it gets through 1-1 into 1-2 (verified: 23% of its frames in 1-2) |
| its-mistakes: "its" now points at the steady player (1, 2) | "of the neuron's mistakes on the third session" |
| not-a-fluke: nothing to compare 94% against; "how we check" overstates (1, 2) | Gives the score on the sessions it was chosen on (new `smb1_height_train_accuracy`, 94%); "one way to check" |
| steady-trade: the 5 no longer derived after school was cut (2) | Derived: going right, 5 more depth adds 5, one less speed × 5 takes 5 away |
| depth credits the game with our zero line (2) | The game measures to a point above his head; his feet are always 32 dots lower |
| the-tilt: "the border's numbers" undefined (2) | "a neuron's weights and bias" |
| why-plus repeats why-zero (2) | Shortened to a callback to the 100 |
| how-found: "a later video" is now the next one (1, 2) | "is the next video" |
| Number and picture load: twelve percentages; ten lines of algebra in a row (2) | For Nate: cut the SMB3 and Castlevania baselines, the depth-alone 75%, or move them to an on-screen scoreboard; consider the balance scale before the graph |
