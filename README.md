# Greenstone

An NES emulator written in Rust, with a focus on cycle accuracy and understanding the hardware at a deep level.

## Why an NES Emulator?

The NES is a fascinating machine to emulate. It's simple enough to be tractable (the 6502 has only 56 instructions, though with addressing modes that expands to 151 official opcodes plus dozens of "illegal" ones), but complex enough to be interesting. The real challenge isn't the CPU—it's the PPU.

The Picture Processing Unit is where things get weird. It's not a framebuffer-based system. Instead, the PPU renders pixels in real-time, racing the electron beam across the screen. Games exploit this timing in creative ways: they change scroll positions mid-scanline, swap pattern tables during vblank, and trigger sprite-0 hits to split the screen. Getting these timing-dependent tricks right requires understanding not just *what* the hardware does, but *when* it does it.

This project is my attempt to understand that timing at a deep level.

## Current State

**What works:**
- Complete 6502 CPU implementation with all official opcodes
- All "illegal" (undocumented) opcodes that games actually use
- Cycle-accurate instruction timing
- Cartridge loading with iNES format parsing
- Mapper 0 (NROM) support—enough for Donkey Kong, Ice Climber, and similar early titles
- PPU register interface and VRAM/OAM memory
- Horizontal and vertical nametable mirroring
- NMI generation on vblank
- Optional WebSocket debug server for external tool integration

**What's in progress:**
- PPU rendering pipeline (scanline/dot timing, background tiles, sprites)
- Sprite evaluation and priority
- Scroll register behavior during rendering

**What's planned:**
- Additional mappers (MMC1, MMC3, etc.)
- APU (audio)
- Controller input beyond the current test harness

## Architecture

The emulator is structured around trait-based abstractions that mirror the NES hardware:

```
┌─────────────────────────────────────────────────────────┐
│                         Bus                             │
│  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌───────────┐  │
│  │   CPU   │  │   PPU   │  │   RAM   │  │ Cartridge │  │
│  │  6502   │  │  2C02   │  │   2KB   │  │  + Mapper │  │
│  └─────────┘  └─────────┘  └─────────┘  └───────────┘  │
└─────────────────────────────────────────────────────────┘
```

**Key traits:**
- `Addressable` — Read/write operations at 16-bit addresses
- `Busable` — Full bus interface combining Addressable + Interruptible
- `Interruptible` — NMI and IRQ signal handling
- `Mappable` — Cartridge mapper interface for bank switching

The CPU ticks the bus, which ticks the PPU at a 3:1 ratio (the PPU runs at 5.37 MHz vs the CPU's 1.79 MHz). This timing relationship is critical for accurate emulation.

## What I've Learned

Building this has taught me things about hardware design that I wouldn't have learned any other way:

**The PPU is a state machine, not a graphics card.** Modern GPUs accept commands and render frames. The NES PPU is a circuit that outputs one pixel per cycle, consulting memory as it goes. It doesn't "know" what the final frame will look like—it just follows its state machine, and games manipulate that state to create effects.

**Memory-mapped I/O is elegant but tricky.** Reading $2002 (PPU status) has side effects—it clears the vblank flag and resets the address latch. Writing $2006 twice sets a 14-bit address. These aren't just memory locations; they're hardware interfaces with behavior.

**Cycle accuracy matters more than I expected.** Many games work fine with approximate timing. But the moment you try to run something that uses sprite-0 hits for a status bar, or changes scroll position mid-frame, you discover that "close enough" isn't.

**The 6502's illegal opcodes are real instructions.** They're not random behavior—they're the result of how the CPU's microcode combines operations. `LAX` loads both A and X. `DCP` decrements memory then compares. Games use these, so emulators must support them.

## Running

```bash
# Build and run with a ROM
cargo run --release -- -f path/to/rom.nes

# Run with debug server (WebSocket on port 44553)
cargo run --release -- -f path/to/rom.nes --serve

# Run tests
cargo test

# Run with trace logging
RUST_LOG=trace cargo test test_name
```

## Testing

Each CPU instruction has its own test file under `src/nes/cpu/instructions/`. The test macro validates:
- Register state changes
- Status flag behavior (respecting the instruction's flag mask)
- Memory modifications
- Cycle counts

```rust
test_instruction!("ADC", Immediate, [0x03]{a: 0x02} => []{a: 0x05});
//                 ^      ^          ^     ^           ^   ^
//                 |      |          |     |           |   expected state
//                 |      |          |     initial     expected memory
//                 |      |          operand bytes
//                 |      addressing mode
//                 mnemonic
```

## Acknowledgements

This project draws heavily from the NES development community:

- [NesDev Wiki](https://www.nesdev.org/) — The definitive NES hardware reference
- [Bugzmanov's NES Ebook](https://bugzmanov.github.io/nes_ebook/) — Clear, approachable introduction
- [Starr Horne's nes-rust](https://github.com/starrhorne/nes-rust) — Test cases, test macro design, cartridge structure
- [One Lone Coder's olcNES](https://github.com/OneLoneCoder/olcNES/) — Excellent video explanations
- [masswerk.at 6502 Reference](https://www.masswerk.at/6502/6502_instruction_set.html) — Per-opcode cycle and flag details
- [6502.org](http://www.6502.org/tutorials/6502opcodes.html) — Opcode behavior documentation
