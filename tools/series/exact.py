"""A scene that keeps marks on a game screen frame-exact, whatever order they were added in, and that leaves
whole what is already on screen when part of it is animated."""

from manim import Animation, AnimationGroup, Mobject, Scene


class ExactScene(Scene):
    """A scene whose game footage advances before anything else updates, so a mark that reads a screen's RAM
    sees the frame about to be drawn even when it was added before the screen. Footage opts in by offering
    advance_first(dt)."""

    def update_mobjects(self, dt: float) -> None:
        for footage in _running(self.mobjects):
            footage.advance_first(dt)
        super().update_mobjects(dt)

    def add_mobjects_from_animations(self, animations: list[Animation]) -> None:
        """Add what the animations move that isn't on screen yet. An AnimationGroup (a Succession, say) moves a
        Group made just to hold its mobjects; Manim adds that Group, and adding pulls each of its members out of
        whatever already held it, splitting that parent into loose pieces whose own updaters no longer run. So
        a group's own animations are looked into instead."""
        family = self.get_mobject_family_members()
        for mob in _animated(animations):
            if mob not in family:
                self.add(mob)
                family += mob.get_family()


def _running(mobjects: list[Mobject]):
    """Footage the scene's update will reach this frame: not under anything whose updating is suspended."""
    for m in mobjects:
        if m.updating_suspended:
            continue
        if hasattr(m, "advance_first"):
            yield m
        yield from _running(m.submobjects)


def _animated(animations: list[Animation]):
    """The mobjects the animations move (an introducer adds its own when it begins)."""
    for animation in animations:
        if animation.is_introducer():
            continue
        if isinstance(animation, AnimationGroup):
            yield from _animated(animation.animations)
        elif animation.mobject is not None:
            yield animation.mobject
