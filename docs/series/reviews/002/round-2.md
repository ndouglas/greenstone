# Video 002, council round 2

Script reviewed: commit 2fa7529 (draft 2). The pacing lens waits for
draft 3's storyboard. Each lens's findings in brief, then the response.

## Skepticism

1. tldr-2 / map-intro: "one lowest point" doesn't separate the rules — the perceptron's score also has one (zero weights) and is convex. The difference is the bottom's shape: the square trick's slope flattens, so its pulls shrink; the perceptron's comes to a sharp V and pulls full size beside it. The map chapter really answers "why not aim at fewest mistakes directly?" Fix: tldr-2 "its pulls shrink near the bottom"; perceptron-score "comes to a point at zero"; reframe map-intro.
2. perceptron-score: zero weights score zero for any data; with no splitting border, zero is the only lowest point. Control's weights end ~20 steps from zero vs ~2.6. Fix wording.
3. never-intro / always-wrong: stronger direct proof available: 284 of 845 spots hold both outcomes; 65% of training moments sit in one; no border of any shape gets them all; best possible on the training moments is 92.9%. Fix: "the same depth and speed sometimes landed and sometimes didn't"; tldr "no border".
4. who-moves-it: "no straight border can get right" overclaims; measured is "the best border gets wrong too".
5. how-far: "neither rule" ambiguous; reads as a general law (least squares can equal fewest mistakes on some data); nearly circular; a third of the shortfall is offset (best offset with LS's tilt: 88.5%). Fix: "both of these… here, came close".
6. square-outro / inside: only the batch version is the same every time.
7. perceptron-score: 3 steps ≈ two nudges (a nudge ≈ 1.5 steps); one order only.
Checked and sound: bowl's single low point and flat half; not-batching, hinge, settles numbers; bounded; why-bowls.

## The cross-examiner

1. how-far / tldr-2: the perceptron's score also measures distance, and its best is zero weights, so "measuring distances" isn't what puts a rule close; "neither rule" can be heard as perceptron and square. Fix: "neither of these two"; "different measure, different best".
2. target-example: why-short's premise (pulling on right moments) is never shown. Fix: an overshoot case (total 3, right by plenty, pulled back by 2).
3. why-short doesn't answer its question (partly: 87→88, 6%→4%); hinge has no convergence check. Fix: "a little closer… not the whole reason"; add a check.
4. perceptron-score: "three steps" against what? A nudge is ~1.5–2 steps; measure nudge length against weight size; "stay near zero" as observation.
5. big-drops: off by one (≥50 includes the moment). Fix: "spots holding fifty or more moments".
6. same-scale: dividing by the spread is the step that matters; "usual" means average (numbers.json quotes medians). Fix wording.
7. bowl: single-hollow check not in exp002; the square trick doesn't walk this map (the bowl is in the weights, Bishop 4.1.3). Fix: add the check; Nate ready with the weights-space answer.
8. settles: "fits best" vs "not the best": say "where its measure is lowest"; "comes to rest".
9. codes: "settles on exactly the same border" (per-moment paths differ).
10. inside / square-outro: only the batch version is identical every time. Fix: "nearly", or name it.
11. games/002.md contradicts the script (69–86% drops; 45%; "candidate cause"); dropped: "left it anywhere between".
Checked and defensible: rule-wrong signs; target-example arithmetic; codes maths; settles' one point; 845 spots; lowest at zero.

## Empiricism

1. tldr: 70–90 is the spread across orders at pass 30; across passes 21–30 and all orders it's 43–90. Fix: "depending on the order", or quote the combined range.
2. bowl: the flat half is not "borders facing the wrong way" — every angle has flat and sloped offsets; a cell is flat when its yes side gets the answers backwards overall. Fix the wording; add a one-minimum check to exp002 (now only asserted in notes).
3. who-moves-it: "no straight border can get right" false; 70% needs its baseline (8%); 371 ties at the cutoff, unstable argsort (68–71%). Fix: stable sort.
4. step-size: the check compares 0.01 with 1.0 over 3 passes; over 30 passes 10× and 100× differ from pass 16 by rounding. Fix: check over the passes shown, or soften.
5. not-batching: 0.052 is arbitrary for a direction-only rule; report weights in nudges (~2).
6. perceptron-score: "close to zero" needs a comparison: a nudge is ~1.5 steps; the square trick's border sits ~78 steps from zero.
7. guarantee board: the control made 418 nudges, still from pass 4: "settles within a few passes".
8. big-drops: off by one; "some drops are big" unmeasured (10.7% of neighbouring offset steps change by 50+ moments; largest 2,539); rule-outro: 65% of moments share a spot with the other answer. Add to exp002.
9. games/002.md out of date (69–86, 45%, 30%, degrees).
Checked and sound: control; same-order drops; late-pass counts; hinge converged (5,000 = 50,000 steps); batch square; one-run, order, pocket, codes, short, 845 spots, two-bottoms.

## Pedagogy

1. perceptron-score: not teachable as written: a third score; a counterintuitive claim the viewer can't check; "steps" collides with the step size; contradicts no-slope's "no rule can walk a staircase" unless held together. Fix: shrink to one sentence (a smoother score of its own that keeps pulling it back toward all-zero weights, so every nudge is a big swing there); keep the derivation in the notes; or reorder.
2. no-slope: say "the staircase", not all rules.
3. not-batching: answers a question nobody asked; fold into one clause or cut.
4. bowl: "size that suits it best" has no referent.
5. 006 scope: why-bowls should mark itself as a plant (one clause).
6. Chapter 4's end is dense; cutting perceptron-score and not-batching lets map-outro land.
7. Minor: same-scale ("like a grade on a curve"); test-question shouldn't read as "so why watch"; settles "one point".
Callbacks that work: rule-intro, step-size, guarantee.

## Clarity, legibility and coherence

1. perceptron-score: five ideas in one beat; "steps from zero" unexplained (and step size was said not to matter). Fix: split into short lines; "never grows much beyond a few nudges". not-batching: set up "add up the nudges first", and say it's the perceptron's rule (contrast with settles).
2. bowl: "size that suits it best" undefined; three abstractions; "hollow" vs "bowl" vs "lowest point". Fix: plain line; pick "bowl" throughout.
3. target: four-term product in one breath. Fix: "in proportion to three things…".
4. settles: mode switch without warning; "tug of war".
5. who-moves-it: lead with the point.
6. dropped: unstated purpose; parse. Fix: "A tiny change, a different result."
7. step-size: split after "ten times bigger".
8. rule-wrong: "low" and "above usual" sounds contradictory (depth grows as he gets lower). Fix the wording.
9. why-short: "already gets right by plenty" has no setup; add to target-example that a right total of five still gets pulled back.
10. map-axes: add left/right, up/down.
11. Minor: tldr-2 could say what the viewer can now say.

## The lay viewer

1. Chapter 4: three landscapes at once; perceptron-score unpicturable. Fix: one or two sentences (it keeps getting pulled back toward all-zero, so it hops), and say why a slope matters ("if you can feel which way is downhill, you can keep stepping").
2. map-axes: say "tilt" and "slide", not "faces".
3. target / target-example: four-factor sentence; live arithmetic; "total" and "target" sound alike.
4. step-size: a tangent; re-say 001's fact in a clause or cut.
5. guarantee: say "we removed the troublemakers"; rule-outro bump.
6. Chapter 2 number density, dropped especially. Fix: keep the range; board carries detail.
7. settles: one-at-a-time vs all-at-once switch unnamed.
8. why-short and how-far redundant; hinge a side trip.
9. "percentage point" → "point"; bias recall ("the neuron's built-in lean").
Worked: recap, test-question, never-outro, disclaimer, chapter conclusions.

## The blind listener

Round 1's fixes work (two named maps, height = mistakes, ruler, no degrees).
1. map-axes: never told the border map is a flat sheet; "one spot is one whole border" only implied. Fix: "Picture a flat sheet… every spot is one whole border".
2. staircase / bowl: height drawn up over the sheet never said; "the half where borders face the right way round" hard by ear; "the size that suits it best" jargon. Fix: "imagine the sheet as ground, each spot raised to that height"; plateau and hollow in words.
3. perceptron-score: densest beat, four ideas; "three steps" has no unit. Fix: split; plain image (dials hover near zero because there nothing counts as wrong).
4. short / how-far: 6% lacks an anchor ("nearly the same line, tilted a little"); how-far leans on the board: say "one adds up how far off, the other counts how many wrong".
Minor: who-moves-it "busy moments" → "the moments that caused the most nudges"; big-drops number wall; no-slope add "only a drop tells you"; target: example first; settles "shivered by one percentage point" → "wobbled by about one point"; codes "cut-off halfway between".

## Response (draft 3)

New measurements in `exp002.py`: spots holding both answers (284 of
845; 65% of moments; best possible 93%), the biggest change from
crossing one spot (292), the perceptron's step as a power of two with
an exact 8× check over all 30 passes, the combined late range (43–90%),
weights in nudges (about 2; batch about 1.9), a stable sort for the
busiest tenth (71%), the share of big drops, and a check that the bowl
has one low point off its flat part.

Acted on:

- **Skeptic 1:** the rules differ at the bottom, not in having one:
  `tldr-2` (pulls shrink near the bottom), `map-outro`, `perceptron-score`;
  `map-intro` now asks "why not aim straight at the fewest mistakes?"
- **Skeptic 2 / cross 4 / empiric 5–6 / pedagogy 1 / clarity 1 / lay 1 /
  blind 3:** `perceptron-score` shrunk to one idea in plain words, in
  nudges not steps; `not-batching` folded into it as one clause.
- **Skeptic 3 / empiric 8:** `never-intro` now proves it directly: the
  same depth and speed sometimes landed and sometimes didn't; `tldr` "no
  border can split".
- **Skeptic 4 / empiric 3:** `who-moves-it` says "even the best border
  gets wrong", with its 8% baseline.
- **Skeptic 5 / cross 1 / cross 3:** `how-far` limited to "these two
  versions" and "here"; `why-short` answers its question ("a little
  closer… not the whole reason"); the hinge converges (5,000 = 50,000
  steps, empiricism).
