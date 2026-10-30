"""The formula, set in the series' serif: neutral symbols until a number fills one, each filled term in its
sign's temperature, the threshold carried into the bias when it moves."""

from dataclasses import dataclass

import numpy as np
from manim import ORIGIN, UP, Animation, AnimationGroup, Mobject, Transform, VGroup, smooth

from series import style
from series.ember import serif, swap

GAP = 0.17  # between pieces
TIGHT = 0.03  # inside a product of bare symbols, which reads as one thing
AXIS = 0.42  # where operators sit, as a share of a digit's height
ARC = 3.1  # the bend of the threshold's hop over the comparison into the bias
MINUS = "\N{MINUS SIGN}"
DOT = "\N{MIDDLE DOT}"


@dataclass
class Piece:
    """One drawn piece: a stable key (so it can move), its text, colour and slant, and whether it hugs the
    piece before it."""
    key: str
    text: str
    colour: str
    italic: bool
    tight: bool = False
    operator: bool = False


def window(start: float, end: float):
    """A rate function that holds still until start, eases through to end, then holds."""
    return lambda t: smooth(min(max((t - start) / (end - start), 0.0), 1.0))


class _Ghost(Animation):
    """The old look of a piece whose text changes: it stays in the formula while it fades out and slides
    to where the new piece is, and is gone when it finishes."""

    def __init__(self, ghost: Mobject, holder: VGroup, to: np.ndarray, **kwargs):
        super().__init__(ghost, **kwargs)
        self.holder, self.origin, self.to = holder, ghost.get_center(), to

    def begin(self) -> None:
        self.holder.add(self.mobject)
        super().begin()

    def interpolate_mobject(self, alpha: float) -> None:
        a = self.rate_func(alpha)
        self.mobject.move_to(self.origin + (self.to - self.origin) * a).set_opacity(1 - a)

    def finish(self) -> None:
        super().finish()
        self.holder.remove(self.mobject)


class _From(Transform):
    """Carry a mobject that already shows its final state from where it was."""

    def __init__(self, mobject: Mobject, start: Mobject, **kwargs):
        self.start = start
        super().__init__(mobject, mobject.copy(), **kwargs)

    def create_starting_mobject(self) -> Mobject:
        start = self.start.copy()
        for _ in range(2):  # aligning one pair can pad the shared member, so go round twice
            start.align_data(self.target_copy)
            self.mobject.align_data(start)
            self.target_copy.align_data(self.mobject)
        return start


def figure(value: float) -> str:
    return style.number(value, 0 if float(value).is_integer() else 2)


