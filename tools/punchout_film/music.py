"""The Punch-Out!! film's score: the Tetris film's cues, and a bell."""

from reservoir_film.music.core import Cue, Part, Section
from reservoir_film.music.cues import CUES, PLINK
from reservoir_film.music.cues import STINGS as TETRIS_STINGS

# The bell at a knockout: an original three-note ring.
BELL = Cue(name="bell", title="Bell", bpm=150, loop=Section([Part("p1", PLINK, "E6/16 E6/16 E6/8")]))

STINGS = {**TETRIS_STINGS, "bell": BELL}

CHAPTER_CUES = {1: "coin_run", 2: "underground", 3: "lab", 4: "korobeiniki", 5: "underground", 6: "lab", 7: "korobeiniki_fast", 8: "fooled", 9: "minuet",
                10: "lab", 11: "reprise"}
