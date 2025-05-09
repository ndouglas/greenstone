//! VBlank flag and NMI generation tests.
//!
//! Tests for exact VBlank timing, NMI generation, and flag behavior.

use super::*;

// =============================================================================
// VBlank Flag Set Timing Tests
// =============================================================================

#[test]
fn test_vblank_flag_not_set_at_scanline_241_dot_0() {
    let mut ppu = create_test_ppu();

    // Advance to scanline 241, dot 0
    advance_ppu_to(&mut ppu, 241, 0);

    // VBlank flag should NOT be set yet
    assert_eq!(
        ppu.status_register.get_vertical_blank_flag(),
        false,
        "VBlank flag should not be set at scanline 241, dot 0"
    );
}

#[test]
fn test_vblank_flag_set_at_scanline_241_dot_1() {
    let mut ppu = create_test_ppu();

    // Advance to scanline 241, dot 1
    advance_ppu_to(&mut ppu, 241, 1);

    // VBlank flag should now be set
    assert_eq!(
        ppu.status_register.get_vertical_blank_flag(),
        true,
        "VBlank flag should be set at scanline 241, dot 1"
    );
}

#[test]
fn test_vblank_flag_remains_set_through_vblank() {
    let mut ppu = create_test_ppu();

    // Advance to VBlank start
    advance_ppu_to(&mut ppu, 241, 1);
    assert!(ppu.status_register.get_vertical_blank_flag());

    // Check flag remains set at various points during VBlank
    advance_ppu_to(&mut ppu, 250, 170);
    assert!(
        ppu.status_register.get_vertical_blank_flag(),
        "VBlank flag should remain set at scanline 250"
    );

    advance_ppu_to(&mut ppu, 260, 340);
    assert!(
        ppu.status_register.get_vertical_blank_flag(),
        "VBlank flag should remain set at scanline 260"
    );
}

// =============================================================================
// VBlank Flag Clear Timing Tests
// =============================================================================

#[test]
fn test_vblank_flag_not_cleared_at_scanline_261_dot_0() {
    let mut ppu = create_test_ppu();

    // Set VBlank flag first
    advance_ppu_to(&mut ppu, 241, 1);
    assert!(ppu.status_register.get_vertical_blank_flag());

    // Advance to pre-render scanline, dot 0
    advance_ppu_to(&mut ppu, 261, 0);

    // VBlank flag should still be set
    assert!(
        ppu.status_register.get_vertical_blank_flag(),
        "VBlank flag should not be cleared at scanline 261, dot 0"
    );
}

#[test]
fn test_vblank_flag_cleared_at_scanline_261_dot_1() {
    let mut ppu = create_test_ppu();

    // Set VBlank flag first
    advance_ppu_to(&mut ppu, 241, 1);
    assert!(ppu.status_register.get_vertical_blank_flag());

    // Advance to pre-render scanline, dot 1
    advance_ppu_to(&mut ppu, 261, 1);

    // VBlank flag should now be cleared
    assert_eq!(
        ppu.status_register.get_vertical_blank_flag(),
        false,
        "VBlank flag should be cleared at scanline 261, dot 1"
    );
}

// =============================================================================
// Reading PPUSTATUS Clears VBlank
// =============================================================================

#[test]
fn test_reading_status_clears_vblank_flag() {
    let mut ppu = create_test_ppu();

    // Set VBlank flag
    advance_ppu_to(&mut ppu, 241, 1);
    assert!(ppu.status_register.get_vertical_blank_flag());

    // Read status register - should return vblank set
    let status = ppu.read_status_register();
    assert!(status & 0x80 != 0, "Status read should show VBlank was set");

    // VBlank flag should now be cleared
    assert_eq!(
        ppu.status_register.get_vertical_blank_flag(),
        false,
        "VBlank flag should be cleared after reading PPUSTATUS"
    );
}

#[test]
fn test_reading_status_at_vblank_start_race_condition() {
    let mut ppu = create_test_ppu();

    // Advance to just before VBlank (scanline 241, dot 0)
    advance_ppu_to(&mut ppu, 241, 0);

    // Read status - VBlank not set yet
    let status = ppu.read_status_register();
    assert!(
        status & 0x80 == 0,
        "Status read at dot 0 should not show VBlank"
    );

    // The suppress_vblanks flag should prevent VBlank from being set
    // when we tick to dot 1
    ppu.tick();

    // Check if VBlank was suppressed (this is the race condition behavior)
    // Reading PPUSTATUS on the exact cycle VBlank would be set suppresses it
    assert_eq!(
        ppu.status_register.get_vertical_blank_flag(),
        false,
        "VBlank should be suppressed when PPUSTATUS read just before VBlank"
    );
}

// =============================================================================
// Suppress VBlank Tests
// =============================================================================

