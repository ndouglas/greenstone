"""A Manim scene narrated line by line from a video's script."""

import textwrap

from manim import DOWN, FadeIn, Scene, Text, VGroup

from series import style
from series.credits import credits
from series.script import Video
from series.voice import Voice

CREDITS_SECONDS = 6.0


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
        self.credits_rolled = False

    def say(self, line_id: str, *animations, pad: float = 0.25) -> None:
        lines = self.video.lines
        expected = lines[len(self.said)].id if len(self.said) < len(lines) else None
        if line_id != expected:
            raise ScriptOrderError(f"said {line_id!r} where the script has {expected!r} next")
        voice = self.voices[line_id]
        self.add_sound(str(voice.path))
        if animations:
            self.play(*animations, run_time=voice.seconds)
        else:
            self.wait(voice.seconds)
        if pad > 0:
            self.wait(pad)
        self.said.append(line_id)

    def placeholder(self, line_id: str) -> None:
        """Say a line over a card standing in for its animation."""
        line = next((l for l in self.video.lines if l.id == line_id), None)
        if line is None:
            raise ScriptOrderError(f"placeholder for {line_id!r}, which isn't a line in the script")
        rows = [Text(line.id, font=style.FONT, color=style.INPUT).scale(0.5)]
        rows.append(Text(textwrap.fill(line.text, 48), font=style.FONT, color=style.TEXT).scale(0.6))
        if line.board:
            rows.append(Text(textwrap.fill(f"Animation: {line.board}", 56), font=style.FONT, color=style.WEIGHT).scale(0.45))
        card = VGroup(*rows).arrange(DOWN, buff=0.4)
        self.add(card)
        self.say(line_id)
        self.remove(card)

    def roll_credits(self, seconds: float = CREDITS_SECONDS) -> None:
        """End on the sources and the disclaimer, after every line. Every
        video must."""
        unsaid = [line.id for line in self.video.lines[len(self.said):]]
        if unsaid:
            raise ScriptOrderError(f"credits rolled before {', '.join(unsaid)} was said")
        self.clear()
        card = credits(self.video)
        self.play(FadeIn(card), run_time=0.5)
        self.wait(seconds - 0.5)
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
        for line in self.video.lines:
            self.placeholder(line.id)
        self.roll_credits()
