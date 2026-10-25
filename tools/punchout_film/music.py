"""The Punch-Out!! film's score: the Tetris film's cues, and a bell."""

from reservoir_film.music.core import Cue, Part, Section
from reservoir_film.music.cues import CUES, PLINK
from reservoir_film.music.cues import STINGS as TETRIS_STINGS

# The bell at a knockout: an original three-note ring.
BELL = Cue(name="bell", title="Bell", bpm=150, loop=Section([Part("p1", PLINK, "E6/16 E6/16 E6/8")]))

STINGS = {**TETRIS_STINGS, "bell": BELL}

CHAPTER_CUES = {}
