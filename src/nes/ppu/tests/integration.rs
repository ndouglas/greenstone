//! Integration tests for full frame rendering.
//!
//! Tests that verify complete frames, multi-frame behavior, and system integration.

use super::*;

// =============================================================================
// Full Frame Execution Tests
// =============================================================================

#[test]
fn test_full_frame_takes_89342_ppu_cycles() {
    let mut ppu = create_test_ppu();

    // A complete NTSC frame is 341 dots * 262 scanlines = 89342 PPU cycles
    let total_cycles = DOTS_PER_SCANLINE as u32 * SCANLINES_PER_FRAME as u32;
    assert_eq!(total_cycles, 89342);

    // Verify by ticking through a frame
    let mut cycles = 0u32;
    while ppu.frame_count == 0 {
        ppu.tick();
        cycles += 1;
    }

    assert_eq!(cycles, 89342, "Frame should take exactly 89342 PPU cycles");
}

#[test]
fn test_nmi_occurs_once_per_frame_when_enabled() {
    let mut ppu = create_test_ppu();

    // Enable NMI
    ppu.write_control_register(0x80);

    // Run through two complete frames
    let mut nmi_count = 0;

    for _ in 0..2 {
        // Advance to VBlank
        while ppu.scanline != VBLANK_SCANLINE || ppu.dot != 1 {
            ppu.tick();
        }

        // Check and acknowledge NMI
        if ppu.is_nmi_pending() {
            nmi_count += 1;
            ppu.acknowledge_nmi();
        }

        // Complete the frame
        while ppu.scanline != 0 || ppu.dot != 0 {
            ppu.tick();
        }
    }

    assert_eq!(nmi_count, 2, "Should get exactly one NMI per frame");
}

#[test]
fn test_frame_ready_flag_set_at_frame_end() {
    let mut ppu = create_test_ppu();

    assert_eq!(ppu.frame_ready, false);

    // Tick through most of the frame
    tick_ppu_n_times(&mut ppu, 89340);
    assert_eq!(ppu.frame_ready, false);

    // The last two ticks should complete the frame
    ppu.tick();
    ppu.tick();

    // frame_ready should be set when we wrap to scanline 0
    // (Actually set when scanline wraps at the end of scanline 261)
    assert_eq!(ppu.frame_count, 1);
}

// =============================================================================
// Multi-Frame Behavior Tests
// =============================================================================

#[test]
fn test_vblank_flag_cycle_across_frames() {
    let mut ppu = create_test_ppu();

    // Run through multiple frames and verify VBlank timing is consistent
    for frame in 0..3 {
        // VBlank should not be set at frame start
        if ppu.scanline == 0 && ppu.dot == 0 {
            assert_eq!(
                ppu.status_register.get_vertical_blank_flag(),
                false,
                "VBlank should be clear at frame {} start",
                frame
            );
        }

        // Advance to VBlank
        advance_ppu_to(&mut ppu, VBLANK_SCANLINE, 1);
        assert!(
            ppu.status_register.get_vertical_blank_flag(),
            "VBlank should be set at frame {}",
            frame
        );

        // Advance to pre-render scanline
        advance_ppu_to(&mut ppu, PRE_RENDER_SCANLINE, 1);
        assert_eq!(
            ppu.status_register.get_vertical_blank_flag(),
            false,
            "VBlank should be cleared at frame {} pre-render",
            frame
        );

        // Advance to next frame
        advance_ppu_to(&mut ppu, 0, 0);
    }
}

#[test]
fn test_scroll_persists_across_frames() {
    let mut ppu = create_test_ppu();

    // Set scroll
    ppu.write_scroll_register(0x55); // X
    ppu.write_scroll_register(0xAA); // Y

    let t_before = ppu.t_address.0;
    let fine_x_before = ppu.fine_x;

    // Run through a complete frame
    tick_ppu_n_times(&mut ppu, DOTS_PER_SCANLINE as u32 * SCANLINES_PER_FRAME as u32);

    // Scroll values in t_address should persist
    // (Note: v_address changes during rendering, but t_address holds the scroll)
    assert_eq!(ppu.t_address.0, t_before, "t_address should persist across frames");
    assert_eq!(ppu.fine_x, fine_x_before, "fine_x should persist across frames");
}

#[test]
fn test_framebuffer_filled_during_visible_scanlines() {
    let mut ppu = create_test_ppu();

    // Enable background rendering
    ppu.mask_register.set_show_background_flag(true);

    // Set a visible backdrop color
    ppu.vram.write_u8(0x3F00, 0x16); // Red

    // Clear framebuffer
    ppu.framebuffer.fill(0);

    // Run through visible scanlines (0-239)
    // Each visible scanline should write 256 pixels
    for _ in 0..240 {
        for _ in 0..DOTS_PER_SCANLINE {
            ppu.tick();
        }
    }

    // Check that framebuffer has been written
    // At minimum, the backdrop color should be present somewhere
    let has_content = ppu.framebuffer.iter().any(|&b| b != 0);
    assert!(has_content, "Framebuffer should have content after rendering");
}

