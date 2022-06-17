macro_rules! format_u8 {
  ($var: expr) => {{
    #[cfg(debug_assertions)]
    if $var & 0x80 > 0 {
      format!("{:#04X} {:#010b} (+: {}, ±: {})", $var, $var, $var as u8, $var as i8)
    } else {
      format!("{:#04X} {:#010b} (+/±: {})", $var, $var, $var as u8)
    }
  }};
}

macro_rules! format_u16 {
  ($var: expr) => {{
    #[cfg(debug_assertions)]
    format!("{:#06X} {:#018b} ({})", $var, $var, $var)
  }};
}

macro_rules! trace_u8 {
  ($var: expr) => {
    trace!("{} = {}", stringify!($var), format_u8!($var));
  };
}

macro_rules! debug_u8 {
  ($var: expr) => {
    debug!("{} = {}", stringify!($var), format_u8!($var));
  };
}

macro_rules! info_u8 {
  ($var: expr) => {
    info!("{} = {}", stringify!($var), format_u8!($var));
  };
}

macro_rules! trace_u16 {
  ($var: expr) => {
    #[cfg(debug_assertions)]
    trace!("{} = {}", stringify!($var), format_u16!($var));
  };
}

macro_rules! debug_u16 {
  ($var: expr) => {
    #[cfg(debug_assertions)]
    debug!("{} = {}", stringify!($var), format_u16!($var));
  };
}

macro_rules! info_u16 {
  ($var: expr) => {
    #[cfg(debug_assertions)]
    info!("{} = {}", stringify!($var), format_u16!($var));
  };
}

macro_rules! trace_status_register {
  ($var: expr) => {
    #[cfg(debug_assertions)]
    trace!(
      "N={}, O={}, U={}, B={}, D={}, I={}, Z={}, C={}",
      $var & NEGATIVE_FLAG,
      $var & OVERFLOW_FLAG,
      $var & UNUSED_FLAG,
      $var & BREAK_FLAG,
      $var & DECIMAL_FLAG,
      $var & INTERRUPT_DISABLE_FLAG,
      $var & ZERO_FLAG,
      $var & CARRY_FLAG
    );
  };
}

macro_rules! debug_status_register {
  ($var: expr) => {
    #[cfg(debug_assertions)]
    debug!(
      "N={}, O={}, U={}, B={}, D={}, I={}, Z={}, C={}",
      $var & NEGATIVE_FLAG,
      $var & OVERFLOW_FLAG,
      $var & UNUSED_FLAG,
      $var & BREAK_FLAG,
      $var & DECIMAL_FLAG,
      $var & INTERRUPT_DISABLE_FLAG,
      $var & ZERO_FLAG,
      $var & CARRY_FLAG
    );
  };
}

macro_rules! info_status_register {
  ($var: expr) => {
    #[cfg(debug_assertions)]
    info!(
      "N={}, O={}, U={}, B={}, D={}, I={}, Z={}, C={}",
      $var & NEGATIVE_FLAG,
      $var & OVERFLOW_FLAG,
      $var & UNUSED_FLAG,
      $var & BREAK_FLAG,
      $var & DECIMAL_FLAG,
      $var & INTERRUPT_DISABLE_FLAG,
      $var & ZERO_FLAG,
      $var & CARRY_FLAG
    );
  };
}

macro_rules! trace_var {
  ($var: expr) => {
    #[cfg(debug_assertions)]
    trace!("{} = {:?}", stringify!($var), $var);
  };
}

macro_rules! debug_var {
  ($var: expr) => {
    #[cfg(debug_assertions)]
    debug!("{} = {:?}", stringify!($var), $var);
  };
}

macro_rules! info_var {
  ($var: expr) => {
    #[cfg(debug_assertions)]
    info!("{} = {:?}", stringify!($var), $var);
  };
}

macro_rules! trace_enter {
  () => {
    #[cfg(debug_assertions)]
    trace!("[ENTER] {} @ line {}", function_name!(), line!());
  };
}

