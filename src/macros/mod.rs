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

macro_rules! format_status_register {
  ($var: expr) => {
    format!(
      "N={}, O={}, U={}, B={}, D={}, I={}, Z={}, C={}",
      ($var & NEGATIVE_FLAG) > 0,
      ($var & OVERFLOW_FLAG) > 0,
      ($var & UNUSED_FLAG) > 0,
      ($var & BREAK_FLAG) > 0,
      ($var & DECIMAL_FLAG) > 0,
      ($var & INTERRUPT_DISABLE_FLAG) > 0,
      ($var & ZERO_FLAG) > 0,
      ($var & CARRY_FLAG) > 0
    )
  };
}

macro_rules! trace_u8 {
  ($var: expr) => {
    #[cfg(debug_assertions)]
    trace!("{} = {}", stringify!($var), format_u8!($var));
  };
}

macro_rules! debug_u8 {
  ($var: expr) => {
    #[cfg(debug_assertions)]
    debug!("{} = {}", stringify!($var), format_u8!($var));
  };
}

macro_rules! info_u8 {
  ($var: expr) => {
    #[cfg(debug_assertions)]
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
    trace!("Status: {}", format_status_register!($var));
  };
}

macro_rules! debug_status_register {
  ($var: expr) => {
    #[cfg(debug_assertions)]
    debug!("Status: {}", format_status_register!($var));
  };
}

macro_rules! info_status_register {
  ($var: expr) => {
    #[cfg(debug_assertions)]
    info!("Status: {}", format_status_register!($var));
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

macro_rules! some_or_none {
  () => {
    None
  };
  ($expression:expr) => {
    Some($expression)
  };
}

macro_rules! test_opcode {
  ($opcode:expr, [$($byte:expr),*]{$($start_key:ident : $start_value:expr),*} => [$($returned_byte:expr),*]{$($expected_key:ident : $expected_value:expr),*} $(, $builder:expr)?) => {
    { // Begin test scope.
      info!("\n\n------------------ Starting test! ------------------\n");
      let ref opcodes: HashMap<u8, &'static Opcode> = *OPCODE_MAP;
      let test_opcode = opcodes.get(&$opcode).expect(&format!("Opcode {:#04X} is not recognized", $opcode));
      trace_var!(test_opcode);
      let mut cpu = CPU::new();
      let mut program = Vec::new();
      $(program.push($byte);)*
      program.insert(0, $opcode);
      trace_var!(program);
      cpu.load(program);
      cpu.reset();
      $(cpu.$start_key = $start_value;)*
      let start_cc = cpu.clock_counter;
      trace_var!(start_cc);
      let start_pc = cpu.program_counter;
      trace_var!(start_pc);
      let start_status = cpu.status;
      trace_var!(start_status);
      $(let builder:Option<fn (&mut CPU<'_>, &Opcode)> = some_or_none!($builder);
      if let Some(closure) = builder {
        closure(&mut cpu, &test_opcode);
      })*
      cpu.process_instruction();
      let status_differences = cpu.status ^ start_status;
      trace_u8!(status_differences);
      let status_mask = test_opcode.status_mask;
      trace_u8!(status_mask);
      let status_violations = status_differences & !status_mask;
      trace_u8!(status_violations);
      assert!(status_violations == 0, "Opcode ({}) violated status register mask; mask: {:#010b}, start: {:#010b}, actual: {:#010b}, differences: {:#010b}, violations: {:#010b}", test_opcode, status_mask, start_status, cpu.status, status_differences, status_violations);
      #[allow(unused_assignments)]
      let mut expected_cycles = 0;
      expected_cycles = test_opcode.cycles;
      $(
        let expected_value: u64 = $expected_value;
        match stringify!($expected_key) {
          "clock_counter" => {
            trace!("Updating expected cycle count to {}", expected_value);
            expected_cycles = expected_value as u8;
          },
          "program_counter" => {
            let expected_value_string = format_u16!(expected_value as u16);
            let actual_value = cpu.$expected_key as u16;
            let actual_value_string = format_u16!(actual_value);
            assert!(cpu.$expected_key == $expected_value, "Unexpected opcode ({}) program counter value: expected {} to be {}, found {}.", test_opcode, stringify!(cpu.$expected_key), expected_value_string, actual_value_string);
          },
          _ => {
            let expected_value_string = format_u8!(expected_value as u8);
            let actual_value = cpu.$expected_key as u8;
            let actual_value_string = format_u8!(actual_value);
            assert!(cpu.$expected_key as u8 == expected_value as u8, "Unexpected opcode ({}) register value: expected {} to be {}, found {}.", test_opcode, stringify!(cpu.$expected_key), expected_value_string, actual_value_string);
          },
        }
      )*
      let actual_cycles = (cpu.clock_counter - start_cc) as u8;
      assert!(expected_cycles == actual_cycles, "Invalid opcode ({}) cycles; expected {} cycles, found {}.", test_opcode, expected_cycles, actual_cycles);
      let mut expected_memory = Vec::new();
      $(expected_memory.push($returned_byte);)*
      expected_memory.insert(0, $opcode);
      for (i, &byte) in expected_memory.iter().enumerate() {
        let address = i as u16;
        let expected_value_string = format_u8!(byte);
        let actual_value = cpu.unclocked_read_u8(address);
        let actual_value_string = format_u8!(actual_value);
        assert!(actual_value == byte, "Unexpected opcode ({}) memory value; Expected contents of memory at {} to be {}, got {}", test_opcode, format_u16!(address), expected_value_string, actual_value_string);
      }
    } // End test scope.
  }
}

macro_rules! test_instruction {
  ($instruction:expr, $mode:ident, $($args:tt)*) => {
    test_opcode!(get_opcode!($instruction, $mode), $($args)*)
  }
}
