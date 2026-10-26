# A Punch-Out!! score for the Punch-Out!! film

## Intent

The Punch-Out!! film (`tools/punchout_film/`, built with
`cd tools && python3 -m punchout_film.build`) is scored with the Tetris
film's cues: Korobeiniki, the Minuet in G, and five quiet originals. It
sounds like the wrong game. Nate wants the fights to be **more
energetic**, and wants the music to have **a sense of humour**,
including "some silliness for the segments where nothing works".

The model is Punch-Out!! itself: each opponent gets a pumped-up
public-domain tune from his supposed homeland. We do the same with real
public-domain pieces, arranged for the NES's four channels and played on
Greenstone's APU like the current score. The jokes come from the
arrangement (tempo, timing, instrumentation, a sour note), not from
caricature. Ethnic stereotypes stay in the game's opponents, not in the
music we write.

## Goals

- Every fight scene sounds energetic and Punch-Out!!-ish.
- The failures are funny: Mac flattened while standing still, the bank
  doing nothing, knockdowns that never become a win.
- The teacher's flawless knockout is over-the-top triumphant.
- Each cue can be heard on its own before it goes into the film.

## Non-goals and anti-goals

- **No Nintendo music.** Nothing is transcribed or paraphrased from
  Punch-Out!!'s (or any other game's) score. The repo and the film are
  public. Original material borrows the *style* only (brassy square-wave
  stabs, driving triangle bass, noise-channel crowd hits).
- **No modern copyrighted works**, including the Rocky theme. "Triumphant
  brass" means an original fanfare or a heroic variation on a
  public-domain tune.
- **No arrangement copied from a modern edition or recording.** Work from
  the public-domain melody itself.
- No mock-ethnic jingles written to sound "foreign".
- The Tetris film's score must not change.

## The cue map

A cue now starts at a point in the film, not only at a chapter. A
**cue span** begins at a shot, optionally some seconds into it, and runs
until the next span begins. Spans are crossfaded as chapters are now.
The approved map:

| Ch | Shot(s) | Cue | Notes |
|---|---|---|---|
| 1 | boot, usual | **Ring entrance**: an original fanfare, short and brassy | |
| 2 | idle | **Can-can** (Offenbach, *Galop infernal*, 1858), with a sour kazoo doubling | Mac is flattened standing still. (Was Entry of the Gladiators; see the note below) |
| 2 | idle, from Mac's last knockdown | **Funeral march** (Chopin, Sonata No. 2, 1839), a few bars, deadpan | Starts at Mac's final knockdown, read from the take like chapter 3's `ko_bell`: the last step where `mac_health` reaches zero (step 1106, about 18.4 s into the shot at 4x). Not the take's `mac_down_at`, which is his first. Then carries the `floor` shot |
| 3 | swap | **Lab** (existing calm cue) | The diagram builds |
| 3 | defence | **La Marseillaise** (1792) | The teacher vs Glass Joe |
| 3 | knockout | **Triumph**: over-the-top brass, a heroic variation on Ode to Joy | Ode to Joy is Von Kaiser's tune from here on. The bell still rings at the KO |
| 4 | see_swap … memories | **Lab** | |
| 4 | bank, honest | **Korobeiniki** (existing) | The brain's memory *is* Tetris |
| 4 | decide_swap, decide | **Lab** | |
| 5 | sidestep | **La Marseillaise, fast** | echo0.5 vs standing still |
| 5 | bank | **Marseillaise, elevator music**: slow, soft, polite | The bank does exactly nothing |
| 6 | grid, chart | **Ode to Joy** (Beethoven, Symphony No. 9, 1824), pumped up as a march | The memory sweep vs Von Kaiser |
| 7 | dagger, counter | **Ach du lieber Augustin** (Austrian folk song, traditional), an oompah too fast | stack4 vs Von Kaiser |
| 8 | knockdowns | **Marseillaise that never resolves**: a fanfare that builds and stalls, over and over | Knockdowns, never a win |
| 9 | rounds | **Rondo alla Turca** (Mozart, 1783) | Bald Bull, held out |
| 10 | yardstick, versus | **Ride of the Valkyries** (Wagner, 1856) | The cheat is overpowered |
| 11 | all | **Reprise**: gentle, ends on the bell | Existing `reprise` may serve. The credits change too (below) |

**Tune swaps during building.** Entry of the Gladiators, the Radetzky
March and Alte Kameraden were replaced by the Can-can, Ode to Joy and Ach
du lieber Augustin. The builder couldn't get the originals' melodies note
for note from a free public-domain source. The only written-out versions
found were modern paid arrangements, which this design rules out. A wrong
melody credited to a real composer would be worse than a different tune.
The replacements keep the joke and the nationality, and the original
three can come back if a source is found.

Shot names are as they stand in `tools/punchout_film/chapters/`.

**Timing (Nate's call): the music runs free.** Apart from the funeral
march's cut at Mac's last knockdown, no cue is synced to the action. A
cue loops and is stretched to fill its span as the scorer does now. The
events are marked by stings instead: the bell already rings at
knockdowns and knockouts, and a new **crowd roar** sting (noise channel)
joins it at each knockdown dealt and at each knockout, in every chapter
that rings the bell. So chapter 8's fanfare stalls on its own schedule;
the bells and roars land on the knockdowns.

## Credits

The credits shot in `c11_verdict.py` lists the music and currently says
Korobeiniki (traditional), the Minuet in G and "Everything else:
Original". After this change that is false. The credits list every
public-domain piece used, by composer and year (Rouget de Lisle, Offenbach,
Chopin, Beethoven, Ach du lieber Augustin traditional, Mozart, Wagner,
Korobeiniki traditional),
plus "Everything else: original". The credits shot gets longer to fit
them (my call); chapter 11 and its reprise span grow to match.

## Constraints

- Music is written in the existing tracker (`tools/reservoir_film/music/core.py`):
  two pulse channels, triangle and noise drums, instruments as per-frame
  macros, rendered by `examples/apu_render.rs`. New instruments (a
  brassier pulse, crowd noise) are fine within that model.
- The scorer stretches a cue to its span within `TEMPO_RANGE` (0.85 to
  1.15) by choosing a loop count. Each new cue must fit each span it is
  used for without falling back to trimming or padding, or the builder
  says which span it couldn't fit and why.
- The Tetris film's cues and `CHAPTER_CUES` keep working unchanged; its
  rendered score should be bit-identical.
- Stings (the bell) still mix on top as now.
- Each new cue renders alone through `reservoir_film.music.audition` (or
  a Punch-Out!! equivalent) into `target/punchout-film/music/`, with its
  spectrogram, so Nate can listen before the full build.

## Left to the builder

- The arrangements themselves: keys, tempos, which bars of each tune,
  harmonisation, the exact sourness of the circus and the stall of the
  never-resolving fanfare.
- The ring-entrance fanfare and the triumph variation are original
  compositions.
- How cue spans are expressed (a per-shot field, a map in `music.py`, or
  similar), provided the Tetris film needs no change.
- Whether `reprise` and `lab` are reused as they are or touched up.
