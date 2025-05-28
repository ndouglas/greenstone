//! Test runner for blargg's NES test ROMs.
//!
//! Blargg's tests write results to memory:
//! - $6000: Result code (0 = pass, 1+ = fail, $80 = still running)
//! - $6001-$6003: Signature ($DE $B0 $61 when test is active)
//! - $6004+: Text output (null-terminated)

use greenstone::*;

/// Maximum number of CPU cycles to run before timing out
const MAX_CYCLES: u64 = 10_000_000; // ~5 seconds of NES time

/// Run a blargg test ROM for a limited number of cycles and return debug info.
fn debug_blargg_test(rom_path: &str, max_cycles: u64) -> String {
  let bytes = std::fs::read(rom_path).expect("Failed to read ROM");
  let mut bus = Bus::new();
  bus.load_cartridge_data(&bytes);

  let mut cpu = CPU::new_with_bus(Box::new(bus));
  cpu.handle_reset();

  let mut cycles = 0u64;
  let mut nmi_count = 0u64;
  let mut frame_count = 0u64;
  let mut last_scanline = 0u16;
  let mut vblank_starts = 0u64;
  let mut ppuctrl_value = 0u8;
  let mut first_pc = cpu.program_counter;
  let mut last_pc = cpu.program_counter;
  let mut min_pc = cpu.program_counter;
  let mut max_pc = cpu.program_counter;
  let mut prg_ram_writes: Vec<String> = Vec::new();

  let mut trace_lines: Vec<String> = Vec::new();

  while cycles < max_cycles {
    last_pc = cpu.program_counter;
    if last_pc < min_pc {
      min_pc = last_pc;
    }
    if last_pc > max_pc {
      max_pc = last_pc;
    }

    // Capture all unique PCs visited
    let pc = cpu.program_counter;
    let scan = cpu.get_ppu_scanline();
    if !trace_lines.iter().any(|l| l.starts_with(&format!("{:04X}", pc))) && trace_lines.len() < 500 {
      let opcode = cpu.unclocked_read_u8(pc);
      trace_lines.push(format!(
        "{:04X}: {:02X} A:{:02X} X:{:02X} Y:{:02X} S:{:02X} P:{:02X} SL:{} DOT:{} CYC:{}",
        pc, opcode, cpu.a, cpu.x, cpu.y, cpu.stack_pointer, cpu.status,
        scan, cpu.get_ppu_dot(), cycles
      ));
    }

    // Track NMI invocations
    if cpu.is_nmi_ready() {
      nmi_count += 1;
    }

    let scanline = cpu.get_ppu_scanline();

    // Track VBlank starts
    if last_scanline < 241 && scanline >= 241 {
      vblank_starts += 1;
    }

    if scanline < last_scanline && last_scanline > 100 {
      // Frame wrapped
      frame_count += 1;
    }
    last_scanline = scanline;

    // Before running instruction, note $6000-600F state
    let sig_before = [
      cpu.unclocked_read_u8(0x6000),
      cpu.unclocked_read_u8(0x6001),
      cpu.unclocked_read_u8(0x6002),
      cpu.unclocked_read_u8(0x6003),
    ];

    cpu.process_instruction();
    cycles = cpu.clock_counter;

    // After instruction, check if $6000-6003 changed
    let sig_after = [
      cpu.unclocked_read_u8(0x6000),
      cpu.unclocked_read_u8(0x6001),
      cpu.unclocked_read_u8(0x6002),
      cpu.unclocked_read_u8(0x6003),
    ];
    for i in 0..4 {
      if sig_before[i] != sig_after[i] && prg_ram_writes.len() < 100 {
        prg_ram_writes.push(format!(
          "CYC:{} PC:{:04X} ${:04X}: {:02X} -> {:02X}",
          cycles, last_pc, 0x6000 + i, sig_before[i], sig_after[i]
        ));
      }
    }

    // Check if test completed (only after signature is written)
    let sig1_now = cpu.unclocked_read_u8(0x6001);
    let sig2_now = cpu.unclocked_read_u8(0x6002);
    let sig3_now = cpu.unclocked_read_u8(0x6003);
    if sig1_now == 0xDE && sig2_now == 0xB0 && sig3_now == 0x61 {
      let result_now = cpu.unclocked_read_u8(0x6000);
      if result_now < 0x80 {
        // Test completed with a result (0 = pass, 1+ = fail)
        break;
      }
    }

    // Check PPUCTRL to see if NMI is enabled (only read occasionally to avoid overhead)
    if cycles % 10000 == 0 {
      // Read the PPUCTRL via the PPU (not through the bus)
      ppuctrl_value = cpu.unclocked_read_u8(0x2000);
    }
  }

  // Check test signature and output
  let sig1 = cpu.unclocked_read_u8(0x6001);
  let sig2 = cpu.unclocked_read_u8(0x6002);
  let sig3 = cpu.unclocked_read_u8(0x6003);
  let result_code = cpu.unclocked_read_u8(0x6000);

  // Read output text from $6004
  let mut output = String::new();
  let mut addr = 0x6004u16;
  for _ in 0..1000 {
    let byte = cpu.unclocked_read_u8(addr);
    if byte == 0 {
      break;
    }
    output.push(byte as char);
    addr = addr.wrapping_add(1);
  }

  let first_str = trace_lines.iter().take(30).cloned().collect::<Vec<_>>().join("\n");
  let last_str = if trace_lines.len() > 30 {
    trace_lines.iter().skip(trace_lines.len() - 30).cloned().collect::<Vec<_>>().join("\n")
  } else {
    String::new()
  };
  let prg_ram_str = prg_ram_writes.join("\n");
  format!(
    "Cycles: {}, Frames: {}, VBlank starts: {}\nNMI count: {}, PPUCTRL: {:02X} (NMI enabled: {})\nScanline: {}, Dot: {}\nPC range: {:04X} - {:04X} (first: {:04X}, last: {:04X})\nSignature: {:02X} {:02X} {:02X} (expect DE B0 61)\nResult: {:02X}\nOutput: {}\n\n--- PRG-RAM writes ---\n{}\n\n--- First 30 instructions ---\n{}\n\n--- Last 30 instructions ---\n{}",
    cycles, frame_count, vblank_starts,
    nmi_count, ppuctrl_value, (ppuctrl_value & 0x80) != 0,
    cpu.get_ppu_scanline(), cpu.get_ppu_dot(),
    min_pc, max_pc, first_pc, last_pc,
    sig1, sig2, sig3, result_code, output, prg_ram_str, first_str, last_str
  )
}

