# Video 001, council round 1

Script reviewed: commit 5df0ec2. Each lens's findings, then the response
to each (acted on, or why not), filled in after the revision.

## The blind listener

1. why-speed / two-cuts / no-help: the scatter (standing moments on one flat row at zero speed, flying moments above and below) is never spoken; "a line has two sides" is taken on trust. Fix: describe the row and why one line can't fence it in from both sides.
2. no-help: the best line's shape (it ignores speed; it's the height cut again) is only on screen. Fix: say so.
3. the-picture: "one number across and the other up" names no numbers; "every straight line through the two numbers" sounds like a line through two values. Fix: say position goes across, speed up, each moment is a dot; describe the search as lines across the picture.
4. our-numbers: spoken algebra with no worked example. Fix: one position in the air and one on the floor, with their totals.
5. floor / the-cut: 176 in two roles back to back; "less than" means higher. Fix: say the cut sits at the floor and "less than" means higher than the floor.
6. "cut" and "line" never tied together. Fix: say once that with one input the line is a cut on the number scale.
7. same-question: the comparison lives in on-screen bars; the six percentages came in inconsistent order. Fix: a spoken recap, one consistent order.
8. school: y = mx + b mapped only visually; "plus b" means two things. Fix: map the parts in words, or say the match is loose.
9. smb3-why (minor): 16 to 160 needs "nearly the whole height of the screen".

## The lay viewer

1. the-picture / no-help: the graph is unanchored; "a line through the two numbers" is meaningless. Fix: anchor it (position along the bottom, speed up the side); say "line" means a boundary on the picture; "every way of splitting the picture with one straight line".
2. weight-and-bias … school: seven lines of maths with no Mario and no reason the setting needed replacing; our-numbers says 176 three times. Fix: give the reason (the weight is what lets a neuron use more than one number); say the bias is the old setting; shorten our-numbers; bring Mario back sooner.
3. balance: a negative weight in a pan makes no sense; "counterweight" vs "added"; "weight" is a pun on the scale. Fix: handle negative weights in the analogy (they go in the other pan) or keep it to positive cases; be consistent about the bias.
4. score / place-value: the neuron produces a whole number, contradicting "the only kind of answer it can give". Fix: say it's the weighted sum before the yes/no step, or drop it.
5. why-speed: "negative going up" is backwards to everyday sense. Fix: repeat why in a clause.
6. castlevania-arc: "doesn't store a speed" vs "its slot stayed at zero" sounds contradictory, and feels like trivia. Fix: give the payoff or cut it.
7. castlevania: who is Simon? Fix: introduce him.
8. the-cut vs recording: "set by hand" vs "picked using ten minutes". Fix: reconcile.
9. no-help: "surprise" with no set-up. Fix: set up the expectation.
10. rule-result: hard to parse aloud; what is the rule if not a neuron? Fix: a plain label and what it stands for.
11. doubters: a hook never paid off. Fix: name the argument in a sentence or say it returns later.
12. Minor: "neurons" vs brain cells; "network" undefined; smb3 positions need "bigger means lower"; "weights" plural vs one m.
Worked: the thermostat, the answer key, the baseline, the pipe mistakes, the closing line; the first third is pitched right.

## Pedagogy

1. no-help / why-speed / two-cuts / rule-result / the-idea: the script teaches 004's idea (questions no line can answer), and 001's own idea, a tilted line where two weights both matter, is never shown working on real data; the ending lands on the limitation. Fix: one real two-input case where a tilted line beats either input alone; keep the speed result a short honest aside or move the two-cuts analysis to 004; end on what the line does.
2. score / place-value: contradicts the yes/no definition; arrives late as "one more thing". Fix: make it a yes/no question ("is the score over 10,000?") with digit weights and a bias, or call it the total before the yes/no step; call back the scale.
3. school: the y = mx + b link is claimed, not shown; no mapping of m and b. Fix: the line is where the total is exactly zero; solve for one input to get slope × other + starting value; tie m to the weights and b to the bias.
4. balance: our own neuron (weight −1, bias +176) can't go on the scale as described; the scale is never called back in speech. Fix: a negative weight puts its block in the other pan; redo our-numbers on the scale; reuse the scale in speech.
5. our-numbers: no spoken worked example; two definitions (compare with a setting vs. with zero) left to merge. Fix: 150 gives 26, in the air; 180 gives −4, on the ground; say the setting became the bias.
6. the-cut vs recording: "by hand" vs "picked from data"; "we tried every line" sounds like search. Fix: say we looked at ten minutes and chose it ourselves.
7. castlevania-arc: dangling fact. Fix: "so there's no second number to give the neuron", or cut.
8. memory → series: never says why the NES is here. Fix: the games give numbers whose meaning we know, so we can check what a neuron makes of them.
9. Smaller: why a straight line (one sentence of intuition); why-speed sign needs the callback; doubters callback; smb3-why fine.
Worked: thermostat, padlocked answer key, the score to beat, the pipe mistakes, "the only neuron we'll understand completely".

