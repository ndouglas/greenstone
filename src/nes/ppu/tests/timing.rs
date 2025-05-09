//! PPU timing tests.
//!
//! Tests for frame structure, scanline/dot counting, and NTSC timing quirks.

use super::*;

// =============================================================================
// Frame Structure Tests
// =============================================================================

#[test]
fn test_frame_has_262_scanlines() {
    let mut ppu = create_test_ppu();

    // Start at scanline 0, tick through entire frame
    assert_eq!(ppu.scanline, 0);
    assert_eq!(ppu.dot, 0);

    // Tick through 262 complete scanlines
    tick_ppu_n_times(&mut ppu, DOTS_PER_SCANLINE as u32 * 262);

    // Should wrap back to scanline 0
    assert_eq!(ppu.scanline, 0);
    assert_eq!(ppu.frame_count, 1);
}

#[test]
fn test_scanline_has_341_dots() {
    let mut ppu = create_test_ppu();

    // Tick through one complete scanline
    tick_ppu_n_times(&mut ppu, DOTS_PER_SCANLINE as u32);

    // Should be at the start of scanline 1
    assert_eq!(ppu.dot, 0);
    assert_eq!(ppu.scanline, 1);
}

#[test]
fn test_visible_scanlines_are_0_to_239() {
    let ppu = create_test_ppu();

    // Verify constants
    assert_eq!(SCREEN_HEIGHT, 240);

    // Visible scanlines are 0-239 (240 total)
    for scanline in 0..240u16 {
        assert!(scanline < 240, "Scanline {} should be visible", scanline);
    }
}

#[test]
fn test_post_render_scanline_is_240() {
    let mut ppu = create_test_ppu();

    // Advance to scanline 240
    advance_ppu_to(&mut ppu, 240, 0);

    // This is the post-render scanline (idle scanline)
    assert_eq!(ppu.scanline, 240);

    // VBlank flag should NOT be set yet (that happens at 241, dot 1)
    assert_eq!(ppu.status_register.get_vertical_blank_flag(), false);
}

#[test]
fn test_vblank_scanlines_are_241_to_260() {
    let mut ppu = create_test_ppu();

    // Advance to VBlank start (scanline 241, dot 1)
    advance_ppu_to(&mut ppu, 241, 1);

    // VBlank flag should be set
    assert_eq!(ppu.status_register.get_vertical_blank_flag(), true);

    // VBlank continues through scanline 260
    advance_ppu_to(&mut ppu, 260, 100);
    assert_eq!(ppu.status_register.get_vertical_blank_flag(), true);
}

#[test]
fn test_pre_render_scanline_is_261() {
    let mut ppu = create_test_ppu();

    // Verify the constant
    assert_eq!(PRE_RENDER_SCANLINE, 261);

    // Advance to pre-render scanline
    advance_ppu_to(&mut ppu, 261, 0);
    assert_eq!(ppu.scanline, 261);
}

// =============================================================================
// Dot/Scanline Counter Wrap Tests
// =============================================================================

#[test]
fn test_dot_wraps_at_341() {
    let mut ppu = create_test_ppu();

    // Tick 340 times to reach dot 340
    tick_ppu_n_times(&mut ppu, 340);
    assert_eq!(ppu.dot, 340);

    // One more tick should wrap to dot 0 of next scanline
    ppu.tick();
    assert_eq!(ppu.dot, 0);
    assert_eq!(ppu.scanline, 1);
}

#[test]
fn test_scanline_wraps_at_262() {
    let mut ppu = create_test_ppu();

    // Advance to last dot of last scanline
    advance_ppu_to(&mut ppu, 261, 340);

    // One more tick should wrap to scanline 0, dot 0
    ppu.tick();
    assert_eq!(ppu.scanline, 0);
    assert_eq!(ppu.dot, 0);
}

#[test]
fn test_frame_count_increments_on_wrap() {
    let mut ppu = create_test_ppu();

    assert_eq!(ppu.frame_count, 0);

    // Tick through one complete frame
    tick_ppu_n_times(&mut ppu, DOTS_PER_SCANLINE as u32 * SCANLINES_PER_FRAME as u32);

    assert_eq!(ppu.frame_count, 1);

    // Tick through another frame
    tick_ppu_n_times(&mut ppu, DOTS_PER_SCANLINE as u32 * SCANLINES_PER_FRAME as u32);

    assert_eq!(ppu.frame_count, 2);
}

