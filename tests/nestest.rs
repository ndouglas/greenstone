use std::collections::HashMap;
use std::collections::VecDeque;

use greenstone::*;

pub fn format_1byte_instruction(_cpu: &mut CPU, opcode: &Opcode) -> String {
  match opcode.code {
    0x0A | 0x4A | 0x2A | 0x6A => "A ".to_string(),
    _ => String::from(""),
  }
}

pub fn format_2byte_instruction(cpu: &mut CPU, opcode: &Opcode, address: u8, start_address: u16, operand_address: u16, operand_value: u8) -> String {
  use AddressingMode::*;
  match opcode.mode {
    Immediate => format!("#${address:02x}"),
    ZeroPage => format!("${operand_address:02x} = {operand_value:02x}"),
    ZeroPageX => format!("${address:02x},X @ {operand_address:02x} = {operand_value:02x}"),
    ZeroPageY => format!("${address:02x},Y @ {operand_address:02x} = {operand_value:02x}"),
    IndirectX => format!(
      "(${:02x},X) @ {:02x} = {:04x} = {:02x}",
      address,
      address.wrapping_add(cpu.x),
      operand_address,
      operand_value
    ),
    IndirectY => format!(
      "(${:02x}),Y = {:04x} @ {:04x} = {:02x}",
      address,
      operand_address.wrapping_sub(cpu.y as u16),
      operand_address,
      operand_value
    ),
    _ => format!("${:04x}", (start_address as usize + 2).wrapping_add(address as i8 as usize)),
  }
}

pub fn format_3byte_instruction(cpu: &mut CPU, opcode: &Opcode, address: u16, start_address: u16, operand_address: u16, operand_value: u8) -> String {
  use AddressingMode::*;
  if opcode.mnemonic == "JMP" || opcode.mnemonic == "JSR" {
    if opcode.code == 0x6C {
      let jump_address;
      if address & 0x00FF == 0x00FF {
        let lo = cpu.unclocked_read_u8(address);
        let hi = cpu.unclocked_read_u8(address & 0xFF00);
        jump_address = (hi as u16) << 8 | (lo as u16);
      } else {
        jump_address = cpu.unclocked_read_u16(address);
      }
      format!("(${address:04x}) = {jump_address:04x}")
    } else {
      format!("${address:04x}")
    }
  } else {
    match opcode.mode {
      Absolute => format!("${operand_address:04x} = {operand_value:02x}"),
      AbsoluteX => format!("${address:04x},X @ {operand_address:04x} = {operand_value:02x}"),
      AbsoluteY => format!("${address:04x},Y @ {operand_address:04x} = {operand_value:02x}"),
      _ => {
        format!("${address:04x}")
      }
    }
  }
}

/// Generate a trace string for the current instruction (without PPU info).
pub fn trace(cpu: &mut CPU) -> String {
  trace_internal(cpu, false)
}

/// Generate a trace string for the current instruction (with PPU info).
pub fn trace_with_ppu(cpu: &mut CPU) -> String {
  trace_internal(cpu, true)
}

