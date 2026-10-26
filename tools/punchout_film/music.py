"""The Punch-Out!! film's score. Like the game, each opponent gets a
pumped-up public-domain tune from his supposed homeland; the jokes are in
the arranging. Nothing here comes from Nintendo's score or any modern
arrangement: every tune is written out from the public-domain melody."""

from reservoir_film.music.core import Cue, Instrument, Part, Section
from reservoir_film.music.cues import BASS, CUES as TETRIS_CUES, DRUM, HARM, OCT, PLINK, SOFT, STAC, SWEEP, bars, each
from reservoir_film.music.cues import STINGS as TETRIS_STINGS

# A punchy pulse with a fast attack, for melodies that should blare.
BRASS = Instrument(volume=(13, 12, 11, 10, 10, 9), duty=(2, 2, 1), vibrato=(10, 0.1, 6), release=(4, 2, 0))
# A kazoo: the same tune, wobbling a third of a semitone either side.
KAZOO = Instrument(volume=(8, 8, 7, 7), duty=(0,), vibrato=(0, 0.35, 5), release=(3, 1, 0))

# Each chord's march bass (root, fifth) and one tone to stab.
ROOT = {"C": ("C3", "G2"), "G": ("G2", "D3"), "F": ("F2", "C3"), "D": ("D3", "A2"), "Am": ("A2", "E3"),
        "Bbm": ("Bb2", "F2"), "Gb": ("Gb2", "Db3"), "A": ("A2", "E3"), "E": ("E2", "B2"), "Bm": ("B2", "F#2")}
TONE = {"C": "E4", "G": "B4", "F": "A4", "D": "F#4", "Am": "C5", "Bbm": "Db4", "Gb": "Bb3", "A": "C#5", "E": "G#4",
        "Bm": "D5"}
# A lounge chord: the third and the sixth above the root.
LOUNGE = {"C": ("E4", "A4"), "G": ("B4", "E5"), "D": ("F#4", "B4")}


def march(c):
    r, f = ROOT[c]
    return f"{r}/4 {f}/4 {r}/4 {f}/4"


def oompah(c):
    r, f = ROOT[c]
    return f"{r}/8 r/8 {f}/8 r/8 {r}/8 r/8 {f}/8 r/8"


def stab(c):
    return " ".join([f"r/8 {TONE[c]}/8"] * 4)


MARCH_DRUMS = "k/4 s/8 s/8 k/4 s/4"
GALOP_DRUMS = "k/8 h/8 s/8 h/8 k/8 h/8 s/8 h/8"
ROLL = " ".join(["s/16"] * 16)

# --- Ring entrance: an original fanfare ------------------------------------
RING_CHORDS = ["C", "C", "F", "G"]
ring_entrance = Cue(
    name="ring_entrance", title="Ring Entrance", bpm=132,
    intro=Section([Part("p1", BRASS, "r/1"), Part("p2", STAC, "r/1"), Part("tri", BASS, "r/1"),
                   Part("noise", DRUM, ROLL)]),
    loop=Section([
        Part("p1", BRASS, bars("C5/8 C5/8 C5/8 G4/8 C5/4 E5/4", "G5/4. E5/8 G5/2",
                               "A5/8 G5/8 F5/8 E5/8 D5/4 G5/4", "C6/2. r/4")),
        Part("p2", STAC, each(RING_CHORDS, stab)),
        Part("tri", BASS, each(RING_CHORDS, march)),
        Part("noise", DRUM, bars(*[MARCH_DRUMS] * 4)),
    ]),
    ending=Section([Part("p1", BRASS, "G5/8 G5/8 C6/4 r/2"), Part("p2", STAC, "r/4 E5/4 r/2"),
                    Part("tri", BASS, "G2/8 G2/8 C3/4 r/2"), Part("noise", DRUM, "s/8 s/8 o/4 r/2")]),
)

