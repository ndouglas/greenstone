# Video 001, adversarial reviews, eighth pass

Script reviewed: commit a5aa9af. Both reviewers found the same substantial
error, in a line written to fix pass 7; reviewer 2 found three more.
Fixed in the next commit.

| Finding (reviewer) | Disposition |
|---|---|
| land-mistakes: "most of the rest say 'lands soon' a little too early" is false: of 222 false alarms, about 120 are Goomba stomps and 78 falls into a pit (1, 2; substantial) | New `false_alarm_kind` in exp001 (test first): 91% are bounces or pits (`smb1_land_false_alarms_bounce_or_pit`). The line: "it says 'lands soon', but he bounces off an enemy or falls into a pit". games/001.md corrected |
| The graph and the map blur: same axis along the bottom, both "drop", "likewise" (2; substantial) | the-map: "It isn't the graph: up the side is now a second number from the game, not the neuron's total"; steady-trade no longer "drops"; bias-slides names both pictures and says the bias moves them "in different ways" |
| "Setting" retired then kept; the bias's sign contradicts why-zero's rule; bias = setting presented as general (2; substantial) | why-zero: "the setting is now wherever the total crosses zero"; our-numbers explains both signs; school: "the same number here only because the weight is minus one"; one-dot: "no longer the same number" |
| "Guessing" undefined (1, 2; 2 calls it substantial) | "always saying 'on the ground'" / "'not yet'" |
| balance-mario repeats example-air; the scale is never used with two inputs (2) | balance-mario removed; new land-balance works the landing example on the scale (140 and 20 against 152) |
| one-dot has no stated purpose (2) | "One more thing the graph shows, which we'll need later" |
| why-weights repeats simplest-neuron (2) | Cut to its new point: several numbers, each with its own weight |
| depth: "well above his head" wrong for big Mario (1, 2) | "The game measures it to a point 32 dots above his feet" |
| school: "starts at 176" starts where (2) | "At depth zero, the total is 176, the bias"; the graph's axes named in the narration |
| Spoken quotation marks inconsistent (2) | All removed from `spoken` |
| Section header above school and one-dot (2) | Moved to new-question |
| exp001's raised share is computed on the unrounded border (2) | Also computed for the whole-number border (`smb1_land_whole_raised_share_of_all_mistakes`): 60% too; recorded in games/001.md |
| not-a-fluke states as certain what's likely (1) | "would probably have scored worse" |
| other-games triplet (1) | "found and tested the same way" |
| answer-key / castlevania omit the other states (1) | Kept: the lines describe the in-air values; games/001.md lists every state |
