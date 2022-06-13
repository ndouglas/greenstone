macro_rules! trace_u8 {
  ($var: expr) => {
    #[cfg(debug_assertions)]
    if $var & 0x80 > 0 {
    trace!(
      "{} = {:#04x} {:#010b} (+: {}, ±: {})",
      stringify!($var),
      $var,
      $var,
      $var as i8,
      $var as u8
    );
  }
  else {
    trace!(
      "{} = {:#04x} {:#010b} (+/±: {})",
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
    trace!("{} = {:#06x} {:#018b} ({})", stringify!($var), $var, $var, $var);
  };
}

macro_rules! trace_status_register {
  ($var: expr) => {
    #[cfg(debug_assertions)]
    trace!("N={}, O={}, U={}, B={}, D={}, I={}, Z={}, C={}", $var & NEGATIVE_FLAG, $var & OVERFLOW_FLAG, $var & UNUSED_FLAG, $var & BREAK_FLAG, $var & DECIMAL_FLAG, $var & INTERRUPT_DISABLE_FLAG, $var & ZERO_FLAG, $var & CARRY_FLAG);
  }
}

macro_rules! trace_var {
  ($var: expr) => {
    #[cfg(debug_assertions)]
    trace!("{} = {:?}", stringify!($var), $var);
  };
}

macro_rules! function_path {
  () => {
    #[cfg(debug_assertions)]
    concat!(module_path!(), "::", function_name!())
  };
}

macro_rules! trace_enter {
  () => {
    #[cfg(debug_assertions)]
    trace!("[ENTER] {} @ {}", function_name!(), line!());
  };
}

macro_rules! trace_exit {
  () => {
    #[cfg(debug_assertions)]
    trace!("[EXIT] {} @ {}", function_name!(), line!());
  };
}

macro_rules! trace_result {
  ($var: ident) => {
    #[cfg(debug_assertions)]
    trace!("[EXIT] {} @ {} with {}: {:?}", function_name!(), line!(), stringify!($var), $var);
  };
}
