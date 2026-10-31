# Video 001: the second interview (after chapters and beats)

Nate's review of the re-cut script (133 beats, ten sections, commit
fbb2f00), with the chaptered storyboard in hand
(`target/series/001-one-neuron/001-one-neuron-storyboard.mp4`). As
before: questions by lens, a few at a time; an answer the script and the
storyboard can't support becomes a change.

## Questions

Decisions the reviews left to Nate:
1. Every council lens wanted the 6502 beat (why-zero-3) and the
   bias-as-a-weight beat (why-zero-2b) cut, as a detour into code and into
   learning. Both are now kept, rewritten to claims that hold. Keep them,
   trim them, or cut them?
2. Your premise was that comparing with zero is cheapest across
   processors and GPUs. The skepticism lens says the hardware that runs
   networks compares with any number equally cheaply; the script now says
   only that the NES "does something similar": it compares by subtracting
   and checking against zero. Does that capture what you wanted?

Concepts (pedagogy):
3. In your own words: why does only a test against zero survive
   tripling a neuron's numbers, when any number worked for its answers?
4. Chapter 2 ends with the neuron failing on pipes and blocks, and the
   video moves to a new question rather than fixing that. Why doesn't it
   just add his falling speed? (The script doesn't say; the answer is
   video 004's.) Does leaving that unsaid bother you?

Structure and pacing (continuity and pacing):
5. Ten sections: an Introduction, eight chapters, Conclusions, about
   24 minutes. Do the breaks fall where you'd want them, and do the
   titles work? Is "Weighing it" (the balance scale) worth its own
   chapter?
6. After watching the storyboard: is two seconds of stillness right, and
   do the chapter intros and conclusions earn their time or feel like
   padding?
7. Is the narrator, at 10% slower, still faster than you'll read?

Hearing it (lay viewer, blind listener):
8. Where in the storyboard did you have to stop and think?

Animation:
9. The title screens: do the drifting walls of the chapter's own
   pictures read as you pictured option A (they're text placeholders
   until real pictures exist)?
10. The depth ruler now starts 32 dots below the top of the screen, so
    its reading at Mario's feet is the game's number. Is that the way you
    want the offset shown?

The series:
11. The Conclusions end by promising that 002 will show a neuron trying
    to find its own weights, and why, on these games, that never quite
    settles. Is that the 002 you want?

## Answers and changes

Asked 2026-10-30, after the visual language was built.

1. Cut both. why-zero-3 (the 6502) and why-zero-2b (bias as a weight)
   are gone, with their two citations (masswerk-6502, smbdis); the
   balance chapter's "which we treated as a weight on an input that's
   always one" went with them.
2. Fine as it was; moot now that why-zero-3 is cut.
3. Nate: "You're asking me why 0·x = 0?" That is the reason, and
   one-dot-4 already says it in those terms (three times zero is still
   zero), so no change.
4. Yes, add a line. New beat its-mistakes-3, before chapter 2's
   conclusion: falling speed would get the pipes and blocks right
   ("in the air" whenever it isn't zero is right 97% of the time), but
   that's both sides of zero and one neuron draws one line; a puzzle for
   a later video (004).
7. Moot: Nate narrates himself.
10. Moot: the ruler now sits at Mario's feet (the visual language).
