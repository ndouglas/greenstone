# Video 001: the interview

Nate's review of the script (revision 9, commit 17521d8), as questions
from each council lens, asked a few at a time. An answer that can't be
given from the script, or that needs the video, becomes a script change.

## Questions

Concepts (pedagogy):
1. In your own words: what do the weight, the input and the bias each do?
2. Why does the neuron say yes when the total is "above zero", rather than above some other number?
3. Why does our first neuron's weight come out as minus one, and the landing neuron's as plus one?
4. If you multiplied the landing neuron's weights and bias all by ten, what would change about its answers, and why?

The claims (cross-examiner, skepticism, empiricism):
5. How do you know the 94% isn't a fluke of this recording?
6. What would make "the setting came out at exactly the floor" not true?
7. The neuron is right 94% of the time. What exactly is it getting wrong the other 6%?
8. What does the landing neuron not know about, and how would you show that?
9. Someone says: "your random player isn't how anyone plays Mario." What do you say?
10. What's the difference between how we found the border and a neuron "learning" one?

Hearing it (lay viewer, blind listener):
11. Describe the sheet of paper out loud, as if to someone who can't see it: what goes where, and where the "lands soon" moments are.
12. Is there anywhere in the script you had to stop and work something out? Where?

Animation:
13. How should the balance scale look: real-looking, or simple shapes? What's on screen at once, and what isn't?
14. How would you animate "the bias slides the border without tilting it"?
15. Which moments need real game footage, and which are better as diagrams?
16. The depth number is measured from a point above Mario's head, so a ruler on screen needs an offset. Do you want that shown, or quietly handled?

The series (clarity, skepticism):
17. Is the opening's case for looking inside (and the doubters' argument) pitched right for where we are?
18. Does the ending ("we can see all of it… networks of many are much harder") set up what you want the next video to do?

## Answers and changes

Nate read the script once, straight through, before answering.

### Round 1: concepts

**Q1 (weight, input, bias).** Nate: the weight changes the slope (multiplies),
the bias is an additive offset, and together they form a line from which an
input gives "the other coordinate". Finding: the slope-intercept picture is
the one the y = mx + b line evokes, but in the script nothing *outputs* a
coordinate; the total is compared with zero. "Bias = intercept" also clashes
with that line's own warning that school's b is a different number from the
bias. The script never shows the one graph where Nate's picture is exactly
right: the total plotted against depth, a line of slope w that starts at b
and crosses zero at the setting. Follow-up asked.

**Q2 (why zero).** Nate: not explained. Confirmed: why-plus says *that* the
bias replaces the setting and the comparison is with zero, never *why* zero
(any fixed number would do; it can be folded into the bias, and zero is the
convention). Gap → script change.

**Q3 (sign of the weight).** Nate: in the air when depth − floor < 0; lands
soon when depth plus falling speed is high. Correct; the script carried it.

**Q1a (follow-up: the total plotted against depth).** Nate: a straight line,
slope −1, y-intercept (0, 176), x-intercept (176, 0). Correct and immediate,
so this graph is the bridge from school algebra. He didn't say what the
crossing means (it is the setting: the border between yes and no). Proposed
change: show this graph in the recipe section and move the y = mx + b link
there (where m = w and b = bias exactly), shrinking the two-number
y = mx + b paragraph to the bias-slides-the-border point.

Applied (Nate approved both): new line `why-zero` after `bias-is-setting`;
`school` moved to after `example-ground` and rewritten around the total-vs-
depth graph; the old two-number `school` line cut to `bias-slides`. Script
2,096 → 2,106 words.

### Round 2: claims

