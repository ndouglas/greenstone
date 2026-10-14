#!/usr/bin/env python3
"""Compose the Tetris reservoir explainer video from a `reservoir_dump` directory.

Usage: python3 tools/reservoir_video.py [dump-dir] [out.mp4] [--quick]

`--quick` renders every chapter at a tenth of its length, for checking layout.
"""

import json
import subprocess
import sys
from pathlib import Path

import numpy as np
from PIL import Image, ImageDraw, ImageFont

W, H = 1280, 720
FPS = 60
BG = (18, 18, 24)
PANEL = (30, 30, 40)
TEXT = (235, 235, 235)
DIM = (140, 140, 150)
ACCENT = (255, 200, 60)
RAW_COLOR = (90, 180, 255)
PLAY_COLOR = (255, 120, 90)
TRUTH_COLOR = (120, 230, 120)
RED = (230, 70, 70)

FONT_DIR = Path("/System/Library/Fonts")
FONTS = {}


def font(size, mono=False):
  key = (size, mono)
  if key not in FONTS:
    path = FONT_DIR / ("Menlo.ttc" if mono else "Supplemental/Arial.ttf")
    FONTS[key] = ImageFont.truetype(str(path), size)
  return FONTS[key]


# SMB 1-1 pits by level x, from the level data; the first is where the search
# fell in before pit deaths were detected.
PITS = [(1104, 1136), (1376, 1424), (2448, 2480)]
FLAGPOLE_X = 3175
LEVEL_END = 3300
ENEMY_NAMES = {0x00: "Koopa", 0x06: "Goomba", 0x0D: "Piranha"}
WINDOW_LABELS = {
  0x86: "Mario x",
  0xCE: "Mario y",
  0x57: "x speed",
  0x9F: "y speed",
  0x87: "enemy x",
  0x1D: "airborne",
  0x0E: "state",
}


class Dump:
  def __init__(self, directory):
    d = Path(directory)
    self.demo = json.loads((d / "demo.json").read_text())
    self.steps = self.demo["steps"]
    for s in self.steps:
      s["window_bytes"] = bytes.fromhex(s["window"])
      s["board_bytes"] = bytes.fromhex(s["board"])
      s["tetris_bytes"] = bytes.fromhex(s.get("tetris_ram", ""))
    n_steps = len(self.steps)
    self.smb = np.memmap(d / "demo.smb.rgb", dtype=np.uint8, mode="r").reshape(-1, 240, 256, 3)
    self.tetris = np.memmap(d / "demo.tetris.rgb", dtype=np.uint8, mode="r").reshape(n_steps, 240, 256, 3)
    self.frames_per_step = self.smb.shape[0] // n_steps
    self.conditions = self.demo["conditions"]
    self.episodes = {}
    for name in ("raw", "clock"):
      meta = json.loads((d / f"episode-{name}.json").read_text())
      frames = np.memmap(d / f"episode-{name}.smb.rgb", dtype=np.uint8, mode="r").reshape(-1, 240, 256, 3)
      self.episodes[name] = (meta["steps"], frames)
    self.progress = self._progress()
    self.enemies = self._enemies()

  def _progress(self):
    """Highest x so far, frozen once the level is cleared and x resets."""
    out, best = [], 0
    for s in self.steps:
      if s["x"] >= best - 500:
        best = max(best, s["x"])
      out.append(best)
    return out

  def _enemies(self):
    """One entry per enemy, at the x where it first appeared, before the clear."""
    seen, last = [], {}
    for s in self.steps:
      if s["cutscene"] or s["x"] < self.progress[0] - 500:
        continue
      for e in s["enemies"]:
        prev = last.get(e["slot"])
        fresh = prev is None or prev[0] != e["kind"] or e["x"] > prev[1] + 64
        if fresh and e["x"] <= LEVEL_END:
          seen.append((e["kind"], e["x"]))
        last[e["slot"]] = (e["kind"], e["x"])
      for slot in list(last):
        if not any(e["slot"] == slot for e in s["enemies"]):
          del last[slot]
    return sorted(seen, key=lambda e: e[1])

  def a_output(self, step, condition):
    i = self.conditions.index(condition)
    return self.steps[step]["outputs"][i][0]

  def a_pressed(self, step):
    return self.steps[step]["buttons"] & 1 == 1