macro_rules! trace_exit {
  () => {
    #[cfg(debug_assertions)]
    trace!("[EXIT] {} @ line {}", function_name!(), line!());
  };
}

macro_rules! trace_result {
  ($var: ident) => {
    #[cfg(debug_assertions)]
    trace!("[EXIT] {} @ line {} with {}: {:?}", function_name!(), line!(), stringify!($var), $var);
  };
}

macro_rules! function_path {
  () => {
    #[cfg(debug_assertions)]
    concat!(module_path!(), "::", function_name!())
  };
}

macro_rules! get_opcode {
  ($mnemonic:expr, $mode:ident) => {{
    let instruction_option = INSTRUCTION_MODE_OPCODE_MAP
      .get($mnemonic)
      .expect(&format!("Opcode {} is not recognized.", $mnemonic));
    let opcode_option = instruction_option.get(&crate::nes::AddressingMode::$mode).expect(&format!(
      "Opcode {} doesn't work with addressing mode {}.",
      $mnemonic,
      stringify!($mode)
    ));
    opcode_option.code
  }};
}

macro_rules! test_opcode {
  ($opcode:expr, [$($byte:expr),*]{$($start_key:ident : $start_value:expr),*} => [$($returned_byte:expr),*]{$($expected_key:ident : $expected_value:expr),*}) => {
    { // Begin test scope.
      let ref opcodes: HashMap<u8, &'static Opcode> = *OPCODE_MAP;
      let test_opcode = opcodes.get(&$opcode).expect(&format!("Opcode {:x} is not recognized", $opcode));
      let mut cpu = CPU::new();
      let mut program = Vec::new();
      $(program.push($byte);)*
      program.insert(0, $opcode);
      cpu.load(program);
      cpu.reset();
      $(cpu.$start_key = $start_value;)*
      let start_pc = cpu.program_counter;
      let start_cycles = cpu.cycles;
      let start_status = cpu.status;
      cpu.process_instruction();
      let status_differences = cpu.status ^ start_status;
      trace_u8!(status_differences);
      let status_mask = test_opcode.status_mask;
      trace_u8!(status_mask);
      let status_violations = status_differences & !status_mask;
      trace_u8!(status_violations);
      assert!(status_violations == 0, "Instruction violated status register mask; mask: {:#010b}, start: {:#010b}, actual: {:#010b}, differences: {:#010b}, violations: {:#010b}", status_mask, start_status, cpu.status, status_differences, status_violations);
      assert!(test_opcode.length == (cpu.program_counter - start_pc) as u8, "Invalid instruction length; expected {} bytes, found {}.", test_opcode.length, (cpu.program_counter - start_pc) as u8);
      assert!(test_opcode.cycles == (cpu.cycles - start_cycles), "Invalid instruction cycles; expected {} cycles, found {}.", test_opcode.cycles, cpu.cycles - start_cycles);
      $(
        let expected_value = $expected_value as u8;
        let expected_value_string = format_u8!(expected_value);
        let actual_value = cpu.$expected_key as u8;
        let actual_value_string = format_u8!(actual_value);
        assert!(cpu.$expected_key == $expected_value, "Unexpected register value: expected {} to be {}, found {}.", stringify!(cpu.$expected_key), expected_value_string, actual_value_string);
      )*
      let mut expected_memory = Vec::new();
      $(expected_memory.push($returned_byte);)*
      expected_memory.insert(0, $opcode);
      for (i, &byte) in expected_memory.iter().enumerate() {
        let address = i as u16;
        let expected_value_string = format_u8!(byte);
        let actual_value = cpu.read_u8(address);
        let actual_value_string = format_u8!(actual_value);
        assert!(actual_value == byte, "Unexpected memory value; Expected contents of memory at {} to be {}, got {}", format_u16!(address), expected_value_string, actual_value_string);
      }
    } // End test scope.
  }
}

macro_rules! test_instruction {
  ($instruction:expr, $mode:ident, $($args:tt)*) => {
    test_opcode!(get_opcode!($instruction, $mode), $($args)*)
  }
}