# --- Can-can: Offenbach's galop, with a kazoo an octave down ---------------
CANCAN_A = ["C5/4 C5/4 D5/8 F5/8 E5/8 D5/8", "G5/4 G5/4 G5/8 A5/8 E5/8 F5/8", "D5/4 D5/4 D5/8 F5/8 E5/8 D5/8"]
CANCAN = bars(*CANCAN_A, "C5/8 C6/8 B5/8 A5/8 G5/8 F5/8 E5/8 D5/8",
              *CANCAN_A, "C5/8 G5/8 D5/8 E5/8 C5/4 r/4")
CANCAN_CHORDS = ["C", "C", "G", "C", "C", "C", "G", "C"]


def down_an_octave(notes):
    out = []
    for token in notes.split():
        if token[0] in "ABCDEFG":
            pitch, rest = token.split("/", 1)
            out.append(f"{pitch[:-1]}{int(pitch[-1]) - 1}/{rest}")
        else:
            out.append(token)
    return " ".join(out)


cancan = Cue(
    name="cancan", title="Can-can", bpm=160, credit="Jacques Offenbach, Galop infernal (Can-can, 1858)",
    loop=Section([
        Part("p1", BRASS, CANCAN), Part("p2", KAZOO, down_an_octave(CANCAN)),
        Part("tri", BASS, each(CANCAN_CHORDS, oompah)), Part("noise", DRUM, bars(*[GALOP_DRUMS] * 8)),
    ]),
    ending=Section([Part("p1", BRASS, "C6/4 r/4 C5/4 r/4"), Part("p2", KAZOO, "C5/4 r/4 C4/4 r/4"),
                    Part("tri", BASS, "C3/4 r/4 C2/4 r/4"), Part("noise", DRUM, "k/4 r/4 o/4 r/4")]),
)

# --- Funeral march: Chopin, deadpan ----------------------------------------
FUNERAL = bars("Bb4/4 Bb4/8. Bb4/16 Bb4/2", "Db5/8. C5/16 C5/8. Bb4/16 Bb4/8. A4/16 Bb4/4")
funeral = Cue(
    name="funeral", title="Funeral March", bpm=60,
    credit="Frédéric Chopin, Funeral March from Piano Sonata No. 2 (1839)",
    loop=Section([
        Part("p1", SOFT, bars(FUNERAL, FUNERAL)),
        Part("p2", HARM, bars(*["F4/4 F4/4 F4/4 F4/4", "Gb4/4 F4/4 F4/4 F4/4"] * 2)),
        Part("tri", BASS, bars(*["Bb2/4 Gb2/4 Bb2/4 Gb2/4"] * 4)),
        Part("noise", DRUM, bars(*["k/4 r/4 k/4 r/4"] * 4)),
    ]),
    ending=Section([Part("p1", SOFT, "Bb3/2"), Part("p2", HARM, "r/2"), Part("tri", BASS, "Bb1/2"),
                    Part("noise", DRUM, "k/2")]),
)

# --- La Marseillaise, four ways --------------------------------------------
MARS = bars("G4/4 G4/4 A4/4 A4/4", "D5/4. B4/8 G4/8. G4/16 B4/8. G4/16", "E4/4 C5/2 A4/8. F#4/16",
            "G4/2 r/4 G4/8. A4/16", "B4/4 B4/4 B4/4 C5/8. B4/16", "B4/4 A4/4 r/4 A4/8. B4/16",
            "C5/4 C5/4 C5/4 D5/8. C5/16", "B4/2 r/4 D4/8. D4/16")
MARS_CHORDS = ["G", "G", "C", "G", "G", "D", "C", "G"]
MARS_CREDIT = "Claude Joseph Rouget de Lisle, La Marseillaise (1792)"
# The pickup into the first bar, over a snare roll.
MARS_INTRO = [Part("p1", BRASS, "r/2 r/4 D4/8. D4/16"), Part("p2", STAC, "r/1"), Part("tri", BASS, "r/1"),
              Part("noise", DRUM, ROLL)]
MARS_ENDING = [Part("p1", BRASS, "G4/2 r/2"), Part("p2", STAC, "B4/2 r/2"), Part("tri", BASS, "G2/2 r/2"),
               Part("noise", DRUM, "o/2 r/2")]


def octave_arp(c):
    r, f = ROOT[c]
    return f"{r[:-1]}4/8 {TONE[c]}/8 {f[:-1]}4/8 {TONE[c]}/8 " * 2


