"""Render the Punch-Out!! film's cues and stings alone, with a spectrogram
each, for listening and checking:

    cd tools && python3 -m punchout_film.audition [name ...]
"""

import sys

from reservoir_film.music import audition

from .film import OUT_DIR
from .music import CUES, STINGS

if __name__ == "__main__":
    audition.main(sys.argv[1:], CUES, STINGS, OUT_DIR / "music")
