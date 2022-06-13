use super::super::*;

#[inline]
pub fn add_u8s(augend: u8, addend: u8, carry: bool) -> (u8, bool, bool) {
  trace!("augend: {:X} ({} or {})", augend, augend, augend as i8);
  trace!("addend: {:X} ({} or {})", addend, addend, addend as i8);
  trace!("carry: {}", carry);
  let sum = (augend as u16) + (addend as u16) + (carry as u16);
  trace!("sum: {:X} ({} or {})", sum, sum, sum as i16);
  let result = sum as u8;
  trace!("result: {} or {}", result, result as i8);
  let set_carry = sum > 0xFF;
  trace!("set_carry: {}", set_carry);
  let set_overflow = (addend ^ result) & (augend ^ result) & 0x80 != 0;
  trace!("set_overflow: {}", set_overflow);
  (result, set_carry, set_overflow)
}

#[cfg(test)]
mod test {
  use super::*;

  #[test]
  fn test_add_u8s() {
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
    augend = (-3i8).wrapping_neg().wrapping_sub(-1) as u8;
    addend = (-2i8).wrapping_neg().wrapping_sub(-1) as u8;
    carry = false;
    add_u8s_result = add_u8s(augend, addend, carry);
    result = add_u8s_result.0;
    set_carry = add_u8s_result.1;
    set_overflow = add_u8s_result.2;
    assert_eq!(result, (augend.wrapping_add(addend)));
    assert!(set_carry == false, "should not have set the carry bit");
    assert!(set_overflow == false, "should not have set the overflow bit");
    augend = (-32i8).wrapping_neg().wrapping_sub(-1) as u8;
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
    addend = (-32i8).wrapping_neg().wrapping_sub(-1) as u8;
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