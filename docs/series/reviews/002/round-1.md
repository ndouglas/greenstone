# Video 002, council round 1

Script reviewed: commit c726057 (draft 1). Each lens's findings in
brief, then the checks run on the disputed claims, then the response
(filled in after the revision).

## Skepticism

1. no-slope / not-batching / never-outro: misdescribes the perceptron. It does follow a score with a slope: the perceptron criterion (how far each wrong moment sits on the wrong side; Bishop 4.1.7). Its pulls never shrink near the best border (and its lowest point is all weights at zero), which is why even the batch version swings. "Aims at the right thing, fewer mistakes" is wrong as stated. Fix: it follows a score of its own, with full-size pulls; the staircase is what we want, which has no slope; "only reacts to mistakes".
2. too-right: stated as the cause, but the logistic trick, which barely pulls on moments it gets right, leans 34° (vs 35°) and scores 88%. Correlation only. Fix: hedge, or test (squared hinge); say both smooth scores lean.
3. bowl: landscape() clamps scale at 0, so borders facing the wrong way are a flat plateau (error 1); the square trick doesn't walk this map. "Sloping everywhere toward one lowest point" is false as drawn. Fix: describe the map honestly, or check it.
4. inside: the moral contradicts the findings: the bowl rule ends in the same place every time; only the perceptron wanders. Fix: limit it to the perceptron.
5. order / tldr: the spread is as much "when we stopped" as order; the drop experiment reshuffles everything (permutation of a different length), so it's another order experiment. Fix: "order and when we stopped"; fix or cut the drop.
6. why-bowls: hinge/ReLU aren't smooth; deep nets have no single bowl; with flexible models smooth scores share fewest-mistakes' best answer (Bartlett, Jordan & McAuliffe 2006); main reason is gradients through layers. Fix: "a score with a slope"; make the gap specific to one line.
7. degrees depend on standardizing; 127° is the widest gap between any two of the last ten, not a turn. Fix: lead with disagreement; "spread across".
8. settles: each moment's miss doesn't shrink; the average does. Fix: "the nudges cancel out on average".
9. One dataset, correlated frames, no error bars; 1–4 point gaps are near-ties. Fix: say so; "close to".
10. Minor: too-many fine; bounded fine (the bound can be large).

## The cross-examiner

1. not-batching / no-slope / never-outro / map-outro: the batch perceptron follows the perceptron criterion (Bishop 4.1.7), which has a slope; on inseparable data its lowest point is zero weights. "A staircase has no slope" doesn't explain the swing, and "aims at fewer mistakes" is wrong. Fix: measure |w| in the batch run and restate, or narrow the lines.
2. bowl / map-axes: which scale is the bowl drawn at? Best scale, clamped at zero: a flat plateau where borders face the wrong way. "Downhill from anywhere" is true in the weights, not on this map. Fix: say how it's drawn.
3. too-right: notes call it a candidate; logistic leans 34° anyway. Fix: test (clip overshoots) or "one likely reason".
4. swing / short / two-bottoms: degrees are in standardized units the viewer never sees; 127° is the widest spread, not a turn. Fix: drawn units or lean on 6%; "spread across".
5. square-intro: bigger inputs are nudged harder. Fix: "a step that doesn't depend on how wrong it was".
6. step-size: why exactly ten times? The bigger-step neuron makes the same mistakes, so it takes the same nudges, only bigger. Say it; perceptron only.
7. rule-why: why does the total always move the right way? It changes by step × (sum of squared inputs + 1). Board's "rises toward zero" only for a should-be-yes moment.
8. target: why ±1 (0/1 gives the same border: say so); "how far" needs "and which way"; mention the bias.
9. settles: only the average nudge shrinks; individual ones don't.
10. rule-name: is this exact rule in the 1958 paper? Closer to Principles of Neurodynamics (1962); did the Mark I run this rule? Fix: page refs or "Rosenblatt's work (1958–62)".
11. bounded: what did Block & Levin prove; read? "Never stop moving" follows from every pass having a mistake. Split the claims; record the reading.
12. Minor: order "where it stops" vs never settles → "where it ends after thirty passes"; pocket keeps the best of 30 pass-end lines, not every line; who-moves-it board's "band along the border" isn't measured.
Checked and defensible: guarantee, square-name, too-many, staircase's whole-numbers point.

## Empiricism

