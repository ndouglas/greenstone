# NES Test ROMs

Test ROMs for validating NES emulator accuracy. Contains 119 test ROMs covering CPU, PPU, APU, and mappers. Most tests are from [blargg's test suite](https://github.com/christopherpow/nes-test-roms).

## Quick Reference

| File/Directory                                       | Category | Description                                                            |
| ---------------------------------------------------- | -------- | ---------------------------------------------------------------------- |
| [nestest.nes](./nestest.nes)                         | CPU      | Comprehensive CPU instruction test. Best first test for new emulators. |
| [instr_test_v5/](./instr_test_v5/)                   | CPU      | Detailed instruction tests by addressing mode.                         |
| [cpu_interrupts_v2/](./cpu_interrupts_v2/)           | CPU      | Interrupt behavior (IRQ, NMI, BRK interactions).                       |
| [instr_timing/](./instr_timing/)                     | CPU      | Instruction cycle timing accuracy.                                     |
| [cpu_dummy_reads.nes](./cpu_dummy_reads.nes)         | CPU      | Dummy read behavior on indexed addressing.                             |
| [cpu_dummy_writes_*.nes](./cpu_dummy_writes_oam.nes) | CPU      | Dummy write behavior on RMW instructions.                              |
| [ppu_vbl_nmi/](./ppu_vbl_nmi/)                       | PPU      | VBlank flag and NMI timing (PPU clock-accurate).                       |
| [sprite_hit_tests/](./sprite_hit_tests/)             | PPU      | Sprite 0 hit detection behavior and timing.                            |
| [sprite_overflow_tests/](./sprite_overflow_tests/)   | PPU      | Sprite overflow flag (including hardware bug).                         |
| [blargg_ppu_tests/](./blargg_ppu_tests/)             | PPU      | Palette RAM, sprite RAM, VRAM access, VBL timing.                      |
| [oam_read.nes](./oam_read.nes)                       | PPU      | OAM reading at $2004.                                                  |
| [ppu_open_bus.nes](./ppu_open_bus.nes)               | PPU      | Open-bus behavior on PPU registers.                                    |
| [apu_test/](./apu_test/)                             | APU      | Length counter, IRQ flag, timing.                                      |
| [apu_reset/](./apu_reset/)                           | APU      | APU state after reset.                                                 |
| [mmc3_test/](./mmc3_test/)                           | Mapper   | MMC3 mapper tests.                                                     |
| [AccuracyCoin.nes](./AccuracyCoin.nes)               | Mixed    | Edge cases (B flag, dummy ops, open bus).                              |

---

## CPU Tests

### nestest.nes
The gold standard for CPU testing. Tests all official and many unofficial opcodes. Run in "automated mode" by setting PC to $C000. Compare output against the reference log.

### instr_test_v5/
Detailed CPU instruction tests organized by addressing mode.

| Test                | Description                           |
| ------------------- | ------------------------------------- |
| `01-basics.nes`     | Basic instruction functionality       |
| `02-implied.nes`    | Implied addressing mode               |
| `03-immediate.nes`  | Immediate addressing                  |
| `04-zero_page.nes`  | Zero page addressing                  |
| `05-zp_xy.nes`      | Zero page X/Y indexed                 |
| `06-absolute.nes`   | Absolute addressing                   |
| `07-abs_xy.nes`     | Absolute X/Y indexed                  |
| `08-ind_x.nes`      | Indexed indirect (X)                  |
| `09-ind_y.nes`      | Indirect indexed (Y)                  |
| `10-branches.nes`   | Branch instructions                   |
| `11-stack.nes`      | Stack operations                      |
| `12-jmp_jsr.nes`    | JMP and JSR                           |
| `13-rts.nes`        | RTS instruction                       |
| `14-rti.nes`        | RTI instruction                       |
| `15-brk.nes`        | BRK instruction                       |
| `16-special.nes`    | Special cases                         |
| `official_only.nes` | All official opcodes combined         |
| `all_instrs.nes`    | All instructions including unofficial |

### cpu_interrupts_v2/
Tests interrupt behavior and timing. **Important for accuracy.**

| Test                      | Description                                  |
| ------------------------- | -------------------------------------------- |
| `1-cli_latency.nes`       | CLI/SEI/PLP effect delay on IRQ inhibition   |
| `2-nmi_and_brk.nes`       | NMI interrupting BRK instruction             |
| `3-nmi_and_irq.nes`       | NMI interrupting IRQ vectoring               |
| `4-irq_and_dma.nes`       | IRQ timing around sprite DMA                 |
| `5-branch_delays_irq.nes` | Branch instruction ignores IRQ on last cycle |
| `cpu_interrupts.nes`      | Combined test                                |

**Key behaviors tested:**
- RTI affects I flag immediately
- CLI/SEI/PLP delay I flag effect until after next instruction
- Taken branches ignore IRQ/NMI on last clock

### instr_timing/
Tests CPU instruction cycle counts.

| Test                  | Description                                 |
| --------------------- | ------------------------------------------- |
| `1-instr_timing.nes`  | All instruction timing                      |
| `2-branch_timing.nes` | Branch timing (taken/not taken, page cross) |
| `instr_timing.nes`    | Combined test                               |

### instr_misc/
Miscellaneous instruction tests for edge cases.

### branch_timing_tests/
Detailed branch instruction timing tests.

### cpu_reset/
Tests CPU state after reset.

### cpu_exec_space/
Tests CPU executing from various address spaces (RAM, ROM, etc.).

### cpu_dummy_reads.nes
Tests dummy read behavior when using indexed addressing modes that cross page boundaries.

### cpu_dummy_writes_oam.nes / cpu_dummy_writes_ppumem.nes
Tests dummy write behavior on read-modify-write instructions (ASL, ROL, LSR, ROR, INC, DEC). The CPU writes the original value before the modified value.

### cpu.nes / official.nes
Older blargg CPU tests (from blargg_nes_cpu_test5).

---

## PPU Tests

### ppu_vbl_nmi/
Tests VBlank flag and NMI timing to **PPU-clock accuracy**. Run tests in order.

| Test                     | Description                                     |
| ------------------------ | ----------------------------------------------- |
| `01-vbl_basics.nes`      | Basic VBL operation and period                  |
| `02-vbl_set_time.nes`    | Exact PPU cycle when VBL flag is set            |
| `03-vbl_clear_time.nes`  | Exact PPU cycle when VBL flag is cleared        |
| `04-nmi_control.nes`     | NMI enable/disable during VBL                   |
| `05-nmi_timing.nes`      | Which instruction NMI occurs after              |
| `06-suppression.nes`     | VBL flag suppression reading $2002 at VBL start |
| `07-nmi_on_timing.nes`   | NMI when enabled near VBL clear                 |
| `08-nmi_off_timing.nes`  | NMI when disabled near VBL set                  |
| `09-even_odd_frames.nes` | Odd-frame cycle skip (BG enabled)               |
| `10-even_odd_timing.nes` | Timing of odd-frame cycle skip                  |
| `ppu_vbl_nmi.nes`        | Combined test                                   |

### sprite_hit_tests/
Tests sprite 0 collision detection. Run tests in order.

| Test                   | Description                       |
| ---------------------- | --------------------------------- |
| `01.basics.nes`        | Basic sprite hit behavior         |
| `02.alignment.nes`     | Sprite/background pixel alignment |
| `03.corners.nes`       | Single-pixel sprite corners       |
| `04.flip.nes`          | Horizontal/vertical flipping      |
| `05.left_clip.nes`     | Left-edge clipping                |
| `06.right_edge.nes`    | X=255 and beyond                  |
| `07.screen_bottom.nes` | Y >= 239                          |
| `08.double_height.nes` | 8x16 sprite mode                  |
| `09.timing_basics.nes` | Hit timing (~12 PPU clocks)       |
| `10.timing_order.nes`  | Which pixel triggers hit          |
| `11.edge_timing.nes`   | Hit timing with clipping          |

### sprite_overflow_tests/
Tests sprite overflow flag (bit 5 of $2002). Run tests in order.

| Test             | Description                                  |
| ---------------- | -------------------------------------------- |
| `1.Basics.nes`   | Basic overflow flag operation                |
| `2.Details.nes`  | Edge cases (clipping, Y coords)              |
| `3.Timing.nes`   | Overflow flag timing                         |
| `4.Obscure.nes`  | **Hardware bug**: diagonal sprite evaluation |
| `5.Emulator.nes` | Common emulator mistakes                     |

**Note:** Test 4 tests the famous overflow bug where the PPU incorrectly evaluates sprite Y coordinates after finding 8 sprites.

### blargg_ppu_tests/
General PPU tests.

| Test                   | Description                          |
| ---------------------- | ------------------------------------ |
| `palette_ram.nes`      | Palette RAM read/write and mirroring |
| `sprite_ram.nes`       | OAM access via $2003/$2004/$4014     |
| `vram_access.nes`      | VRAM read/write and read buffer      |
| `vbl_clear_time.nes`   | VBL flag clear timing                |
| `power_up_palette.nes` | Initial palette values               |

### Individual PPU Tests

| Test                       | Description                    |
| -------------------------- | ------------------------------ |
| `ppu_open_bus.nes`         | Open-bus bits on PPU registers |
| `test_ppu_read_buffer.nes` | $2007 read buffer behavior     |
| `oam_read.nes`             | OAM reading via $2004          |
| `oam_stress.nes`           | OAM stress testing             |
| `scroll.nes`               | Scrolling mechanics            |
| `nmi_sync_ntsc.nes`        | NMI timing visual pattern      |
| `full_palette.nes`         | Full palette display           |

---

## APU Tests

### apu_test/
Tests APU functionality visible to CPU.

| Test                    | Description                 |
| ----------------------- | --------------------------- |
| `1-len_ctr.nes`         | Length counter operation    |
| `2-len_table.nes`       | Length table entries        |
| `3-irq_flag.nes`        | Frame IRQ flag              |
| `4-jitter.nes`          | APU clock jitter            |
| `5-len_timing.nes`      | Length counter clock timing |
| `6-irq_flag_timing.nes` | Frame IRQ flag timing       |
| `7-dmc_basics.nes`      | DMC channel basics          |
| `8-dmc_rates.nes`       | DMC rate table              |

### apu_reset/
Tests APU state after reset.

| Test                    | Description                       |
| ----------------------- | --------------------------------- |
| `4015_cleared.nes`      | $4015 cleared on reset            |
| `4017_timing.nes`       | $4017 timing after reset          |
| `4017_written.nes`      | $4017 write behavior              |
| `irq_flag_cleared.nes`  | IRQ flag cleared on reset         |
| `len_ctrs_enabled.nes`  | Length counters after reset       |
| `works_immediately.nes` | APU works immediately after reset |

### apu_mixer/
Tests APU mixer operation.

### dmc_tests/
DMC (Delta Modulation Channel) tests.

---

## Mapper Tests

### mmc3_test/
Tests MMC3 (mapper 4) functionality - one of the most common mappers.

| Test                    | Description                |
| ----------------------- | -------------------------- |
| `1-clocking.nes`        | IRQ counter clocking       |
| `2-details.nes`         | IRQ counter details        |
| `3-A12_clocking.nes`    | A12 line clocking behavior |
| `4-scanline_timing.nes` | Scanline IRQ timing        |
| `5-MMC3.nes`            | General MMC3 functionality |
| `6-MMC3_alt.nes`        | Alternative MMC3 behavior  |

### mmc1_a12.nes
MMC1 mapper A12 behavior test.

---

## Mixed/Video Tests

### AccuracyCoin.nes
Tests various accuracy edge cases:
- B flag behavior during BRK/IRQ/NMI
- Dummy read/write behavior
- Open bus behavior
- Interrupt timing

### 240pee.nes / 240pee-bnrom.nes
240p Test Suite for video output:
- Color bars and gradients
- Scroll tests
- Grid patterns
- MDFourier tone generator

### full_palette.nes / full_palette_smooth.nes / flowing_palette.nes
Palette display utilities.

---

## Test Output Format

Most blargg tests report results in multiple ways:

1. **Screen** - Text showing pass/fail
2. **Audio** - Beeps indicating result code
3. **Memory** - Result at $6000, text at $6004+

**Result codes:**
- `$00` = Passed
- `$01` = Failed
- `$02+` = Specific error (see readme files)
- `$80` = Test still running
- `$81` = Needs reset button

**Memory signature:** Tests write `$DE $B0 $G1` to $6001-$6003 to identify themselves.

---

## Recommended Test Order

### Phase 1: Basic CPU
1. **nestest.nes** - CPU instruction basics
2. **instr_test_v5/official_only.nes** - All official opcodes

### Phase 2: CPU Timing & Edge Cases
3. **instr_timing/** - Instruction cycle counts
4. **cpu_dummy_reads.nes** - Page crossing reads
5. **cpu_dummy_writes_*.nes** - RMW dummy writes
6. **cpu_interrupts_v2/** - Interrupt behavior

### Phase 3: Basic PPU
7. **blargg_ppu_tests/** - VRAM, palette, OAM access
8. **ppu_vbl_nmi/01-vbl_basics.nes** - Basic VBlank

### Phase 4: PPU Timing
9. **ppu_vbl_nmi/** - Full VBlank/NMI timing
10. **sprite_hit_tests/** - Sprite 0 collision
11. **sprite_overflow_tests/** - Sprite overflow

### Phase 5: APU
12. **apu_test/** - APU functionality
13. **apu_reset/** - APU reset behavior

### Phase 6: Mappers
14. **mmc3_test/** - MMC3 mapper (if implemented)

---

## Sources

- [christopherpow/nes-test-roms](https://github.com/christopherpow/nes-test-roms)
- [NESdev Wiki - Emulator Tests](https://www.nesdev.org/wiki/Emulator_tests)
- [blargg's NES test ROMs](http://blargg.8bitalley.com/nes-tests/)
