"""The Prospector's contact sheet: one static page, worst levels first.

Usage, from tools/: python3 -m prospector.sheet
"""

import html
import json
from pathlib import Path

from PIL import Image

from reservoir_film.model import REPO

ROOT = REPO / "target" / "prospector"
SECTION_TITLES = ("Cleared, worst first", "Unsolved", "Walked, not searched", "Going nowhere",
                  "Instant death", "Hall of shame")
SECTION_CLASSES = (None, {"unsolved"}, {"walked"}, {"stuck", "walker_died"}, {"instant_death"},
                   {"crashed", "jammed", "blank"})
CLEARED = {"cleared_distance", "cleared_flag"}


def load(root=ROOT):
    """The deduplicated catalog `prospector rank` writes, the ranking and the calibration."""
    root = Path(root)
    view = root / "catalog_view.jsonl"
    if not view.exists():
        raise SystemExit(f"no {view}; run: cargo run --release --example prospector -- rank")
    entries = [json.loads(line) for line in view.read_text().splitlines() if line.strip()]
    ranking = json.loads((root / "ranking.json").read_text()) if (root / "ranking.json").exists() else []
    calibration = json.loads((root / "calibration.json").read_text()) if (root / "calibration.json").exists() else {}
    return entries, ranking, calibration


def sections(entries, ranking):
    """(title, entries) in SECTION_TITLES order. Duplicates (marked by the
    Rust side's dedupe) are left out; cleared levels missing from the
    ranking go after the ranked ones."""
    by_offset = {e["offset"]: e for e in entries}
    ranked = {r["offset"] for r in ranking}
    leaders = [by_offset[r["offset"]] for r in ranking if r["offset"] in by_offset]
    leaders += [e for e in entries if e["class"] in CLEARED and e["offset"] not in ranked]
    out = [(SECTION_TITLES[0], leaders)]
    for title, classes in zip(SECTION_TITLES[1:], SECTION_CLASSES[1:]):
        out.append((title, [e for e in entries if e["class"] in classes]))
    return out


def card(e, row):
    fp = e.get("fingerprint")
    img = f'<img src="thumbs/{fp}.png" alt="" loading="lazy">' if fp else '<div class="none"></div>'
    lines = [f'<b>{html.escape(e["hex"])}</b> {html.escape(e["class"])}']
    if row:
        lines.append(f'#{row["rank"]} bottleneck {row["bottleneck"]}, killed {row["kill_rate"]:.0%}, {row["rounds"]} rounds')
        if row.get("narrower_than"):
            lines.append("narrower than " + " and ".join(row["narrower_than"]))
        lines.append(f'<a href="clips/{row["rank"]:02d}-{e["hex"].lstrip("$")}.mp4">clip</a>')
    if e.get("walk_x") is not None:
        lines.append(f'walker reached x {e["walk_x"]}')
    if e.get("message"):
        lines.append(f'<code>{html.escape(e["message"])}</code>')
    return f'<figure>{img}<figcaption>{"<br>".join(lines)}</figcaption></figure>'


def render(entries, ranking, calibration):
    rows = {r["offset"]: r for r in ranking}
    counts = {}
    for e in entries:
        counts[e["class"]] = counts.get(e["class"], 0) + 1
    parts = ["<!doctype html><html><head><meta charset=\"utf-8\"><title>The Prospector</title>",
             "<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\"><style>",
             "body{background:#0c0e14;color:#ebebeb;font:15px system-ui,sans-serif;margin:16px}",
             "figure{display:inline-block;width:256px;margin:8px;vertical-align:top}",
             "img,.none{width:256px;height:240px;image-rendering:pixelated;background:#1a1d28}",
             "code{color:#eb5a50;word-break:break-all}a{color:#f5c850}h2{margin-top:32px}</style></head><body>",
             "<h1>The Prospector</h1><p>Every offset of Super Mario Bros.' ROM, loaded as World 1-1.</p>",
             "<p>" + ", ".join(f"{html.escape(k)}: {v}" for k, v in sorted(counts.items())) + "</p>"]
    if calibration:
        parts.append("<p>Real levels, solved the same way: " + "; ".join(
            f"{html.escape(name)} bottleneck {c['bottleneck']}, killed {c['kill_rate']:.0%}, {c['rounds']} rounds"
            for name, c in calibration.items()) + "</p>")
    for title, es in sections(entries, ranking):
        parts.append(f"<h2>{html.escape(title)} ({len(es)})</h2>")
        parts.extend(card(e, rows.get(e["offset"])) for e in es)
    parts.append("</body></html>")
    return "\n".join(parts)


def main():
    entries, ranking, calibration = load()
    for ppm in (ROOT / "thumbs").glob("*.ppm"):
        png = ppm.with_suffix(".png")
        if not png.exists():
            Image.open(ppm).save(png)
    out = ROOT / "catalog.html"
    out.write_text(render(entries, ranking, calibration))
    print(out)


if __name__ == "__main__":
    main()