**Q4 (scale weights and bias by ten).** Nate: the border becomes steeper
and moves left. Wrong, and the script states the right answer (line
`the-tilt`: scaling doesn't move the border) without making it
stick. His answer is right about the *total's* graph (ten times steeper,
starting at 1,760) and wrong about the *border* (where it crosses zero:
still 176). Finding: two different lines (the total's graph and the border)
are easy to conflate, and the new `school` line introduces the first just
before the second. Proposed change: show the scaling on the total's graph,
where it visibly pivots about the zero crossing, and say the two lines apart.

**Q5–Q7 (the 94%).** Nate couldn't place "94%" after one reading. Partly the
question's fault (no cue), but the script has twelve percentages
(55, 94, 88, 87, 50, 92, 67, 75, 75, 84, 91, 85). Lay-viewer finding: too
many numbers to hold; consider fewer, or a persistent on-screen scoreboard.
Q6 was worded unclearly. All three re-asked with cues.

**Q4 follow-up.** Nate's two ideas: (a) explain the steepening, then point
out that all we care about is one dot, where the line crosses zero; (b) two
views, one holding the axes still and moving the line, the other holding
the border still as a horizon and turning the points around it. Proposed:
(a) for 001; (b) parked in the bible (it is projection onto the weight
direction, a later video's picture).

**Q5 (re-asked with cues: why isn't the 94% a fluke?).** Nate: no idea.
Finding: the script says the test session is new play, but never says that
is how we check for a fluke, and gives no other evidence. Checked
(SMB1, cut chosen on sessions 1–2): 93.7% / 94.0% / 94.0% on sessions 1/2/3;
choosing the cut on any one session alone also gives 176; the rule-teacher
recording (a different style of play) gets 91.3% with it. SMB3 (128) and
Castlevania (176) likewise pick the same cut on every session. Proposed: a
line saying so, with the numbers added to exp001's numbers.json.

**Q6.** Nate: Mario would have to spend more time standing somewhere other
than the floor. Correct.

**Q7.** Nate: standing on a pipe, block, Goomba. Correct (the script says
block or pipe).

### Round 3: hearing it

**Q11 (describe the two-number sheet).** Nate: "What two-number sheet?"
Finding: the script's "sheet of paper" (line `the-sheet`, every in-the-air
moment as a dot, depth along the bottom, falling speed up the side) didn't
stick under that name after one reading, though it carries the whole second
half. His Q12 answer calls something "the graph"; he read the script before
the `school` change, so that must be the sheet. Risk: the script now has
two graphs (the total against depth, and the sheet), so each needs a name
that can't be confused with the other. Follow-up asked.

**Q12 (where did you stop to work something out?).** Nate: picturing the
graph and what it means, and why a negative weight works the way it does.
The negative weight is explained once (`our-numbers`: "like the thermostat,
its yes is below the setting"); the new total-vs-depth graph may carry it
(a negative weight is a line going down), to be checked with him.

**Q11 follow-up.** Nate: "the sheet isn't in the script" (it is, once, as
"a sheet of paper" in `the-sheet`, which is the finding). 11a: yes, his
"graph" was the sheet; unsure of a name (distribution? population? data?).
Ruling: "the map" (a map of every moment), plain and distinct from "the
graph" of the total; recorded in the bible's visual language. 12a: the
negative weight still benefits from the thermostat; kept.

Applied (Nate approved): `not-a-fluke` after `result` (backed by the new
`{game}_every_session_cut` numbers: each session alone picks 176, 128, 176);
`one-dot` before `the-tilt`, which now just scales; sheet → map throughout
(`the-sheet` → `the-map`). Script 2,106 → 2,201 words (~12:35): over the
12-minute ceiling, to trim.

Correction to Q11: Nate meant the map isn't *visualized* in a text script
(he raised aphantasia as a possible viewer lens, then clarified he can
picture it himself and had misread the question). Lens kept: anything the
narration asks a viewer to "picture" must be on screen. On length: 12 minutes isn't a
ceiling; 20 or 30 is fine. Spec updated.

### Round 4: animation

**Q13 (balance scale).** Simple shapes. Show the graph beside it, so each
change visibly moves both: things varying together help. Mario on screen,
moving and jumping, helps. Not every view at once; split into subsets that
focus on one aspect.

**Q14 (bias slides the border).** Move the line straight up or down, linked
to the other views as in Q13. (On the graph, the bias raises or lowers the
total's line, so its zero crossing slides; on the map, the border slides
without changing its lean. One slider, both views.)

**Q15 (footage vs diagrams).** As much game footage as possible: its
physical feel balances the abstract views. Storyboard note: put footage
under, or beside, every line that's about Mario, not only the opening.

### Round 5: claims

**Q8 (what the landing neuron doesn't know).** Nate: whether there's floor
beneath him, an enemy, or something else. Correct; `land-mistakes` carries
it (most misses are landings onto raised things).

**Q9 (the random player).** Nate: nobody plays like that, but what's
recorded (jumping, falling, running) is the same information we use to
understand Mario. Fair. Unsaid in the script: a different player changes
how often each situation comes up, so the percentages would move, though
the setting needn't (the rule-teacher recording, steadier play, also picks
176 and scores 91%). Offered as an optional line.

**Q10 (search vs learning).** Nate: a learner adjusts from each example
and never knows what "best" means, only "a little better in this
direction". Correct; `how-found` defers learning to a later video. Nate's
note for 003 recorded in the bible: linear regression and his update
methods (`ndouglas/linear-regression`, `bitterbridge/p5js-sketches`);
basic network material only as far as the interpretability thread needs.

### Round 6: animation, and the series

**Q16 (the depth offset).** Nate: show it; directly portray as much as
possible. Applied: `depth` now says the game counts from a line a little
below the top of the screen, and its board draws that zero line on the
footage with a ruler that follows Mario.

**Q17 (the doubters, this early).** Nate: worth acknowledging. Kept.

**Q18 (what comes next).** Nate: stay on one neuron and show it being
trained, perhaps over a whole playthrough of Super Mario Bros., watching
the neuron change. Proposed: make learning 002 and move "when the line
lies" to 003 (Nate to decide). Needs: play beyond 1-1 (an existing input
recording of the whole game, or the rule teacher), and exp001's finding
that the learning rule reaches 84%, below search's 94%, and never settles.

## Outcome

Twenty-one questions asked across six rounds (with follow-ups 1a, 11a, 12a).
Changes to the script: why-zero; the graph (school, moved, then one-dot);
not-a-fluke; steady-player; the sheet renamed the map; the depth offset
shown; fewer percentages; the balance scale moved. Changes to the series:
learning becomes 002 and "when the line lies" 003; 5 to 12 minutes is a
comfortable size, not a ceiling. The changed script then needed five more
adversarial passes (adversarial-6 to -10) before both reviewers found
nothing substantial.
