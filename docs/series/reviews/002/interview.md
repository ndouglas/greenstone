# Video 002: the interview

Nate's review of the script (draft 12, commit 2fd7800), with the draft-11
storyboard in hand (`target/series/002-every-mistake/002-every-mistake-storyboard.mp4`).
Questions by lens, a few at a time; an answer the script and its sources
can't support becomes a change.

## Questions

Decisions left to Nate:
1. The bible's 006 row now says it calls back to 002's ground over the
   weights (both round-6 reviewers flagged the downhill picture as 006's).
   Keep that, or move the picture out of 002?
2. Chapter 4 ("Close, but not quite") is 1:10. Keep it, fold it back
   into chapter 3 (about 4:31, just over the limit), or give it chapter
   5's opening?
3. The line pointing at your linear-regression repo was cut. Restore
   it, put it in the description, or leave it out?

Concepts (pedagogy):
4. In your own words: what does the perceptron rule do after one moment,
   when the neuron was right and when it was wrong?
5. Why doesn't the step size matter for the perceptron starting from
   zero, and why does it matter for the square trick?
6. Why does the perceptron stop on the control set but never on Mario's
   moments?
7. What is the total squared miss, and why does the square trick come to
   rest where the perceptron doesn't?
8. Why does the square trick pull on a moment it already gets right?
9. What is the perceptron's own measure, why is it lowest at zero, and
   why can't the rule stay there?

The claims (cross-examiner, skepticism, empiricism):
10. How do you know the wobble isn't just a step that's too big?
11. The square trick rests at 87%, not 91%. Someone asks why. What do you
    say, and what did we test?
12. What would make "the perceptron never settles on Mario" false?
13. What do Höffgen, Simon and Van Horn show, and what don't they?
14. Rescorla–Wagner is "essentially the same" as Widrow–Hoff. In what
    sense, and where do they differ?
15. Check against Bishop §4.1.7: is the perceptron's measure lowest at
    zero? (No reviewer could open the page.)

Hearing it (lay viewer, blind listener):
16. Describe the sheet of borders out loud: what each direction means,
    and what the shelf and the bowl are.
17. Anywhere you had to stop and work something out? Read `rule-wrong`
    aloud: one beat or two?

Animation:
18. How do we show that one spot on the sheet is one ruler on Mario's
    map?
19. There are two surfaces: the ground over the weights (the U) and the
    sheet of borders (staircase, bowl). How do we keep them apart on
    screen?
20. Which beats need game footage (`unseen`: the pipe and the pit), and
    which are diagrams?
21. The Mark I: a period photo, or a drawing?

The series:
22. Does the ending set up 003 the way you want?

## Answers and changes

### Round 1: decisions, and the rule

**Q1 (006).** Keep: the bible's 006 row stands.

**Q2 (chapter 4).** Nate: merge it with chapter 5. Applied: one chapter,
"Close, but not quite" (`short` to `staircase-outro`); its length is
checked on the next storyboard.

**Q3 (the repo line).** Leave it out.

**Q4 (the rule).** Nate: right, nothing changes; wrong, the weights and
the bias change to swing the border toward getting it right. Correct.
Not said: how much each weight moves (the step times its own number) and
which way (up if he landed, down if not, then times the number's sign).
Follow-up asked.

### Round 2: the rule's signs, and the step

**Q4a (follow-up: high and rising, said yes, didn't land).** Nate: didn't
know; couldn't picture it. Finding: `rule-wrong` uses a moment above
average on both numbers, so the signs never show, and `rule-flip` only
covers that same moment. Walked through: depth −1.5, speed −0.5, step
0.1: both weights turn *up* (+0.15, +0.05), the bias down (−0.1), and the
moment's total drops by 0.35, toward no. The total always moves the
right way, by the step times (depth² + speed² + 1); depth's weight moves
more because its number is further from average.

**Q5 (step size).** Nate: didn't know, for either rule. Finding:
`step-size-2` never said the weights are only the nudges added up (the
link to 001's scaling), and nothing said why the square trick needs a
small step. Walked through: doubling the step doubles every nudge, so
every weight and the bias, so (001) the same border, the same mistakes,
for ever; the square trick measures misses against fixed targets, and a
step too big overshoots by more than the miss it closes.

Applied (Nate approved):
- New beat `rule-signs` after `rule-flip`: a number below average counts
  as negative and turns its weight the other way (high and rising, said
  yes, didn't land: both weights turn up); the board shows the moment's
  total beside the dials. `rule-why` follows: the total always moves the
  right way.
- `step-size-2`: "Its weights are only its nudges, added up, so a bigger
  step makes every weight and the bias bigger by the same amount."
- New beat `too-big` after `settles-3`, from a new measurement in exp002
  (`too_big_*`, with a claim check and a test): with 20 times the batch
  square trick's step, the total squared miss grew at every one of 60
  steps.

**Q4b (asked again: low but rising, said not yet, landed).** Nate: depth
weight up, speed weight down, bias up; the total goes up. Correct.

**Q5b (asked again: steps of 1 and 3).** Nate: three times the weights,
the same border; for the square trick, the bigger step moves it more
dramatically. The perceptron half is right. The square-trick half is
true but misses why the scaling breaks (the targets don't triple, so the
misses, and so the nudges, differ). Follow-up asked.

**Q5c (follow-up: why the square trick's weights stop being three times
apart).** Nate: because the targets stay at plus and minus one. Correct.

### Round 3 (paused)

**`rule-signs` (Nate, reading it).** Too many negations to follow.
Finding: the inputs' meanings fight the examples. Depth grows as Mario
gets *lower* (high = small depth) and speed is *falling* speed (rising =
negative), so any nudge example told by Mario's position and motion makes
the viewer flip each word before the rule starts; `rule-wrong` has the
same problem ("his depth, which grows as he gets lower"). Proposed, not
yet applied: tell the nudge examples in the numbers ("both numbers above
average, he landed: both weights turn up"), replace `rule-signs` with the
rule stated by its effect ("each weight turns whichever way pushes this
moment's total toward the right answer, and further the further its
number is from average"), and drop the high-and-rising example. The
board shows the moment's total moving toward its side beside the dials.

**Chapter length.** The merged "Close, but not quite" is 4:29 on the
narrator's voice (storyboard of commit 8949166; whole video 21:18).
Proposed: move `perceptron-score` to `batch-2` (about 1:30) to open the
bowl chapter, retitled (e.g. "Two measures").

Still to ask: Q6 to Q22.

### Round 4: the questions, re-asked on draft 25

Draft 13 changed the question (will the screen scroll soon?), and
drafts 14–25 went through the council and adversarial reviews. Q1–Q5
stand as answered. Round 3's `rule-signs` finding is moot: both numbers
now count up toward the mark (position in blocks, rightward speed), so
no sign ever flips. Q14 and Q15 went with the beats they were about
(the animals and `perceptron-score` were cut). The rest, rewritten for
draft 25 (commit b047c80), with the minor notes from adversarial-14:

Concepts (pedagogy):
- R1. Why does the perceptron stop on the control set but never on all
  of Mario's moments? (`guarantee-check`, `never-intro`)
- R2. `spots`: why can no border, of any shape, get both answers right
  at one of the points where both happened?
- R3. What is the total squared miss, and why does the square trick,
  added up, come to rest where the perceptron doesn't?
- R4. Why does the square trick pull back on a total of three, which is
  right? (`target-over`)
- R5. Why does the square trick one moment at a time still move around,
  when added up it rests? (`settles-2`)
- R6. Why does too big a step make the square trick worse, not just
  faster, when the perceptron didn't care? (`too-big`)

The claims (cross-examiner, skepticism, empiricism):
- R7. The square trick rests at 88%, not 91%. Someone asks why. What do
  you say, and what did we test? (`why-short`)
- R8. What would make "the perceptron never came to rest" false, one
  moment at a time and added up? How do you know its wobble isn't just
  a step that's too big?
- R9. `no-fast-way`: what do Höffgen, Simon and Van Horn show, and what
  don't they?
- R10. `bounded`: what did Minsky and Papert, and Block and Levin,
  prove?
- R11. `tldr-3`: why is the pocket version "a search of its own"?
- R12. (minor) `too-many`: "each extra number multiplies the borders to
  try". Can you defend it from counting alone?
- R13. (minor) `how-far` says "close to" the best border; the chapter is
  "Close, but not quite". 88% against 91% is about a third more
  mistakes. Which do you mean?
- R14. (minor) `search-2` calls the best border "a compromise" before
  `rule-outro` says no straight border splits the moments. Fine, or out
  of order?

Hearing it (lay viewer, blind listener):
- R15. Describe the sheet of borders out loud: what turning and sliding
  mean, and what the staircase, the shelf and the bowl are.
- R16. (minor) `too-big` has two thoughts (why the step matters; the
  overshoot), and the first has no picture. One beat or two?
- R17. (minor) "Mark One" a beat after "the mark". Does it trip a
  listener?

Animation:
- R18. How do we show that one spot on the sheet is one ruler on
  Mario's map?
- R19. Two surfaces: the U over one weight (`downhill`) and the sheet
  of borders (staircase, bowl). How do we keep them apart on screen?
- R20. Which beats need game footage (`the-line`, `unseen`, `big-drops`'
  start point), and which are diagrams?
- R21. The Mark I: a period photo, or a drawing?

The series:
- R22. Does `next` set up 003 the way you want?

### Round 5: R1–R3

**R1 (control set against all moments).** Nate: the full set can't be
divided into yes and no by a single straight line. Correct; the step
left unsaid (a border that splits them makes no mistakes, so no nudges)
is in `guarantee`.

**R2 (no border of any shape).** Nate: at those points the neuron's
answer is sometimes right and sometimes wrong, so the line never
stabilizes. That is `always-wrong`'s consequence, not the reason. The
reason: a border, of any shape, gives one answer per point on the map;
at a point where both outcomes happened, either answer is wrong about
some of its moments. Finding: the script never says it. `never-intro`
claims "no border of any shape" before `spots` shows the points, and
`spots` stops at the evidence without the step from it. Follow-up asked.

**R3 (total squared miss; why it rests).** Nate: the sum of the squares
of how far right or wrong each moment is; there's a border where no
adjustment lowers it, and that's where it rests. Mostly right. The miss
is measured from the target (plus or minus one), so "too right" counts
too (`target-over`). Not said: why the rule stops there (the added-up
pulls cancel at the bottom, `balance`), and the perceptron half.
Follow-up asked.

**R2a (follow-up: a border at a point where both happened).** Nate:
didn't know how to answer; "the border cannot separate a point." That
sentence is the answer, but the step after it (so every moment at that
point gets the same answer, and some are wrong) didn't come from the
script. Confirms the finding. Proposed: `never-intro` asks whether any
border, even a curved one, could get every moment right; `spots` ends
"A border can't split a point: whatever its shape, every moment at one
of those points gets the same answer, so some of them are wrong." On
screen, one point holding both colours, and a curved border that can
only put it on one side.

**R3a (what the added-up pull comes to at the bottom).** Nate: it
balances at zero, so it isn't pulled either way. Correct.

**R3b (the batch perceptron).** Nate: it keeps moving. Right; the
measured detail (`perceptron-rest-2`): it rocks between two borders,
each move almost undoing the last, and its moves don't shrink.