/// Run a blargg test ROM and return the result.
/// Returns Ok(()) on pass, Err(message) on failure.
fn run_blargg_test(rom_path: &str) -> std::result::Result<(), String> {
  let bytes = std::fs::read(rom_path).map_err(|e| format!("Failed to read ROM: {}", e))?;
  let mut bus = Bus::new();
  bus.load_cartridge_data(&bytes);

  let mut cpu = CPU::new_with_bus(Box::new(bus));
  cpu.handle_reset();

  let mut cycles = 0u64;
  let mut result_code = 0x80u8;

  // Run until test completes or times out
  while cycles < MAX_CYCLES {
    cpu.process_instruction();
    cycles = cpu.clock_counter;

    // Check test signature at $6001-$6003
    let sig1 = cpu.unclocked_read_u8(0x6001);
    let sig2 = cpu.unclocked_read_u8(0x6002);
    let sig3 = cpu.unclocked_read_u8(0x6003);

    // Signature should be $DE $B0 $61
    if sig1 == 0xDE && sig2 == 0xB0 && sig3 == 0x61 {
      result_code = cpu.unclocked_read_u8(0x6000);

      // $80 = still running, $81 = needs reset
      if result_code < 0x80 {
        break;
      }
    }
  }

  // Read output text from $6004
  let mut output = String::new();
  let mut addr = 0x6004u16;
  loop {
    let byte = cpu.unclocked_read_u8(addr);
    if byte == 0 {
      break;
    }
    output.push(byte as char);
    addr = addr.wrapping_add(1);
    if addr == 0 {
      break; // Wrapped around, something is wrong
    }
  }

  if cycles >= MAX_CYCLES {
    return Err(format!("Test timed out after {} cycles. Output:\n{}", cycles, output));
  }

  if result_code == 0 {
    Ok(())
  } else {
    Err(format!("Test failed with code {}. Output:\n{}", result_code, output))
  }
}