marseillaise = Cue(
    name="marseillaise", title="La Marseillaise", bpm=112, credit=MARS_CREDIT,
    intro=Section(MARS_INTRO),
    loop=Section([
        Part("p1", BRASS, MARS), Part("p2", STAC, each(MARS_CHORDS, stab)),
        Part("tri", BASS, each(MARS_CHORDS, march)), Part("noise", DRUM, bars(*[MARCH_DRUMS] * 8)),
    ]),
    ending=Section(MARS_ENDING),
)

marseillaise_fast = Cue(
    name="marseillaise_fast", title="La Marseillaise (quick march)", bpm=168, credit=MARS_CREDIT,
    intro=Section(MARS_INTRO),
    loop=Section([
        Part("p1", BRASS, MARS), Part("p2", OCT, each(MARS_CHORDS, lambda c: octave_arp(c).strip())),
        Part("tri", BASS, each(MARS_CHORDS, oompah)), Part("noise", DRUM, bars(*[GALOP_DRUMS] * 8)),
    ]),
    ending=Section(MARS_ENDING),
)

# Lift music: slow, soft, polite, brushed.
marseillaise_elevator = Cue(
    name="marseillaise_elevator", title="La Marseillaise (lift music)", bpm=76, credit=MARS_CREDIT,
    loop=Section([
        Part("p1", SOFT, MARS),
        Part("p2", HARM, each(MARS_CHORDS, lambda c: "{0}/2 {1}/2".format(*LOUNGE.get(c, (TONE[c], TONE[c]))))),
        Part("tri", BASS, each(MARS_CHORDS, lambda c: "{0}/2 {1}/2".format(*ROOT[c]))),
        Part("noise", DRUM, bars(*["r/4 h/4 r/4 h/4"] * 8)),
    ]),
    ending=Section([Part("p1", SOFT, "G4/1"), Part("p2", HARM, "E5/1"), Part("tri", BASS, "G2/1"),
                    Part("noise", DRUM, "r/1")]),
)

# The anthem that never arrives: two bars, a snare roll and a climb, then
# a deflating slide where the cadence should be. Then it tries again.
marseillaise_stall = Cue(
    name="marseillaise_stall", title="La Marseillaise (never quite)", bpm=120, credit=MARS_CREDIT,
    intro=Section(MARS_INTRO),
    loop=Section([
        Part("p1", BRASS, bars("G4/4 G4/4 A4/4 A4/4", "D5/4. B4/8 G4/8. G4/16 B4/8. G4/16",
                               "G4/8 B4/8 D5/8 G5/8 B5/8 D6/8 G6/4", "r/2 r/4 D4/8. D4/16")),
        Part("p2", SWEEP, bars("r/1", "r/1", "r/1", "G5/2 r/2")),
        Part("tri", BASS, bars(march("G"), march("G"), "G2/8 A2/8 B2/8 C3/8 D3/8 E3/8 F#3/8 G3/8", "r/1")),
        Part("noise", DRUM, bars(MARCH_DRUMS, MARCH_DRUMS, ROLL, "r/2 k/4 r/4")),
    ]),
    ending=Section([Part("p1", BRASS, "r/1"), Part("p2", SWEEP, "D5/2 r/2"), Part("tri", BASS, "r/1"),
                    Part("noise", DRUM, "r/2 k/4 r/4")]),
)

# --- Ode to Joy: Beethoven, as a march, then as a triumph -----------------
ODE = bars("F#5/4 F#5/4 G5/4 A5/4", "A5/4 G5/4 F#5/4 E5/4", "D5/4 D5/4 E5/4 F#5/4", "F#5/4. E5/8 E5/2",
           "F#5/4 F#5/4 G5/4 A5/4", "A5/4 G5/4 F#5/4 E5/4", "D5/4 D5/4 E5/4 F#5/4", "E5/4. D5/8 D5/2")
ODE_CHORDS = ["D", "A", "D", "A", "D", "A", "D", "D"]
ODE_CREDIT = "Ludwig van Beethoven, Ode to Joy from Symphony No. 9 (1824)"


