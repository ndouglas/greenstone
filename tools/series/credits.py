"""A video's sources and disclaimer: in its description, and on screen."""

import textwrap

from manim import DOWN, LEFT, Text, VGroup

from series import style
from series.ember import serif
from series.script import Citation, Video

# Each page of the end cards holds this long.
PAGE_SECONDS = 6.0
# Text size, the widest a line may be, and the tallest a page may be.
SIZE = 0.6
WIDTH = 12.0
HEIGHT = 6.0
BUFF = 0.35
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


def stack(*rows: Text) -> VGroup:
    return VGroup(*rows).arrange(DOWN, aligned_edge=LEFT, buff=BUFF)


def heading(page: int, pages: int) -> Text:
    return serif(f"Sources ({page} of {pages})", style.WARM[0], SIZE)


def credit_pages(video: Video) -> list[VGroup]:
    """The end cards: the sources a page at a time, every one at the same
    readable size, closing on the disclaimer (on its own page if it won't fit)."""
    entries = [serif(textwrap.fill(f"{c.authors} ({c.year}). {c.title}.", 64), style.TEXT, 1.0, italic=False)
               for c in cited(video)]
    # One size for every page, narrower only if a line is too wide.
    size = min([SIZE] + [WIDTH / e.width for e in entries])
    for e in entries:
        e.scale(size)
    disclaimer = serif(textwrap.fill(DISCLAIMER, 64), style.TEXT, SIZE)
    groups: list[list[Text]] = []
    for e in entries:
        if not groups or stack(heading(1, 1), *groups[-1], e).height > HEIGHT:
            groups.append([])
        groups[-1].append(e)
    fits = bool(groups) and stack(heading(1, 1), *groups[-1], disclaimer).height <= HEIGHT
    pages = [stack(heading(i + 1, len(groups)), *g, *([disclaimer] if fits and i == len(groups) - 1 else []))
             for i, g in enumerate(groups)]
    return pages if fits else pages + [stack(disclaimer)]


def credits_seconds(video: Video) -> float:
    return len(credit_pages(video)) * PAGE_SECONDS
