"""How a video breathes: the pauses, holds and limits every video is built
with (docs/superpowers/specs/2026-10-29-chapters-and-beats-design.md)."""

import math
from typing import TYPE_CHECKING

if TYPE_CHECKING:  # voice.py imports this module
    from series.script import Section, Video
    from series.voice import Voice

# Seconds of stillness after each beat, so the viewer can take it in.
STILLNESS = 2.0
# Seconds a section's (silent) title screen holds.
TITLE_HOLD = 4.0
# No section longer than this, voiced, with its pauses and title screen.
SECTION_LIMIT = 240.0
# The narrator's speed: 10% slower than Kokoro's default, until it's
# calibrated against Nate's own reading.
NARRATION_SPEED = 0.9


def section_seconds(section: "Section", voices: dict[str, "Voice"]) -> float:
    """How long a section runs: its title screen, then each beat's voice and
    the stillness after it."""
    return TITLE_HOLD + sum(voices[l.id].seconds + (l.pause if l.pause is not None else STILLNESS)
                            for l in section.lines)


def lengths(video: "Video", voices: dict[str, "Voice"]) -> dict[str, float]:
    """Every section's length as the storyboard would run it."""
    return {s.id: section_seconds(s, voices) for s in video.sections}


def clock(seconds: float) -> str:
    """Minutes and seconds, rounded up, so a length never reads shorter than it is."""
    whole = math.ceil(round(seconds, 6))
    return f"{whole // 60}:{whole % 60:02d}"


def too_long(video: "Video", lengths: dict[str, float]) -> list[str]:
    """One message per section over SECTION_LIMIT, to be split."""
    return [f"{s.id} ({s.title}) runs {clock(t)}, over {clock(SECTION_LIMIT)}; split it"
            for s in video.sections if (t := lengths.get(s.id, 0.0)) > SECTION_LIMIT]
