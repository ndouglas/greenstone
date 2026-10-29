"""How a video breathes: the pauses, holds and limits every video is built
with (docs/superpowers/specs/2026-10-29-chapters-and-beats-design.md)."""

# Seconds of stillness after each beat, so the viewer can take it in.
STILLNESS = 2.0
# Seconds a section's (silent) title screen holds.
TITLE_HOLD = 4.0
# No section longer than this, voiced, with its pauses and title screen.
SECTION_LIMIT = 240.0
# The narrator's speed: 10% slower than Kokoro's default, until it's
# calibrated against Nate's own reading.
NARRATION_SPEED = 0.9