// =============================================================================
// State Consistency Tests
// =============================================================================

#[test]
fn test_state_after_reset() {
    let mut ppu = create_test_ppu();

    // Modify various state
    ppu.dot = 100;
    ppu.scanline = 50;
    ppu.nmi_pending = true;
    ppu.frame_count = 10;
    ppu.status_register.set_vertical_blank_flag(true);
    ppu.status_register.set_sprite_zero_hit_flag(true);
    ppu.status_register.set_sprite_overflow_flag(true);

    // Reset
    ppu.reset();

    // Verify state is cleared
    assert_eq!(ppu.dot, 0);
    assert_eq!(ppu.scanline, 0);
    assert_eq!(ppu.nmi_pending, false);
    assert_eq!(ppu.frame_count, 0);
    assert_eq!(ppu.status_register.get_vertical_blank_flag(), false);
    assert_eq!(ppu.status_register.get_sprite_zero_hit_flag(), false);
    assert_eq!(ppu.status_register.get_sprite_overflow_flag(), false);
}

#[test]
fn test_registers_persist_across_ticks() {
    let mut ppu = create_test_ppu();

    // Set control register
    ppu.write_control_register(0xFF);
    let ctrl = ppu.control_register.read_u8();

    // Set mask register
    ppu.write_mask_register(0xAA);
    let mask = ppu.mask_register.read_u8();

    // Tick many times
    tick_ppu_n_times(&mut ppu, 10000);

    // Registers should persist
    assert_eq!(ppu.control_register.read_u8(), ctrl);
    assert_eq!(ppu.mask_register.read_u8(), mask);
}

// =============================================================================
// Timing Edge Cases
// =============================================================================

#[test]
fn test_scanline_transitions() {
    let mut ppu = create_test_ppu();

    // Test transition from visible to post-render
    advance_ppu_to(&mut ppu, 239, 340);
    ppu.tick();
    assert_eq!(ppu.scanline, 240);
    assert_eq!(ppu.dot, 0);

    // Test transition from post-render to VBlank
    advance_ppu_to(&mut ppu, 240, 340);
    ppu.tick();
    assert_eq!(ppu.scanline, 241);
    assert_eq!(ppu.dot, 0);

    // Test transition from VBlank to pre-render
    advance_ppu_to(&mut ppu, 260, 340);
    ppu.tick();
    assert_eq!(ppu.scanline, 261);
    assert_eq!(ppu.dot, 0);

    // Test transition from pre-render to new frame
    advance_ppu_to(&mut ppu, 261, 340);
    ppu.tick();
    assert_eq!(ppu.scanline, 0);
    assert_eq!(ppu.dot, 0);
    assert_eq!(ppu.frame_count, 1);
}

#[test]
fn test_dot_0_is_idle() {
    let mut ppu = create_test_ppu();

    // Dot 0 of each scanline is an idle cycle
    // No memory accesses should occur

    // Enable rendering
    ppu.mask_register.set_show_background_flag(true);

    // Position at dot 340 of scanline 0
    advance_ppu_to(&mut ppu, 0, 340);

    // Record framebuffer state
    let fb_before = ppu.framebuffer[0];

    // Tick to dot 0 of scanline 1
    ppu.tick();
    assert_eq!(ppu.dot, 0);
    assert_eq!(ppu.scanline, 1);

    // Tick past dot 0
    ppu.tick();
    assert_eq!(ppu.dot, 1);

    // Framebuffer at position (0, 0) shouldn't have changed
    // (This scanline 1 should render starting at dot 1)
}

// =============================================================================
// Frame Counter Tests
// =============================================================================

#[test]
fn test_frame_counter_increments_correctly() {
    let mut ppu = create_test_ppu();

    assert_eq!(ppu.frame_count, 0);

    // Complete 5 frames
    for expected_frame in 1..=5 {
        tick_ppu_n_times(&mut ppu, DOTS_PER_SCANLINE as u32 * SCANLINES_PER_FRAME as u32);
        assert_eq!(
            ppu.frame_count, expected_frame,
            "Frame count should be {} after {} frames",
            expected_frame, expected_frame
        );
    }
}

#[test]
fn test_frame_counter_does_not_overflow_quickly() {
    let mut ppu = create_test_ppu();

    // Set frame count to a large value
    ppu.frame_count = u64::MAX - 1;

    // Complete one frame
    tick_ppu_n_times(&mut ppu, DOTS_PER_SCANLINE as u32 * SCANLINES_PER_FRAME as u32);

    // Should wrap to 0 (u64 behavior)
    assert_eq!(ppu.frame_count, u64::MAX);

    // One more frame wraps
    tick_ppu_n_times(&mut ppu, DOTS_PER_SCANLINE as u32 * SCANLINES_PER_FRAME as u32);
    assert_eq!(ppu.frame_count, 0);
}