def run_up(c):
    """A triangle run from the chord's root up an octave, in eighths."""
    scale = ["D", "E", "F#", "G", "A", "B", "C#"]
    root = ROOT[c][0][:-1]
    k, octave, notes = scale.index(root), 2, []
    for i in range(8):
        letter = scale[(k + i) % 7]
        # Octaves turn over at C.
        if i > 0 and letter.startswith("C"):
            octave += 1
        notes.append(f"{letter}{octave}/8")
    return " ".join(notes)


ode = Cue(
    name="ode", title="Ode to Joy (march)", bpm=132, credit=ODE_CREDIT,
    intro=Section([Part("p1", BRASS, "r/1"), Part("p2", STAC, "r/1"), Part("tri", BASS, march("D")),
                   Part("noise", DRUM, ROLL)]),
    loop=Section([
        Part("p1", BRASS, ODE), Part("p2", STAC, each(ODE_CHORDS, stab)),
        Part("tri", BASS, each(ODE_CHORDS, oompah)), Part("noise", DRUM, bars(*[MARCH_DRUMS] * 8)),
    ]),
    ending=Section([Part("p1", BRASS, "D6/2 r/2"), Part("p2", STAC, "F#5/2 r/2"), Part("tri", BASS, "D3/2 r/2"),
                    Part("noise", DRUM, "o/2 r/2")]),
)

# Over the top: the tune on blaring brass, an arpeggio
# shimmering under it, a triangle run every bar and a pounding drum.
triumph = Cue(
    name="triumph", title="Triumph", bpm=140, credit=ODE_CREDIT,
    intro=Section([Part("p1", BRASS, "A4/8 A4/8 A4/8 D5/8 F#5/8 A5/8 D6/4"), Part("p2", OCT, "r/1"),
                   Part("tri", BASS, run_up("D")), Part("noise", DRUM, ROLL)]),
    loop=Section([
        Part("p1", BRASS, ODE),
        Part("p2", OCT, each(ODE_CHORDS, lambda c: octave_arp(c).strip())),
        Part("tri", BASS, each(ODE_CHORDS, run_up)),
        Part("noise", DRUM, bars(*["k/8 k/8 s/4 k/8 k/8 s/4"] * 8)),
    ]),
    ending=Section([Part("p1", BRASS, "D6/1"), Part("p2", OCT, "D5/1"), Part("tri", BASS, "D2/1"),
                    Part("noise", DRUM, ROLL)]),
)

# --- Ach du lieber Augustin: an oompah band in too much of a hurry ---------
AUGUSTIN = bars("C5/4. D5/8 C5/8 Bb4/8", "A4/4 F4/4 F4/4", "G4/4 C4/4 C4/4", "A4/4 F4/4 F4/4",
                "C5/4. D5/8 C5/8 Bb4/8", "A4/4 F4/4 F4/4", "G4/4 C4/4 C4/4", "F4/2.")
AUGUSTIN_CHORDS = ["F", "F", "C", "F", "F", "F", "C", "F"]
augustin = Cue(
    name="augustin", title="Ach du lieber Augustin", bpm=200, credit="Ach du lieber Augustin (traditional)",
    loop=Section([
        Part("p1", BRASS, AUGUSTIN), Part("p2", STAC, each(AUGUSTIN_CHORDS, lambda c: f"r/4 {TONE[c]}/4 {TONE[c]}/4")),
        Part("tri", BASS, each(AUGUSTIN_CHORDS, lambda c: f"{ROOT[c][0]}/4 r/4 {ROOT[c][1]}/4")),
        Part("noise", DRUM, bars(*["k/4 h/4 h/4"] * 8)),
    ]),
    ending=Section([Part("p1", BRASS, "F5/4 F4/4 r/4"), Part("p2", STAC, "A4/4 r/2"),
                    Part("tri", BASS, "F2/4 F2/4 r/4"), Part("noise", DRUM, "k/4 o/4 r/4")]),
)

