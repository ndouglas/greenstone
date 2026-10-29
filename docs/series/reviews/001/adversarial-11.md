# Video 001, adversarial reviews, eleventh pass

Script reviewed: uncommitted changes after af46eb0 (the final review's
minors: exp001 counts only frames of play, the worked examples' inputs
as placeholders, steady-trade moved, why-weights trimmed). **Both
reviewers: nothing substantial.** Reviewer 1 re-ran exp001 byte for byte
and found one flaw in the new mask. Applied:

| Finding (reviewer) | Change |
|---|---|
| `in_play` counted state 0A as play, but 0A is the freeze while the game shrinks a hurt Mario, and its first 16 frames have the in-the-air fault the mask removes; 09 (growing) was already masked (1) | Only 08 counts. Moved its-mistakes 91% → 92%, the baseline 55% → 54%, the raised share 60% → 61%, bounces-or-pits 91% → 94% (all via placeholders). `false_alarm_kind` still lets the look-ahead run through 0A, since it doesn't end a life. games/001.md updated with the masked counts per session |
| steady-trade uses the +1 depth weight a line before why-plus explains it (1, 2) | Order is now the-tilt → why-plus → steady-trade → land-example |
| steady-trade's last sentence states the border's rule as the game's (2) | "In words, the border says: …" |
| why-weights now only repeats simplest-neuron (1, 2) | "But the recipe works for any number of inputs, each with its own weight, and soon we'll need two." |
| The mask isn't disclosed to the viewer (1, 2) | frames: "…leaving out the moments when the game moves Mario by itself, like when he dies or slides down the flagpole." |
| games/001.md: "about 90 frames a session", 09 unmentioned, "no other quoted number", flagpole note (1) | Corrected |