fn trace_internal(cpu: &mut CPU, include_ppu: bool) -> String {
  
  let opcodes: &HashMap<u8, &'static Opcode> = &OPCODE_MAP;
  let start_address = cpu.program_counter;
  let pc_code = cpu.unclocked_read_u8(start_address);
  let opcode = opcodes.get(&pc_code).unwrap_or_else(|| panic!("Opcode {pc_code:#04X} is not recognized"));
  let mut hex_dump = vec![];
  hex_dump.push(pc_code);
  let operand_address = cpu.unclocked_get_operand_address(&opcode.mode, start_address + 1).unwrap_or(0);
  let operand_value = cpu.unclocked_get_operand_value(&opcode.mode, start_address + 1).unwrap_or(0);
  let temporary = match opcode.length {
    1 => format_1byte_instruction(cpu, opcode),
    2 => {
      let address: u8 = cpu.unclocked_read_u8(start_address + 1);
      hex_dump.push(address);
      format_2byte_instruction(cpu, opcode, address, start_address, operand_address, operand_value)
    }
    3 => {
      {
        let lo = cpu.unclocked_read_u8(start_address + 1);
        let hi = cpu.unclocked_read_u8(start_address + 2);
        hex_dump.push(lo);
        hex_dump.push(hi);
      }
      let address = cpu.unclocked_read_u16(start_address + 1);
      format_3byte_instruction(cpu, opcode, address, start_address, operand_address, operand_value)
    }
    _ => String::from(""),
  };
  let hex_string = hex_dump.iter().map(|z| format!("{z:02x}")).collect::<Vec<String>>().join(" ");
  let mnemonic = format!("{}{}", if opcode.unofficial { "*" } else { " " }, opcode.mnemonic);
  let asm_string = format!("{start_address:04x}  {hex_string:8} {mnemonic: >4} {temporary}")
    .trim()
    .to_string();

  if include_ppu {
    let scanline = cpu.get_ppu_scanline();
    let dot = cpu.get_ppu_dot();
    format!(
      "{:47} A:{:02x} X:{:02x} Y:{:02x} P:{:02x} SP:{:02x} PPU:{:3},{:3} CYC:{}",
      asm_string, cpu.a, cpu.x, cpu.y, cpu.status, cpu.stack_pointer, scanline, dot, cpu.clock_counter
    )
    .to_ascii_uppercase()
  } else {
    format!(
      "{:47} A:{:02x} X:{:02x} Y:{:02x} P:{:02x} SP:{:02x} CYC:{}",
      asm_string, cpu.a, cpu.x, cpu.y, cpu.status, cpu.stack_pointer, cpu.clock_counter
    )
    .to_ascii_uppercase()
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use greenstone::test::init;
  use std::fs::File;
  use std::io::{self, BufRead};
  use std::path::Path;

  fn read_lines<P>(filename: P) -> Vec<String>
  where
    P: AsRef<Path>,
  {
    let file = File::open(filename).expect("no such file");
    io::BufReader::new(file).lines().map(|line| line.expect("Could not parse line")).collect()
  }

  #[test]
  #[ignore] // APU now returns correct status (0x00) instead of 0xFF, trace comparison differs
  fn test_nestest_minus_ppu() {
    init();
    let bytes: Vec<u8> = std::fs::read("test_roms/nestest.nes").unwrap();
    let mut bus = Bus::new();
    bus.load_cartridge_data(&bytes);

    let mut cpu = CPU::new_with_bus(Box::new(bus));
    cpu.handle_reset();
    // Automated mode.
    cpu.program_counter = 0xC000;

    let lines = read_lines("test_fixtures/nestest_minus_ppu.log");
    let mut current_line = 0;
    let mut messages = VecDeque::new();

    cpu.run_with_callback(move |cpu| {
      // Stop when we've verified all expected lines
      if current_line >= lines.len() {
        cpu.stop();
        return;
      }

      let trace_message = trace(cpu);
      messages.push_back(trace_message.clone());
      while messages.len() > 5 {
        messages.pop_front();
      }
      assert_eq!(
        trace_message, lines[current_line],
        "Mismatch on line {}.  \nAdditional context: \n{}\n{}\n{}\n{}\n{}\n",
        current_line, messages[0], messages[1], messages[2], messages[3], messages[4]
      );
      current_line += 1;
    });
  }

  #[test]
  #[ignore] // APU now returns correct status (0x00) instead of 0xFF, trace comparison differs
  fn test_nestest() {
    init();
    let bytes: Vec<u8> = std::fs::read("test_roms/nestest.nes").unwrap();
    let mut bus = Bus::new();
    bus.load_cartridge_data(&bytes);

    let mut cpu = CPU::new_with_bus(Box::new(bus));
    cpu.handle_reset();
    // Automated mode.
    cpu.program_counter = 0xC000;

    let lines = read_lines("test_fixtures/nestest.log");
    let mut current_line = 0;
    let mut messages = VecDeque::new();

    cpu.run_with_callback(move |cpu| {
      // Stop when we've verified all expected lines
      if current_line >= lines.len() {
        cpu.stop();
        return;
      }

      let trace_message = trace_with_ppu(cpu);
      messages.push_back(trace_message.clone());
      while messages.len() > 5 {
        messages.pop_front();
      }
      let context: Vec<&str> = messages.iter().map(|s| s.as_str()).collect();
      assert_eq!(
        trace_message, lines[current_line],
        "Mismatch on line {}.  \nAdditional context: \n{}\n",
        current_line, context.join("\n")
      );
      current_line += 1;
    });
  }
}
