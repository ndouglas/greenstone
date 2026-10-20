# The Prospector: SMB's accidental levels

Status: approved design, 2026-10-20. Source idea: `docs/ideas/brain-cartridge.md`,
"Build order" item 1.

## What it is

SMB finds each level's layout through a pointer table. Rewrite 1-1's entry in
our copy of the ROM to point at offset X, and the game's own loader builds a
level out of whatever bytes live at X. Every offset in the 32 KB PRG ROM is an
accidental level nobody designed. The Prospector loads all of them, throws
out the ones that crash or can't be played, searches the rest for a way
through, and ranks the cleared ones by how narrowly they can be cleared.

## Deliverable

A ranked catalog plus short replay clips of the top finds. Film-ready
material, not a film.

- `target/prospector/catalog.jsonl`: one line per offset, every stage's verdict.
- `target/prospector/takes/<offset>/`: take for each top find (film format).
- `target/prospector/clips/<rank>-<offset>.mp4`: silent captioned clips.
- `target/prospector/catalog.html`: static contact sheet.

## Decisions

- **Worst means hard but clearable**: the search gets through, but only just.
- **Cleared** means SMB moved on from 1-1 (flagpole, castle, warp pipe:
  `Clear::Flag`, badged) or Mario is alive at x >= `GOAL_X` = 3072 px
  (`Clear::Distance`).
- **Hijack by ROM patch**, not RAM poke, so the first screens come from X too.
- **Terrain only in v1**: 1-1's enemy pointer is aimed at a `$FF` byte (empty list).
- **Ground type only in v1**: the area type comes from the table index, which stays 1-1's.

## ROM facts (SMB, PRG at file offset 16, CPU `$8000`)

- Area type offset tables: layouts at `$9D28` (water 0, ground 3,
  underground `$19`, castle `$1C`), enemies at `$9CE0` (ground 6).
- 1-1 is ground area 5, so its layout entry is index 8: low byte `$9D34`,
  high byte `$9D56`; it points at `$A68E` (header `$50 $21`).
- 1-1's enemy entry is index 11: low byte `$9CE4+11`, high byte
  `$9D06+11`; it points at `$9F01`.
- The first `$FF` byte in PRG is at `$8008`; a test pins whichever empty-list
  byte the patch uses.

## Architecture

New module `src/nes/gym/prospector/`, beside `reservoir/`.

### `patch.rs`

- `patch_rom(rom: &[u8], area_addr: u16) -> Vec<u8>`: writes `area_addr`
  into 1-1's layout entry and points 1-1's enemy entry at an empty list.
- `area_pointer(rom: &[u8]) -> u16`: reads 1-1's layout entry back.

### `funnel.rs`: one function per stage, each takes a patched ROM

1. `load(rom) -> Load`: `Crashed(String)` (panic caught with
   `catch_unwind`), `Jammed` (no frame within a step budget), `Blank`
   (the frame after load is one colour), `InstantDeath` (dead or lives lost
   within 5 s idle), or `Alive { start_state, thumb }`. `thumb` is the first
   loaded frame.
2. `fingerprint(start_state) -> u64`: hash of both block buffers after load.
   The scan keeps the first offset per fingerprint; later ones are
   `duplicate_of`.
3. `walk(rom, start_state, seed) -> Walk`: 600 frames of right+B with random
   jumps. `Stuck` (x moved < 64 px), `Died(x)`, or `Reached(x)`.
4. `solve(rom, start_state) -> Solve`: `beam_search_until` with beam 16 and
   at most 150 rounds. Returns `clear: Option<Clear>`, `x_reached`,
   `rounds`, `path`, and the per-round `RoundStats`.

### `scan.rs`

Walks offsets `$8000..=$FFFF` across threads. Stages 1 to 3 run on every
offset in one pass; stage 4 runs on the top K walker survivors by distance
(K = 50 first, then raised). Appends to `catalog.jsonl`, flushing each line;
on restart, offsets already in the file are skipped.

### Narrowness

Only for cleared levels, worst first, keys in order:

1. **Bottleneck**: minimum `distinct` over all rounds. 1 means every
   surviving branch collapsed to one state: one way through.
2. **Kill rate**: sum(`expanded` - `alive`) / sum(`expanded`). Higher is worse.
3. **Rounds**: more is worse.

Calibration: unpatched 1-1 and 4-1 go through the same `solve`. The catalog
reports where each find sits relative to them ("narrower than 4-1").

### Classes

`crashed`, `jammed`, `blank`, `instant_death`, `duplicate`, `stuck`,
`walker_died`, `unsolved`, `cleared_distance`, `cleared_flag`. Walker
survivors outside the top K are `walked` (not searched).

### Changes to `reservoir/demo.rs`

- `beam_search_until(rom, start, noops, beam, max_rounds, seed, goal, on_round)`,
  where `goal: &(dyn Fn(&mut NesEmulator) -> Option<Clear> + Sync)` and
  `on_round: &mut dyn FnMut(RoundStats)`. `beam_search` becomes a wrapper
  whose goal is "stage changed", so its behaviour and tests are unchanged.
- `RoundStats { expanded, alive, distinct, kept }`, counted inside
  `prune_and_rank`'s caller: children produced, children neither dead nor
  stuck, distinct after the state-hash dedupe, survivors kept.
- Each expansion runs under `catch_unwind`; a panic marks the branch dead.
  A search whose every branch panicked reports it, and the level is `crashed`.

### CLI: `examples/prospector.rs`

- `scan [--k N]`: run the funnel.
- `rank`: print the cleared leaderboard, worst first, with the calibration rows.
- `clips --top N`: replay each winner's path on its patched ROM, with 2 s of
  idle first, and write `takes/<offset>/take.json` + `smb.rgb`.

A missing ROM exits non-zero with a message.

### `tools/prospector/` (Python)

- `clips.py`: reuses `tools/reservoir_film/draw.py`. NES frame at x3 with a
  caption strip: offset (`$C4A0`), clear kind (flag badge), bottleneck, kill
  rate, rounds, calibration comparison. Silent `.mp4`, 15 to 40 s.
- `sheet.py`: `catalog.html`, one static page. Counts per class and the
  calibration rows on top, then the cleared leaderboard (clips linked),
  unsolved, walker-stuck, and the hall of shame (crashed, jammed, blank,
  with panic messages). Thumbnails from `thumbs/<offset>.png`.

## Budget

Beam 16 on 1-1 takes about six minutes; a 150-round cap bounds a level near
that. K = 50 is an evening; K = 200 is roughly 20 hours, and the scan
resumes.

## Testing

Rust (ROM-dependent tests skip through `rom_or_skip`; slow ones are `#[ignore]`):

- `patch_rom(rom, 0xA68E)` returns the ROM unchanged; `area_pointer` reads
  back any patched value.
- The empty-list byte the patch uses is `$FF`.
- `load` on unpatched 1-1 is `Alive`; a pointer into a run of `$FF` is
  `Blank` or `InstantDeath`.
- The patched 1-1 with empty enemies has no enemies loaded after start.
- `fingerprint` is equal across two loads of one offset, and differs
  between 1-1 and 4-1.
- Narrowness ordering on synthetic `RoundStats`.
- `beam_search`'s existing 1-1 tests pass through the wrapper.
- A goal that panics marks branches dead instead of killing the process.

Python:

- `sheet.py` on a tiny fake catalog: section order and leaderboard order.
- `clips.py` smoke test on a 3-frame take.

## Out of scope (follow-ups)

- Enemies (the second pointer, scanned on its own or paired).
- Water, underground and castle area types (4x the scan).
- Music and a film.
- Playing the finds in Greenstone.
