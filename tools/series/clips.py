"""Each built beat's busiest few seconds, kept as a small looping clip for
the title walls of later renders, as its still is."""

import os
from collections import deque
from pathlib import Path

import av
import numpy as np
from PIL import Image

from manim import ImageMobject
from manim.utils.images import change_to_rgba_array

# A clip's length, its frame rate at most, and how much smaller than the frame each side is.
CLIP_SECONDS = 2.5
CLIP_FPS = 30
SHRINK = 4
# Seconds a clip takes to fade out at its loop's end, and in again at its start.
FADE = 0.2


class Capture:
    """The frames of one beat as they're drawn, `fps` a second, keeping the
    CLIP_SECONDS whose pictures change the most (the latest, of equals, so
    a beat's motion is seen with where it settles)."""

    def __init__(self, fps: int):
        self.every = max(1, round(fps / CLIP_FPS))
        self.rate = fps // self.every
        self.size = round(CLIP_SECONDS * self.rate)
        self.window: deque[tuple[np.ndarray, float]] = deque()
        self.change = 0.0
        self.best: list[np.ndarray] | None = None
        self.best_change = -1.0
        self.count = 0
        self.last: np.ndarray | None = None

    def add(self, frame: np.ndarray, repeat: int = 1) -> None:
        kept = sum(1 for i in range(self.count, self.count + repeat) if i % self.every == 0)
        self.count += repeat
        if kept == 0:
            return
        h, w = frame.shape[:2]
        small = np.asarray(Image.fromarray(np.ascontiguousarray(frame[..., :3])).resize(
            (max(1, w // SHRINK), max(1, h // SHRINK)), Image.Resampling.BOX))
        for _ in range(kept):
            change = 0.0 if self.last is None else float(np.abs(
                small[::4, ::4].astype(np.int16) - self.last[::4, ::4]).mean())
            self.last = small
            self.window.append((small, change))
            self.change += change
            if len(self.window) > self.size:
                self.change -= self.window.popleft()[1]
            if len(self.window) == self.size and self.change >= self.best_change:
                self.best, self.best_change = [f for f, _ in self.window], self.change

    def frames(self) -> list[np.ndarray]:
        return self.best if self.best is not None else [f for f, _ in self.window]


def save_clip(frames: list[np.ndarray], fps: int, path: Path) -> None:
    """Write `frames` (RGB) as an H.264 clip, trimmed to even sides."""
    path.parent.mkdir(parents=True, exist_ok=True)
    h, w = (frames[0].shape[0] // 2) * 2, (frames[0].shape[1] // 2) * 2
    partial = path.with_name(f".{path.name}")
    with av.open(str(partial), "w", format="mp4") as out:
        stream = out.add_stream("libx264", rate=fps)
        stream.width, stream.height, stream.pix_fmt = w, h, "yuv420p"
        stream.options = {"crf": "20"}
        for f in frames:
            out.mux(stream.encode(av.VideoFrame.from_ndarray(np.ascontiguousarray(f[:h, :w]), format="rgb24")))
        out.mux(stream.encode())
    os.replace(partial, path)


class Clip:
    def __init__(self, frames: list[np.ndarray], fps: int):
        self.frames, self.fps = frames, fps

    @property
    def seconds(self) -> float:
        return len(self.frames) / self.fps


def load_clip(path: Path) -> Clip:
    with av.open(str(path)) as source:
        stream = source.streams.video[0]
        frames = [f.to_ndarray(format="rgb24") for f in source.decode(stream)]
        return Clip(frames, round(float(stream.average_rate)))


class ClipTile(ImageMobject):
    """A clip, looping, fading through black at each seam; `phase` (0 to 1)
    starts it partway, so a wall's tiles don't all blink at once."""

    def __init__(self, clip: Clip, phase: float = 0.0, **kwargs):
        self.clip, self.elapsed = clip, phase * clip.seconds
        super().__init__(self.picture(), **kwargs)
        self.add_updater(ClipTile.advance)

    def __deepcopy__(self, memo: dict) -> "ClipTile":
        memo[id(self.clip)] = self.clip  # copies share the frames
        return super().__deepcopy__(memo)

    def picture(self) -> np.ndarray:
        length = self.clip.seconds
        t = self.elapsed % length
        index = min(int(t * self.clip.fps), len(self.clip.frames) - 1)
        light = max(0.0, min(1.0, t / FADE, (length - t) / FADE))
        return change_to_rgba_array((self.clip.frames[index] * light).astype(np.uint8))

    def advance(self, dt: float) -> None:
        self.elapsed += dt
        self.pixel_array = self.picture()