- **Skeptic 6 / cross 10:** "nearly the same place every time".
- **Cross 2 / clarity 9:** `target-example` adds the overshoot (total 3,
  pulled back by 2).
- **Cross 5 / empiric 8:** "spots holding fifty or more" in the notes;
  `big-drops` now quotes the measured single-spot change.
- **Cross 6:** `same-scale` says average and typical spread.
- **Cross 7 / empiric 2:** one-low-point check added; the flat part
  described as borders whose yes side gets the answers backwards.
- **Cross 8–9:** "near where its measure is lowest", "comes to rest",
  "settles on exactly the same border".
- **Cross 11 / empiric 9:** `games/002.md`'s experiment section
  regenerated from `numbers.json`.
- **Empiric 1:** `tldr` says "depending only on the order" (pass 30).
- **Empiric 4:** `step-size` uses 8×, checked exactly over all passes.
- **Empiric 7:** `guarantee`'s board says it settles within a few passes.
- **Blind / clarity / lay:** the border sheet as ground with heights;
  tilt and slide; "bowl" throughout; `dropped` leads with its point;
  `settles` names the two modes; "about one point"; `rule-wrong`'s depth
  direction; `step-size` split; "cut-off halfway between"; the bias as
  "its built-in lean".

Not acted on:

- **Pedagogy 5** (mark `why-bowls` as a plant for 006): no clause added;
  the rules forbid "we'll see later", and the line stands on its own.
- **Lay 8** (cut `why-short`): kept; it answers the obvious objection,
  and the skeptic and cross-examiner both asked it.
- **Lay 3** ("total" and "target" sound alike): kept; both are defined
  where they first appear, and renaming would break 001's "total".