## Clarity, legibility and coherence

1. score / place-value contradict the yes/no definition and the ending. Fix: the adding-up half only, or a yes/no question with place-value weights; or cut/move.
2. formula: w, x, b spoken before they're named. Fix: name each as spoken, or put `parts` first.
3. balance: signs backwards for our own neuron (−1 weight, +176 bias). Fix: a negative weight puts the block in the other pan; the bias can go in either pan.
4. our-numbers: algebra by ear. Fix: speak one example (150 → 26, in the air); say why the weight is negative (bigger means lower).
5. the-picture: "changes along a straight line" is ambiguous; this is the one idea. Fix: "the border between yes and no is a straight line", same wording in the-idea.
6. The input has many names (how far down, the number, position, his position); "position" sounds left-right. Fix: one plain name throughout.
7. "standing"/"flying" vs "on the ground"/"in the air" (and SMB3 Mario can fly). Fix: use the latter throughout.
8. the-cut vs recording: two accounts of where the cut came from. Fix: one account.
9. smb3: "the same neuron" isn't (cut 112); "much worse" has no stated comparison. Fix: "the same kind of neuron, with its own cut".
10. castlevania-arc: no consequence; "slot" jargon; introduce Simon; "dropping" → "falling".
11. no-help: say the best line ignores speed; "a line through the two numbers" → "across that picture".
12. rule-result hard to parse; two-cuts' "a line has two sides" doesn't land (yes on both sides of a narrow no-band).
13. Opening never says why a game console. Fix: games give us an answer key. Minor: "numbers" double meaning.
14. school: "the same line" and the two b's invite equating them. Fix: "the same kind of line"; don't equate the b's.
15. Smaller: answer-key (1 and 2 both count); "frame" undefined; all-of-it lists "a line" as a part; weight-and-bias's "compares" target.
16. The climax leans on 004's idea. Fix: frame the failure as a consequence of the line.

## The cross-examiner

1. place-value breaks the yes/no definition. Fix: "the weighing part, before the yes/no step", or loosen the definition.
2. The piece shown isn't what modern AI neurons are (they give a graded number, e.g. ReLU); the-idea reads as a claim about them. Fix: say this is the oldest, simplest version; modern neurons say how far past the line; the line still holds.
3. castlevania-arc can't be defended: one slot ($03C4) checked, by observation not exp001, and it's zero on *every* frame, including knockback; consistent with a wrong address. Fix: narrow to "the slot the map lists never changed", or verify properly.
4. smb3-why: "levels go up as well as across" is untested (all recordings are 1-1; the high byte is 1 on 98–99% of frames); the spread is platforms and slopes, like SMB1; $00A2 is a level coordinate, not screen position; 16–160 is the 1st–99th percentile, and the low end is plausibly death-hop noise (~2,500 "standing" frames below 100). Fix: "stands at many heights even in the first level"; drop death frames or say so.
5. the-cut vs recording, and never-see: the cut was chosen by trying every value on training data with the answer key. Fix: say how; say we used the key to choose, the neuron never reads it.
6. recording: not fifteen minutes of play but three five-minute sessions of random button-pressing, restarted at each death (~35 per five minutes), all in 1-1. Fix: say so.
7. baseline: "Mario spends a little more than half his time standing" is about this player. Fix: "in our recordings".
8. no-help: "every straight line" is false (a 360 × 199 grid). Fix: "hundreds of thousands of lines".
9. balance: reversed for our example. Fix: negative weights as counterweights; bias on either side.
10. memory: 2,048 is the main work RAM; there's also video, sprite, palette and cartridge RAM. Fix: "in its main memory".
11. school: the two b's differ (intercept is −b/w₂); vertical lines aren't y = mx + b. Fix: "the same kind of line".
12. rosenblatt1958 describes a multi-layer machine; the single threshold unit goes back to McCulloch and Pitts (1943). Fix: cite a source for the modern single-unit form, or cite Rosenblatt as historical origin.
13. only-one: promises about future videos; undercut by the other games' neurons. Fix: "the networks after this get harder to see into".
14. its-mistakes: true (962 of 1,089 test errors are Mario standing at 112, 128 or 144), but exp001 doesn't compute it. Fix: add the breakdown.
Defensible: nobody-knows, doubters, why-speed (speed 0 on every standing frame, 0 on 7% of airborne), our-numbers, answer-key.

