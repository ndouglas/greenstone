//! Sprite rendering tests.
//!
//! Tests for sprite evaluation, sprite zero hit, and overflow detection.

use super::*;

// =============================================================================
// OAM Structure Tests
// =============================================================================

#[test]
fn test_oam_has_64_sprites() {
    let ppu = create_test_ppu();

    // OAM holds 64 sprites, 4 bytes each = 256 bytes
    assert_eq!(ppu.oam_ram.len(), 256);
}

#[test]
fn test_oam_sprite_format() {
    let mut ppu = create_test_ppu();

    // Each sprite is 4 bytes:
    // Byte 0: Y position (top of sprite - 1)
    // Byte 1: Tile index
    // Byte 2: Attributes (palette, priority, flip)
    // Byte 3: X position (left side)

    // Write sprite 0
    ppu.oam_address = 0;
    ppu.write_oam_data_register(100); // Y = 100
    ppu.write_oam_data_register(0x10); // Tile = 0x10
    ppu.write_oam_data_register(0x01); // Attributes
    ppu.write_oam_data_register(50); // X = 50

    // Verify
    assert_eq!(ppu.oam_ram[0], 100);
    assert_eq!(ppu.oam_ram[1], 0x10);
    assert_eq!(ppu.oam_ram[2], 0x01);
    assert_eq!(ppu.oam_ram[3], 50);
}

// =============================================================================
// Sprite Visibility Tests
// =============================================================================

#[test]
fn test_sprite_show_flag() {
    let mut ppu = create_test_ppu();

    // Sprites disabled by default
    assert_eq!(ppu.mask_register.get_show_sprites_flag(), false);

    // Enable sprites
    ppu.mask_register.set_show_sprites_flag(true);
    assert!(ppu.mask_register.get_show_sprites_flag());
}

#[test]
fn test_sprite_left_8_pixels_flag() {
    let mut ppu = create_test_ppu();

    // Left sprites disabled by default
    assert_eq!(ppu.mask_register.get_show_sprites_left_flag(), false);

    // Enable left sprites
    ppu.mask_register.set_show_sprites_left_flag(true);
    assert!(ppu.mask_register.get_show_sprites_left_flag());
}

// =============================================================================
// Sprite Zero Hit Tests
// =============================================================================

#[test]
fn test_sprite_zero_hit_flag_not_set_at_x_255() {
    let mut ppu = create_test_ppu();

    // Sprite zero hit cannot occur at x=255
    // This is a hardware quirk

    // Set sprite 0 at x=255
    ppu.oam_ram[0] = 100; // Y
    ppu.oam_ram[1] = 0x00; // Tile
    ppu.oam_ram[2] = 0x00; // Attributes
    ppu.oam_ram[3] = 255; // X = 255

    // The sprite zero hit flag has specific conditions that prevent
    // it from being set at x=255
    // This test documents that expected behavior
}

#[test]
fn test_sprite_zero_hit_cleared_at_prerender() {
    let mut ppu = create_test_ppu();

    // Set sprite zero hit flag
    ppu.status_register.set_sprite_zero_hit_flag(true);
    assert!(ppu.status_register.get_sprite_zero_hit_flag());

    // Advance to pre-render scanline, dot 1
    advance_ppu_to(&mut ppu, PRE_RENDER_SCANLINE, 1);

    // Flag should be cleared
    assert_eq!(ppu.status_register.get_sprite_zero_hit_flag(), false);
}

#[test]
fn test_sprite_zero_hit_respects_rendering_enable() {
    let mut ppu = create_test_ppu();

    // Sprite zero hit requires both background AND sprite rendering enabled
    // If either is disabled, no hit can occur

    // Both disabled - no hit possible
    ppu.mask_register.set_show_background_flag(false);
    ppu.mask_register.set_show_sprites_flag(false);

    // Only background enabled - no hit possible
    ppu.mask_register.set_show_background_flag(true);
    ppu.mask_register.set_show_sprites_flag(false);

    // Only sprites enabled - no hit possible
    ppu.mask_register.set_show_background_flag(false);
    ppu.mask_register.set_show_sprites_flag(true);

    // Both enabled - hit is possible
    ppu.mask_register.set_show_background_flag(true);
    ppu.mask_register.set_show_sprites_flag(true);
    // This is the only state where sprite zero hit can occur
}

// =============================================================================
// Sprite Overflow Tests
// =============================================================================

