# Video 001, adversarial reviews, tenth pass

Script reviewed: commit 325b95f. **Both reviewers: nothing substantial.**
The script passes. Reviewer 1 re-ran exp001 and matched numbers.json byte
for byte; reviewer 2 re-checked the first neuron's mistakes against the
session-3 recording.

Polish applied (each corrects a fact or removes an ambiguity, adding no
new claim, so not re-reviewed):

| Finding (reviewer) | Change |
|---|---|
| land-results: depth alone (74.8%) is a hair *better* than always "not yet" (74.5%) (1) | "does only about as well as" |
| steady-trade: "toward the top of the screen" while on the map (1, 2) | "toward smaller depths, where Mario is higher up" |
| bias-slides: no reason for "without changing its lean"; "weights against each other" vague (1, 2) | "because the bias adds the same amount to every total"; "changing one weight compared with the other" |
| bias-is-setting: "setting" shifts from a threshold on the total to a value of the input (1, 2) | "For a neuron reading one number, the setting is still there in a new form: it's the value of that number where the total comes out exactly zero, where the answer flips." |
| land-example: second sum skips "5 times minus 2" (2) | Spelled out |
| modern: "past the border" doesn't say which side (2) | "onto the yes side" |
| the-tilt: rounding nudges the border (2) | "Here, very nearly, is the border it found" |
| steady-player: 1-2's floor unsaid (2) | "even in the second level, it's at the same depth" (games/001.md) |

Deferred (polish, for the storyboard stage): the map is called straight
11 lines before steady-trade says why (2); depth's "32 dots" isn't used
again in the narration (2); why-weights still partly restates
simplest-neuron (1, 2); land-example and land-balance type their sums as
literals, so a rerun that changed the border's 5 or 152 wouldn't flow
through (2).
