"""A video's sources and disclaimer: in its description, and on screen."""

import textwrap

from manim import DOWN, LEFT, Text, VGroup

from series import style
from series.script import Citation, Video

DISCLAIMER = "I'm not an expert, just a guy learning this stuff, in public. If you spot a mistake, please tell me."


def cited(video: Video) -> list[Citation]:
    """Every source the script cites, once, in the order first cited."""
    keys = dict.fromkeys(key for line in video.lines for key in line.cite)
    return [video.citations[key] for key in keys]


def reference(c: Citation) -> str:
    return f"{c.authors} ({c.year}). {c.title}. {c.url}"


def description(video: Video) -> str:
    parts = [video.title]
    sources = cited(video)
    if sources:
        parts.append("Sources:\n" + "\n".join(f"- {reference(c)}" for c in sources))
    parts.append(DISCLAIMER)
    return "\n\n".join(parts) + "\n"


def credits(video: Video) -> VGroup:
    """The end card: the sources, then the disclaimer, fitted to the frame."""
    rows = [Text("Sources", font=style.FONT, color=style.TEXT, weight="BOLD")]
    rows += [Text(textwrap.fill(f"{c.authors} ({c.year}). {c.title}.", 64), font=style.FONT, color=style.TEXT)
             for c in cited(video)]
    rows.append(Text(textwrap.fill(DISCLAIMER, 64), font=style.FONT, color=style.TEXT, slant="ITALIC"))
    card = VGroup(*rows).arrange(DOWN, aligned_edge=LEFT, buff=0.35)
    return card.scale(min(0.6, 12 / card.width, 6.5 / card.height))
