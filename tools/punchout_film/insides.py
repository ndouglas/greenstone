"""What a Punch-Out!! take recorded about the brain, step by step."""

# A brain moves once per 4-frame step, and a take keeps every frame.
MOVE_FRAMES = 4


def step_at(take, i):
    """The step shown at frame `i` of the take, the last one past the end."""
    steps = take.meta["steps"]
    return steps[min(max(int(i), 0) // MOVE_FRAMES, len(steps) - 1)]


def moves(take):
    """The phase's moves, in the order of each step's `scores`."""
    return take.meta["moves"]
