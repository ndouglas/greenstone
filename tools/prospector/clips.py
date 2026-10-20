"""Turn the Prospector's takes into short captioned clips.

Usage, from tools/: python3 -m prospector.clips
"""

import subprocess
from pathlib import Path

from reservoir_film import draw
from reservoir_film.model import REPO, Take

ROOT = REPO / "target" / "prospector"
W, H, FPS = 1280, 720, 60
CLEAR_NAMES = {"flag": "cleared: SMB moved on (flag)", "distance": "cleared: reached x 3072 alive"}


def caption_lines(meta):
    lines = [f"{meta['hex']}  (#{meta['rank']})", CLEAR_NAMES.get(meta.get("clear"), "not cleared"),
             f"bottleneck: {meta['bottleneck']} way(s) through",
             f"search killed {meta['kill_rate']:.0%} of what it tried",
             f"{meta['rounds']} rounds"]
    if meta.get("narrower_than"):
        lines.append("narrower than " + " and ".join(meta["narrower_than"]))
    return lines


def compose(take, i):
    img = draw.blank()
    draw.nes(img, take.frames("smb")[i], (0, 0, 768, 720))
    for k, line in enumerate(caption_lines(take.meta)):
        if k == 0:
            draw.text(img, (800, 60), line, 40, draw.GOLD, face=draw.mono(40))
        else:
            draw.text(img, (800, 60 + k * 56), line, 26, draw.INK)
    draw.text(img, (800, 660), "The Prospector: SMB, loaded from the wrong address", 18, draw.DIM)
    return img


def encode(take, out):
    out = Path(out)
    out.parent.mkdir(parents=True, exist_ok=True)
    ff = subprocess.Popen(["ffmpeg", "-y", "-loglevel", "error", "-f", "rawvideo", "-pix_fmt", "rgb24",
                           "-s", f"{W}x{H}", "-r", str(FPS), "-i", "-", "-c:v", "libx264", "-crf", "20",
                           "-pix_fmt", "yuv420p", str(out)], stdin=subprocess.PIPE)
    for i in range(len(take.frames("smb"))):
        ff.stdin.write(compose(take, i).tobytes())
    ff.stdin.close()
    if ff.wait() != 0:
        raise RuntimeError(f"ffmpeg failed on {take.name}")


def main():
    takes = sorted((Take(p.parent) for p in (ROOT / "takes").glob("*/take.json")), key=lambda t: t.meta["rank"])
    if not takes:
        raise SystemExit("no takes; run: cargo run --release --example prospector -- clips")
    for take in takes:
        out = ROOT / "clips" / f"{take.meta['rank']:02d}-{take.name}.mp4"
        encode(take, out)
        print(out)


if __name__ == "__main__":
    main()
