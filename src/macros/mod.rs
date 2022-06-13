macro_rules! trace_u8 {
  ($var: expr) => {
    #[cfg(debug_assertions)]
    if $var & 0x80 > 0 {
    trace!(
      "{} = {:#04X} {:#010b} (+: {}, ±: {})",
      stringify!($var),
      $var,
      $var,
      $var as i8,
      $var as u8
    );
  }
  else {
    trace!(
      "{} = {:#04X} {:#010b} (+/±: {})",
      stringify!($var),
      $var,
      $var,
      $var as u8
    );
  }
  };
}

macro_rules! debug_u8 {
  ($var: expr) => {
    #[cfg(debug_assertions)]
    if $var & 0x80 > 0 {
    debug!(
      "{} = {:#04X} {:#010b} (+: {}, ±: {})",
      stringify!($var),
      $var,
      $var,
      $var as i8,
      $var as u8
    );
  }
  else {
    debug!(
      "{} = {:#04X} {:#010b} (+/±: {})",
      stringify!($var),
      $var,
      $var,
      $var as u8
    );
  }
  };
}

macro_rules! info_u8 {
  ($var: expr) => {
    #[cfg(debug_assertions)]
    if $var & 0x80 > 0 {
    info!(
      "{} = {:#04X} {:#010b} (+: {}, ±: {})",
      stringify!($var),
      $var,
      $var,
      $var as i8,
      $var as u8
    );
  }
  else {
    info!(
      "{} = {:#04X} {:#010b} (+/±: {})",
      stringify!($var),
      $var,
      $var,
      $var as u8
    );
  }
  };
}

macro_rules! trace_u16 {
  ($var: expr) => {
    #[cfg(debug_assertions)]
    trace!("{} = {:#06X} {:#018b} ({})", stringify!($var), $var, $var, $var);
  };
}

macro_rules! debug_u16 {
  ($var: expr) => {
    #[cfg(debug_assertions)]
    debug!("{} = {:#06X} {:#018b} ({})", stringify!($var), $var, $var, $var);
  };
}

macro_rules! info_u16 {
  ($var: expr) => {
    #[cfg(debug_assertions)]
    info!("{} = {:#06X} {:#018b} ({})", stringify!($var), $var, $var, $var);
  };
}

macro_rules! trace_status_register {
  ($var: expr) => {
    #[cfg(debug_assertions)]
    trace!("N={}, O={}, U={}, B={}, D={}, I={}, Z={}, C={}", $var & NEGATIVE_FLAG, $var & OVERFLOW_FLAG, $var & UNUSED_FLAG, $var & BREAK_FLAG, $var & DECIMAL_FLAG, $var & INTERRUPT_DISABLE_FLAG, $var & ZERO_FLAG, $var & CARRY_FLAG);
  }
}

macro_rules! debug_status_register {
  ($var: expr) => {
    #[cfg(debug_assertions)]
    debug!("N={}, O={}, U={}, B={}, D={}, I={}, Z={}, C={}", $var & NEGATIVE_FLAG, $var & OVERFLOW_FLAG, $var & UNUSED_FLAG, $var & BREAK_FLAG, $var & DECIMAL_FLAG, $var & INTERRUPT_DISABLE_FLAG, $var & ZERO_FLAG, $var & CARRY_FLAG);
  }
}

macro_rules! info_status_register {
  ($var: expr) => {
    #[cfg(debug_assertions)]
    info!("N={}, O={}, U={}, B={}, D={}, I={}, Z={}, C={}", $var & NEGATIVE_FLAG, $var & OVERFLOW_FLAG, $var & UNUSED_FLAG, $var & BREAK_FLAG, $var & DECIMAL_FLAG, $var & INTERRUPT_DISABLE_FLAG, $var & ZERO_FLAG, $var & CARRY_FLAG);
  }
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