1. swing / short / two-bottoms: degrees are in standardized units; on the board's raw map the square border leans 8°, not 35°; 127° is a median over 10 runs, run 0 turned 103° standardized and 17° raw. Fix: measure on drawn axes from the run shown, or drop degrees for the 6% disagreement.
2. too-right: cause never tested; 97% rising lacks a baseline (54% of all moments rise); all are "no" moments. Fix: intervene (cap the pull past target) and report; "against about half".
3. guarantee: no control on our data. Trained on only the moments search gets right, the perceptron reaches 100% and stays. Fix: add it, and show it in place of a made-up tidy cloud.
4. who-moves-it: the "band" is unmeasured; 70% of the busiest tenth are moments the best line gets wrong (vs 8.5% of all); counts include early passes; one order. Fix: late passes, several orders; say "mostly moments no line gets right".
5. staircase: stairs_flat 37% — at the grid's resolution most steps change the count; whole-number claim measured outside exp002 (845 spots, up to 292 moments in one, 57% of moments in spots of 50+). Fix: add to exp002; "between steps it's perfectly flat".
6. settles: only the average nudge shrinks. 
7. not-batching: 43% is one extreme; median 84%, 10th percentile 73%. Fix: typical range.
8. remember: pocket chooses among 30 pass-end lines; averaged uses 3 orders vs 10. Fix wording and orders.
9. rule-wrong: inputs were standardized; the script never says. Fix: say so, or rerun raw.
10. order: pass 30 is an arbitrary snapshot.
Checked and sound: held-out scoring; train/test similar; one-run typical; step-size and least-squares asserted in code; ranges from all 10 orders.

## Pedagogy

1. target / settles / too-right: densest idea, no worked example (001 did one for every formula); "square" never explained. Fix: worked moment (total 0.2, target +1, missed by 0.8); call it the miss map, or explain squared.
2. square-name: three digressions in one beat. Fix: move or shrink the dog; move "my own code".
3. step-size: "starting from zero" contradicts learn; reads as trivia; pre-empts 006. Fix: say start at zero; call back tripling; say why it matters (no step to tune), or cut.
4. rule-wrong: sign logic easy to lose; rule-why asserts. Fix: one worked mistake with 001's real numbers.
5. staircase / bowl / why-bowls: "downhill" and "learning aims at a smooth score" is 006's thesis in advance. Fix: keep 002 at "the bowl tells you which way is better"; plant 006; drop "almost always"/"nobody can".
6. too-many: abstract claim. Fix: a concrete count of borders, or "as far as anyone knows".
7. short / two-bottoms: metrics pile up; lean introduced without set-up. Fix: keep percentages; lean once.
8. one-run / order / swing: ten numbers; three measures of one wobble. Fix: range in prose; drop or merge swing; lead with order.
9. Callbacks: "the total, the number tested against zero" once; move remember after chapter 3 or cut; not-batching answers a question no one asked: cut or notes.
10. tldr-2: add what the viewer can now say.

## Clarity, legibility and coherence

1. one-run: ten percentages unlistenable; "pass" and "shuffled" undefined. Fix: define a pass ("one trip through all the moments"); tell it as a story; all ten on the board; swing: "more than a half-circle"... (127° is more than a quarter turn, less than a half).
2. Core vocabulary (weight, bias, total) undefined in 002; rule-wrong hardest to say. Fix: one recall sentence before rule-intro; concrete case in rule-wrong; define "total" once.
3. settles vs swing use different units; batch averaging not marked as a different mode. Fix: one measure per rule; say "or, add up all the nudges, then take one step".
4. square-name: "square trick" used before it's named; dog aside is a tangent; drop "my own code". Fix: name it in square-intro.
5. too-right: needs an example (target +1, total +5: right, yet 4 away); tie the rising moments to the lean.
6. who-moves-it: two bases (share of nudges vs share of moments); no point. Fix: end on the point, or cut.
7. remember: "the average of all of them" unclear; 89–90 range odd. Fix: "about ninety percent… a patch; the rule never stops".
8. step-size: dense; split.
9. last-time: too long; too-many: "two thousand and forty-eight" needs "numbers in its memory".
10. map chapter abstract. Fix: a ruler laid on the map: its tilt, and how far it sits from the middle. staircase: split the three-clause chain.
11. not-batching: "It" has no referent; seems to contradict settles (averaging fixes one, not the other). Fix: reword.
12. why-bowls: "nobody can" overclaims. Fix: "nobody has a fast way".
13. order: give the point of dropping moments.
14. inside: "one stop on a walk" not set up; rule-name: "learned it" → "learned this way".

## The lay viewer