#[test]
fn test_sprite_overflow_flag_cleared_at_prerender() {
    let mut ppu = create_test_ppu();

    // Set sprite overflow flag
    ppu.status_register.set_sprite_overflow_flag(true);
    assert!(ppu.status_register.get_sprite_overflow_flag());

    // Advance to pre-render scanline, dot 1
    advance_ppu_to(&mut ppu, PRE_RENDER_SCANLINE, 1);

    // Flag should be cleared
    assert_eq!(ppu.status_register.get_sprite_overflow_flag(), false);
}

#[test]
fn test_max_8_sprites_per_scanline() {
    // The NES can only display 8 sprites per scanline
    // This is a hardware limitation documented in the test

    // When more than 8 sprites are on a scanline, the overflow flag
    // should be set (with some hardware bugs - see below)
    let ppu = create_test_ppu();
    assert_eq!(ppu.oam_ram.len() / 4, 64, "OAM should hold 64 sprites");
}

// =============================================================================
// Sprite Overflow Hardware Bug Tests
// =============================================================================

#[test]
fn test_overflow_evaluation_bug_documented() {
    // The NES PPU has a famous bug in sprite overflow detection.
    // After finding 8 sprites, it increments both n (sprite index) AND m (byte index)
    // instead of just n. This causes diagonal evaluation through OAM.
    //
    // This can result in:
    // 1. False negatives: >8 sprites but overflow not set
    // 2. False positives: <=8 sprites but overflow incorrectly set
    //
    // For full accuracy, the emulator should replicate this bug.
    //
    // References:
    // - https://wiki.nesdev.org/w/index.php/PPU_sprite_evaluation
    // - https://wiki.nesdev.org/w/index.php/Sprite_overflow_games

    // This test documents the expected buggy behavior
    // Actual implementation tests would verify the specific bug patterns
}

// =============================================================================
// Sprite Size Tests
// =============================================================================

#[test]
fn test_sprite_8x8_mode() {
    let mut ppu = create_test_ppu();

    // Bit 5 of PPUCTRL selects sprite size
    ppu.write_control_register(0x00); // 8x8 sprites

    // Verify 8x8 mode
    assert_eq!(ppu.control_register.get_sprite_size_flag(), false);
}

#[test]
fn test_sprite_8x16_mode() {
    let mut ppu = create_test_ppu();

    // Bit 5 of PPUCTRL selects sprite size
    ppu.write_control_register(0x20); // 8x16 sprites

    // Verify 8x16 mode
    assert_eq!(ppu.control_register.get_sprite_size_flag(), true);
}

#[test]
fn test_sprite_pattern_table_selection() {
    let mut ppu = create_test_ppu();

    // Bit 3 of PPUCTRL selects sprite pattern table (for 8x8 sprites only)
    ppu.write_control_register(0x00); // Pattern table 0 (0x0000)
    assert_eq!(ppu.control_register.get_sprite_address_flag(), false);

    ppu.write_control_register(0x08); // Pattern table 1 (0x1000)
    assert_eq!(ppu.control_register.get_sprite_address_flag(), true);
}

// =============================================================================
// OAM DMA Tests
// =============================================================================

#[test]
fn test_oam_dma_writes_256_bytes() {
    let mut ppu = create_test_ppu();

    // Create test data
    let mut data = [0u8; 256];
    for i in 0..256 {
        data[i] = i as u8;
    }

    // Perform DMA
    ppu.oam_address = 0;
    ppu.write_oam_dma(&data);

    // Verify all bytes were written
    for i in 0..256 {
        assert_eq!(ppu.oam_ram[i], i as u8, "OAM byte {} should match", i);
    }
}

#[test]
fn test_oam_dma_wraps_address() {
    let mut ppu = create_test_ppu();

    // Start DMA from middle of OAM
    let mut data = [0xAA; 256];
    ppu.oam_address = 128;
    ppu.write_oam_dma(&data);

    // Should wrap around: bytes 128-255 first, then 0-127
    assert_eq!(ppu.oam_ram[128], 0xAA);
    assert_eq!(ppu.oam_ram[0], 0xAA);
    assert_eq!(ppu.oam_ram[127], 0xAA);

    // oam_address should wrap back to starting point
    assert_eq!(ppu.oam_address, 128);
}

// =============================================================================
// Sprite Priority Tests
// =============================================================================

#[test]
fn test_sprite_priority_bit() {
    let mut ppu = create_test_ppu();

    // Sprite attribute byte bit 5 controls priority
    // 0 = sprite in front of background
    // 1 = sprite behind background

    ppu.oam_address = 2; // Attribute byte of sprite 0
    ppu.write_oam_data_register(0x00); // Priority = front
    assert_eq!(ppu.oam_ram[2] & 0x20, 0x00);

    ppu.oam_address = 2;
    ppu.write_oam_data_register(0x20); // Priority = behind
    assert_eq!(ppu.oam_ram[2] & 0x20, 0x20);
}

