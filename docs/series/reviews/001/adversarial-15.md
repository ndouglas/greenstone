# Video 001, adversarial reviews, fifteenth pass

Script reviewed: commit f26f4ad. **Reviewer 1: nothing substantial.**
Reviewer 2: one "borderline-substantial" finding, which reviewer 1 listed
as minor with the same fix; everything else minor. **The script passes.**

Polish applied, with the wording both reviewers gave (not re-reviewed):

| Finding | Change |
|---|---|
| one-dot-4b states its claim before the turn, so the reminder sounds like evidence against it (1 minor, 2 borderline) | "Here, zero matters. In the last chapter, any number worked: moving the 100 into the bias gave the same answers as testing against 100." |
| testing-end: "A harder question needs two numbers" oversells (speed alone gets 84%), "harder" is unearned, and it repeats recipe-intro (1, 2) | "Some questions depth can't answer well at all, and for those we'll want a neuron that reads two numbers." |
| draw-intro is `framing` but promises a mathematical fact (1) | Cites bishop2006 |

Left for the storyboard stage: one-dot-4c's closing rule could fold into
one-dot-5 (2).