1. rule-wrong: three ideas in one sentence; weight/bias forgotten since 001; "by its input's number" unparseable; bigger input, bigger turn never said. Fix: re-ground the dials; one concrete case (falling fast, said no, he landed: turn both dials up), then the rule.
2. rule-why: "total" and "side" need a callback; "may jump past it" unexplained. Fix: one-line callback; drop "jump past".
3. step-size: "starting from zero" unexplained; 001's tripling only vaguely remembered. Fix: replay 001 in a sentence; say all dials start at zero.
4. one-run: ten percentages are noise; held-out and shuffled unexplained. Fix: say the range; "like an exam it hasn't seen".
5. order: two ranges; the point of dropping moments unclear. Fix: lead with "so the result is partly luck"; cut or explain the drop.
6. bounded: abstract; names unknown. Fix: cut, or "mathematicians proved it wanders but never runs away".
7. who-moves-it: why it matters unsaid. Fix: "a few awkward moments near the border keep dragging it around" (if true).
8. remember: three claims; name the trick once; "about eighty-nine percent".
9. target / square-name: why ±1 unexplained; "my own code" jars; dog aside one too many. Fix: drop code reference; keep dog short.
10. short / too-right: hardest move in two sentences. Fix: one image; payoff for the rising stat.
11. map chapter arrives cold; whole-numbers clause one too many; "nobody can" vs "no known method" inconsistent.
12. not-batching: batch is a new idea; 43% looks like a typo. Explain or cut.
13. "perceptron" bare on first use; Höffgen claim unhedged.
Worked: recap and test-question; learn; always-wrong; staircase vs bowl; map-outro; chapter outros.

## The blind listener