## Empiricism

Every number reproduces; the SMB1 core is sound (cut chosen on seeds 1–2, test on 3; the oracle cut on test is also 176; no leakage).
1. SMB3's result is mostly a death-animation artifact: death is detected ~200 frames late, so 28–37% of each recording is a dead Mario labelled "ground" (vy −64, then sinking to y 160). Without death frames: cut 112 gives 78% vs a 58% baseline, best cut 128 gives 81%; "speed isn't zero" at 63% is the same artifact. Nothing shows "levels go up". Fix: detect the death when it starts and restart there; re-record; claim only causes the data shows.
2. Castlevania's key is wrong on knockback: all 235 "in the air, actually ground" errors are state 05, when Simon is airborne. Fix: count 05 as airborne and re-run. Also the cut is re-chosen per game (176 again by coincidence).
3. its-mistakes: 962 of 1,089 SMB1 test errors (88%) are ground frames at 112/128/144; exp001 should emit it.
4. castlevania-arc: $03C4 is 0 on all 54,000 frames (an all-zero byte also looks like a wrong address); $0488 counts 0–40 through every jump. Fix: measure both in exp001, or soften.
5. "set by hand" isn't what the code does (best_cut searches). Fix: describe the search, or fix 176 by hand.
6. "every straight line" overstates (360 × 199 grid).
7. recording: restarts every 7–9 s; all SMB1 data is the first ~6 screens of 1-1; a random button-holder, not a person. Fix: one clause each.
8. Minor: SMB3 key differs between recorder (non-zero) and exp001 (== 1); memory line and SMB3's 8KB cartridge RAM; place value demonstrated only by a unit test; $00CE is likely the top of Mario's box (feet near 208), so the floor board drawing needs checking.

## Skepticism

1. smb3 / smb3-why: 53% of SMB3 "ground" frames have nonzero speed (the death hop); ~a third of test frames are death animation; 160 is the bottom of the death fall. Without deaths, standing heights 48–128 in clear platform levels (128 down to 48): the same "standing on blocks" effect as SMB1, not "levels go up"; baseline 57% airborne, cut 78%. $00A2 is a level position, not a screen position. Fix: detect deaths properly and recompute; don't present the old numbers as facts about level shape.
2. castlevania-arc: $03C4 is 0 in all 54,000 frames, including 2,843 knockback frames; $0488 "Jump State" counts 0, 1, 2… through each jump and $04DC steps 144–163 in the air: the game stores where Simon is in the jump, as a step counter into a table. Fix: say that, cite $0488/$04DC.
3. place-value contradicts the definition; the ram.rs score test is circular (it writes and reads back with its own function). The recordings do support the layout ($07E1 takes 1, 2, 3, 5; $07E2 always 0). Fix: "the weighted-sum part reads the score; with a bias it answers 'is the score above X?'".
4. Yes/no threshold units presented as what AI is built from; modern neurons are graded (ReLU, GELU). Fix: one clause.
5. Rosenblatt 1958 doesn't present w·x + b with a bias; the threshold unit is McCulloch and Pitts (1943). Fix: cite McCulloch–Pitts for the unit, a modern textbook for the notation, Rosenblatt for the learning perceptron.
6. the-cut: "set by hand" isn't what was done. Fix: describe the search, or set 176 by eye and say so.
7. memory: the NES also has video RAM, sprite memory, palette RAM, registers; SMB3 adds 8KB of cartridge RAM. Fix: "its main working memory holds 2,048 numbers".
8. school: the line is y = −(w1/w2)x − b/w2; the b's differ, and w2 moves the intercept too. Fix: tilt from the ratio of weights, position from the bias divided by a weight; or say the letters differ.
9. castlevania key: knockback (05, ~5%) airborne but labelled ground; "dropping" (07) never occurs. Fix: add 05.
10. recording / baseline: a random button-masher restarting at the start of 1-1 every 7–9 s, never past x 1,438 px; train and test are near-duplicates. Fix: "in our recordings"; "pressing random buttons for random lengths of time".
11. its-mistakes: 88% of errors are ground frames at 112/128/144; exp001 doesn't compute it.
12. Minor: "every straight line" (a grid); doubters slightly stronger than the source ("misguided", "scale down"); the bible's disclaimer is missing from the script.

