"""A Manim scene narrated line by line from a video's script."""

import textwrap

from manim import DOWN, LEFT, FadeIn, Mobject, Scene, Text, VGroup, config, linear

from series import style
from series.credits import PAGE_SECONDS, credit_pages
from series.pacing import STILLNESS, TITLE_HOLD
from series.script import Video
from series.title import board_tiles, title_card
from series.voice import Voice


class ScriptOrderError(Exception):
    pass


class NarratedScene(Scene):
    """Say every script line once, in order; each line's animations last as
    long as its voice. The build sets `video` and `voices`."""

    video: Video
    voices: dict[str, Voice]
    # The build sets this, so every finished video must end on the credits.
    require_credits = False

    def setup(self):
        self.camera.background_color = style.BACKGROUND
        self.said: list[str] = []
        self.titled: list[str] = []
        self.credits_rolled = False
        # When the credits began, in seconds: the room tone stops there.
        self.credits_at: float | None = None
        # Each section's length as rendered: from its title screen to the end of its last beat.
        self.section_starts: dict[str, float] = {}
        self.section_lengths: dict[str, float] = {}

    def show_title(self, section_id: str, tiles: list[Mobject] | None = None) -> None:
        """A section's title screen, silent, for TITLE_HOLD seconds: the wall
        drifts left as it holds. Tiles default to the section's boards."""
        section = next((s for s in self.video.sections if s.id == section_id), None)
        if section is None:
            raise ScriptOrderError(f"title screen for {section_id!r}, which isn't a section in the script")
        if section_id in self.titled:
            raise ScriptOrderError(f"{section_id}'s title screen is shown twice")
        # A title screen comes just before its section's first beat.
        lines = self.video.lines
        upcoming = lines[len(self.said)] if len(self.said) < len(lines) else None
        if upcoming is None or upcoming.id != section.lines[0].id:
            where = f"before {upcoming.id!r} ({upcoming.section})" if upcoming else "after every line"
            raise ScriptOrderError(f"{section_id}'s title screen is shown {where}; "
                                   f"it goes just before {section.lines[0].id!r}")
        start = self.time
        card = title_card(section, board_tiles(section) if tiles is None else tiles)
        # A section starts from a clear screen; the last beat's picture
        # mustn't show through the wall's gaps.
        self.clear()
        self.add(card)
        self.play(card[0].animate.shift(LEFT * 0.22 * config.frame_width), run_time=TITLE_HOLD, rate_func=linear)
        self.remove(card)
        self.titled.append(section_id)
        self.section_starts[section_id] = start
        self.section_lengths[section_id] = self.time - start

    def say(self, line_id: str, *animations, pad: float | None = None) -> None:
        """Say a beat, then hold still: `pad` seconds, else the beat's own
        pause, else STILLNESS."""
        lines = self.video.lines
        expected = lines[len(self.said)].id if len(self.said) < len(lines) else None
        if line_id != expected:
            raise ScriptOrderError(f"said {line_id!r} where the script has {expected!r} next")
        line = lines[len(self.said)]
        first = line.section is not None and (len(self.said) == 0 or lines[len(self.said) - 1].section != line.section)
        if first and line.section not in self.titled:
            raise ScriptOrderError(f"said {line_id!r}, the first beat of {line.section}, before its title screen")
        voice = self.voices[line_id]
        self.add_sound(str(voice.path))
        if animations:
            self.play(*animations, run_time=voice.seconds)
        else:
            self.wait(voice.seconds)
        pad = pad if pad is not None else line.pause if line.pause is not None else STILLNESS
        if pad > 0:
            self.wait(pad)
        self.said.append(line_id)
        if line.section is not None:
            # Up to the end of its latest beat, the stillness and any waits before it included.
            self.section_lengths[line.section] = self.time - self.section_starts[line.section]

    def placeholder(self, line_id: str) -> None:
        """Say a line over a card standing in for its animation."""
        line = next((l for l in self.video.lines if l.id == line_id), None)
        if line is None:
            raise ScriptOrderError(f"placeholder for {line_id!r}, which isn't a line in the script")
        section = next((s for s in self.video.sections if s.id == line.section), None)
        rows = [Text(f"{section.title} · {line.id}" if section else line.id, font=style.FONT, color=style.WARM[0]).scale(0.5)]
        rows.append(Text(textwrap.fill(line.text, 48), font=style.FONT, color=style.TEXT).scale(0.6))
        if line.board:
            rows.append(Text(textwrap.fill(f"Animation: {line.board}", 56), font=style.FONT, color=style.COOL[0]).scale(0.45))
        card = VGroup(*rows).arrange(DOWN, buff=0.4)
        self.add(card)
        self.say(line_id)
        self.remove(card)

    def roll_credits(self) -> None:
        """End on the sources, a page at a time, and the disclaimer, after
        every line. Every video must."""
        unsaid = [line.id for line in self.video.lines[len(self.said):]]
        if unsaid:
            raise ScriptOrderError(f"credits rolled before {', '.join(unsaid)} was said")
        self.clear()
        self.credits_at = self.time
        for page in credit_pages(self.video):
            self.play(FadeIn(page), run_time=0.5)
            self.wait(PAGE_SECONDS - 0.5)
            self.remove(page)
        self.credits_rolled = True

    def tear_down(self):
        unsaid = [line.id for line in self.video.lines[len(self.said):]]
        if unsaid:
            raise ScriptOrderError(f"the scene never says {', '.join(unsaid)}")
        if self.require_credits and not self.credits_rolled:
            raise ScriptOrderError("the scene never rolls the credits (call self.roll_credits() last)")


class StoryboardScene(NarratedScene):
    """Every line over its placeholder card, then the credits: the whole
    video's pacing before any of it is animated."""

    def construct(self):
        for section in self.video.sections:
            self.show_title(section.id)
            for line in section.lines:
                self.placeholder(line.id)
        for line in self.video.lines:
            if line.section is None:
                self.placeholder(line.id)
        self.roll_credits()