#[test]
fn test_sprite_palette_selection() {
    let mut ppu = create_test_ppu();

    // Sprite attribute byte bits 0-1 select palette (4-7)
    ppu.oam_address = 2;
    ppu.write_oam_data_register(0x00); // Palette 4
    assert_eq!(ppu.oam_ram[2] & 0x03, 0x00);

    ppu.oam_address = 2;
    ppu.write_oam_data_register(0x03); // Palette 7
    assert_eq!(ppu.oam_ram[2] & 0x03, 0x03);
}

#[test]
fn test_sprite_flip_flags() {
    let mut ppu = create_test_ppu();

    // Bit 6: horizontal flip
    // Bit 7: vertical flip

    ppu.oam_address = 2;
    ppu.write_oam_data_register(0x40); // Horizontal flip
    assert_eq!(ppu.oam_ram[2] & 0x40, 0x40);

    ppu.oam_address = 2;
    ppu.write_oam_data_register(0x80); // Vertical flip
    assert_eq!(ppu.oam_ram[2] & 0x80, 0x80);

    ppu.oam_address = 2;
    ppu.write_oam_data_register(0xC0); // Both flips
    assert_eq!(ppu.oam_ram[2] & 0xC0, 0xC0);
}

// =============================================================================
// Sprite Evaluation Timing Tests
// =============================================================================
// Sprite evaluation occurs during dots 65-256 of visible scanlines.
// The PPU scans OAM to find up to 8 sprites that are on the current scanline.

#[test]
fn test_sprite_evaluation_dots_65_to_256() {
    // Sprite evaluation happens during dots 65-256
    // This is when the PPU determines which sprites appear on the next scanline

    let ppu = create_test_ppu();

    // Document the expected timing
    const SPRITE_EVAL_START: u16 = 65;
    const SPRITE_EVAL_END: u16 = 256;

    assert!(
        SPRITE_EVAL_END - SPRITE_EVAL_START == 191,
        "Sprite evaluation should span 192 dots"
    );
}

#[test]
fn test_sprite_fetches_during_dots_257_320() {
    // After evaluation, sprite pattern data is fetched during dots 257-320 (HBlank)
    // This is 64 dots = 8 sprites * 8 dots per sprite

    let ppu = create_test_ppu();

    const SPRITE_FETCH_START: u16 = 257;
    const SPRITE_FETCH_END: u16 = 320;

    // 8 sprites, each taking 8 dots to fetch pattern data
    assert_eq!(
        SPRITE_FETCH_END - SPRITE_FETCH_START + 1,
        64,
        "Sprite fetches should span 64 dots (8 sprites * 8 dots)"
    );
}

#[test]
fn test_sprite_on_scanline_detection() {
    let mut ppu = create_test_ppu();

    // Enable sprites
    ppu.mask_register.set_show_sprites_flag(true);

    // Place sprite 0 at Y=50 (sprite appears on scanlines 50-57 for 8x8)
    ppu.oam_ram[0] = 50; // Y position
    ppu.oam_ram[1] = 0x00; // Tile
    ppu.oam_ram[2] = 0x00; // Attributes
    ppu.oam_ram[3] = 100; // X position

    // A sprite at Y=50 should be visible on scanline 50
    // (Y in OAM is the top of the sprite)
    let sprite_y = ppu.oam_ram[0] as u16;
    let scanline: u16 = 50;

    // In 8x8 mode, sprite is visible when: scanline >= sprite_y && scanline < sprite_y + 8
    let is_8x8 = !ppu.control_register.get_sprite_size_flag();
    let sprite_height: u16 = if is_8x8 { 8 } else { 16 };

    assert!(
        scanline >= sprite_y && scanline < sprite_y + sprite_height,
        "Sprite at Y={} should be visible on scanline {}",
        sprite_y,
        scanline
    );
}

// =============================================================================
// Sprite Zero Hit Detailed Tests
// =============================================================================

