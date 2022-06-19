use super::super::*;

impl CPU<'_> {
  //
  // (from https://www.masswerk.at/6502/6502_instruction_set.html#DCP)
  //
  // DEC oper + CMP oper
  //
  // M - 1 -> M, A - M
  //
  // N       Z       C       I       D       V
  // +       +       +       -       -       -
  // addressing      assembler       opc     bytes   cycles
  // zeropage        DCP oper        C7      2       5
  // zeropage,X      DCP oper,X      D7      2       6
  // absolute        DCP oper        CF      3       6
  // absolut,X       DCP oper,X      DF      3       7
  // absolut,Y       DCP oper,Y      DB      3       7
  // (indirect,X)    DCP (oper,X)    C3      2       8
  // (indirect),Y    DCP (oper),Y    D3      2       8
  //
  #[inline]
  #[named]
  pub fn instruction_dcp(&mut self, opcode: &Opcode) {
    trace_enter!();
    trace_u8!(opcode.length);
    let output = self.decrement_u8(opcode);
    trace_u8!(output);
    let a = self.a;
    trace_u8!(a);
    self.set_value_flags(a.wrapping_sub(output));
    self.set_carry_flag(a >= output);
    trace_exit!();
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use crate::test::init;

  #[test]
  #[named]
  fn test_dcp() {
    init();
    // These test cases are based on Starr Horne's `nes-rust`.
    // See https://github.com/starrhorne/nes-rust/blob/master/src/cpu_test.rs
    //
    // TODO: Add tests.
  }
}
