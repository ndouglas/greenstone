# Adversarial review Q (draft 16, commit d52f255)

Two substantial findings. Every number checked matches; search's border
is essentially the true fewest-mistakes border (an exhaustive-cut search
at 3,600 angles: 92.21% against 92.19% on training, 90.6% on test for
both).
1. spots: "the game keeps both numbers in small whole units" is false;
   it keeps sub-pixel parts the two bytes don't show, a second reason
   the same point goes both ways. → spots says "the two numbers we read
   come in whole pixels and sixteenths of a pixel a frame"; new
   `unseen-2` names the hidden finer parts. Checked first: `$0400`
   differs at 66% of moving points (now in exp002).
2. only-these: "can't move until he turns round, so we leave them out"
   doesn't follow (6.5% moved). → the sign reason.
Minor, taken: bowl-2 ("like the U"); too-big (why size matters now: the
targets are fixed); short split (lean on speed into `short-2`);
this-video's triplet cut to two; unseen's numbers now at the very points
holding both answers (88% against 53%); numbers-2 now a callback to
001's sixty frames; too-many cites hoffgen1995; tldr no longer cites
Rosenblatt; settles-2 "in a hundred"; rule-name "rules of this kind".
Not taken: downhill's four thoughts (chapter 3 has no room for another
pause; they build one picture).

# Adversarial review R (draft 16, commit d52f255)

Seven substantial findings.
1. batch: "moved on every one of 300 steps" isn't evidence; the batch
   square trick moves on every step too, its steps shrinking to nothing.
   → compares step sizes ("its last hundred were all about the same
   size, while the square trick's shrink toward nothing"), checked.
2. perceptron-rest: "leans back toward the start" was unexplained (no
   place to lean in), and rest-2 carried two pictures. → the weights as
   a place, "like a spot on the ground from before"; the reason in
   total terms (the wrong total is the weights' doing, so pushing it
   back takes some back); three beats.
3. never-outro / square-intro / balance taught that shrinking pulls let
   a rule stop; the per-moment square trick doesn't stop, and at the
   bottom the pulls balance rather than vanish. → never-outro asks
   "What would a rule need, for its pulls to cancel out?"; square-intro
   "pulls on every moment, right or wrong, from both sides"; balance
   "the pulls are still there, but they balance".
4. unseen: its numbers were a different group from its board's (same
   point). → the mixed points' figures (Q8).
5. target-over: 2.5× holds only for the same numbers. → "for the same
   position and speed".
6. learn: "a fixed amount added on" read as never changing. → "one more
   number added on" (001's words).
7. bowl-2 / map-outro: the sheet isn't the U, and the square trick
   doesn't slide across the sheet. → "like the U"; map-outro "a bowl
   whose bottom is where the square trick rests".
Minor, taken: shelf's reason (totals point the wrong way); the-map
split (`player`); guarantee's unused second sentence cut; tldr's "with
a fixed step" dropped (a true rest needs cancelling pulls whatever the
step); search "a great many borders"; too-many "only two numbers";
the-line's "a third of the way across" cites NESdev's 256-pixel
picture (`nesdev-ppu`, verified); staircase-outro's hook; map-outro's
filler line. Not taken: big-drops' "where each life starts" cites
exp002 (verified in the data: every moment at that point is within
300 frames of a restart); the number of beats opening "So".