# --- drawing helpers -------------------------------------------------------


def screen(fb, scale):
  return Image.fromarray(np.asarray(fb)).resize((256 * scale, 240 * scale), Image.NEAREST)


def panel(draw, box, title=None):
  draw.rounded_rectangle(box, radius=8, fill=PANEL)
  if title:
    draw.text((box[0] + 10, box[1] + 6), title, font=font(16), fill=DIM)


def caption(img, text, sub=None):
  draw = ImageDraw.Draw(img, "RGBA")
  draw.rectangle((0, H - 96, W, H), fill=(0, 0, 0, 200))
  f = font(26)
  lines = wrap(text, f, W - 80)
  y = H - 88
  for line in lines[:2]:
    draw.text((40, y), line, font=f, fill=TEXT)
    y += 32
  if sub:
    draw.text((40, H - 24), sub, font=font(16), fill=DIM)


def wrap(text, f, width):
  words = text.split()
  lines, cur = [], ""
  for w in words:
    trial = (cur + " " + w).strip()
    if f.getlength(trial) > width and cur:
      lines.append(cur)
      cur = w
    else:
      cur = trial
  if cur:
    lines.append(cur)
  return lines


def byte_grid(draw, origin, data, cols, cell, changed=None, outline_first=None, labels=None, signed=False):
  """Draw bytes (or floats when signed) as a grid of cells lit by value."""
  ox, oy = origin
  rows = (len(data) + cols - 1) // cols
  for i, v in enumerate(data):
    r, c = divmod(i, cols)
    x0, y0 = ox + c * cell, oy + r * cell
    if signed:
      mag = min(1.0, abs(v))
      color = (int(60 + 195 * mag), 60, 60) if v < 0 else (60, int(60 + 195 * mag), 80) if v > 0 else (40, 40, 48)
    else:
      t = v / 255.0
      color = (int(30 + 200 * t), int(30 + 180 * t), int(40 + 120 * t))
    draw.rectangle((x0, y0, x0 + cell - 1, y0 + cell - 1), fill=color)
    if changed is not None and changed[i]:
      draw.rectangle((x0, y0, x0 + cell - 1, y0 + cell - 1), outline=ACCENT, width=2)
  if outline_first:
    r = (outline_first + cols - 1) // cols
    draw.rectangle((ox - 2, oy - 2, ox + cols * cell + 1, oy + r * cell + 1), outline=ACCENT, width=2)
  if labels:
    for addr, name in labels.items():
      r, c = divmod(addr, cols)
      x0, y0 = ox + c * cell, oy + r * cell
      draw.rectangle((x0, y0, x0 + cell - 1, y0 + cell - 1), outline=(255, 255, 255), width=2)
      draw.text((x0 + cell + 2, y0 - 2), name, font=font(13), fill=TEXT)
  return rows


def board(draw, origin, cells, cell):
  """Tetris playfield from RAM: 10 columns by 20 rows, 0xEF is empty."""
  ox, oy = origin
  draw.rectangle((ox - 2, oy - 2, ox + 10 * cell + 1, oy + 20 * cell + 1), fill=(10, 10, 14), outline=DIM)
  for i, v in enumerate(cells):
    r, c = divmod(i, 10)
    x0, y0 = ox + c * cell, oy + r * cell
    if v != 0xEF:
      draw.rectangle((x0 + 1, y0 + 1, x0 + cell - 2, y0 + cell - 2), fill=(230, 160, 70))
    elif r < 4:
      draw.rectangle((x0 + 1, y0 + 1, x0 + cell - 2, y0 + cell - 2), fill=(24, 24, 30))


def gauge(draw, box, value, color, label):
  x0, y0, x1, y1 = box
  draw.rectangle(box, fill=(40, 40, 50))
  v = max(0.0, min(1.0, value))
  draw.rectangle((x0, y0, x0 + (x1 - x0) * v, y1), fill=color)
  tx = x0 + (x1 - x0) * 0.5
  draw.line((tx, y0 - 4, tx, y1 + 4), fill=TEXT, width=2)
  draw.text((x0, y0 - 22), f"{label}  {value:+.2f}", font=font(16), fill=TEXT)
  if value > 0.5:
    draw.text((x1 - 60, y0 - 22), "JUMP", font=font(16), fill=color)


