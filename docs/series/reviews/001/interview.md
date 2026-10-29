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