#[test]
fn test_suppress_vblanks_prevents_flag_set() {
    let mut ppu = create_test_ppu();

    // Manually set suppress_vblanks to test the behavior
    ppu.suppress_vblanks = true;

    // Advance to when VBlank would normally be set
    advance_ppu_to(&mut ppu, 241, 1);

    // VBlank flag should NOT be set due to suppression
    assert_eq!(
        ppu.status_register.get_vertical_blank_flag(),
        false,
        "VBlank should be suppressed"
    );

    // suppress_vblanks should be cleared after the check
    assert_eq!(
        ppu.suppress_vblanks, false,
        "suppress_vblanks should be cleared"
    );
}

// =============================================================================
// NMI Generation Tests
// =============================================================================

#[test]
fn test_nmi_fires_when_vblank_starts_with_nmi_enabled() {
    let mut ppu = create_test_ppu();

    // Enable NMI generation (bit 7 of PPUCTRL)
    ppu.write_control_register(0x80);

    // Verify NMI is not pending yet
    assert_eq!(ppu.nmi_pending, false);

    // Advance to VBlank start
    advance_ppu_to(&mut ppu, 241, 1);

    // NMI should now be pending
    assert!(ppu.nmi_pending, "NMI should be pending when VBlank starts with NMI enabled");
}

#[test]
fn test_nmi_does_not_fire_when_nmi_disabled() {
    let mut ppu = create_test_ppu();

    // NMI is disabled by default (bit 7 of PPUCTRL is 0)
    assert_eq!(ppu.control_register.get_generate_nmi_flag(), false);

    // Advance to VBlank start
    advance_ppu_to(&mut ppu, 241, 1);

    // NMI should NOT be pending
    assert_eq!(
        ppu.nmi_pending, false,
        "NMI should not be pending when NMI generation is disabled"
    );

    // But VBlank flag should still be set
    assert!(ppu.status_register.get_vertical_blank_flag());
}

#[test]
fn test_nmi_fires_when_enabled_during_vblank() {
    let mut ppu = create_test_ppu();

    // NMI starts disabled
    assert_eq!(ppu.control_register.get_generate_nmi_flag(), false);

    // Advance to VBlank
    advance_ppu_to(&mut ppu, 241, 1);

    // VBlank is set but no NMI
    assert!(ppu.status_register.get_vertical_blank_flag());
    assert_eq!(ppu.nmi_pending, false);

    // Now enable NMI during VBlank
    ppu.write_control_register(0x80);

    // Tick once to process the force_nmi
    ppu.tick();

    // NMI should now be pending (enabling NMI while VBlank is set triggers NMI)
    assert!(
        ppu.nmi_pending,
        "NMI should fire when enabled during active VBlank"
    );
}

#[test]
fn test_nmi_does_not_fire_when_disabled_during_vblank() {
    let mut ppu = create_test_ppu();

    // Enable NMI
    ppu.write_control_register(0x80);

    // Advance to VBlank start
    advance_ppu_to(&mut ppu, 241, 1);

    // NMI is pending
    assert!(ppu.nmi_pending);

    // Acknowledge the NMI
    ppu.acknowledge_nmi();
    assert_eq!(ppu.nmi_pending, false);

    // Disable NMI during VBlank
    ppu.write_control_register(0x00);

    // Tick a few times
    tick_ppu_n_times(&mut ppu, 10);

    // NMI should not be pending (disabling NMI prevents new ones)
    assert_eq!(
        ppu.nmi_pending, false,
        "NMI should not fire after being disabled"
    );
}

// =============================================================================
// NMI Acknowledgment Tests
// =============================================================================

#[test]
fn test_nmi_pending_cleared_on_acknowledge() {
    let mut ppu = create_test_ppu();

    // Enable NMI and trigger VBlank
    ppu.write_control_register(0x80);
    advance_ppu_to(&mut ppu, 241, 1);

    assert!(ppu.nmi_pending);

    // Acknowledge the NMI
    ppu.acknowledge_nmi();

    assert_eq!(ppu.nmi_pending, false, "NMI pending should be cleared after acknowledge");
}

#[test]
fn test_only_one_nmi_per_vblank() {
    let mut ppu = create_test_ppu();

    // Enable NMI
    ppu.write_control_register(0x80);

    // Trigger VBlank
    advance_ppu_to(&mut ppu, 241, 1);
    assert!(ppu.nmi_pending);

    // Acknowledge it
    ppu.acknowledge_nmi();
    assert_eq!(ppu.nmi_pending, false);

    // Continue through VBlank - NMI should not fire again
    advance_ppu_to(&mut ppu, 250, 0);
    assert_eq!(
        ppu.nmi_pending, false,
        "Only one NMI should occur per VBlank period"
    );

    // But next frame's VBlank should trigger a new NMI
    advance_ppu_to(&mut ppu, 241, 1);
    assert!(ppu.nmi_pending, "New VBlank should trigger new NMI");
}