def strip(draw, box, dump, x_now=None, a_ticks=None, deaths=None):
  """The level laid out left to right by x."""
  x0, y0, x1, y1 = box
  draw.rectangle(box, fill=(26, 26, 34))
  ground = y1 - 14

  def sx(x):
    return x0 + (x1 - x0) * x / LEVEL_END

  draw.line((x0, ground, x1, ground), fill=(110, 80, 50), width=3)
  for a, b in PITS:
    draw.rectangle((sx(a), ground - 1, sx(b), ground + 2), fill=(26, 26, 34))
    draw.rectangle((sx(a), ground + 3, sx(b), y1), fill=RED)
  for kind, ex in dump.enemies:
    color = (200, 120, 60) if kind == 0x06 else (90, 200, 90)
    draw.ellipse((sx(ex) - 4, ground - 10, sx(ex) + 4, ground - 2), fill=color)
  draw.line((sx(FLAGPOLE_X), y0 + 8, sx(FLAGPOLE_X), ground), fill=(200, 200, 200), width=2)
  draw.text((sx(FLAGPOLE_X) + 4, y0 + 6), "flag", font=font(13), fill=DIM)
  if a_ticks:
    for ax in a_ticks:
      draw.line((sx(ax), ground - 24, sx(ax), ground - 14), fill=TRUTH_COLOR, width=2)
  if deaths:
    for dx in deaths:
      draw.text((sx(dx) - 6, ground - 34), "x", font=font(20), fill=RED)
  if x_now is not None:
    draw.polygon([(sx(x_now), ground - 30), (sx(x_now) - 6, ground - 42), (sx(x_now) + 6, ground - 42)], fill=ACCENT)


def trace_panel(draw, box, dump, step, span=150):
  """The last `span` steps: truth as blocks, readouts as lines, threshold at 0.5."""
  x0, y0, x1, y1 = box
  draw.rectangle(box, fill=(26, 26, 34))
  draw.text((x0 + 8, y0 + 4), "A button, last 10 seconds", font=font(14), fill=DIM)
  lo = max(0, step - span + 1)
  plot_top, plot_bot = y0 + 24, y1 - 8
  mid = plot_bot - (plot_bot - plot_top) * 0.5

  def px(s):
    return x0 + (x1 - x0) * (s - (step - span + 1)) / span

  def py(v):
    v = max(-0.25, min(1.25, v))
    return plot_bot - (plot_bot - plot_top) * (v + 0.25) / 1.5

  for s in range(lo, step + 1):
    if dump.a_pressed(s):
      draw.rectangle((px(s), plot_top, px(s + 1), plot_bot), fill=(40, 70, 40))
  draw.line((x0, mid, x1, mid), fill=(120, 120, 130), width=1)
  for cond, color in (("raw", RAW_COLOR), ("driven-playfield", PLAY_COLOR)):
    pts = [(px(s), py(dump.a_output(s, cond))) for s in range(lo, step + 1)]
    if len(pts) > 1:
      draw.line(pts, fill=color, width=2)
  legend_y = y0 + 4
  draw.rectangle((x1 - 300, legend_y + 2, x1 - 290, legend_y + 12), fill=(40, 70, 40))
  draw.text((x1 - 284, legend_y - 2), "example pressed A", font=font(13), fill=DIM)
  draw.line((x1 - 150, legend_y + 7, x1 - 140, legend_y + 7), fill=RAW_COLOR, width=2)
  draw.text((x1 - 134, legend_y - 2), "raw", font=font(13), fill=DIM)
  draw.line((x1 - 100, legend_y + 7, x1 - 90, legend_y + 7), fill=PLAY_COLOR, width=2)
  draw.text((x1 - 84, legend_y - 2), "Tetris", font=font(13), fill=DIM)


def buttons_lamp(draw, origin, buttons):
  ox, oy = origin
  for i, (name, bit) in enumerate((("A", 1), ("B", 2), ("Right", 128))):
    on = buttons & bit
    x = ox + i * 70
    draw.rounded_rectangle((x, oy, x + 60, oy + 28), radius=6, fill=ACCENT if on else (50, 50, 60))
    draw.text((x + 8, oy + 4), name, font=font(16), fill=(20, 20, 20) if on else DIM)


