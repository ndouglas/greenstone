"""The film's music: five originals, Korobeiniki (traditional) and the
Minuet in G (Petzold, long attributed to Bach). Nothing here is taken from
Koji Kondo's or Hirokazu Tanaka's scores; the Mario-flavoured cues borrow
a style (bouncy, syncopated, major), not a tune."""

from .core import Cue, Instrument, Part, Section

LEAD = Instrument(volume=(11, 10, 9, 9, 8), duty=(2,), vibrato=(12, 0.15, 6), release=(5, 3, 1, 0))
SOFT = Instrument(volume=(8, 7, 7, 6), duty=(1,), vibrato=(16, 0.12, 7), release=(4, 2, 1, 0))
HARM = Instrument(volume=(7, 6, 5), duty=(1,), release=(3, 1, 0))
PLINK = Instrument(volume=(10, 6, 4, 2, 1, 0), duty=(1,))
STAC = Instrument(volume=(9, 5, 2, 0), duty=(2,))
OCT = Instrument(volume=(6, 5, 5, 4), duty=(1,), arp=(0, 0, 12, 12))
# No vibrato: writing the period each frame would undo the sweep unit's slide.
SWEEP = Instrument(volume=(12, 12, 11, 11, 10, 10, 9), duty=(2,), release=(6, 4, 2, 1, 0), sweep=0xA5)
BASS = Instrument()
DRUM = Instrument()


def bars(*b):
    return " | ".join(b)


def each(chords, pattern):
    """One bar of `pattern` per chord; `pattern` is a function of the chord."""
    return bars(*(pattern(c) for c in chords))


def rest(n_beats_per_bar, n_bars):
    return bars(*([{4: "r/1", 3: "r/2."}[n_beats_per_bar]] * n_bars))


# --- Coin Run: bouncy, syncopated C major, calypso bass -------------------
CR_CHORDS = ["C", "Am", "F", "G", "C", "Am", "G", "C"]
CR_ROOT = {"C": ("C", "G"), "Am": ("A", "E"), "F": ("F", "C"), "G": ("G", "D")}
CR_THIRD = {"C": "E4", "Am": "C5", "F": "A4", "G": "B4"}
CR_MELODY = bars(
    "G4/8 C5/8 r/8 E5/8 r/8 G5/8 r/8 E5/8",
    "A5/8 G5/8 r/8 E5/8 C5/4 r/4",
    "A4/8 C5/8 r/8 F5/8 r/8 A5/8 r/8 F5/8",
    "G5/8 F5/8 r/8 D5/8 B4/4 r/4",
    "E5/8 E5/8 r/8 E5/8 r/8 C5/8 E5/4",
    "G5/4 r/4 G4/4 r/4",
    "F5/8 E5/8 r/8 D5/8 r/8 B4/8 G4/4",
    "C5/4 G4/8 C5/8 r/2",
)


def calypso(c):
    r, f = CR_ROOT[c]
    return f"{r}3/8. {f}3/16 r/8 {r}3/8 r/8 {f}2/8 {r}3/8 r/8"


def skank(c):
    t = CR_THIRD[c]
    return f"r/8 {t}/8 r/8 {t}/8 r/8 {t}/8 r/8 {t}/8"


CR_DRUMS = "k/8 h/8 s/8 h/16 h/16 k/8 h/8 s/8 h/8"
coin_run = Cue(
    name="coin_run", title="Coin Run", bpm=132,
    intro=Section([
        Part("p1", LEAD, rest(4, 2)), Part("p2", STAC, rest(4, 2)),
        Part("tri", BASS, each(["C", "C"], calypso)), Part("noise", DRUM, bars(CR_DRUMS, CR_DRUMS)),
    ]),
    loop=Section([
        Part("p1", LEAD, CR_MELODY), Part("p2", STAC, each(CR_CHORDS, skank)),
        Part("tri", BASS, each(CR_CHORDS, calypso)), Part("noise", DRUM, bars(*[CR_DRUMS] * 8)),
    ]),
    ending=Section([
        Part("p1", LEAD, "G4/8 C5/8 E5/8 G5/8 C6/2"), Part("p2", STAC, "r/2 E5/2"),
        Part("tri", BASS, "C3/8 C3/8 C3/8 C3/8 C2/2"), Part("noise", DRUM, "k/8 k/8 k/8 k/8 o/2"),
    ]),
)

