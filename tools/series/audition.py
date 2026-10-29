"""Read one passage in several narrator voices, to pick one by ear.

Usage: cd tools && series/.venv/bin/python -m series.audition [VOICE ...]
"""

import sys

from series.script import REPO
from series.voice import KokoroEngine

PASSAGE = (
    "A series of short videos that teaches mechanistic interpretability from "
    "the ground up, using Greenstone and NES games as the subject of every "
    "experiment. Someone who watches the whole series in order should come out "
    "able to take on an interpretability problem in a domain the series never "
    "touches, such as image models."
)
VOICES = ["af_heart", "am_michael", "bm_george"]
OUT = REPO / "target" / "series" / "audition"


def main(names: list[str]) -> None:
    for voice in names or VOICES:
        out = OUT / f"{voice}.wav"
        KokoroEngine(voice).synthesize(PASSAGE, out)
        print(out)


if __name__ == "__main__":
    main(sys.argv[1:])
