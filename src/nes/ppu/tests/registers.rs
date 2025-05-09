//! PPU register behavior tests.
//!
//! Tests for status register side effects, scroll/address latching,
//! and flag timing during rendering.

use super::*;

// =============================================================================
// Status Register Side Effects
// =============================================================================

#[test]
fn test_status_read_resets_address_latch() {
    let mut ppu = create_test_ppu();

    // Write first byte to address register
    ppu.write_address_register(0x21);
    assert!(ppu.is_latched, "Latch should be set after first write");

    // Read status register
    ppu.read_status_register();

    // Latch should be reset
    assert_eq!(
        ppu.is_latched, false,
        "Reading PPUSTATUS should reset the address latch"
    );
}

#[test]
fn test_status_read_returns_open_bus_in_lower_5_bits() {
    let mut ppu = create_test_ppu();

    // Set the latching bus to a known value by writing to some register
    ppu.write_register(CONTROL_REGISTER_INDEX, 0b0001_1111);

    // Set status register to have specific upper bits
    ppu.status_register.set_vertical_blank_flag(true);
    ppu.status_register.set_sprite_zero_hit_flag(true);
    ppu.status_register.set_sprite_overflow_flag(false);

    // Read status - upper 3 bits from status, lower 5 from open bus
    let status = ppu.read_status_register();

    // Upper bits: VBlank=1, Sprite0=1, Overflow=0 = 0b110
    // Lower 5 bits from bus: 0b11111
    assert_eq!(
        status & 0b1110_0000,
        0b1100_0000,
        "Upper 3 bits should be from status register"
    );
    assert_eq!(
        status & 0b0001_1111,
        0b0001_1111,
        "Lower 5 bits should be from open bus"
    );
}

// =============================================================================
// Flag Timing During Pre-Render
// =============================================================================

#[test]
fn test_sprite_zero_hit_cleared_at_prerender() {
    let mut ppu = create_test_ppu();

    // Set sprite zero hit flag
    ppu.status_register.set_sprite_zero_hit_flag(true);
    assert!(ppu.status_register.get_sprite_zero_hit_flag());

    // Advance to pre-render scanline, dot 1
    advance_ppu_to(&mut ppu, PRE_RENDER_SCANLINE, 1);

    // Flag should be cleared
    assert_eq!(
        ppu.status_register.get_sprite_zero_hit_flag(),
        false,
        "Sprite zero hit should be cleared at pre-render scanline dot 1"
    );
}

#[test]
fn test_sprite_overflow_cleared_at_prerender() {
    let mut ppu = create_test_ppu();

    // Set sprite overflow flag
    ppu.status_register.set_sprite_overflow_flag(true);
    assert!(ppu.status_register.get_sprite_overflow_flag());

    // Advance to pre-render scanline, dot 1
    advance_ppu_to(&mut ppu, PRE_RENDER_SCANLINE, 1);

    // Flag should be cleared
    assert_eq!(
        ppu.status_register.get_sprite_overflow_flag(),
        false,
        "Sprite overflow should be cleared at pre-render scanline dot 1"
    );
}

// =============================================================================
// Scroll/Address Two-Write Latch Behavior
// =============================================================================

#[test]
fn test_first_scroll_write_sets_x() {
    let mut ppu = create_test_ppu();

    // First write sets X scroll
    ppu.write_scroll_register(0b10101_010);

    // Check fine_x and coarse_x
    assert_eq!(ppu.fine_x, 0b010, "Fine X should be lower 3 bits");
    assert_eq!(
        ppu.t_address.coarse_x(),
        0b10101,
        "Coarse X should be upper 5 bits"
    );

    // Latch should now be set
    assert!(ppu.is_latched);
}

#[test]
fn test_second_scroll_write_sets_y() {
    let mut ppu = create_test_ppu();

    // First write (X)
    ppu.write_scroll_register(0x00);
    assert!(ppu.is_latched);

    // Second write sets Y scroll
    ppu.write_scroll_register(0b01010_101);

    // Check fine_y and coarse_y
    assert_eq!(ppu.t_address.fine_y(), 0b101, "Fine Y should be lower 3 bits");
    assert_eq!(
        ppu.t_address.coarse_y(),
        0b01010,
        "Coarse Y should be upper 5 bits"
    );

    // Latch should be reset
    assert_eq!(ppu.is_latched, false);
}

#[test]
fn test_status_read_resets_scroll_latch() {
    let mut ppu = create_test_ppu();

    // First scroll write
    ppu.write_scroll_register(0x10);
    assert!(ppu.is_latched);

    // Read status resets latch
    ppu.read_status_register();
    assert_eq!(ppu.is_latched, false);

    // Next write should be treated as first write (X)
    ppu.write_scroll_register(0b11111_000);
    assert_eq!(ppu.fine_x, 0b000);
    assert_eq!(ppu.t_address.coarse_x(), 0b11111);
}