reprise = Cue(
    name="reprise", title="Coin Run (reprise)", bpm=96,
    loop=Section([
        Part("p1", SOFT, CR_MELODY),
        Part("tri", BASS, each(CR_CHORDS, lambda c: f"{CR_ROOT[c][0]}3/2 {CR_ROOT[c][1]}2/2")),
    ]),
    ending=Section([Part("p1", SOFT, "C5/1"), Part("tri", BASS, "C3/1")]),
)

# --- Underground Numbers: sparse A minor ----------------------------------
UG_CHORDS = ["Am", "Am", "F", "E", "Am", "Am", "Dm", "E"]
UG_ROOT = {"Am": ("A2", "E2"), "F": ("F2", "C2"), "E": ("E2", "B1"), "Dm": ("D2", "A1")}
UG_TONES = {"Am": "A4 C5 E5 A5", "F": "F4 A4 C5 F5", "E": "E4 G#4 B4 E5", "Dm": "D4 F4 A4 D5"}


def ug_bass(c):
    r, q = UG_ROOT[c]
    return f"{r}/8 r/8 {r}/8 r/8 r/4 {q}/4"


def ug_plink(c):
    a, b, d, e = UG_TONES[c].split()
    return f"{a}/16 {b}/16 {d}/16 {e}/16 r/4 r/2"


UG_TICK = "h/4 r/4 h/8 h/8 r/4"
underground = Cue(
    name="underground", title="Underground Numbers", bpm=100,
    intro=Section([
        Part("p1", LEAD, "r/1"), Part("p2", OCT, "r/1"),
        Part("tri", BASS, ug_bass("Am")), Part("noise", DRUM, UG_TICK),
    ]),
    loop=Section([
        Part("p1", LEAD, bars("r/2 E5/4 C5/4", "B4/2 r/2", "r/2 A4/4 C5/4", "B4/1",
                              "r/2 E5/4 G5/4", "F5/2 E5/4 r/4", "D5/4 F5/4 A5/4 G5/4", "G#5/2 r/2")),
        Part("p2", OCT, each(UG_CHORDS, ug_plink)),
        Part("tri", BASS, each(UG_CHORDS, ug_bass)),
        Part("noise", DRUM, bars(*[UG_TICK] * 8)),
    ]),
    ending=Section([
        Part("p1", LEAD, "A4/1"), Part("p2", OCT, "r/1"), Part("tri", BASS, "A2/1"), Part("noise", DRUM, "r/1"),
    ]),
)

# --- The Lab: plinky F major ----------------------------------------------
LAB_CHORDS = ["F", "C", "Dm", "Bb", "F", "C", "Bb", "F"]
LAB_ROOT = {"F": ("F2", "C3"), "C": ("C3", "G2"), "Dm": ("D3", "A2"), "Bb": ("Bb2", "F2")}
LAB_THIRD = {"F": "A4", "C": "E4", "Dm": "F4", "Bb": "D4"}
LAB_TICK = "h/4 h/4 h/4 h/4"
lab = Cue(
    name="lab", title="The Lab", bpm=112,
    intro=Section([
        Part("p1", PLINK, "C6/8 r/8 C6/8 r/8 r/2"), Part("p2", HARM, "r/1"),
        Part("tri", BASS, "F2/4 C3/4 F2/4 C3/4"), Part("noise", DRUM, LAB_TICK),
    ]),
    loop=Section([
        Part("p1", PLINK, bars(
            "F5/8 A5/8 C6/8 A5/8 F5/4 r/4", "E5/8 G5/8 C6/8 G5/8 E5/4 r/4",
            "D5/8 F5/8 A5/8 F5/8 D5/4 r/4", "D5/8 F5/8 Bb5/8 F5/8 D5/4 r/4",
            "C5/8 F5/8 A5/8 C6/8 A5/4 F5/4", "G5/8 E5/8 C5/8 E5/8 G5/2",
            "Bb5/8 A5/8 G5/8 F5/8 E5/8 F5/8 G5/4", "F5/2 r/2")),
        Part("p2", HARM, each(LAB_CHORDS, lambda c: f"r/4 {LAB_THIRD[c]}/4 r/4 {LAB_THIRD[c]}/4")),
        Part("tri", BASS, each(LAB_CHORDS, lambda c: "{0}/4 {1}/4 {0}/4 {1}/4".format(*LAB_ROOT[c]))),
        Part("noise", DRUM, bars(*[LAB_TICK] * 8)),
    ]),
    ending=Section([
        Part("p1", PLINK, "F5/4 C5/4 F4/2"), Part("p2", HARM, "r/1"),
        Part("tri", BASS, "F2/1"), Part("noise", DRUM, "r/1"),
    ]),
)

