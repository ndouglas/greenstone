# Video 001, adversarial reviews, ninth pass

Script reviewed: commit 0db9223. Neither reviewer found anything factually
wrong; reviewer 1 found nothing substantial, reproducing every new number
(the false-alarm breakdown event by event). Reviewer 2 found three
problems of following, all from pass 8's moves. Fixed in the next commit.

| Finding (reviewer) | Disposition |
|---|---|
| The bias/setting story told four ways in seven lines, and "crosses zero" used before the graph (2; substantial) | bias-is-setting: "The setting is still there: it's the input where the total comes out exactly zero." why-zero ends "Zero is just the custom." our-numbers explains each sign. school keeps "only because the weight is minus one", now a confirmation |
| balance orphaned before the graph, used 13 lines later (2; substantial) | Moved to just before land-balance, its only use |
| bias-slides blurs graph and map; "one weight on its own" overstates (2; substantial) | Moved to just after settings-are-borders: "the border is the map's version of the graph's crossing point… changing the weights against each other changes the lean" |
| one-dot: "which we'll need later" defers; colon non-sequitur; bias value unsaid (1, 2) | "which we'll use to write the next neuron in whole numbers"; "The bias is now three times 176, but the setting is still 176" |
| depth: "measures it to a point" heard as "partly" (2) | "The number marks a spot 32 dots above his feet." |
| why-weights repeats simplest-neuron (1, 2) | "So far ours has read just one number. What makes a neuron more than a thermostat is that it can read several at once…" |
| our-numbers: thermostat never had a "yes"; bias sign unexplained (1, 2) | "like the thermostat, whose heat comes on below its setting"; "A bias of plus 176 then puts the zero at a depth of 176" |
| steady-trade: "it" unclear; walks right then reasons left (2) | Walks left, toward the top of the screen, then the in-words sentence |
| land-results: "more than a sixth of a second from landing" (pits never land) (2) | "aren't within a sixth of a second of a landing" |
| land-mistakes cites the unrounded border's share (1) | Cites `smb1_land_whole_raised_share_of_all_mistakes` |
| The 6 "died" false alarms are Mario hit and shrinking (1) | Classifier counts state `$0A` as play: now 19 late, 0 died; 91% unchanged; games/001.md corrected, with the stomp check |
| land-balance spoken "5" (1) | "five" |
| Bounce frames include ~10 still rising from a stomp (2) | Noted in games/001.md |
