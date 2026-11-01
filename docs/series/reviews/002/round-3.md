# Video 002, council round 3

Script reviewed: commit d2f82fe (draft 3), the last council round. Each
lens's findings in brief, then the response. The pacing lens reads draft
4's storyboard instead (below).

## Skepticism

1. who-moves-it: "mostly moments even the best border gets wrong" is false: they cause 38–39% of late nudges (all ten orders), from 8% of moments: about five times their share. Fix: quote 8% → 39%; add the measurement.
2. perceptron-score: at zero weights the neuron says no to everything, so every landing is a mistake; what's zero is the cost (how far on the wrong side). Fix: "every total is zero, so no mistake costs anything"; "two nudges" from one order.
3. why-bowls: "isn't quite the lowest step" generalizes; "here" or "may not be".
4. map-outro / tldr-2: only the all-at-once version stops; "settles".
Minor: bowl's flat condition is weighted, not a head count: "overall".
Checked and sound: guarantee, step-size, never-intro, bounded, remember, order, dropped, codes, settles, short, why-short, two-bottoms, too-many, square-name.

## The cross-examiner

1. who-moves-it: "mostly moments even the best border gets wrong" is false: those moments cause 39% of the late nudges (orders 0–2), from 8% of moments. The 71% is a share of the busiest tenth's moments, not of nudges. Fix: lead with 8% of moments, about two in five nudges; add to exp002.
2. perceptron-score: at zero weights the neuron says no to everything, so "nothing counts as wrong" is false of mistakes; true only of its measure (how far wrong moments sit on the wrong side). Say what the measure is; "so" overclaims cause; two nudges from one order; "a long way" unmeasured.
3. bowl: a border's square-trick height needs a weight size; say "each border given the weights that suit it best; where the border is no use at all, they're zero and the ground is flat". Nate: the rule walks a bowl in the weights, not this map.
4. tldr-2: only the all-at-once version comes to rest.
Minor: guarantee "within a few passes" not checked in code; games/002.md's spine section stale.
Checked and defensible: rule-wrong, rule-why, step-size, never-intro, big-drops, codes, numbers, bounded, too-many, guarantee's theorem, square-name.

## Empiricism

1. perceptron-score: "nothing counts as wrong" is false (at zero it says no to everything; 25% of moments are wrong); "about 2 nudges" holds across all ten orders (1.64–1.67).
2. bowl: the flat part isn't "yes side mostly backwards" (66% of sloped cells fit that too); "borders that get most moments wrong" covers 95% of flat cells and 4.5% of sloped ones.
3. games/002.md's spine section stale (0.05; "same place every time"; "facing the wrong way"; "not the cause" vs "not the whole reason"; angles bullet twice).
4. who-moves-it's 71% depends on how ties at the cutoff are broken (67–72% by order); tie-neutral about 70%. The 47% is unaffected.
5. never-intro: 845 spots counts only the moments learned from (920 with the test recording).
Sound: 8× check; 292; in_spots_with_both; best_possible; one low point; narrow lowest ledge with a cliff; control (last nudge in pass 3); order, dropped, one-run, tldr, pocket, codes, settles, short, why-short, two-bottoms, step-size-2.

## Pedagogy

Round 2's big problems fixed; recap, staircase/bowl, callbacks and conclusions work.
1. perceptron-score still too much: one picture, one sentence (a sharp point at zero; nudges full size beside it); drop the nudge count and the batch clause.
2. "measure" undefined and collides with "score" (accuracy); why-bowls' "score with a slope" reads as accuracy. Fix: name it once ("the miss total"); keep "score" for percent right.
3. "pass" used in guarantee before it's defined.
4. Scope: bowl's "step downhill… you reach it" is 006's; keep to the contrast.
5. Side trips: codes; step-size's "we checked… exactly"; square-name's dog without payoff.
6. target-example: live fractions; keep the overshoot case.
7. bowl's plateau clause hard to hear: "a high flat shelf off to one side".

## Clarity, legibility and coherence

