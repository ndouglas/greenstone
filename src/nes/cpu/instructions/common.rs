#[inline]
#[named]
pub fn add_u8s(augend: u8, addend: u8, carry: bool) -> (u8, bool, bool) {
  trace_enter!();
  trace_u8!(augend);
  trace_u8!(addend);
  trace_var!(carry);
  let sum = (augend as u16).wrapping_add(addend as u16).wrapping_add(carry as u16);
  trace_u16!(sum);
  let result = sum as u8;
  trace_u8!(result);
  let set_carry = sum > 0xFF;
  trace_var!(set_carry);
  let set_overflow = (addend ^ result) & (augend ^ result) & 0x80 != 0;
  trace_var!(set_overflow);
  trace_exit!();
  (result, set_carry, set_overflow)
}

#[cfg(test)]
mod test {
  use super::*;
  use crate::test::init;

  #[test]
  #[named]
  fn test_add_u8s() {
    init();
    let mut augend = 0x05;
    let mut addend = 0x02;
    let mut carry = false;
    let mut add_u8s_result = add_u8s(augend, addend, carry);
    let mut result = add_u8s_result.0;
    let mut set_carry = add_u8s_result.1;
    let mut set_overflow = add_u8s_result.2;
    assert_eq!(result, (augend.wrapping_add(addend)));
    assert!(set_carry == false, "should not have set the carry bit");
    assert!(set_overflow == false, "should not have set the overflow bit");
    augend = (-3i8).wrapping_neg().wrapping_add(1) as u8;
    addend = (-2i8).wrapping_neg().wrapping_add(1) as u8;
    carry = false;
    add_u8s_result = add_u8s(augend, addend, carry);
    result = add_u8s_result.0;
    set_carry = add_u8s_result.1;
    set_overflow = add_u8s_result.2;
    assert_eq!(result, (augend.wrapping_add(addend)));
    assert!(set_carry == false, "should not have set the carry bit");
    assert!(set_overflow == false, "should not have set the overflow bit");
    augend = (-32i8).wrapping_neg().wrapping_add(1) as u8;
    addend = 27;
    carry = false;
    add_u8s_result = add_u8s(augend, addend, carry);
    result = add_u8s_result.0;
    set_carry = add_u8s_result.1;
    set_overflow = add_u8s_result.2;
    assert_eq!(result, (augend.wrapping_add(addend)));
    assert!(set_carry == false, "should not have set the carry bit");
    assert!(set_overflow == false, "should not have set the overflow bit");
    augend = 27;
    addend = (-32i8).wrapping_neg().wrapping_add(1) as u8;
    carry = false;
    add_u8s_result = add_u8s(augend, addend, carry);
    result = add_u8s_result.0;
    set_carry = add_u8s_result.1;
    set_overflow = add_u8s_result.2;
    assert_eq!(result, (augend.wrapping_add(addend)));
    assert!(set_carry == false, "should not have set the carry bit");
    assert!(set_overflow == false, "should not have set the overflow bit");
  }

}