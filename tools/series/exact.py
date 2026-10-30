"""A scene that keeps marks on a game screen frame-exact, whatever order they were added in."""

from manim import Mobject, Scene


class ExactScene(Scene):
    """A scene whose game footage advances before anything else updates, so a mark that reads a screen's RAM
    sees the frame about to be drawn even when it was added before the screen. Footage opts in by offering
    advance_first(dt)."""

    def update_mobjects(self, dt: float) -> None:
        for footage in _running(self.mobjects):
            footage.advance_first(dt)
        super().update_mobjects(dt)


def _running(mobjects: list[Mobject]):
    """Footage the scene's update will reach this frame: not under anything whose updating is suspended."""
    for m in mobjects:
        if m.updating_suspended:
            continue
        if hasattr(m, "advance_first"):
            yield m
        yield from _running(m.submobjects)