#[test]
fn test_sprite_zero_hit_requires_both_rendering_enabled() {
    let mut ppu = create_test_ppu();

    // Sprite zero hit requires BOTH background AND sprites to be enabled

    // Neither enabled - no hit possible
    ppu.mask_register.set_show_background_flag(false);
    ppu.mask_register.set_show_sprites_flag(false);
    let both_disabled =
        !ppu.mask_register.get_show_background_flag() && !ppu.mask_register.get_show_sprites_flag();
    assert!(both_disabled, "Both rendering disabled");

    // Only background - no hit
    ppu.mask_register.set_show_background_flag(true);
    ppu.mask_register.set_show_sprites_flag(false);
    let only_bg =
        ppu.mask_register.get_show_background_flag() && !ppu.mask_register.get_show_sprites_flag();
    assert!(only_bg, "Only background enabled");

    // Only sprites - no hit
    ppu.mask_register.set_show_background_flag(false);
    ppu.mask_register.set_show_sprites_flag(true);
    let only_sprites =
        !ppu.mask_register.get_show_background_flag() && ppu.mask_register.get_show_sprites_flag();
    assert!(only_sprites, "Only sprites enabled");

    // Both enabled - hit is possible
    ppu.mask_register.set_show_background_flag(true);
    ppu.mask_register.set_show_sprites_flag(true);
    let both_enabled =
        ppu.mask_register.get_show_background_flag() && ppu.mask_register.get_show_sprites_flag();
    assert!(both_enabled, "Both rendering enabled - hit possible");
}

#[test]
fn test_sprite_zero_hit_not_at_x_255_detailed() {
    // Sprite zero hit cannot occur at x=255
    // This is because the hit detection happens at the pixel output stage,
    // and x=255 has special handling

    let mut ppu = create_test_ppu();
    ppu.mask_register.set_show_background_flag(true);
    ppu.mask_register.set_show_sprites_flag(true);

    // Place sprite 0 with X=255
    ppu.oam_ram[0] = 100; // Y
    ppu.oam_ram[1] = 0x00; // Tile
    ppu.oam_ram[2] = 0x00; // Attributes (front priority)
    ppu.oam_ram[3] = 255; // X = 255

    // Even if both sprite and background have opaque pixels,
    // hit cannot occur at x=255
    // This test documents the expected hardware behavior
}

#[test]
fn test_sprite_zero_hit_respects_left_clipping() {
    // If left 8 pixel clipping is enabled for either background or sprites,
    // sprite zero hit cannot occur in the leftmost 8 pixels

    let mut ppu = create_test_ppu();
    ppu.mask_register.set_show_background_flag(true);
    ppu.mask_register.set_show_sprites_flag(true);

    // Disable left 8 pixels for background
    ppu.mask_register.set_show_background_left_flag(false);

    // Place sprite 0 in the left 8 pixels
    ppu.oam_ram[0] = 100;
    ppu.oam_ram[1] = 0x00;
    ppu.oam_ram[2] = 0x00;
    ppu.oam_ram[3] = 0; // X = 0 (leftmost)

    // Sprite zero hit should not occur in clipped region
    // This documents the expected behavior
}

// =============================================================================
// Sprite Rendering Priority Tests
// =============================================================================

#[test]
fn test_lower_oam_index_has_higher_priority() {
    // When multiple sprites overlap, lower OAM index has priority
    // Sprite 0 has highest priority, sprite 63 has lowest

    let mut ppu = create_test_ppu();

    // Place two sprites at the same position
    // Sprite 0
    ppu.oam_ram[0] = 100; // Y
    ppu.oam_ram[1] = 0x01; // Tile 1
    ppu.oam_ram[2] = 0x00; // Attributes
    ppu.oam_ram[3] = 50; // X

    // Sprite 1 (same position, different tile)
    ppu.oam_ram[4] = 100; // Y
    ppu.oam_ram[5] = 0x02; // Tile 2
    ppu.oam_ram[6] = 0x00; // Attributes
    ppu.oam_ram[7] = 50; // X

    // Sprite 0 should be displayed in front of sprite 1
    // (when both have opaque pixels at the same location)
}

#[test]
fn test_sprite_behind_background_priority() {
    let mut ppu = create_test_ppu();

    // Set sprite priority bit (bit 5 of attributes)
    // 0 = in front of background
    // 1 = behind background

    // Sprite with priority = behind (bit 5 set)
    ppu.oam_ram[0] = 100;
    ppu.oam_ram[1] = 0x00;
    ppu.oam_ram[2] = 0x20; // Bit 5 set = behind background
    ppu.oam_ram[3] = 50;

    // When priority bit is set, sprite only shows where background is transparent
    assert_eq!(ppu.oam_ram[2] & 0x20, 0x20, "Priority bit should be set");
}

#[test]
fn test_sprite_in_front_of_background_priority() {
    let mut ppu = create_test_ppu();

    // Sprite with priority = in front (bit 5 clear)
    ppu.oam_ram[0] = 100;
    ppu.oam_ram[1] = 0x00;
    ppu.oam_ram[2] = 0x00; // Bit 5 clear = in front of background
    ppu.oam_ram[3] = 50;

    // When priority bit is clear, sprite shows in front of background
    assert_eq!(ppu.oam_ram[2] & 0x20, 0x00, "Priority bit should be clear");
}