#[cfg(test)]
mod debug_tests {
  use super::*;

  #[test]
  fn test_prg_ram_works() {
    greenstone::test::init();
    let bytes = std::fs::read("test_roms/blargg_ppu_tests/palette_ram.nes").expect("Failed to read ROM");
    let mut bus = Bus::new();
    bus.load_cartridge_data(&bytes);

    // Write test values to PRG-RAM
    bus.unclocked_write_u8(0x6000, 0x42);
    bus.unclocked_write_u8(0x6001, 0xDE);
    bus.unclocked_write_u8(0x6002, 0xB0);
    bus.unclocked_write_u8(0x6003, 0x61);

    // Read them back
    let v0 = bus.unclocked_read_u8(0x6000);
    let v1 = bus.unclocked_read_u8(0x6001);
    let v2 = bus.unclocked_read_u8(0x6002);
    let v3 = bus.unclocked_read_u8(0x6003);

    assert_eq!(v0, 0x42, "PRG-RAM at $6000 failed");
    assert_eq!(v1, 0xDE, "PRG-RAM at $6001 failed");
    assert_eq!(v2, 0xB0, "PRG-RAM at $6002 failed");
    assert_eq!(v3, 0x61, "PRG-RAM at $6003 failed");
    println!("PRG-RAM test passed!");
  }

  #[test]
  #[ignore] // Debug test - intentionally fails to show output
  fn test_debug_palette_ram() {
    greenstone::test::init();
    let result = debug_blargg_test("test_roms/blargg_ppu_tests/palette_ram.nes", 1_000_000);
    println!("{}", result);
    // This test always fails to show output
    panic!("Debug output:\n{}", result);
  }

  #[test]
  #[ignore] // Debug test - intentionally fails to show output
  fn test_debug_vbl_basics() {
    greenstone::test::init();
    // Run for 10M cycles (~5 seconds of NES time)
    let result = debug_blargg_test("test_roms/ppu_vbl_nmi/01-vbl_basics.nes", 10_000_000);
    println!("{}", result);
    // This test always fails to show output
    panic!("Debug output:\n{}", result);
  }
}

#[cfg(test)]
mod ppu_vbl_nmi {
  use super::*;

  #[test]
  fn test_01_vbl_basics() {
    greenstone::test::init();
    if let Err(e) = run_blargg_test("test_roms/ppu_vbl_nmi/01-vbl_basics.nes") {
      panic!("{}", e);
    }
  }

  #[test]
  fn test_02_vbl_set_time() {
    greenstone::test::init();
    if let Err(e) = run_blargg_test("test_roms/ppu_vbl_nmi/02-vbl_set_time.nes") {
      panic!("{}", e);
    }
  }

  #[test]
  fn test_03_vbl_clear_time() {
    greenstone::test::init();
    if let Err(e) = run_blargg_test("test_roms/ppu_vbl_nmi/03-vbl_clear_time.nes") {
      panic!("{}", e);
    }
  }

  #[test]
  fn test_04_nmi_control() {
    greenstone::test::init();
    if let Err(e) = run_blargg_test("test_roms/ppu_vbl_nmi/04-nmi_control.nes") {
      panic!("{}", e);
    }
  }

  #[test]
  #[ignore] // NMI timing off by 1 instruction for some cases
  fn test_05_nmi_timing() {
    greenstone::test::init();
    if let Err(e) = run_blargg_test("test_roms/ppu_vbl_nmi/05-nmi_timing.nes") {
      panic!("{}", e);
    }
  }

  #[test]
  fn test_06_suppression() {
    greenstone::test::init();
    if let Err(e) = run_blargg_test("test_roms/ppu_vbl_nmi/06-suppression.nes") {
      panic!("{}", e);
    }
  }
}

#[cfg(test)]
mod sprite_hit_tests {
  // Note: These tests use screen-only output (no $6000 memory format)
  // They will timeout with our test runner. Need visual inspection or
  // a screen-reading test framework.
  use super::*;

  #[test]
  #[ignore] // Screen-only output
  fn test_01_basics() {
    greenstone::test::init();
    if let Err(e) = run_blargg_test("test_roms/sprite_hit_tests/01.basics.nes") {
      panic!("{}", e);
    }
  }

