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

**Q5b (asked again: steps of 1 and 3).** Open.