#[test]
fn test_first_address_write_sets_high_byte() {
    let mut ppu = create_test_ppu();

    // First write sets high byte of t_address
    ppu.write_address_register(0b11_101010);

    // High byte should be set (bit 14 is cleared)
    assert_eq!(ppu.t_address.high_byte(), 0b00_101010);

    // v_address should NOT be affected yet
    assert_ne!(ppu.v_address.address(), ppu.t_address.address());

    assert!(ppu.is_latched);
}

#[test]
fn test_second_address_write_sets_low_byte_and_copies_to_v() {
    let mut ppu = create_test_ppu();

    // First write (high byte)
    ppu.write_address_register(0x21);

    // Second write (low byte)
    ppu.write_address_register(0x05);

    // t_address should be 0x2105
    assert_eq!(ppu.t_address.address(), 0x2105);

    // v_address should now equal t_address
    assert_eq!(ppu.v_address.address(), 0x2105);

    assert_eq!(ppu.is_latched, false);
}

#[test]
fn test_status_read_resets_address_latch_mid_write() {
    let mut ppu = create_test_ppu();

    // First address write
    ppu.write_address_register(0x21);
    assert!(ppu.is_latched);

    // Read status resets latch
    ppu.read_status_register();
    assert_eq!(ppu.is_latched, false);

    // Now write address again - should be treated as first write (high byte)
    ppu.write_address_register(0x23);
    assert!(ppu.is_latched);

    // And second write (low byte)
    ppu.write_address_register(0x05);
    assert_eq!(ppu.v_address.address(), 0x2305);
}

// =============================================================================
// t_address vs v_address Behavior
// =============================================================================

#[test]
fn test_scroll_writes_only_affect_t_address() {
    let mut ppu = create_test_ppu();

    // Set v_address to a known value
    ppu.v_address.0 = 0x1234;

    // Write scroll
    ppu.write_scroll_register(0xFF); // X
    ppu.write_scroll_register(0xFF); // Y

    // v_address should be unchanged
    assert_eq!(ppu.v_address.0, 0x1234, "Scroll writes should not affect v_address");

    // t_address should have scroll values
    assert_ne!(ppu.t_address.0, 0x1234);
}

#[test]
fn test_address_second_write_copies_t_to_v() {
    let mut ppu = create_test_ppu();

    // Set up t_address with scroll writes
    ppu.write_scroll_register(0x00);
    ppu.write_scroll_register(0x00);

    // Modify t_address through address writes
    ppu.write_address_register(0x23);
    let t_before_second_write = ppu.t_address.0;

    // v should not match t yet
    assert_ne!(ppu.v_address.0, t_before_second_write);

    // Second write copies t to v
    ppu.write_address_register(0x05);

    // Now v should equal t
    assert_eq!(ppu.v_address.0, ppu.t_address.0);
}

// =============================================================================
// PPUDATA Read/Write Tests
// =============================================================================

#[test]
fn test_data_write_increments_v_address_by_1() {
    let mut ppu = create_test_ppu();

    // Set increment mode to +1 (bit 2 of PPUCTRL = 0)
    ppu.write_control_register(0x00);

    // Set address
    ppu.write_address_register(0x20);
    ppu.write_address_register(0x00);
    assert_eq!(ppu.v_address.address(), 0x2000);

    // Write data
    ppu.write_data_register(0x55);

    // v_address should increment by 1
    assert_eq!(ppu.v_address.address(), 0x2001);
}

#[test]
fn test_data_write_increments_v_address_by_32() {
    let mut ppu = create_test_ppu();

    // Set increment mode to +32 (bit 2 of PPUCTRL = 1)
    ppu.write_control_register(0x04);

    // Set address
    ppu.write_address_register(0x20);
    ppu.write_address_register(0x00);

    // Write data
    ppu.write_data_register(0x55);

    // v_address should increment by 32
    assert_eq!(ppu.v_address.address(), 0x2020);
}

#[test]
fn test_data_read_increments_v_address() {
    let mut ppu = create_test_ppu();

    // Set increment mode to +1
    ppu.write_control_register(0x00);

    // Set address
    ppu.write_address_register(0x20);
    ppu.write_address_register(0x00);

    // Read data (dummy read)
    ppu.read_data_register();

    // v_address should increment by 1
    assert_eq!(ppu.v_address.address(), 0x2001);

    // Read again
    ppu.read_data_register();
    assert_eq!(ppu.v_address.address(), 0x2002);
}