# --- Korobeiniki (traditional): A minor -----------------------------------
KORO_MELODY = bars(
    "E5/4 B4/8 C5/8 D5/4 C5/8 B4/8", "A4/4 A4/8 C5/8 E5/4 D5/8 C5/8",
    "B4/4. C5/8 D5/4 E5/4", "C5/4 A4/4 A4/2",
    "r/8 D5/4 F5/8 A5/4 G5/8 F5/8", "E5/4. C5/8 E5/4 D5/8 C5/8",
    "B4/4 B4/8 C5/8 D5/4 E5/4", "C5/4 A4/4 A4/2",
)
KORO_HARM = bars("C5/2 B4/2", "A4/2 C5/2", "G#4/2 B4/2", "A4/1", "F4/2 A4/2", "G4/2 E4/2", "G#4/2 B4/2", "A4/1")
KORO_CHORDS = ["Am", "Am", "E", "Am", "Dm", "Am", "E", "Am"]
KORO_ROOT = {"Am": "A", "E": "E", "Dm": "D"}


def octaves(c):
    r = KORO_ROOT[c]
    return " ".join([f"{r}2/8 {r}3/8"] * 4)


KORO_DRUMS = "k/8 h/8 s/8 h/8 k/8 h/8 s/8 h/8"


def korobeiniki_cue(name, title, bpm, drums):
    noise = [Part("noise", DRUM, bars(*[KORO_DRUMS] * 8))] if drums else []
    return Cue(
        name=name, title=title, bpm=bpm, credit="traditional",
        intro=Section([Part("p1", LEAD, "r/1"), Part("p2", HARM, "r/1"), Part("tri", BASS, octaves("Am"))]
                      + ([Part("noise", DRUM, KORO_DRUMS)] if drums else [])),
        loop=Section([Part("p1", LEAD, KORO_MELODY), Part("p2", HARM, KORO_HARM),
                      Part("tri", BASS, each(KORO_CHORDS, octaves))] + noise),
        ending=Section([Part("p1", LEAD, "A4/2. r/4"), Part("p2", HARM, "E4/2. r/4"), Part("tri", BASS, "A2/2. r/4")]
                       + ([Part("noise", DRUM, "k/4 k/4 o/2")] if drums else [])),
    )


korobeiniki = korobeiniki_cue("korobeiniki", "Korobeiniki", 140, drums=False)
korobeiniki_fast = korobeiniki_cue("korobeiniki_fast", "Korobeiniki (fast)", 176, drums=True)

# --- Fooled Again: sneaky staccato D minor --------------------------------
FA_CHORDS = ["Dm", "Dm", "Bb", "A", "Dm", "Gm", "A", "Dm"]
FA_ROOT = {"Dm": ("D", "A"), "Bb": ("Bb", "F"), "A": ("A", "E"), "Gm": ("G", "D")}
FA_THIRD = {"Dm": "F4", "Bb": "D4", "A": "C#4", "Gm": "Bb3"}
FA_TICK = "h/8 r/8 h/8 r/8 h/8 r/8 h/8 h/8"


def tiptoe(c):
    r, q = FA_ROOT[c]
    return f"{r}2/8 r/8 {r}3/8 r/8 {r}2/8 r/8 {q}2/8 r/8"