1. map-intro / map-axes: "a new map" collides with "the map" (Mario's); axes have no left/right/up/down. Fix: "the Mario map" vs "the border map"; each spot is one whole border; left/right turns it, up/down moves it nearer/farther from the middle.
2. staircase / bowl / two-bottoms: "colour" means nothing by ear; height never said to be mistakes; lower = better unstated; "degrees apart" may be heard as a distance on the sheet. Fix: height = mistakes, lower is better; "make the height the square trick's miss"; say the degrees are between the two borders.
3. no-slope: add "on a flat step every direction looks the same; only a drop tells you anything".
4. short: "leans" with no picture; wedge is board-only. Fix: say it's tilted, e.g. clock hands.
5. never-intro / always-wrong / who-moves-it: ringed dots, sized dots, band along the border are board-only; say once each moment is a dot; say the busiest moments are near the border (if measured).
6. one-run: ten percentages is a wall of numbers. Fix: range plus two or three values.
7. swing: 127 degrees has no feel. Fix: "more than a quarter turn".
8. rule-wrong / target: "by its input's number" hard by ear; answer key needs a clause. Fix: one concrete example; "nudged in proportion to the number it multiplies".
9. rule-why: "toward the dot" ambiguous. Fix: so the dot ends on the correct side, or nearer it.
10. step-size / bounded: "within bounds" abstract. Fix: "never run off to infinity, but never stop changing".
11. too-right: thread is board-only. Fix: say it plainly.
12. remember: "the learning itself" for "the rule itself".

## Continuity and pacing

1. one-run: setup plus ten scores in one beat, no pause. Fix: split setup and scores; say the range; pause 3.
2. order: two ranges in one beat. Fix: split or drop the drop experiment; pause 3.
3. Conclusions: give inside pause 3; TL;DR omits why learning aims at bowls. Add a clause.
4. last-time: 60+ words. Fix: split at the colon.
5. test-question interrupts the recap. Fix: move after last-time-2.
6. 001 promised "weights and bias"; learn says only border. Fix: "its own weights and bias, which is to say its own border".
7. step-size: "start from zero" contradicts learn's "start anywhere"; a digression mid-chapter. Fix: say start from zero earlier; move step-size before rule-outro.
8. guarantee: pause 3 before the turn.
9. bounded interrupts never-settles thread. Fix: move after always-wrong; who-moves-it after order; remember: "about ninety".
10. short: four numbers. Fix: keep 87 vs 91; lean once, in two-bottoms.
11. map-intro: state the problem ("why did one settle and the other not?"); map-outro: forward pointer.
12. next: name 003; bridges (outros, map-intro, learn, this-video) could take shorter pauses.

## Checks run on the disputed claims (scratch, 2026-11-01)

- **Angles depend on scaling.** On the depth-and-speed map as drawn
  (raw units), the least-squares border leans 8° from search's, not 35°
  (standardized); logistic 8°. Degrees can't be quoted.
- **"Right by too much" is not the main cause.** With no pull past the
  target (a squared hinge), the border scores 88% and still disagrees
  with search's on 3.7% of moments (least squares: 87%, 6%). Logistic
  is 88% too. Every smooth score tried lands close to, but not on, the
  fewest-mistakes border.
- **The batch perceptron hovers near zero weights.** Its weights' size
  stays around 0.01–0.03 for 300 steps (the perceptron's own score, how
  far wrong the wrong moments are, is lowest at zero on data no line
  splits); last 100 steps: median 84%, tenth percentile 73%, low 43%.
- **Control: when a line can split them, it stops.** Trained on only
  the moments search's border gets right, the perceptron is 100% right
  and its line identical from pass to pass.
- **Leaving out 30 moments, same order otherwise:** 81–90% over 5
  drops (base 90%). The tiny-change effect survives without reshuffling.

## Response (draft 2)

Nate approved the revised spine (perceptron: only reacts to mistakes,
never stops; square trick: measures distances, settles near but not on
the best border; fewest mistakes is a staircase). Acted on:

- **Skeptic 1 / cross 1:** the perceptron follows its own score; new
  beat `perceptron-score`, measured (weights about 3 steps from zero);
  "aims at the right thing" gone; `not-batching` restated.
- **Skeptic 2 / cross 3 / empiric 2:** tested; a no-pull-past-target rule
  lands at 88% too. `too-right` lines replaced by `why-short` and
  `how-far`.
- **Skeptic 3 / cross 2:** the map is described as drawn: each border at
  its best size, flat on the wrong-facing half, one hollow (checked).
- **Skeptic 4:** `inside` limited to what was shown.
- **Skeptic 5 / empiric order:** "order and when we stopped"; the drop
  experiment now keeps the order (81–90%).
- **Skeptic 6:** "a score with a slope"; the gap limited to one border.
- **Skeptic 7 / cross 4 / empiric 1:** no angles anywhere.
- **Skeptic 8 / cross 9 / empiric 6:** `settles` says the average shrinks.
- **Empiric 3:** control added to `guarantee` and to `exp002.py`'s checks.
- **Empiric 4:** busiest moments counted over the last ten passes, ten
  orders; "70% are ones the best border gets wrong" replaces the band.
- **Empiric 5:** whole-number spots measured (845 spots; 57% of moments
  in spots of 50+), new beat `big-drops`; "flat between crossings".
- **Empiric 7:** `not-batching` uses the typical range.
- **Empiric 8 / cross 12:** pocket worded as the best of the pass-end
  lines; averaging dropped from the script.
- **Empiric 9:** new beat `same-scale`.
- **Cross 5–8:** `square-intro` "ignores how wrong", `step-size` says
  why (same mistakes, bigger nudges), `rule-why` adds the should-be-no
  case, new beats `target-example` and `codes`.
- **Cross 10:** "Rosenblatt's work between 1958 and 62"; the Mark I
  claim limited to what the source says (weights on motor-driven dials).
- **Cross 11:** `always-wrong` gives the reason it never stops; `bounded`
  cites the theorem only for not running off.
- **Pedagogy / lay / clarity / blind:** recall of the neuron
  (`rule-intro`), a worked mistake (`rule-wrong`), a worked miss
  (`target-example`), one measure per wobble (degrees and `swing` cut),
  ten scores on the board only, "Mario's map" and "the border map",
  height is mistakes and lower is better, `remember` simplified, "nobody
  has a fast way".
- **Pacing:** `last-time` split (`both-matter`), `test-question` after
  the recap, `learn` names weights and bias and the zero start, pauses
  on `guarantee`, `one-run`, `order`, `settles`, `inside`; bridges at
  1 s; `next` names 003.

Not acted on:

- **Pedagogy 5** (cut "downhill" as 006's): kept, limited to "a slope you
  can step down"; loss and gradients stay 006's.
- **Pedagogy 3** (cut `step-size`): kept; it's the 001 callback the bible
  promised, now with its reason.
- **Lay 9 / clarity 4** (drop "my own code"): kept in a plainer form
  ("the first rule I wrote myself, learning all this"); it's Nate's
  stance and the repo is cited.
- **Skeptic 9** (one dataset, no error bars): noted in `games/002.md`;
  the script avoids ranking near-ties except search against the rest.
- **Block & Levin read in full:** not yet; the claim used is only that
  the weights stay bounded.
