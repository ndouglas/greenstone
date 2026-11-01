"""A network too big to follow: thousands of dots in columns, each joined to a few in the next, drawn as one image
every frame. It grows outward from its middle dot (`shown`), the camera can push in (`zoom` about `focus`), its
dots light warm or cool (`lit`, -1 to 1 each), and it fades by `level`."""

import numpy as np
from PIL import Image, ImageDraw

from manim import ImageMobject, ValueTracker, config

from series import style
from series.ember import rgb

COLUMNS = 33
LINKS = 3  # each dot's lines to the next column
SUPER = 2  # drawn at twice the frame's pixels, then halved, so lines and dots are smooth


class Web(ImageMobject):
    def __init__(self, count: int = 2400, half_width: float = 6.4, half_height: float = 3.5, seed: int = 0):
        rng = np.random.default_rng(seed)
        xs = np.linspace(-half_width, half_width, COLUMNS)
        # Each column as tall as an ellipse there, so the web is round-edged; dots shared out by that height.
        tall = np.sqrt(np.clip(1 - (xs / (half_width * 1.08)) ** 2, 0.05, 1)) * half_height
        per = np.maximum(1, np.round(count * tall / tall.sum())).astype(int)
        points, column = [], []
        for c, (x, h, n) in enumerate(zip(xs, tall, per)):
            ys = np.linspace(-h, h, n) + rng.normal(0, h / n * 0.35, n)
            points += [(x + rng.normal(0, 0.11), y) for y in ys]
            column += [c] * n
        self.spots, self.column = np.array(points), np.array(column)  # not .points: that is Manim's
        self.count = len(self.spots)
        links = []
        for c in range(COLUMNS - 1):
            here, there = np.flatnonzero(self.column == c), np.flatnonzero(self.column == c + 1)
            for i in here:
                near = there[np.argsort(np.abs(self.spots[there, 1] - self.spots[i, 1]))[:LINKS * 6]]
                links += [(i, j) for j in rng.choice(near, size=min(LINKS, len(near)), replace=False)]
        self.links = np.array(links)
        # Grown outward from the dot nearest the middle; `rank` is each dot's place in that order.
        order = np.argsort(np.hypot(self.spots[:, 0] / half_width, self.spots[:, 1] / half_height)
                           + rng.uniform(0, 0.04, self.count))
        self.rank = np.empty(self.count, float)
        self.rank[order] = np.arange(self.count)
        self.middle = self.spots[order[0]]
        self.shown = ValueTracker(float(self.count))
        self.zoom = ValueTracker(1.0)
        self.focus = np.array(self.middle)
        self.level = ValueTracker(1.0)
        self.lit = np.zeros(self.count)
        super().__init__(self.picture())
        self.set(width=config.frame_width, height=config.frame_height).move_to([0, 0, 0])
        self.add_updater(lambda m: m.draw())

    def where(self, points: np.ndarray) -> np.ndarray:
        """Frame coordinates, through the camera's push-in."""
        return self.focus + (points - self.focus) * self.zoom.get_value()

    def picture(self) -> np.ndarray:
        w, h = config.pixel_width * SUPER, config.pixel_height * SUPER
        per_unit = h / config.frame_height
        shown, zoom, level = self.shown.get_value(), self.zoom.get_value(), self.level.get_value()
        # Each dot fades in over the twentieth of the web grown just before it.
        appear = np.clip((shown - self.rank) / max(1.0, shown * 0.05), 0, 1) * level
        at = self.where(self.spots)
        px = np.column_stack([w / 2 + at[:, 0] * per_unit, h / 2 - at[:, 1] * per_unit])
        image = Image.new("RGBA", (w, h), (0, 0, 0, 0))
        draw = ImageDraw.Draw(image, "RGBA")
        ash, warm, cool = rgb(style.ASH), rgb(style.WARM[0]), rgb(style.COOL[1])
        line_w = max(1, round(SUPER * min(2.0, 0.8 * zoom)))
        for i, j in self.links:
            a = min(appear[i], appear[j])
            if a <= 0.01:
                continue
            heat = (abs(self.lit[i]) + abs(self.lit[j])) / 2
            colour = ash if heat < 0.05 else (warm if self.lit[i] + self.lit[j] > 0 else cool)
            alpha = (0.10 + 0.35 * heat) * a
            draw.line([tuple(px[i]), tuple(px[j])], fill=(*colour.astype(int), round(255 * alpha)), width=line_w)
        # A lone dot is drawn large, a crowd small.
        base = SUPER * (1.6 + 6.0 * np.clip(1 - np.log10(max(shown, 1)) / 2, 0, 1)) * zoom ** 0.5
        for i in np.flatnonzero(appear > 0.01):
            x, y = px[i]
            if not (-20 < x < w + 20 and -20 < y < h + 20):
                continue
            heat = abs(self.lit[i])
            colour = ash * (1 - heat) + (warm if self.lit[i] >= 0 else cool) * heat
            r = base * (1 + 0.7 * heat)
            alpha = (0.55 + 0.45 * heat) * appear[i]
            draw.ellipse([x - r, y - r, x + r, y + r], fill=(*colour.astype(int), round(255 * alpha)))
        return np.asarray(image.resize((config.pixel_width, config.pixel_height), Image.Resampling.BOX))

    def draw(self) -> None:
        self.pixel_array = self.picture()