## Responses (revision 2)

Data and experiments, changed first:
- SMB3 deaths are detected when they begin (`$058C`, found by experiment), and all games re-recorded; the recorder logs restart frames. SMB3 now reads 87% against a 50% baseline, with the cut at its floor (128). "Levels go up" and the 16–160 range are gone. (empiricism 1, skepticism 1, cross-examiner 4)
- Castlevania's key counts knockback (05); 92% against 67%. (empiricism 2, skepticism 9)
- exp001 now measures: the mistake breakdown (88% standing on something), Castlevania's speed slot (never non-zero) and jump counter (to 39), the top score, seconds between deaths, and a new two-number question. (cross-examiner 14, empiricism 3–4, skepticism 2, 11)
- The new question, "will Mario land in the next 10 frames?", gives 001 a real tilted line where both inputs matter: 75% always-no, 75% height cut, 84% speed cut, 91% the line. (pedagogy 1)

Script:
- The "speed isn't zero" two-cuts analysis is moved to 004, where it's the opening example; 001 ends on what the line does. (pedagogy 1, clarity 16)
- The setting is described as it was chosen: every value tried on the first two sessions, the best kept; it came out at the floor. The answer key is used to check and to choose, never read by the neuron. (all lenses on the-cut/recording; cross-examiner 5)
- The recording is described honestly: random buttons for random lengths, restarting at every death, the first level only, three sessions; the baseline is "in the third session". (cross-examiner 6–7, skepticism 10, empiricism 7)
- The weights get a reason (reading more than one number); w, x and b are named as they're spoken; the bias is said to do the setting's job; our-numbers has two spoken worked examples; the negative weight is explained. (lay viewer 2, pedagogy 5, clarity 2, 4)
- The balance scale has yes and no pans, negative blocks go in the no pan, the bias in whichever pan its sign says, and our own neuron is worked on it; it comes back for the landing neuron. (lay viewer 3, pedagogy 4, clarity 3, cross-examiner 9)
- The picture is anchored (a sheet of paper, height along the bottom, speed up the side, a dot per moment), and the line is named as a border between yes and no; why it tilts is given in words. (blind listener 1–3, lay viewer 1, clarity 5)
- y = mx + b is "the same kind of line"; tilt from how the weights compare, position from the bias; the letters don't match one for one. (pedagogy 3, clarity 14, skepticism 8, cross-examiner 11)
- Place value is "the adding-up half"; with a bias it answers "more than a thousand points?". (all lenses)
- One name for the input ("height number"); "on the ground" / "in the air" throughout; Simon introduced; "falling" for dropping. (clarity 6–7, lay viewer 7)
- The opening says why games: their memory holds the right answers. "Main working memory". "Artificial neurons, loosely inspired by brain cells". (pedagogy 8, clarity 13, skepticism 7, cross-examiner 10, lay viewer 12)
- doubters names the argument (study patterns across many pieces) and says it gets its own video. (lay viewer 11)
- "Oldest and simplest kind", cited to McCulloch and Pitts; a closing line on modern neurons (graded output, same weighted sum and bias), cited to Goodfellow et al.; Rosenblatt kept for the learning perceptron (003). (cross-examiner 2, 12; skepticism 4–5)
- "Only neuron we'll understand completely" became "networks of thousands are much harder to see into". (cross-examiner 13)
- The disclaimer is spoken at the end. (skepticism 12)

Not acted on:
- The SMB1 score layout is supported by the RAM map and the recordings ($07E1 takes 1, 2, 3, 5); the circular unit test is left as a decoding test and noted. (skepticism 3)
- The floor board drawing (is `$00CE` the top of Mario's box?) is an animation note for the storyboard stage, not a script claim. (empiricism 8)