def changed_mask(dump, step):
  if step == 0:
    return [False] * 256
  a, b = dump.steps[step - 1]["window_bytes"], dump.steps[step]["window_bytes"]
  return [x != y for x, y in zip(a, b)]


def caption_at(t, schedule):
  """`schedule` is a list of (start_seconds, text)."""
  text = schedule[0][1]
  for start, s in schedule:
    if t >= start:
      text = s
  return text


# --- chapters --------------------------------------------------------------


def title_card(dump, n, title, sub):
  for i in range(n):
    img = Image.new("RGB", (W, H), BG)
    d = ImageDraw.Draw(img)
    d.text((80, 260), title, font=font(56), fill=TEXT)
    d.text((80, 340), sub, font=font(26), fill=DIM)
    yield img


def chapter_bytes(dump, n):
  schedule = [
    (0, "Super Mario Bros. runs on a computer with 2,048 bytes of memory. Everything Mario is, is in those numbers."),
    (7, "The first 256 bytes are where the action lives. We call them the window. One square per byte, brighter means bigger."),
    (14, "Yellow squares just changed. Mario's position, his speed, and where the enemies are, all live here."),
    (21, "A policy is any rule that turns these 256 numbers into buttons: A to jump, B to run, Right to go right."),
  ]
  fps = dump.frames_per_step
  for i in range(n):
    frame = min(i, dump.smb.shape[0] - 1)
    step = frame // fps
    img = Image.new("RGB", (W, H), BG)
    d = ImageDraw.Draw(img)
    d.text((40, 20), "1. The game is 2,048 numbers", font=font(30), fill=TEXT)
    img.paste(screen(dump.smb[frame], 2), (40, 70))
    panel(d, (600, 70, 1240, 600), "the window: bytes $00 to $FF")
    t = i / FPS
    labels = WINDOW_LABELS if t >= 14 else None
    byte_grid(d, (640, 110), dump.steps[step]["window_bytes"], 16, 28, changed=changed_mask(dump, step), labels=labels)
    buttons_lamp(d, (40, 560), dump.steps[step]["buttons"])
    caption(img, caption_at(t, schedule))
    yield img


def chapter_demo(dump, n):
  schedule = [
    (0, "To teach anything, we need an example. We searched for one."),
    (6, "Every 24 frames the search tries six moves: run, jump now, jump a little later, walk, wait. It keeps the sixteen most advanced positions and goes again."),
    (14, "Green ticks mark where the example pressed A. It clears the level in 540 steps."),
    (24, "We did this six times from slightly different starts, so no two examples agree on exactly when to jump."),
  ]
  fps = dump.frames_per_step
  total_frames = dump.smb.shape[0]
  play = max(1, n - 8 * FPS)
  ticks = []
  prev_a = False
  for i in range(n):
    t = i / FPS
    frame = min(int((i / play) * total_frames), total_frames - 1)
    step = frame // fps
    a = dump.a_pressed(step)
    if a and not prev_a:
      ticks.append(dump.progress[step])
    prev_a = a
    img = Image.new("RGB", (W, H), BG)
    d = ImageDraw.Draw(img)
    d.text((40, 20), "2. A demonstration", font=font(30), fill=TEXT)
    strip(d, (40, 70, 1240, 150), dump, x_now=dump.progress[step], a_ticks=ticks)
    img.paste(screen(dump.smb[frame], 2), (40, 170))
    panel(d, (600, 170, 1240, 600), "the search, each round")
    for k, line in enumerate((
      "six moves from every kept position",
      "drop anything that dies, or would die next round",
      "drop anything that stopped moving right",
      "keep one copy of each distinct game state",
      "keep the sixteen furthest right, highest up",
      f"step {step} of {len(dump.steps)}   x = {dump.progress[step]}",
    )):
      d.text((630, 215 + k * 36), line, font=font(20), fill=TEXT if k < 5 else ACCENT)
    buttons_lamp(d, (40, 560), dump.steps[step]["buttons"])
    caption(img, caption_at(t, schedule))
    yield img


