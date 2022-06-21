use std::collections::HashMap;

use greenstone::*;

pub fn format_1byte_instruction(_cpu: &CPU, opcode: &Opcode) -> String {
  match opcode.code {
    0x0A | 0x4A | 0x2A | 0x6A => format!("A "),
    _ => String::from(""),
  }
}

pub fn format_2byte_instruction(cpu: &CPU, opcode: &Opcode, address: u8, start_address: u16, operand_address: u16, operand_value: u8) -> String {
  use AddressingMode::*;
  match opcode.mode {
    Immediate => format!("#${:02x}", address),
    ZeroPage => format!("${:02x} = {:02x}", operand_address, operand_value),
    ZeroPageX => format!("${:02x},X @ {:02x} = {:02x}", address, operand_address, operand_value),
    ZeroPageY => format!("${:02x},Y @ {:02x} = {:02x}", address, operand_address, operand_value),
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

pub fn format_3byte_instruction(cpu: &CPU, opcode: &Opcode, address: u16, start_address: u16, operand_address: u16, operand_value: u8) -> String {
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
      format!("(${:04x}) = {:04x}", address, jump_address)
    } else {
      format!("${:04x}", address)
    }
  } else {
    match opcode.mode {
      Absolute => format!("${:04x} = {:02x}", operand_address, operand_value),
      AbsoluteX => format!("${:04x},X @ {:04x} = {:02x}", address, operand_address, operand_value),
      AbsoluteY => format!("${:04x},Y @ {:04x} = {:02x}", address, operand_address, operand_value),
      _ => {
        format!("${:04x}", address)
      }
    }
  }
}

pub fn trace(cpu: &mut CPU) -> String {
  use AddressingMode::*;
  let ref opcodes: HashMap<u8, &'static Opcode> = *OPCODE_MAP;
  let start_address = cpu.program_counter;
  let pc_code = cpu.unclocked_read_u8(start_address);
  let opcode = opcodes.get(&pc_code).expect(&format!("Opcode {:#04X} is not recognized", pc_code));
  let mut hex_dump = vec![];
  hex_dump.push(pc_code);
  let operand_address = cpu.unclocked_get_operand_address(&opcode.mode, start_address + 1).unwrap_or(0);
  let operand_value = cpu.unclocked_get_operand_value(&opcode.mode, start_address + 1).unwrap_or(0);
  let temporary = match opcode.length {
    1 => format_1byte_instruction(&cpu, &opcode),
    2 => {
      let address: u8 = cpu.unclocked_read_u8(start_address + 1);
      hex_dump.push(address);
      format_2byte_instruction(&cpu, &opcode, address, start_address, operand_address, operand_value)
    }
    3 => {
      {
        let lo = cpu.unclocked_read_u8(start_address + 1);
        let hi = cpu.unclocked_read_u8(start_address + 2);
        hex_dump.push(lo);
        hex_dump.push(hi);
      }
      let address = cpu.unclocked_read_u16(start_address + 1);
      format_3byte_instruction(&cpu, &opcode, address, start_address, operand_address, operand_value)
    }
    _ => String::from(""),
  };
  let hex_string = hex_dump.iter().map(|z| format!("{:02x}", z)).collect::<Vec<String>>().join(" ");
  let mnemonic = format!("{}{}", if opcode.unofficial { "*" } else { " " }, opcode.mnemonic);
  let asm_string = format!("{:04x}  {:8} {: >4} {}", start_address, hex_string, mnemonic, temporary)
    .trim()
    .to_string();
  format!(
    "{:47} A:{:02x} X:{:02x} Y:{:02x} P:{:02x} SP:{:02x} CYC:{}",
    asm_string, cpu.a, cpu.x, cpu.y, cpu.status, cpu.stack_pointer, cpu.clock_counter
  )
  .to_ascii_uppercase()
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
  fn test_nestest() {
    init();
    let bytes: Vec<u8> = std::fs::read("/Users/nathan/Projects/greenstone/roms/nestest.nes").unwrap();
    let mut bus = Bus::new();
    bus.load_cartridge_data(&bytes);

    let mut cpu = CPU::new_with_bus(Box::new(bus));
    cpu.handle_reset();
    // Automated mode.
    cpu.program_counter = 0xC000;

    let lines = read_lines("/Users/nathan/Projects/greenstone/test_fixtures/nestest_minus_ppu.log");
    let mut current_line = 0;
    cpu.run_with_callback(move |cpu| {
      println!("{}", trace(cpu));
      assert_eq!(trace(cpu), lines[current_line], "Mismatch on line {}", current_line);
      current_line = current_line + 1;
    });
  }
}