  #[test]
  #[ignore] // Screen-only output
  fn test_02_alignment() {
    greenstone::test::init();
    if let Err(e) = run_blargg_test("test_roms/sprite_hit_tests/02.alignment.nes") {
      panic!("{}", e);
    }
  }

  #[test]
  #[ignore] // Screen-only output
  fn test_03_corners() {
    greenstone::test::init();
    if let Err(e) = run_blargg_test("test_roms/sprite_hit_tests/03.corners.nes") {
      panic!("{}", e);
    }
  }

  #[test]
  #[ignore] // Screen-only output
  fn test_04_flip() {
    greenstone::test::init();
    if let Err(e) = run_blargg_test("test_roms/sprite_hit_tests/04.flip.nes") {
      panic!("{}", e);
    }
  }

  #[test]
  #[ignore] // Screen-only output
  fn test_08_double_height() {
    greenstone::test::init();
    if let Err(e) = run_blargg_test("test_roms/sprite_hit_tests/08.double_height.nes") {
      panic!("{}", e);
    }
  }
}

#[cfg(test)]
mod blargg_ppu_tests {
  use super::*;

  #[test]
  #[ignore]
  fn test_palette_ram() {
    greenstone::test::init();
    if let Err(e) = run_blargg_test("test_roms/blargg_ppu_tests/palette_ram.nes") {
      panic!("{}", e);
    }
  }

  #[test]
  #[ignore] // Screen-only output
  fn test_sprite_ram() {
    greenstone::test::init();
    if let Err(e) = run_blargg_test("test_roms/blargg_ppu_tests/sprite_ram.nes") {
      panic!("{}", e);
    }
  }

  #[test]
  #[ignore]
  fn test_vram_access() {
    greenstone::test::init();
    if let Err(e) = run_blargg_test("test_roms/blargg_ppu_tests/vram_access.nes") {
      panic!("{}", e);
    }
  }

  #[test]
  #[ignore]
  fn test_vbl_clear_time() {
    greenstone::test::init();
    if let Err(e) = run_blargg_test("test_roms/blargg_ppu_tests/vbl_clear_time.nes") {
      panic!("{}", e);
    }
  }
}

#[cfg(test)]
mod cpu_tests {
  use super::*;

  #[test]
  #[ignore]
  fn test_instr_test_v5_official_only() {
    greenstone::test::init();
    if let Err(e) = run_blargg_test("test_roms/instr_test_v5/official_only.nes") {
      panic!("{}", e);
    }
  }

  #[test]
  #[ignore]
  fn test_cpu_dummy_reads() {
    greenstone::test::init();
    if let Err(e) = run_blargg_test("test_roms/cpu_dummy_reads.nes") {
      panic!("{}", e);
    }
  }
}

#[cfg(test)]
mod instr_timing {
  use super::*;

  #[test]
  #[ignore]
  fn test_1_instr_timing() {
    greenstone::test::init();
    if let Err(e) = run_blargg_test("test_roms/instr_timing/1-instr_timing.nes") {
      panic!("{}", e);
    }
  }

  #[test]
  #[ignore]
  fn test_2_branch_timing() {
    greenstone::test::init();
    if let Err(e) = run_blargg_test("test_roms/instr_timing/2-branch_timing.nes") {
      panic!("{}", e);
    }
  }
}

#[cfg(test)]
mod apu_tests {
  use super::*;

  #[test]
  fn test_1_len_ctr() {
    greenstone::test::init();
    if let Err(e) = run_blargg_test("test_roms/apu_test/1-len_ctr.nes") {
      panic!("{}", e);
    }
  }

  #[test]
  fn test_2_len_table() {
    greenstone::test::init();
    if let Err(e) = run_blargg_test("test_roms/apu_test/2-len_table.nes") {
      panic!("{}", e);
    }
  }

  #[test]
  fn test_3_irq_flag() {
    greenstone::test::init();
    if let Err(e) = run_blargg_test("test_roms/apu_test/3-irq_flag.nes") {
      panic!("{}", e);
    }
  }

  #[test]
  #[ignore] // APU jitter test - very timing sensitive
  fn test_4_jitter() {
    greenstone::test::init();
    if let Err(e) = run_blargg_test("test_roms/apu_test/4-jitter.nes") {
      panic!("{}", e);
    }
  }