def chapter_readout(dump, n):
  schedule = [
    (0, "A readout is the simplest possible brain: one weight per byte."),
    (6, "Multiply each byte by its weight, add them up. If the total is over one half, press A."),
    (13, "Training means choosing the 257 weights that best copy the six examples. This is what it chose: green pulls toward jumping, red away."),
    (22, "Watch the gauge against the real presses. It rises at the jumps, but rarely clears the line. A weighted sum cannot say 'jump when the Goomba is close'."),
  ]
  weights = dump.demo["weights"]["raw_a"]
  scale = max(1e-9, max(abs(w) for w in weights[1:]))
  w_cells = [w / scale for w in weights[1:]]
  steps = len(dump.steps)
  play = max(1, n - 6 * FPS)
  for i in range(n):
    t = i / FPS
    step = min(int((i / play) * steps), steps - 1)
    img = Image.new("RGB", (W, H), BG)
    d = ImageDraw.Draw(img)
    d.text((40, 20), "3. A readout", font=font(30), fill=TEXT)
    panel(d, (40, 70, 440, 470), "window bytes")
    byte_grid(d, (60, 105), dump.steps[step]["window_bytes"], 16, 22)
    d.text((455, 260), "x", font=font(40), fill=DIM)
    panel(d, (500, 70, 900, 470), "weights for A, one per byte")
    byte_grid(d, (520, 105), w_cells, 16, 22, signed=True)
    d.text((915, 260), "=", font=font(40), fill=DIM)
    panel(d, (960, 70, 1240, 470), "A gauge")
    gauge(d, (980, 200, 1220, 240), dump.a_output(step, "raw"), RAW_COLOR, "raw readout")
    truth = dump.a_pressed(step)
    d.text((980, 290), "example pressed A:", font=font(16), fill=DIM)
    d.text((980, 312), "yes" if truth else "no", font=font(30), fill=TRUTH_COLOR if truth else DIM)
    trace_panel(d, (40, 490, 1240, 610), dump, step)
    caption(img, caption_at(t, schedule))
    yield img


def chapter_reservoir(dump, n):
  schedule = [
    (0, "Now the strange part. Instead of reading Mario's bytes directly, we pour them into Tetris."),
    (7, "Each step: bytes of 192 or more become blocks, pasted into the board. Only the first 160 bytes fit; the rest never reach Tetris."),
    (16, "Then Tetris runs four frames. The piece falls, lands on the pasted blocks, lines clear. Its own 2,048 bytes are now what the readout sees."),
    (25, "The screen lies: Tetris only redraws rows when a piece lands. The board in memory is what matters, and it is what we draw here."),
    (33, "The hope: a running game scrambles the numbers in ways a weighted sum cannot, and maybe 'jump now' is easier to read in the scramble."),
  ]
  steps = len(dump.steps)
  fps = dump.frames_per_step
  for i in range(n):
    t = i / FPS
    frame = min(i, dump.smb.shape[0] - 1)
    step = frame // fps
    s = dump.steps[step]
    img = Image.new("RGB", (W, H), BG)
    d = ImageDraw.Draw(img)
    d.text((40, 20), "4. A reservoir", font=font(30), fill=TEXT)
    panel(d, (30, 70, 300, 480), "Mario")
    img.paste(screen(dump.smb[frame], 1), (37, 100))
    buttons_lamp(d, (40, 360), s["buttons"])
    d.text((40, 400), f"step {step}   x {s['x']}", font=font(16), fill=DIM)
    panel(d, (310, 70, 590, 480), "window, first 160 outlined")
    byte_grid(d, (322, 100), s["window_bytes"], 16, 16, outline_first=160)
    d.text((596, 250), ">", font=font(30), fill=DIM)
    panel(d, (620, 70, 800, 480), "board, from RAM")
    board(d, (630, 100), s["board_bytes"], 16)
    panel(d, (810, 70, 1080, 480), "Tetris screen")
    img.paste(screen(dump.tetris[step], 1), (817, 100))
    d.text((1086, 250), ">", font=font(30), fill=DIM)
    panel(d, (1110, 70, 1250, 480), "RAM")
    if s["tetris_bytes"]:
      byte_grid(d, (1118, 96), s["tetris_bytes"], 32, 4)
    gauge(d, (60, 560, 600, 590), dump.a_output(step, "driven-playfield"), PLAY_COLOR, "Tetris readout, A")
    gauge(d, (660, 560, 1200, 590), dump.a_output(step, "raw"), RAW_COLOR, "raw readout, A")
    caption(img, caption_at(t, schedule))
    yield img