1. "measure" never defined; "square trick" never explained. Fix: name it once and use it every time. (Our note: what the rule makes small is the misses *squared* and added up, not the misses added up; the name should say so.)
2. perceptron-score: three ideas; link unstated; dangling batch clause.
3. bowl: plateau clause unpicturable; "step downhill" implies the rule walks this map.
4. settles: split; "point" singular.
5. map-outro "it stops" vs settles' one-at-a-time wobble: "it settles".
6. who-moves-it: "those" ambiguous.
7. remember: best by what, judged on which moments.
8. rule-why: clumsy; "the border swings a little toward getting that moment right".
9. target-example: fractions by ear; "two and a half times as hard".
10. how-far: shorten.
11. Chapter 1 title says "line", narration "border" (title is the video's title, kept); bounded: "the weights", not "they".

## The lay viewer

Worked: recap; grading on a curve; worked examples; tug of war; ruler on a sheet; "the first rule I wrote myself"; outros. No condescension.
1. Final chapter: add a one-sentence recap of the sheet; "mostly backwards"; "give each border a different height: how far off its answers are, added up".
2. perceptron-score: why zero is best is unintuitive; "nudges" as distance odd; "adding up" lacks an antecedent for the perceptron.
3. "weights" never glossed; "measure" loose: define once.
4. too-many: needs an image.
5. Chapter 2 number density; who-moves-it three percentages in one sentence: say it in words, figures on screen.
6. guarantee: "to see the rule at its best".
7. codes: a side detail; cut or move.
8. why-short / how-far meander: say once that judging by distance and by mistakes pick different borders.
9. square-name: say what the dog learns.

## The blind listener

Round 2's fixes landed. Remaining, small:
1. bowl: "a border's yes side mostly gets the answers backwards" hard by ear; say it's the same sheet with a different height painted on.
2. two-bottoms: say "on the same sheet".
3. short: anchor the 6% ("nearly the same line, tilted a little").
4. how-far: say the contrast (adds up distances vs counts wrong).
5. perceptron-score: "two nudges" scale; split the last sentence off.
6. rule-outro / never-intro: say the dots overlap near the border.
7. same-scale (mild): a number-free example.
8. rule-wrong (mild): acceptable.
9. big-drops: 292 needs a baseline ("nearly three hundred moments at once").
Items 1–2 worth fixing; nothing else substantial.

## Response (draft 4)

New in `exp002.py`: the share of late nudges caused by moments search's
border gets wrong (39%, no cut-off), the control's last moving pass (3),
and how closely the flat part of the squared-miss map matches borders
that get most moments wrong (95% of flat, 4% of sloped).

Acted on:

- **Skeptic 1 / cross 1 / empiric 4:** `who-moves-it` was false
  ("mostly"); now "only 8% of moments, but 39% of the nudges".
- **Skeptic 2 / cross 2 / empiric 1:** `perceptron-score` says what its
  measure is and why zero is its lowest point ("no mistake costs
  anything"); the batch clause and "a long way" are gone.
- **Clarity 1 / pedagogy 2 (and a slip of ours):** the square trick's
  measure named once, correctly, as the total squared miss (new beat
  `square-why`), and used throughout; "score" kept for percent right.
- **Skeptic 3:** `why-bowls` "may not be".
- **Skeptic 4 / cross 4 / clarity 5:** `map-outro` and `tldr-2` say it
  settles (the all-at-once version comes to rest).
- **Cross 3 / empiric 2 / blind 1 / lay 1 / pedagogy 7:** `bowl` says the
  same sheet with a different height, each border with the weights that
  suit it best, and "almost everywhere a border gets most moments wrong"
  for the flat part; "step downhill" removed (pedagogy 4, 006's scope).
- **Empiric 5:** `never-intro` "the moments it learns from".
- **Empiric 3 / cross minor:** `games/002.md`'s spine and stale lines
  regenerated.
- **Pedagogy 3:** `guarantee` no longer uses "pass" before it's defined.
- **Blind / clarity / lay:** `two-bottoms` "on the same sheet"; `short`
  "nearly the same line, tilted a little"; `how-far` shortened;
  `settles` split in two; `rule-why`, `remember`, `bounded` reworded;
  `target-example` "two and a half times"; `big-drops` "moments at once";
  `square-name` says what the animal learns; `learn` glosses weights;
  `guarantee` "to see the rule at its best".

Not acted on:

- **Pedagogy 5 / lay 7** (cut `codes`): kept; the cross-examiner asked
  "why plus and minus one?" in round 1, and Nate should be able to answer.
- **Pedagogy 5** (`step-size`'s "we checked"): kept; it says the claim was
  tested, which the rule of every line being demonstrated asks for.
- **Lay 4** (an image for `too-many`): left; the line is already plain.
- **Clarity 11** (chapter 1's title says "line"): kept; it's the video's
  title.