# --- Rondo alla Turca: Mozart's own joke about a Janissary band ------------
TURCA = bars("B4/16 A4/16 G#4/16 A4/16 C5/4 D5/16 C5/16 B4/16 C5/16 E5/4",
             "F5/16 E5/16 D#5/16 E5/16 B5/16 A5/16 G#5/16 A5/16 B5/16 A5/16 G#5/16 A5/16 C6/4",
             "A5/8 C6/8 B5/8 A5/8 G#5/8 A5/8 B5/8 A5/8", "G#5/8 A5/8 B5/8 A5/8 G#5/8 F#5/8 E5/4")
TURCA_CHORDS = ["Am", "Am", "Am", "E"]
turca = Cue(
    name="turca", title="Rondo alla Turca", bpm=132, credit="Wolfgang Amadeus Mozart, Rondo alla Turca (1783)",
    loop=Section([
        Part("p1", BRASS, TURCA), Part("p2", STAC, each(TURCA_CHORDS, stab)),
        Part("tri", BASS, each(TURCA_CHORDS, oompah)),
        Part("noise", DRUM, bars(*["k/8 k/8 s/4 k/8 k/8 s/4"] * 4)),
    ]),
    ending=Section([Part("p1", BRASS, "A5/4 A4/4 r/2"), Part("p2", STAC, "C5/4 r/4 r/2"),
                    Part("tri", BASS, "A2/4 A2/4 r/2"), Part("noise", DRUM, "k/4 o/4 r/2")]),
)

# --- Ride of the Valkyries: the cheat rides in -----------------------------
VALKYRIES = bars("F#4/8. B4/16 D5/8. B4/16 D5/2", "B4/8. D5/16 F#5/8. D5/16 F#5/2",
                 "D5/8. F#5/16 A5/8. F#5/16 A5/2", "A4/8. D5/16 F#5/8. D5/16 F#5/2")
VALKYRIES_CHORDS = ["Bm", "Bm", "D", "D"]
valkyries = Cue(
    name="valkyries", title="Ride of the Valkyries", bpm=112,
    credit="Richard Wagner, Ride of the Valkyries from Die Walküre (1856)",
    loop=Section([
        Part("p1", BRASS, VALKYRIES),
        # The strings' trill, as a pulse shaking between two notes.
        Part("p2", STAC, each(VALKYRIES_CHORDS, lambda c: " ".join([f"{TONE[c]}/16 {TONE[c][:-1]}5/16"] * 8)
                              if TONE[c].endswith("4") else " ".join([f"{TONE[c]}/16 C#5/16"] * 8))),
        Part("tri", BASS, each(VALKYRIES_CHORDS, lambda c: " ".join([f"{ROOT[c][0]}/8 {ROOT[c][0]}/16 {ROOT[c][0]}/16"] * 4))),
        Part("noise", DRUM, bars(*[" ".join(["k/8 h/16 h/16"] * 4)] * 4)),
    ]),
    ending=Section([Part("p1", BRASS, "B4/2 r/2"), Part("p2", STAC, "r/1"), Part("tri", BASS, "B1/2 r/2"),
                    Part("noise", DRUM, "o/2 r/2")]),
)

PUNCHOUT_CUES = [ring_entrance, cancan, funeral, marseillaise, marseillaise_fast, marseillaise_elevator,
                 marseillaise_stall, ode, triumph, augustin, turca, valkyries]
CUES = {**TETRIS_CUES, **{c.name: c for c in PUNCHOUT_CUES}}

# The bell at a knockout: an original three-note ring.
BELL = Cue(name="bell", title="Bell", bpm=150, loop=Section([Part("p1", PLINK, "E6/16 E6/16 E6/8")]))
# The crowd at a knockdown: one long swell of noise.
ROAR = Cue(name="roar", title="Roar", bpm=160, loop=Section([Part("noise", Instrument(), "c/1")]))

STINGS = {**TETRIS_STINGS, "bell": BELL, "roar": ROAR}

CHAPTER_CUES = {1: "coin_run", 2: "underground", 3: "lab", 4: "korobeiniki", 5: "underground", 6: "lab", 7: "korobeiniki_fast", 8: "fooled", 9: "minuet",
                10: "lab", 11: "reprise"}