  #[test]
  fn test_5_len_timing() {
    greenstone::test::init();
    if let Err(e) = run_blargg_test("test_roms/apu_test/5-len_timing.nes") {
      panic!("{}", e);
    }
  }

  #[test]
  fn test_6_irq_flag_timing() {
    greenstone::test::init();
    if let Err(e) = run_blargg_test("test_roms/apu_test/6-irq_flag_timing.nes") {
      panic!("{}", e);
    }
  }

  #[test]
  #[ignore] // DMC not fully implemented
  fn test_7_dmc_basics() {
    greenstone::test::init();
    if let Err(e) = run_blargg_test("test_roms/apu_test/7-dmc_basics.nes") {
      panic!("{}", e);
    }
  }

  #[test]
  #[ignore] // DMC not fully implemented
  fn test_8_dmc_rates() {
    greenstone::test::init();
    if let Err(e) = run_blargg_test("test_roms/apu_test/8-dmc_rates.nes") {
      panic!("{}", e);
    }
  }
}

#[cfg(test)]
mod apu_reset_tests {
  use super::*;

  // These tests require hardware reset functionality during test execution.
  // They display "Press RESET" and wait for a reset signal.
  // Ignored until reset mechanism is implemented.

  #[test]
  #[ignore = "Requires hardware reset support"]
  fn test_4015_cleared() {
    greenstone::test::init();
    if let Err(e) = run_blargg_test("test_roms/apu_reset/4015_cleared.nes") {
      panic!("{}", e);
    }
  }

  #[test]
  #[ignore = "Requires hardware reset support"]
  fn test_4017_timing() {
    greenstone::test::init();
    if let Err(e) = run_blargg_test("test_roms/apu_reset/4017_timing.nes") {
      panic!("{}", e);
    }
  }

  #[test]
  #[ignore = "Requires hardware reset support"]
  fn test_4017_written() {
    greenstone::test::init();
    if let Err(e) = run_blargg_test("test_roms/apu_reset/4017_written.nes") {
      panic!("{}", e);
    }
  }

  #[test]
  #[ignore = "Requires hardware reset support"]
  fn test_irq_flag_cleared() {
    greenstone::test::init();
    if let Err(e) = run_blargg_test("test_roms/apu_reset/irq_flag_cleared.nes") {
      panic!("{}", e);
    }
  }

  #[test]
  #[ignore = "Requires hardware reset support"]
  fn test_len_ctrs_enabled() {
    greenstone::test::init();
    if let Err(e) = run_blargg_test("test_roms/apu_reset/len_ctrs_enabled.nes") {
      panic!("{}", e);
    }
  }

  #[test]
  #[ignore = "Requires hardware reset support"]
  fn test_works_immediately() {
    greenstone::test::init();
    if let Err(e) = run_blargg_test("test_roms/apu_reset/works_immediately.nes") {
      panic!("{}", e);
    }
  }
}

#[cfg(test)]
mod apu_mixer_tests {
  use super::*;

  // These tests use DMC DAC to generate inverse waveforms to cancel sound.
  // They require DMC functionality to work properly.

  #[test]
  #[ignore = "Requires DMC for inverse waveform cancellation"]
  fn test_square() {
    greenstone::test::init();
    if let Err(e) = run_blargg_test("test_roms/apu_mixer/square.nes") {
      panic!("{}", e);
    }
  }

  #[test]
  #[ignore = "Requires DMC for inverse waveform cancellation"]
  fn test_triangle() {
    greenstone::test::init();
    if let Err(e) = run_blargg_test("test_roms/apu_mixer/triangle.nes") {
      panic!("{}", e);
    }
  }

  #[test]
  #[ignore = "Requires DMC for inverse waveform cancellation"]
  fn test_noise() {
    greenstone::test::init();
    if let Err(e) = run_blargg_test("test_roms/apu_mixer/noise.nes") {
      panic!("{}", e);
    }
  }

  #[test]
  #[ignore = "DMC not fully implemented"]
  fn test_dmc() {
    greenstone::test::init();
    if let Err(e) = run_blargg_test("test_roms/apu_mixer/dmc.nes") {
      panic!("{}", e);
    }
  }
}
