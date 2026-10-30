"""The series' visual language, Ember: sign is temperature, magnitude is light.

Warm colours are positive, cool are negative, ash is zero. One light scale
per video says how bright a value is. Words and numbers are set in Charter.
"""

import math

BACKGROUND = "#030303"
EMBER = "#24100A"  # the ground's warm side
TEXT = "#FBE3C4"  # parchment
ASH = "#A88A6C"
WARM = ("#F2B34A", "#EE8660", "#FFD59A", "#E8597A", "#C0392B")  # amber, coral, gold, rose, crimson
COOL = ("#4F8FB8", "#7FB7E0", "#5EC8D8", "#9A8CF0")  # blue, light blue, teal, violet
FONT = "XCharter"  # Charter, bundled in series/fonts and registered by series.ember

LIGHT_SCALE = 60.0  # the video's one light scale; a video may set its own, once


def temperature(value: float, hue: int = 0) -> str:
    """Warm for positive values, cool for negative, ash at zero."""
    if value > 0:
        return WARM[hue]
    if value < 0:
        return COOL[hue]
    return ASH


def light(value: float, scale: float = LIGHT_SCALE) -> float:
    """How bright a value shines, in [0, 1): smooth, and saturating at large magnitudes."""
    return min(math.tanh(abs(value) / scale), math.nextafter(1.0, 0.0))  # tanh rounds to 1.0 far out


def number(value: float, places: int = 0) -> str:
    """A number with a true minus; a value that rounds to zero has no sign."""
    text = f"{value:.{places}f}"
    if float(text) == 0:
        text = text.lstrip("-")
    return text.replace("-", "\N{MINUS SIGN}")
