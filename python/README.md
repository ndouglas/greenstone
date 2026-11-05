# greenstone (Python)

The emulator from Python, for notebooks: `gs.boot("smb1")`, then `step`, `play`, `ram`, `peek`/`poke`,
`picture`, `save`/`load`. ROMs come from `roms2/` (or `$GREENSTONE_ROMS`).

- Tests: `uv venv .venv && uv pip install --python .venv/bin/python -e . && .venv/bin/python -m unittest discover -s tests`
- After changing the Rust, rebuild where it's installed: `uv sync --reinstall-package greenstone`
