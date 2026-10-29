# Chapters, beats and breathing room: design

The series' videos are restructured so they stop feeling like "a steady
stream of information": chapters with title screens, one thought per
beat, and stillness after each. This applies to every video in the series,
starting with 001.

Builds on: `docs/superpowers/specs/2026-10-29-interpretability-series-design.md`
(the series spec) and the pipeline in `tools/series/`.

## Why

Nate watched 001's storyboard (52 lines, 14:25) and found it dense,
unnatural and mechanical: a long line carries several thoughts and several
pictures at once, nothing pauses, and there's no sense of where the video
is going or what was just achieved. The narrator is also faster than Nate
will read.

## Decided (Nate)

**Beats.** A "slide" is a beat: one short spoken thought paired with one
picture or animation that shows it. It's a unit of timing, not
necessarily a new screen: consecutive beats can share a scene and change
what's in it. If a beat needs two pictures, it's two beats. Splitting can
call for a picture the script didn't have (Nate's example: "All this
border knows about is the floor" gets its own animation, Mario moving with
the neuron's answer updating frame by frame). Worked example, today's
`land-mistakes` (one 17–18 s line) becomes three beats of about 25–30 s
in total at Nate's pace:

1. "About 61% of its mistakes are landings on something raised, a block or
   a pipe, that it said weren't coming yet." / Mario landing on a pipe
   while the neuron says "not yet".
2. "Most of the rest are the opposite mistake: it says 'lands soon', but he
   bounces off an enemy or falls into a pit." / Mario bouncing off a
   Goomba, and dropping into a pit, while it says "lands soon".
3. "All this border knows about is the floor." / Mario jumping and
   falling, "lands soon" and "not yet" updating frame by frame.

**Stillness.** Every beat ends with about 2 seconds in which nothing new
arrives, so the viewer can absorb it.

**Chapters.** Every video opens with an **Introduction**, is divided into
numbered chapters, and closes with **Conclusions** that TL;DR the whole
video. The Introduction and Conclusions are sections like chapters (each
with its title screen) but unnumbered, and they don't need their own
introduction and conclusion beats. Each numbered chapter has:
- a **title screen** that lingers a few seconds, **silent** (the title is
  shown, not spoken; for a listener, the chapter's introduction beat
  carries what the chapter is about): the chapter number and
  title over a dimmed wall of tiles (the chapter's own footage and
  pictures, unevenly sized), the whole wall drifting slowly in one
  direction like a panning camera (option A of the mockups in
  `.superpowers/brainstorm/`);
- a brief **introduction**: the problem this chapter solves;
- the **contents**, more or less as now (re-cut into beats);
- a **conclusion** that zooms out a step: recap the chapter, tie it to the
  one before ("this is how we got from X to Y"), say why it mattered (the
  problem just solved or the need just met), and what comes next.

**Chapter length.** No chapter longer than about 3–4 minutes; a chapter
that runs over is split, with its own title screen.

**001's sections**, at Nate's breaks (his markers are in the reading copy,
saved as `001-script-nate-chapters.md` in the session scratchpad; the
breaks after the first three are the builder's reading of "at the section
headers, roughly"):

- Introduction (why look inside): from `memory` to `start-with-one`.
1. Building one neuron: from `thermostat` to `the-neuron`.
2. Is it any good? (testing it): from `baseline` to `castlevania`.
3. The recipe: from `why-weights` to `one-dot`.
4. Two numbers: from `new-question` to `land-mistakes`.
- Conclusions: a TL;DR of the whole video, then `the-idea`, `modern`,
  `harder`; its "what's next" points at 002 (learning).
- The disclaimer (`disclaimer`) follows the Conclusions, outside the
  sections, before the credits.

Chapter titles above are working titles. Any chapter over the length limit
after re-cutting is split.

**Why zero.** `why-zero`'s "Zero is just the custom" undersells it. Replace
it with the real reasons, kept brief:
- **Learning (foreshadowing 002):** with the setting folded into the bias,
  the bias is one more weight (on an input that is always 1), so the
  learning rule can nudge it the same way as the others.
- **The machine (an aside, not the reason the field chose zero):**
  processors compare with zero most cheaply, because arithmetic itself
  tells them whether a result is zero or negative, with no extra step;
  comparing with 100 or 26 takes a compare first. Nate's understanding is
  that this holds across processors generally. Keep it brief and out of
  the weeds: one sentence, with the NES's own 6502 as the example on the
  board (a real zero-flag branch from Super Mario Bros.' code, via
  Greenstone's disassembly). Cite a 6502 instruction reference, and a
  general source if the sentence claims more than the 6502.

**Narration speed.** Slow the narrator by about 10% now. Proper
calibration against Nate's own reading pace is a separate, later step, so
the speed is one setting.

**Reviews.** The full process for the re-cut script: the council (all
lenses), the adversarial reviews until both reviewers find nothing
substantial, and an interview round if the changes are large. Add a
**continuity and pacing** lens to the council: chapter and beat length,
one picture per beat, intros and conclusions doing their job, the flow
from beat to beat, nothing arriving before it's set up.

**Series-wide.** All of the above is how every video is made from now on:
the bible, the spec and the pipeline change, not just 001.

## The builder's calls (Claude's, open to Nate's correction)

- **Script format.** Chapters are declared in `script.toml` (number,
  title, and their beats in order); the introduction and conclusion are
  ordinary beats, marked as such so the pacing check can find them. Every
  beat keeps today's rules: an id, text, optional spoken form, a board,
  and citations or `framing`. Numbers still come only from `numbers.json`.
- **Pause and hold lengths** are settings with defaults: 2.0 s of
  stillness after each beat, about 4 s for a (silent) title screen.
- **Order of work** (Nate agreed): the pipeline changes come first, then
  the re-cut script is rendered as a chaptered, voiced storyboard, so the
  pacing lens judges real beat lengths; then the council and adversarial
  reviews; then Nate's interview, with the storyboard in hand.
- **Title-screen tiles** are chosen automatically from the chapter's own
  footage and boards, so a title screen always previews its own chapter;
  a chapter can name its tiles to override.
- **Speed** is a parameter of the narrator (Kokoro's `speed`, 0.9 for
  "10% slower"), part of the voice cache key so changing it re-voices.
- **The storyboard** shows chapter title screens (as placeholder cards
  until the tiles exist), one card per beat, and the stillness, so pacing
  can be judged before anything is animated.
- **Checks the build enforces:** a chapter over the length limit (as
  voiced, with pauses) is an error naming the chapter; every chapter has
  an introduction and a conclusion (the Introduction and Conclusions
  sections are exempt).
- Beats are split from today's lines with the narration's wording kept
  where it survives; the split, the new intros and conclusions, and the
  new pictures are drafted by Claude and then go through the reviews.

## Non-goals

- Calibrating the narration to Nate's pace (later; this only slows it 10%).
- Animating 001's beats for real (a later plan; the storyboard comes first).
- Re-recording any gameplay or re-running experiments beyond what new
  claims (the 6502) need.

## Anti-goals

- Beats that are still several thoughts with several pictures.
- Intros and conclusions that are filler ("In this chapter, we'll…") rather
  than the problem and the zoom-out.
- Title screens that show pictures from other chapters.
- Lengthening the video with stillness where nothing was said worth
  absorbing; the pauses are for absorbing, not padding (the builder may
  shorten a pause after a beat that's a bridge, if the pacing lens agrees).