class Formula(VGroup):
    """w·x + b > 0 and the like. Everything is drawn from the start; fill() and move_into_bias() change
    what is written at once and return the animation that shows it."""

    def __init__(self, terms: list[str], comparison: str = "> 0"):
        super().__init__()
        self.terms = [self._parse(term) for term in terms]
        self.relation, self.right = comparison.split(None, 1)
        self.values: dict[str, float] = {}
        self.moved: float | None = None
        self.zoom = 1.0  # how far the formula has been scaled, so pieces rebuilt later keep its size
        self.pieces: list[Piece] = []
        self.tokens: dict[str, Mobject] = {}
        self.digit_height = serif("0", style.TEXT, italic=False).height
        self._set(self._pieces(), ORIGIN, animate=False)

    @staticmethod
    def _parse(term: str) -> tuple[str | None, list[str]]:
        term = term.strip().replace("-", MINUS)
        op = term[0] if term[0] in "+" + MINUS else None
        body = term[1:] if op else term
        return op, [f.strip() for f in body.split(DOT)]

    def _pieces(self) -> list[Piece]:
        out: list[Piece] = []
        for i, (op, factors) in enumerate(self.terms):
            filled = any(f in self.values for f in factors)
            if len(factors) == 1 and factors[0] in self.values:
                value = self.values[factors[0]]
                if op is None:
                    out.append(Piece(f"f:{factors[0]}", figure(value), style.temperature(value), False))
                else:
                    sign = op if value >= 0 else ({"+": MINUS, MINUS: "+"}[op])
                    out.append(Piece(f"op{i}", sign, style.TEXT, False, operator=True))
                    out.append(Piece(f"f:{factors[0]}", figure(abs(value)), style.temperature(value), False))
                continue
            if op:
                out.append(Piece(f"op{i}", op, style.TEXT, False, operator=True))
            for j, f in enumerate(factors):
                if j:
                    out.append(Piece(f"dot{i}.{j}", DOT, style.TEXT, False, not filled, True))
                if f in self.values:
                    value = self.values[f]
                    out.append(Piece(f"f:{f}", figure(value), style.temperature(value), False))
                else:
                    out.append(Piece(f"f:{f}", f, style.TEXT, True, j > 0 and not filled))
        if self.moved is not None:
            out.append(Piece("minus", MINUS, style.TEXT, False, operator=True))
            out.append(Piece("thr", figure(self.moved), style.temperature(-self.moved), False))
        out.append(Piece("cmp", self.relation, style.TEXT, False, operator=True))
        if self.moved is None:
            out.append(Piece("thr" if self.right != "0" else "zero", self.right, style.TEXT, False))
        else:
            out.append(Piece("zero", "0", style.TEXT, False))
        return out

    def scale(self, scale_factor: float, **kwargs) -> "Formula":
        self.zoom *= scale_factor
        return super().scale(scale_factor, **kwargs)

    def text(self) -> str:
        """The formula as written now."""
        return "".join(("" if p.tight or i == 0 else " ") + p.text for i, p in enumerate(self._pieces()))

    def token(self, key: str) -> Mobject:
        """The piece with this key: "f:w", "f:b", "thr", "cmp"..."""
        return self.tokens[key]

    def _build(self, pieces: list[Piece], at: np.ndarray) -> dict[str, Mobject]:
        """The pieces laid in a row on a baseline, centred on at's x and standing on at's y."""
        mobs = {p.key: serif(p.text, p.colour, italic=p.italic) for p in pieces}
        x = 0.0
        for i, p in enumerate(pieces):
            m = mobs[p.key]
            x += 0 if i == 0 else (TIGHT if p.tight else GAP)
            m.move_to([x + m.width / 2, 0, 0])
            m.shift(UP * (self.digit_height * AXIS - m.get_center()[1] if p.operator else -m.get_bottom()[1]))
            x += m.width
        row = VGroup(*mobs.values()).scale(self.zoom, about_point=ORIGIN)
        row.shift(at - np.array([x * self.zoom / 2, 0, 0]))
        return mobs

    def _set(self, pieces: list[Piece], at: np.ndarray, animate: bool, when: dict | None = None,
             arcs: dict | None = None) -> list[Animation]:
        """Lay the pieces out and return their animations. A piece whose text or slant is unchanged is
        carried from where it was; one whose text changes is crossfaded (morphing one glyph into another
        scribbles); a new piece fades in. `when` gives pieces their window of the animation."""
        new = self._build(pieces, at)
        old = self.tokens
        moves: list[Animation] = []
        if animate:
            for p in pieces:
                m, rate = new[p.key], window(*(when or {}).get(p.key, (0.0, 1.0)))
                before = old.get(p.key)
                if before is not None and (before.original_text, before.slant) == (m.original_text, m.slant):
                    moves.append(_From(m, before.copy(), path_arc=(arcs or {}).get(p.key, 0), rate_func=rate))
                elif before is not None:
                    moves.append(_Ghost(before.copy(), self, m.get_center(), rate_func=rate))
                    moves.append(_From(m, m.copy().set_opacity(0).move_to(before.get_center()), rate_func=rate))
                else:
                    moves.append(_From(m, m.copy().set_opacity(0), rate_func=rate))
        swap(self, *[new[p.key] for p in pieces])
        self.tokens, self.pieces = new, pieces
        return moves

    def _change(self, when: dict | None = None, arcs: dict | None = None) -> Animation:
        letters = [m for p, m in zip(self.pieces, self.tokens.values()) if not p.operator]
        at = np.array([self.get_center()[0], letters[0].get_bottom()[1], 0])
        return AnimationGroup(*self._set(self._pieces(), at, True, when, arcs), lag_ratio=0)

    def fill(self, name: str, value: float) -> Animation:
        """Replace a symbol with its number, in its sign's temperature."""
        self.values[name] = value
        return self._change()

    def move_into_bias(self, threshold: float) -> Animation:
        """The "> 100" becomes "− 100" beside the bias, leaving "> 0"."""
        self.moved = threshold
        # The threshold hops over the comparison first; the comparison slides out as it clears, the minus
        # appears where the comparison was, and the new zero comes last.
        when = {"thr": (0.0, 0.6), "cmp": (0.3, 0.75), "minus": (0.45, 0.8), "zero": (0.7, 1.0)}
        return self._change(when, {"thr": ARC})
