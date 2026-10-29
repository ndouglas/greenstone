"""A Manim scene narrated line by line from a video's script."""

from manim import Scene

from series import style
from series.script import Video
from series.voice import Voice


class ScriptOrderError(Exception):
    pass


class NarratedScene(Scene):
    """Say every script line once, in order; each line's animations last as
    long as its voice. The build sets `video` and `voices`."""

    video: Video
    voices: dict[str, Voice]

    def setup(self):
        self.camera.background_color = style.BACKGROUND
        self.said: list[str] = []

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
        self.wait(pad)
        self.said.append(line_id)

    def tear_down(self):
        unsaid = [line.id for line in self.video.lines[len(self.said):]]
        if unsaid:
            raise ScriptOrderError(f"the scene never says {', '.join(unsaid)}")