fooled = Cue(
    name="fooled", title="Fooled Again", bpm=120,
    intro=Section([Part("p1", STAC, "r/1"), Part("p2", HARM, "r/1"), Part("tri", BASS, tiptoe("Dm")),
                   Part("noise", DRUM, FA_TICK)]),
    loop=Section([
        Part("p1", STAC, bars(
            "D5/8 r/8 F5/8 r/8 A5/8 r/8 G5/8 F5/8", "E5/8 r/8 D5/8 r/8 r/2",
            "D5/8 r/8 F5/8 r/8 Bb5/8 r/8 A5/8 G5/8", "A5/8 r/8 C#5/8 r/8 r/2",
            "D5/8 F5/8 A5/8 D6/8 r/8 A5/8 F5/8 r/8", "G5/8 r/8 Bb5/8 r/8 D6/8 r/8 Bb5/8 r/8",
            "A5/8 G5/8 F5/8 E5/8 C#5/4 r/4", "D5/4 r/4 r/2")),
        Part("p2", HARM, each(FA_CHORDS, lambda c: f"r/4 {FA_THIRD[c]}/8 r/8 r/4 {FA_THIRD[c]}/8 r/8")),
        Part("tri", BASS, each(FA_CHORDS, tiptoe)),
        Part("noise", DRUM, bars(*[FA_TICK] * 8)),
    ]),
    ending=Section([Part("p1", STAC, "D5/8 r/8 A4/8 r/8 D4/4 r/4"), Part("p2", HARM, "r/1"),
                    Part("tri", BASS, "D2/8 r/8 A2/8 r/8 D2/4 r/4"), Part("noise", DRUM, "r/1")]),
)

# --- Minuet in G (Petzold), 3/4 -------------------------------------------
MINUET_A = [
    "D5/4 G4/8 A4/8 B4/8 C5/8", "D5/4 G4/4 G4/4", "E5/4 C5/8 D5/8 E5/8 F#5/8", "G5/4 G4/4 G4/4",
    "C5/4 D5/8 C5/8 B4/8 A4/8", "B4/4 C5/8 B4/8 A4/8 G4/8", "F#4/4 G4/8 A4/8 B4/8 G4/8", "A4/2.",
]
MINUET_B = MINUET_A[:6] + ["A4/4 B4/8 A4/8 G4/8 F#4/8", "G4/2."]
MINUET_BASS_A = ["G2/2 A2/4", "B2/2.", "C3/2.", "B2/2.", "A2/2.", "G2/2.", "D3/4 B2/4 G2/4", "D3/4 D2/4 C3/4"]
MINUET_BASS_B = MINUET_BASS_A[:6] + ["C3/4 D3/4 D2/4", "G2/2 G1/4"]
minuet = Cue(
    name="minuet", title="Minuet in G", bpm=120,
    credit="Christian Petzold, Minuet in G (BWV Anh. 114, long attributed to Bach)",
    loop=Section([Part("p1", SOFT, bars(*MINUET_A, *MINUET_B)),
                  Part("tri", BASS, bars(*MINUET_BASS_A, *MINUET_BASS_B))]),
    ending=Section([Part("p1", SOFT, "G4/2."), Part("tri", BASS, "G2/2.")]),
)

CUES = {c.name: c for c in [coin_run, underground, lab, korobeiniki, korobeiniki_fast, fooled, minuet, reprise]}

# --- Stings (all original) -------------------------------------------------
STINGS = {
    "blip": Cue(name="blip", title="Blip", bpm=150, loop=Section([Part("p1", PLINK, "C6/32 G6/32")])),
    "oneup": Cue(name="oneup", title="One up", bpm=150,
                 loop=Section([Part("p1", PLINK, "C6/16 E6/16 G6/16 C7/8")])),
    "wahwah": Cue(name="wahwah", title="Wah wah", bpm=90,
                  loop=Section([Part("p1", SWEEP, "Bb4/4 A4/4 Ab4/2"), Part("tri", BASS, "Bb2/4 A2/4 Ab2/2")])),
    "fanfare": Cue(name="fanfare", title="Fanfare", bpm=140, loop=Section([
        Part("p1", LEAD, "C5/8 E5/8 G5/8 C6/8 r/8 G5/8 C6/2"),
        Part("p2", HARM, "E4/8 G4/8 C5/8 E5/8 r/8 B4/8 E5/2"),
        Part("tri", BASS, "C3/2 r/8 G2/8 C3/2")])),
    "death": Cue(name="death", title="Death", bpm=120, loop=Section([
        Part("p1", STAC, "G4/8 F#4/8 F4/8 E4/2."), Part("tri", BASS, "G2/8 F#2/8 F2/8 E2/2.")])),
}

CHAPTER_CUES = {0: "coin_run", 1: "underground", 2: "lab", 3: "korobeiniki", 4: "lab",
                5: "fooled", 6: "korobeiniki_fast", 7: "minuet", 8: "minuet", 9: "reprise"}