def chapter_realtime(dump, n):
  schedule = [
    (0, "Everything together, in real time. The example plays. The two readouts watch the same bytes and say what they would press."),
    (10, ""),
    (30, "Level clear. Both readouts agreed with the examples about 89% of the time. Nearly every miss is the A button."),
  ]
  fps = dump.frames_per_step
  total = dump.smb.shape[0]
  for i in range(n):
    t = i / FPS
    frame = min(i, total - 1)
    step = frame // fps
    s = dump.steps[step]
    img = Image.new("RGB", (W, H), BG)
    d = ImageDraw.Draw(img)
    d.text((40, 14), "5. The whole level, in real time", font=font(26), fill=TEXT)
    img.paste(screen(dump.smb[frame], 2), (40, 50))
    panel(d, (570, 50, 800, 530), "board, from RAM")
    board(d, (605, 80), s["board_bytes"], 16)
    buttons_lamp(d, (582, 440), s["buttons"])
    d.text((582, 480), f"step {step}   x {s['x']}", font=font(16), fill=DIM)
    panel(d, (820, 50, 1240, 300), "window")
    byte_grid(d, (840, 82), s["window_bytes"], 16, 13, changed=changed_mask(dump, step))
    img.paste(screen(dump.tetris[step], 1), (820, 320))
    d.text((1090, 330), "Tetris screen", font=font(14), fill=DIM)
    d.text((1090, 352), "(it only redraws", font=font(14), fill=DIM)
    d.text((1090, 370), "when a piece lands)", font=font(14), fill=DIM)
    strip(d, (40, 545, 1240, 600), dump, x_now=dump.progress[step])
    gauge(d, (60, 630, 420, 650), 1.0 if dump.a_pressed(step) else 0.0, TRUTH_COLOR, "example")
    gauge(d, (460, 630, 820, 650), dump.a_output(step, "raw"), RAW_COLOR, "raw readout")
    gauge(d, (860, 630, 1220, 650), dump.a_output(step, "driven-playfield"), PLAY_COLOR, "Tetris readout")
    text = caption_at(t, schedule)
    if text:
      caption(img, text)
    yield img