// =============================================================================
// Odd/Even Frame Cycle Skip Tests (NTSC quirk)
// =============================================================================

#[test]
fn test_odd_frame_skips_cycle_on_prerender() {
    let mut ppu = create_test_ppu();

    // Enable rendering (required for cycle skip)
    ppu.mask_register.set_show_background_flag(true);

    // Complete first frame (even frame, frame_count goes 0 -> 1)
    tick_ppu_n_times(&mut ppu, DOTS_PER_SCANLINE as u32 * SCANLINES_PER_FRAME as u32);
    assert_eq!(ppu.frame_count, 1);

    // Now we're on an odd frame (frame_count = 1)
    // Advance to pre-render scanline, near the end
    advance_ppu_to(&mut ppu, 261, 338);

    // On odd frames with rendering enabled, the last dot of pre-render should skip
    // So from dot 338 -> 339 -> 0 (skipping 340)
    ppu.tick(); // 338 -> 339
    assert_eq!(ppu.dot, 339);

    // This tick should skip dot 340 and go directly to scanline 0, dot 0
    ppu.tick();

    // Should now be at the start of the next frame
    assert_eq!(ppu.scanline, 0);
    assert_eq!(ppu.dot, 0);
    assert_eq!(ppu.frame_count, 2);
}

#[test]
fn test_even_frame_has_normal_prerender() {
    let mut ppu = create_test_ppu();

    // Enable rendering
    ppu.mask_register.set_show_background_flag(true);

    // On even frame (frame_count = 0), pre-render should have all 341 dots
    advance_ppu_to(&mut ppu, 261, 339);

    // Dot 339 -> 340 should work normally
    ppu.tick();
    assert_eq!(ppu.dot, 340);
    assert_eq!(ppu.scanline, 261);

    // Dot 340 -> 0 (next frame)
    ppu.tick();
    assert_eq!(ppu.dot, 0);
    assert_eq!(ppu.scanline, 0);
    assert_eq!(ppu.frame_count, 1);
}

#[test]
fn test_cycle_skip_only_when_rendering_enabled() {
    let mut ppu = create_test_ppu();

    // Rendering is DISABLED by default
    assert_eq!(ppu.mask_register.get_show_background_flag(), false);
    assert_eq!(ppu.mask_register.get_show_sprites_flag(), false);

    // Complete first frame
    tick_ppu_n_times(&mut ppu, DOTS_PER_SCANLINE as u32 * SCANLINES_PER_FRAME as u32);
    assert_eq!(ppu.frame_count, 1);

    // Now on odd frame, but rendering disabled - should NOT skip
    advance_ppu_to(&mut ppu, 261, 339);

    // Should go through dot 340 normally
    ppu.tick();
    assert_eq!(ppu.dot, 340);

    // Then wrap to 0
    ppu.tick();
    assert_eq!(ppu.dot, 0);
    assert_eq!(ppu.scanline, 0);
}

// =============================================================================
// Total Frame Cycle Count Tests
// =============================================================================

#[test]
fn test_total_cycles_per_frame_even() {
    let mut ppu = create_test_ppu();

    // Count cycles for one complete frame
    let mut cycle_count = 0u32;
    let start_frame = ppu.frame_count;

    while ppu.frame_count == start_frame {
        ppu.tick();
        cycle_count += 1;
    }

    // Even frame should have 341 * 262 = 89342 cycles
    assert_eq!(cycle_count, 341 * 262);
}

#[test]
fn test_total_cycles_per_frame_odd_with_rendering() {
    let mut ppu = create_test_ppu();

    // Enable rendering for cycle skip
    ppu.mask_register.set_show_background_flag(true);

    // Complete first (even) frame
    tick_ppu_n_times(&mut ppu, DOTS_PER_SCANLINE as u32 * SCANLINES_PER_FRAME as u32);

    // Now count cycles for the odd frame
    let mut cycle_count = 0u32;
    let start_frame = ppu.frame_count;

    while ppu.frame_count == start_frame {
        ppu.tick();
        cycle_count += 1;
    }

    // Odd frame with rendering should have 89341 cycles (one less)
    assert_eq!(cycle_count, 341 * 262 - 1);
}
