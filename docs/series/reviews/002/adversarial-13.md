# Adversarial review EE (draft 23, commit b88d4fd)

One substantial finding.
1. settles-2: draft 23's trim cut "Single moments still tug", the only
   reason one-at-a-time learning keeps moving right after balance's
   "add up to exactly nothing"; "the small step we gave it" then read as
   the cause. → "Only the sum is nothing; each moment's own pull isn't.
   So if it learns one moment at a time, with the small step we gave
   it, …"
Minor, taken: numbers "counted in blocks of sixteen pixels" (the byte
truncates, so "to the nearest pixel" was wrong; and a block is now said
to be sixteen pixels); tldr-2 states only what was shown ("Added up, its
pulls cancel at the bottom of a bowl"), not a general law. Not taken:
boards that show list placeholders raw (board notes for the animator,
not spoken).

# Adversarial review FF (draft 23, commit b88d4fd)

Two substantial findings.
1. shelf / bowl: the flat shelf sits at the all-zero start, where the
   square trick begins, and no-slope has just taught that a
   slope-follower can't move on flat ground; nothing said why it isn't
   stuck (draft 14's no-shelf said so; round 4 cut it). → a new
   `no-shelf`: "The square trick starts with every weight at zero, right
   on that shelf. But it isn't stuck there: it moves its weights, not
   this sheet, and in its valley over the weights, zero sits on a
   slope." (Round 4's skepticism checked the slope at zero.)
2. settles-2 (EE1).
Minor, taken: remember's "One cheap fix" against tldr-3's "a search of
its own": now "One fix".