def chapter_coda(dump, n):
  raw_steps, raw_frames = dump.episodes["raw"]
  clock_steps, clock_frames = dump.episodes["clock"]
  fps = dump.frames_per_step
  raw_n = raw_frames.shape[0]
  clock_n = clock_frames.shape[0]
  hold = 2 * FPS
  schedule_raw = [(0, "What happens when the raw readout actually drives? Its buttons, its consequences.")]
  part1 = raw_n + hold
  part2 = n - part1
  for i in range(n):
    img = Image.new("RGB", (W, H), BG)
    d = ImageDraw.Draw(img)
    if i < part1:
      frame = min(i, raw_n - 1)
      step = min(frame // fps, len(raw_steps) - 1)
      d.text((40, 14), "5b. Closed loop: the raw readout drives", font=font(26), fill=TEXT)
      img.paste(screen(raw_frames[frame], 2), (40, 50))
      s = raw_steps[step]
      buttons_lamp(d, (600, 80), s["buttons"])
      d.text((600, 130), f"step {step}   x {s['x']}", font=font(20), fill=TEXT)
      deaths = [raw_steps[-1]["x"]] if raw_steps[-1]["dead"] and i >= raw_n - 1 else None
      strip(d, (40, 545, 1240, 600), dump, x_now=s["x"], deaths=deaths)
      text = schedule_raw[0][1] if i < raw_n - 1 else "Dead at the first Goomba. Every episode, every state-based readout, the same spot."
      caption(img, text)
    else:
      j = i - part1
      frame = min(int(j / max(1, part2 - hold) * clock_n), clock_n - 1)
      step = min(frame // fps, len(clock_steps) - 1)
      d.text((40, 14), "5c. Closed loop: a clock drives", font=font(26), fill=TEXT)
      img.paste(screen(clock_frames[frame], 2), (40, 50))
      s = clock_steps[step]
      buttons_lamp(d, (600, 80), s["buttons"])
      d.text((600, 130), f"step {step}   x {s['x']}", font=font(20), fill=TEXT)
      deaths = [clock_steps[-1]["x"]] if clock_steps[-1]["dead"] and frame >= clock_n - 1 else None
      strip(d, (40, 545, 1240, 600), dump, x_now=s["x"], deaths=deaths)
      caption(img, "A readout that knows only what time it is gets further, by replaying the examples' timing. It is not playing; it is reciting.", "shown at double speed")
    yield img


def chapter_verdict(dump, n):
  rows = [
    "condition          train acc  held-out acc  mean x (closed loop)",
    "clock                  0.861         0.808   999",
    "raw                    0.896         0.891   313",
    "undriven Tetris        0.806         0.797   814",
    "Tetris, controller     0.805         0.793   765",
    "Tetris, playfield      0.888         0.886   314",
  ]
  lines = [
    "Tetris carries at best what the window already carries, a little degraded.",
    "The readout cannot copy its own examples: a weighted sum of bytes cannot see 'the Goomba is close'.",
    "So the bottleneck is the readout, not the reservoir. Next: give the readout bits instead of bytes.",
  ]
  for i in range(n):
    img = Image.new("RGB", (W, H), BG)
    d = ImageDraw.Draw(img)
    d.text((40, 20), "6. The verdict", font=font(30), fill=TEXT)
    for k, row in enumerate(rows):
      d.text((60, 90 + k * 30), row, font=font(20, mono=True), fill=TEXT if k else DIM)
    for k, line in enumerate(lines):
      if i > (k + 1) * FPS:
        d.text((60, 320 + k * 60), line, font=font(24), fill=TEXT)
    yield img


def chapters(dump, quick):
  spec = [
    (title_card, 4, ("Can Tetris play Mario?", "A reservoir computer made of a commercial ROM, explained piece by piece")),
    (chapter_bytes, 28, ()),
    (chapter_demo, 32, ()),
    (chapter_readout, 34, ()),
    (chapter_reservoir, 42, ()),
    (title_card, 3, ("Now all of it, in real time", "One level of Super Mario Bros., 36 seconds")),
    (chapter_realtime, None, ()),
    (chapter_coda, None, ()),
    (chapter_verdict, 14, ()),
  ]
  for fn, seconds, extra in spec:
    if fn is chapter_realtime:
      n = dump.smb.shape[0] + 4 * FPS
    elif fn is chapter_coda:
      raw_n = dump.episodes["raw"][1].shape[0]
      n = raw_n + 2 * FPS + 10 * FPS
    else:
      n = seconds * FPS
    if quick:
      n = max(FPS, n // 10)
    yield from fn(dump, n, *extra)


def main():
  args = [a for a in sys.argv[1:] if not a.startswith("--")]
  quick = "--quick" in sys.argv
  dump_dir = args[0] if args else "target/tetris-reservoir-dump"
  out = args[1] if len(args) > 1 else "tetris-reservoir.mp4"
  dump = Dump(dump_dir)
  cmd = [
    "ffmpeg", "-y", "-loglevel", "error",
    "-f", "rawvideo", "-pix_fmt", "rgb24", "-s", f"{W}x{H}", "-r", str(FPS), "-i", "-",
    "-c:v", "libx264", "-pix_fmt", "yuv420p", "-crf", "20", "-preset", "fast", out,
  ]
  proc = subprocess.Popen(cmd, stdin=subprocess.PIPE)
  count = 0
  for img in chapters(dump, quick):
    proc.stdin.write(img.tobytes())
    count += 1
    if count % 600 == 0:
      print(f"{count} frames ({count / FPS:.0f}s)", file=sys.stderr)
  proc.stdin.close()
  proc.wait()
  print(f"wrote {out}: {count} frames, {count / FPS:.1f}s", file=sys.stderr)


if __name__ == "__main__":
  main()
