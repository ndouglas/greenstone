# The Visual Language and 001's Pieces Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** The series' Ember look as code, plus the ten reusable pieces 001 needs, each tested and each tried as a sketch Nate sees.

**Architecture:** `style.py` holds the tokens and the two rules (temperature is sign; one light scale per video); `ember.py` the ground, glow, serif text and live text; `recording.py` frame-exact recordings with each game's measured lag; `screen.py` the game screen with its TV treatment; `overlays.py` the marks drawn over it; then one module per piece (`neuron.py`, `formula.py`, `graph.py`, `bordermap.py`, `balance.py`, `memory.py`, `records.py`), and `title.py`/`credits.py` restyled. Each piece gets a sketch in `tools/series/videos/001-one-neuron/sketches/`. The throwaway spike (`scratchpad/spike/sketches/ember_neuron_v2.py`) is the reference for look and numbers; nothing is copied from it without its test.

**Tech Stack:** Python 3.12 (tools/series/.venv), Manim Community 0.22.1, numpy, scipy (Manim's), soundfile, unittest; the Rust `series_record` example's footage recordings.

**Spec:** docs/superpowers/specs/2026-10-30-visual-language-design.md

## Global Constraints

- Ground: radial gradient `#24100A` (toward the action) to `#030303`; light means understanding.
- Temperature is sign: warm (amber `#F2B34A`, coral `#EE8660`, gold `#FFD59A`, rose, crimson) above zero; cool (blue `#4F8FB8`, light blue `#7FB7E0`, teal, violet) below; the dark is zero. Hue tells things apart within a temperature; combining paths blend.
- One light scale per video: brightness is a fixed, smooth, saturating function of a total's size, the same for every neuron and beat in a video.
- Type: Charter; italic for labels and words, roman for numbers; true minus (−); formulas in the same serif. Labels ≈ 0.42, verdicts ≈ 0.5 of Manim's text scale at 1080p.
- Lines: hairlines, ≈1.2–1.4 px at 1080p.
- A continuous value glows (one smooth gradient image, easing toward its value over ≈1/9 s); a binary answer is a ring its line runs into, filled warm for yes, empty and cool for no, no glow.
- Motion: instruments arrive one at a time over ≈0.5–1 s, after what they measure; numbers are live from the moment they appear.
- Game screen: whole-number scale (3× at 1080p: 768 × 720) on whole pixels, nearest-neighbour; smooth averaging below 1×; original colours, no dimming; rounded corners (≈26 px at 3×); glow = blur of the rounded screen in the clip's average colour, fading up with the picture; bloom over the rim; soft edge darkening; scanlines (≈12% dark, last screen row of each NES line) only at ≥3×, automatic, with a flag; arrival by a cover fading off, never FadeIn.
- Footage only from per-frame recordings (`series_record --footage`); overlays read RAM one frame behind the picture in Super Mario Bros. (measured, 99%); each game's lag is measured against its sprite, never assumed.
- Neuron: hairline ring, total in roman, its glow; inputs are paths from a screen mark, a memory square, or a bare number; a bias that isn't a place is a number with no path; 001's setting is a fixed gold line at depth 176 (the bias's source); the total is the gap between Mario's feet line (depth + 33 rows from the top) and the setting line, shaded warm above, gone on it, cool below; weight ≠ −1 brightens the band on the light scale; falling speed is an arrow at Mario (length its speed, warm falling, cool rising).
- Formula symbols are neutral parchment until filled; filled terms take their sign's temperature.
- Title tiles stay stand-ins until beats are rendered.
- Never stage `src/main.rs` or `src/nes/bus/mod.rs`; commit with `git commit -F` and the session trailer; work on `main`; don't push without asking.
- Tests: `cd tools && series/.venv/bin/python -m unittest discover -s series/tests -t .`. Sketches: `cd tools && series/.venv/bin/python -m series.sketch PATH [--final]`.

## Review Focus

- A recording whose RAM is missing or shorter than its footage: a clear error naming both lengths, not a silent misalignment. Test: `test_a_recording_with_short_ram_is_refused` (Task 2).
- A game with no measured lag used for overlays: refused, naming the game, rather than drawn a frame off. Test: `test_an_unmeasured_game_is_refused` (Task 2).
- A screen placed at a size or position that isn't a whole number of pixels: snapped to whole pixels (or, below 1×, smoothed), never stretched unevenly. Test: `test_screens_snap_to_whole_pixels` (Task 3).
- A total of exactly zero: dark glow, empty ring, cool verdict ("no"), no warm or cool band. Test: `test_zero_is_dark_and_no` (Task 5).
- Text that becomes negative mid-animation (a total, a formula term): a true minus and the cool temperature, never a hyphen. Test: `test_negative_numbers_use_a_true_minus` (Task 1).

---

### Task 1: Style tokens, ground and light

**Files:**
- Modify: `tools/series/style.py` (rewrite)
- Create: `tools/series/ember.py`
- Modify: `tools/series/scene.py`, `tools/series/title.py`, `tools/series/credits.py`, `tools/series/videos/000-test-clip/scene.py`, `tools/series/videos/000-test-clip/sketches/neuron.py` (old names `INPUT`, `WEIGHT`, `BIAS`, `OUTPUT` replaced)
- Test: `tools/series/tests/test_ember.py`

**Interfaces:**
- Produces (`style.py`): `BACKGROUND = "#030303"`, `EMBER = "#24100A"`, `TEXT = "#FBE3C4"` (parchment), `ASH = "#A88A6C"`, `WARM = ("#F2B34A", "#EE8660", "#FFD59A", "#E8597A", "#C0392B")` (amber, coral, gold, rose, crimson), `COOL = ("#4F8FB8", "#7FB7E0", "#5EC8D8", "#9A8CF0")` (blue, light blue, teal, violet), `FONT = "Charter"`; `temperature(value: float, hue: int = 0) -> str` (warm[hue] above 0, cool[hue] below, `ASH` at 0); `LIGHT_SCALE = 60.0` (the video's one light scale; a video may set its own, once); `light(value: float, scale: float = LIGHT_SCALE) -> float` in [0, 1), `tanh(|value| / scale)`; `number(value: float, places: int = 0) -> str` with a true minus.
- Produces (`ember.py`): `backdrop(focus: np.ndarray = RIGHT * 3) -> ImageMobject` (full frame); `glow(colour: str, radius: float) -> ImageMobject` (smooth, alpha exactly 0 at its border); `serif(text: str, colour: str, size: float = 0.42, italic: bool = True) -> Text`; `live(mobject, place: Callable, shown: ValueTracker) -> Mobject` (redrawn every frame, faded by the tracker); `fade_image(image: ImageMobject, level: ValueTracker) -> None` (scales its own alpha; never `set_opacity`).

- [ ] **Step 1: Write the failing tests** (`test_ember.py`):
  - `test_temperature_is_sign`: `temperature(5) in WARM`, `temperature(-5) in COOL`, `temperature(0) == ASH`, `temperature(2, hue=1) == WARM[1]`.
  - `test_light_is_smooth_and_saturating`: `light(0, 60) == 0`; increasing in |value|; `light(-30, 60) == light(30, 60)`; `light(10_000, 60) < 1`.
  - `test_negative_numbers_use_a_true_minus`: `number(-41) == "−41"`, `number(3) == "3"`, `number(-0.4) == "0"` (rounds to zero, no minus).
  - `test_glow_has_no_edge`: the alpha channel of `glow(...)` is 0 on all four borders, its maximum is at the centre, and it has more than 150 distinct alpha levels (smooth, not banded).
  - `test_serif_is_charter`: `serif("x", TEXT).font == "Charter"`, and `"Charter" in manimpango.list_fonts()` (fail with "install Charter" otherwise).
  - `test_fading_an_image_keeps_its_gradient`: `fade_image` at 0.5 halves every alpha value (within 1) and keeps the gradient's shape.
- [ ] **Step 2: Run** `series.tests.test_ember` — Expected: ImportError.
- [ ] **Step 3: Implement**, then replace the old colour names in the five files with the new tokens (input-like → `WARM[0]`, weight → `COOL[0]`, bias → `WARM[1]`, output → `WARM[2]`).
- [ ] **Step 4: Run** the whole suite — Expected: OK.
- [ ] **Step 5: Sketch** `videos/001-one-neuron/sketches/ground.py`: the backdrop; a warm and a cool glow easing between values; live serif numbers counting through zero (true minus). Render `--final`, read the contact sheet.
- [ ] **Step 6: Commit** — "Add the Ember style: sign as temperature, one light scale, the ground and glow."

### Task 2: Frame-exact recordings and each game's lag

**Files:**
- Create: `tools/series/recording.py`
- Test: `tools/series/tests/test_recording.py`

**Interfaces:**
- Produces: `class Recording(game: str, root: Path = OUT / "001-one-neuron" / "data")` with `frames` (the footage take's frames, `root/<game>/footage`), `ram: np.ndarray` (`root/<game>/1/ram.u8`, frames × 2048), `len(recording)` = the footage's length; `ram_at(frame: int) -> np.ndarray` = the RAM the picture of `frame` shows (`frame − LAG[game]`, clamped at 0); `LAG: dict[str, int]` = `{"smb1": 1, ...}`; `measure_lag(frames, ram, sprite_bottom: Callable[[np.ndarray], int | None], byte: int, offset: int, max_lag: int = 3) -> tuple[int, float]` (best lag and its share of exact matches on moving frames); `smb1_sprite_bottom(frame) -> int | None` (Mario's lowest shoe pixel row: the colours `(152, 34, 32)` red and `(84, 90, 0)` olive, as in the spike's measurement).

- [ ] **Step 1: Write the failing tests:**
  - `test_ram_is_read_behind_the_picture`: a synthetic recording (frames whose "sprite" row is `ram[f-1][byte] + offset`) → `ram_at(10)` is `ram[9]`; `measure_lag(...)` returns `(1, 1.0)`.
  - `test_a_recording_with_short_ram_is_refused`: RAM shorter than footage → `ValueError` naming both lengths.
  - `test_an_unmeasured_game_is_refused`: a game not in `LAG` → `ram_at` raises `KeyError` naming the game and how to measure it.
  - `test_smb1s_lag_is_measured` (skipped when `target/series/001-one-neuron/data/smb1/footage` is absent): `measure_lag` on frames 1000–1940 with `smb1_sprite_bottom`, byte `0x00CE`, offset 32 → lag 1 with ≥ 0.95 exact, and equal to `LAG["smb1"]`.
- [ ] **Step 2: Run** `series.tests.test_recording` — Expected: ImportError.
- [ ] **Step 3: Implement.** Measure Super Mario Bros. 3's and Castlevania's lags the same way if a sprite rule can be found quickly (player's feet byte and colours from `docs/series/games/001.md`); add each with its own skipped-when-absent test. A game whose sprite can't be isolated is left out of `LAG` (refused until measured), with a ledger ruling.
- [ ] **Step 4: Run** — Expected: OK.
- [ ] **Step 5: Commit** — "Add frame-exact recordings, with each game's lag measured against its sprite."

### Task 3: The game screen

**Files:**
- Create: `tools/series/screen.py` (uses `footage.GameFootage` for frames)
- Test: `tools/series/tests/test_screen.py`

**Interfaces:**
- Consumes: `Recording` (Task 2); `fade_image`, `glow` style rules (Task 1).
- Produces: `class Screen(Group)(recording: Recording, start: int, scale: float | None = None, scanlines: bool | None = None, left_px: int | None = None, centre: np.ndarray | None = None)`; `scale=None` means 3 at 1080p (`pixel_height * 2 // 720`); attributes `scale`, `dot` (one NES pixel in scene units), `game` (the GameFootage), `frame() -> int` (the frame on screen now), `ram() -> np.ndarray` (`recording.ram_at(frame())`), `row_y(row: float) -> float`, `col_x(col: float) -> float` (top/left edges of NES pixel rows/columns); `arrive(run_time: float = 0.8) -> Animation` (cover fades off the screen, glow fades up by its alpha; footage holds its first frame until it ends); module functions `screen_overlay(w, h, scale, behind) -> np.ndarray`, `screen_glow(colour, w, h, scale) -> tuple[np.ndarray, int]`, `screen_bloom(colour, w, h, scale) -> np.ndarray`, `rounded_mask(w, h, r) -> np.ndarray` (from the spike).

- [ ] **Step 1: Write the failing tests** (a short synthetic recording; `tempconfig` at 1920 × 1080):
  - `test_screens_snap_to_whole_pixels`: at scale 3 the screen is 768 × 720 px and its left and top edges fall on whole pixels; asking for `left_px=36.4` snaps to 36.
  - `test_scanlines_only_where_they_fit`: scale 3 → overlay has dark rows at every third pixel row; scale 1 → none; scale 3 with `scanlines=False` → none; `scanlines=True` at scale 1 → `ValueError`.
  - `test_small_screens_are_smoothed`: scale 1/3 → the footage's resampling is not nearest (averaged); scale 1 and 3 → nearest.
  - `test_the_glow_has_no_edge_and_follows_the_corners`: `screen_glow` alpha is 0 on its image border; at the screen's corner the glow is dimmer than at the middle of an edge (a blur of a rounded shape).
  - `test_arrival_holds_the_first_frame`: rendering `arrive()` then a 0.5 s wait → `frame()` was `start` throughout the arrival, then advances.
  - `test_the_ram_matches_the_picture`: `screen.ram()` equals `recording.ram_at(screen.frame())`.
- [ ] **Step 2: Run** — Expected: ImportError.
- [ ] **Step 3: Implement** from the spike's numbers (corner radius `26 * scale // 3` px, glow σ = `14 * scale` px with margin 4σ and strength `GLOW = 0.3`, bloom `0.6 × GLOW` stopping at the rounded edge, edge darkening 0.14 at the corners, scanline alpha 0.12; all lengths scale with the screen).
- [ ] **Step 4: Run** — Expected: OK.
- [ ] **Step 5: Sketch** `sketches/screen.py`: one 3× screen arriving and playing 5 s of smb1; then three 1× screens and nine 1/3× screens of the three games side by side. Render `--final`; check the corners with shadows lifted (`ffmpeg … curves`), as in the spike.
- [ ] **Step 6: Commit** — "Add the game screen: exact pixels, an old TV's corners and glow, frame-exact RAM."

### Task 4: Marks on the screen

**Files:**
- Create: `tools/series/overlays.py`
- Test: `tools/series/tests/test_overlays.py`

**Interfaces:**
- Consumes: `Screen` (Task 3); `temperature`, `light`, `serif`, `live` (Task 1).
- Produces: `feet_line(screen, byte: int = 0x00CE, offset: int = 33, label: str = "depth") -> Mobject` (a hairline across the screen at the bottom edge of the sprite's lowest row, with its live reading; exposes `.y() -> float` and `.value() -> int`); `setting_line(screen, row: int, label: str) -> Mobject` (fixed gold hairline, `.y()`); `gap_band(screen, upper, lower, weight: float = -1, scale: float = LIGHT_SCALE) -> Mobject` (the band between two lines, warm when `upper` is above `lower`, cool when below, absent when equal, brightness `light(weight × gap, scale)`); `speed_arrow(screen, x_byte: int, y_byte: int, speed_byte: int) -> Mobject` (at Mario, length ∝ speed, warm falling, cool rising, absent at zero).

- [ ] **Step 1: Write the failing tests** (synthetic recording with known bytes):
  - `test_the_feet_line_sits_under_the_sprite`: depth 100 → `feet_line.y() == screen.row_y(133)`; it reads the lagged RAM (`screen.ram()`).
  - `test_the_gap_is_the_total`: depth 150, setting 176 → band height `26 × dot`, warm; depth 176 → no band; depth 200 → cool.
  - `test_a_bigger_weight_burns_brighter`: weight −3 vs −1 at the same gap → brighter, same height.
  - `test_the_speed_arrow_shows_direction`: falling speed +3 → warm, pointing down; −3 → cool, up; 0 → absent. (The falling-speed byte is signed: `0x009F`, as in `exp001`.)
- [ ] **Step 2–4:** Run (ImportError), implement, run (OK).
- [ ] **Step 5: Commit** — "Add the marks drawn on the game: feet and setting lines, the gap, the speed arrow."

### Task 5: The neuron

**Files:**
- Create: `tools/series/neuron.py`
- Test: `tools/series/tests/test_neuron.py`

**Interfaces:**
- Consumes: Tasks 1, 3, 4.
- Produces: `@dataclass Input(value: Callable[[], float], weight: float, source: Callable[[], np.ndarray] | None, label: str)` (source: where its path starts, or None for a bare number); `class Neuron(Group)(inputs: list[Input], bias: float, bias_source: Callable[[], np.ndarray] | None, centre: np.ndarray, yes: str, no: str, scale: float = LIGHT_SCALE)` with `total() -> float`, `answer() -> bool` (`total() > 0`); parts: ring, live total (roman, true minus), glow (warm/cool, `light(total, scale)`, eased ≈1/9 s), one path per input from its source (warm leaving, cooling or staying warm by the sign of `weight × value`), the bias path (or a bare "+ b" label), the output line into the answer ring, the crisp dot, the live verdict; `arrive() -> list[Animation]` (in order: paths, ring and total, bias, output).

- [ ] **Step 1: Write the failing tests:**
  - `test_the_total_is_the_weighted_sum`: inputs (3, w 2), (5, w −1), bias 4 → total 5, answer True.
  - `test_zero_is_dark_and_no`: total 0 → glow light 0, dot empty, ring cool, verdict is `no` in a cool colour.
  - `test_the_answer_is_crisp`: totals 0.01 and 500 → the dot is fully filled in both (no glow on the answer).
  - `test_paths_take_their_contributions_temperature`: input 176 with weight −1 → its path ends cool; bias +176 → warm.
  - `test_a_bias_with_no_place_has_no_path`: `bias_source=None` → no bias path, a "+ b" label instead.
  - `test_inputs_can_come_from_anywhere`: a source on a screen line, one on a memory square (a fixed point), and `None` all build.
- [ ] **Step 2–4:** Run, implement, run.
- [ ] **Step 5: Sketch** `sketches/neuron.py`: the spike's thirteenth pass rebuilt from the pieces (Screen, feet and setting lines, gap, neuron), on smb1 frames 1247–1943; then a second scene, the two-input neuron (depth and the speed arrow, 001's land-soon weights from `numbers.json`). Render `--final`; compare against `target/series/spike/sketches/ember_neuron_v2.mp4`.
- [ ] **Step 6: Commit** — "Add the neuron, drawn from what it measures."

### Task 6: The formula

**Files:** Create `tools/series/formula.py`; Test `tools/series/tests/test_formula.py`.

**Interfaces:**
- Produces: `class Formula(VGroup)(terms: list[str], comparison: str = "> 0")` (e.g. `["w·x", "+ b"]`), set in Charter, every symbol parchment; `fill(name: str, value: float) -> Animation` (replaces a symbol with its number, coloured by sign, true minus); `move_into_bias(threshold: float) -> Animation` (the "> 100" becomes "− 100" beside the bias, leaving "> 0"); `text() -> str` (the formula as written now).

- [ ] **Step 1: Write the failing tests:** `test_symbols_start_neutral` (all parchment); `test_filled_terms_take_their_sign` (w = −1 cool, b = 176 warm, text `"−1 · x + 176 > 0"`); `test_the_threshold_moves_into_the_bias` (`"w·x + b > 100"` → `"w·x + b − 100 > 0"`).
- [ ] **Step 2–4:** Run, implement, run.
- [ ] **Step 5: Sketch** `sketches/formula.py`: assemble, fill 001's numbers, move "> 100" into the bias.
- [ ] **Step 6: Commit** — "Add the formula, neutral until filled."

### Task 7: The graph

**Files:** Create `tools/series/graph.py`; Test `tools/series/tests/test_graph.py`.

**Interfaces:**
- Produces: `class NeuronGraph(VGroup)(weight: float, bias: float, x_range: tuple, y_range: tuple, x_label: str, y_label: str = "total", yes: str, no: str)`: hairline axes, faint grid, the line coloured by its own sign along its length (warm above the x-axis, dark at the crossing, cool below), the setting as a small mark at the crossing, the sides faintly warm and cool; `setting() -> float` (= −bias / weight); `scale_weights(k: float) -> Animation` (line steepens and pivots about the setting; the setting and the shading don't move).

- [ ] **Step 1: Write the failing tests:** `test_the_setting_is_where_the_line_crosses` (w −1, b 176 → 176); `test_the_line_is_warm_above_and_cool_below`; `test_tripling_never_moves_the_setting` (after `scale_weights(3)`, setting 176, mark unmoved).
- [ ] **Step 2–4:** Run, implement, run.
- [ ] **Step 5: Sketch** `sketches/graph.py`: 001's graph drawn, then "× 3" and the pivot.
- [ ] **Step 6: Commit** — "Add the graph: light by sign, the setting fixed."

### Task 8: The map

**Files:** Create `tools/series/bordermap.py`; Test `tools/series/tests/test_bordermap.py`.

**Interfaces:**
- Produces: `class BorderMap(VGroup)(points: np.ndarray, outcomes: list[str | None], weights: tuple[float, float], bias: float, x_label: str, y_label: str, yes: str, no: str)`: dots dim ash, outcome dots lit (one warm hue per outcome, e.g. floor amber, blocks rose); the border a hairline shading from the first input's hue to the second's; the yes side faintly warm, no side faintly cool; `slide_bias(new: float) -> Animation` (parallel move), `lean(weights) -> Animation`, `border_points() -> tuple[np.ndarray, np.ndarray]`.

- [ ] **Step 1: Write the failing tests:** `test_the_border_is_where_the_total_is_zero` (both end points satisfy w·p + b ≈ 0); `test_outcomes_light_their_dots`; `test_sliding_the_bias_keeps_the_lean` (same slope after `slide_bias`).
- [ ] **Step 2–4:** Run, implement, run.
- [ ] **Step 5: Sketch** `sketches/bordermap.py` from 001's real moments (`exp001`'s land-soon data, a few hundred points), border drawn, bias slid, weights leaned.
- [ ] **Step 6: Commit** — "Add the map and its border."

### Task 9: The balance scale

**Files:** Create `tools/series/balance.py` (supersedes `videos/000-test-clip/sketches/neuron.py`'s scale); Test `tools/series/tests/test_balance.py`.

**Interfaces:**
- Produces: `class Balance(VGroup)(yes: str, no: str)`: hairline beam running warm to cool, yes pan warm, no pan cool; `drop(value: float, label: str, hue: int = 0) -> Animation` (a block sized by |value| falls into the pan its sign says, coloured by sign and hue); `verdict() -> str`, tilt follows the sum.

- [ ] **Step 1: Write the failing tests:** `test_blocks_fall_by_sign`; `test_the_beam_tips_to_the_heavier_pan`; `test_the_verdict_takes_its_sides_colour`; `test_a_balanced_scale_says_no` (sum 0 → no).
- [ ] **Step 2–4:** Run, implement, run.
- [ ] **Step 5: Sketch** `sketches/balance.py`: 001's land-balance example (numbers from `numbers.json`).
- [ ] **Step 6: Commit** — "Add the balance scale."

### Task 10: The memory grid, the tapes and the score bars

**Files:** Create `tools/series/memory.py`, `tools/series/records.py`; Test `tools/series/tests/test_memory.py`, `tools/series/tests/test_records.py`.

**Interfaces:**
- Produces: `class MemoryGrid(VGroup)(screen_or_recording, columns: int = 64)`: 2,048 bytes as a grid of small serif numbers, live with the game (lagged RAM), dim ash; `light(address: int, label: str, hue: int = 0) -> Animation`, `square(address) -> Mobject` (a source for `Input`); `class Tapes(VGroup)(n: int, labels)` with `apart(i) -> Animation`, `grey(i, spans)`; `class ScoreBars(VGroup)(baseline: float)` with `add(value: float, label: str, faint: bool = False) -> Animation`, heights ∝ value, the baseline bar ash.

- [ ] **Step 1: Write the failing tests:** `test_the_grid_holds_every_byte` (2,048 cells, cell at `0x00CE` shows `ram[0xCE]` of the frame on screen); `test_a_lit_square_is_a_source`; `test_bars_are_proportional`; `test_the_baseline_is_ash`.
- [ ] **Step 2–4:** Run, implement, run.
- [ ] **Step 5: Sketch** `sketches/memory.py` (the grid live under smb1, three squares lit and labelled: position, lives, jumping) and `sketches/records.py` (three tapes, the third apart; bars for baseline and result).
- [ ] **Step 6: Commit** — "Add the memory grid, the tapes and the score bars."

### Task 11: Title screens and credits in Ember, and the docs

**Files:**
- Modify: `tools/series/title.py`, `tools/series/credits.py`, `tools/series/scene.py` (backdrop behind every scene; placeholder cards in the new type)
- Modify: `docs/series/bible.md` ("Visual language": this design's rules, replacing "one colour per concept"), `tools/series/videos/001-one-neuron/script.toml` (the two boards naming old colours: "w·x + b > 0 assembles, w in orange, x in blue, b in purple" and "…colour-coded, beside the border"), via the generator in the scratchpad if it's still there, else directly
- Test: `tools/series/tests/test_scene.py`, `tools/series/tests/test_credits.py` (existing tests updated, plus new)

**Interfaces:**
- Consumes: Task 1.
- Produces: title card on the Ember ground: the wall's tiles dim warm stand-ins, its words in Charter (chapter number italic, title roman); credits pages in Charter on the ground; every `NarratedScene` draws `backdrop()` first.

- [ ] **Step 1: Write the failing tests:** `test_titles_are_in_charter`; `test_every_scene_starts_on_the_ground` (the backdrop is the first mobject after `setup`); existing title and credits tests still pass (text content unchanged).
- [ ] **Step 2–4:** Run, implement, run the whole suite.
- [ ] **Step 5: Rebuild the 001 storyboard** (`--storyboard --draft`) and read a title screen, a placeholder and a credits page from it.
- [ ] **Step 6: Commit** — "Put titles, placeholders and credits in the Ember look; the bible's visual language rewritten."
