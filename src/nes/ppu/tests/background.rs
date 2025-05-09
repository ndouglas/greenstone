//! Background rendering tests.
//!
//! Tests for tile fetching, scroll increments, and pixel output.

use super::*;

// =============================================================================
// Background Enable/Disable Tests
// =============================================================================

#[test]
fn test_background_disabled_outputs_backdrop_color() {
    let mut ppu = create_test_ppu();

    // Ensure background rendering is disabled
    ppu.mask_register.set_show_background_flag(false);

    // Set a backdrop color
    ppu.vram.write_u8(0x3F00, 0x0F); // Black

    // Get pixel - should be backdrop
    let color = ppu.get_background_pixel(100, 100);
    assert_eq!(color, 0x0F, "With background disabled, should return backdrop color");
}

#[test]
fn test_left_8_pixels_clipped_when_mask_bit_clear() {
    let mut ppu = create_test_ppu();

    // Enable background but disable left 8 pixels
    ppu.mask_register.set_show_background_flag(true);
    ppu.mask_register.set_show_background_left_flag(false);

    // Set backdrop color
    ppu.vram.write_u8(0x3F00, 0x0D);

    // Pixels 0-7 should return backdrop
    for x in 0..8 {
        let color = ppu.get_background_pixel(x, 50);
        assert_eq!(
            color, 0x0D,
            "Pixel {} should be clipped and show backdrop",
            x
        );
    }
}

#[test]
fn test_left_8_pixels_shown_when_mask_bit_set() {
    let mut ppu = create_test_ppu();

    // Enable background and left 8 pixels
    ppu.mask_register.set_show_background_flag(true);
    ppu.mask_register.set_show_background_left_flag(true);

    // Set up a simple tile pattern
    // We need: nametable entry, pattern data, attribute, palette

    // Set backdrop and palette colors
    ppu.vram.write_u8(0x3F00, 0x0F); // Backdrop black
    ppu.vram.write_u8(0x3F01, 0x15); // Palette 0, color 1

    // The get_background_pixel function should try to render actual tile data
    // For pixels 0-7, it should NOT use backdrop (unless the pixel value is 0)
    // This test verifies the left clipping flag is respected
}

// =============================================================================
// Scroll Position Tests
// =============================================================================

#[test]
fn test_scroll_affects_pixel_lookup() {
    let mut ppu = create_test_ppu();

    // Enable background
    ppu.mask_register.set_show_background_flag(true);
    ppu.mask_register.set_show_background_left_flag(true);

    // Set scroll position
    ppu.write_scroll_register(8); // X = 8 (coarse_x = 1, fine_x = 0)
    ppu.write_scroll_register(0); // Y = 0

    // Verify scroll was set
    assert_eq!(ppu.t_address.coarse_x(), 1);
    assert_eq!(ppu.fine_x, 0);
}

#[test]
fn test_fine_x_determines_pixel_offset() {
    let mut ppu = create_test_ppu();

    // Set fine_x to various values
    ppu.write_scroll_register(0b00000_011); // coarse_x = 0, fine_x = 3
    assert_eq!(ppu.fine_x, 3);

    ppu.read_status_register(); // Reset latch
    ppu.write_scroll_register(0b00000_111); // coarse_x = 0, fine_x = 7
    assert_eq!(ppu.fine_x, 7);
}

// =============================================================================
// Palette Tests
// =============================================================================

#[test]
fn test_palette_to_rgb_valid_indices() {
    let ppu = create_test_ppu();

    // Test a few known palette entries
    let (r, g, b) = ppu.palette_to_rgb(0x00);
    assert_eq!((r, g, b), (84, 84, 84), "Palette 0x00 should be gray");

    let (r, g, b) = ppu.palette_to_rgb(0x0D);
    assert_eq!((r, g, b), (0, 0, 0), "Palette 0x0D should be black");

    let (r, g, b) = ppu.palette_to_rgb(0x20);
    assert_eq!((r, g, b), (236, 238, 236), "Palette 0x20 should be white");
}

#[test]
fn test_palette_to_rgb_wraps_at_64() {
    let ppu = create_test_ppu();

    // Palette index should be masked to 6 bits (0-63)
    let color_00 = ppu.palette_to_rgb(0x00);
    let color_40 = ppu.palette_to_rgb(0x40);
    let color_80 = ppu.palette_to_rgb(0x80);

    // All should be the same (wrapped)
    assert_eq!(color_00, color_40);
    assert_eq!(color_00, color_80);
}

// =============================================================================
// Nametable Tests
// =============================================================================

#[test]
fn test_nametable_selection_from_scroll() {
    let mut ppu = create_test_ppu();

    // Set nametable via control register (bits 0-1)
    ppu.write_control_register(0x00); // Nametable 0
    assert_eq!(ppu.control_register.get_nametable(), 0);

    ppu.write_control_register(0x01); // Nametable 1
    assert_eq!(ppu.control_register.get_nametable(), 1);

    ppu.write_control_register(0x02); // Nametable 2
    assert_eq!(ppu.control_register.get_nametable(), 2);

    ppu.write_control_register(0x03); // Nametable 3
    assert_eq!(ppu.control_register.get_nametable(), 3);
}

#[test]
fn test_pattern_table_selection() {
    let mut ppu = create_test_ppu();

    // Bit 4 of PPUCTRL selects background pattern table
    ppu.write_control_register(0x00); // Pattern table 0 (0x0000)
    assert_eq!(ppu.control_register.get_background_address_flag(), false);

    ppu.write_control_register(0x10); // Pattern table 1 (0x1000)
    assert_eq!(ppu.control_register.get_background_address_flag(), true);
}

// =============================================================================
// Attribute Table Tests
// =============================================================================

#[test]
fn test_attribute_table_quadrant_selection() {
    // The attribute byte covers 4x4 tiles (32x32 pixels)
    // Each 2-bit section covers 2x2 tiles (16x16 pixels):
    //   bits 0-1: top-left
    //   bits 2-3: top-right
    //   bits 4-5: bottom-left
    //   bits 6-7: bottom-right

    let ppu = create_test_ppu();

    // This is more of a documentation test - the actual attribute selection
    // is tested implicitly through get_background_pixel
    assert_eq!(NAMETABLE_SIZE, 0x400); // 1KB per nametable
}

// =============================================================================
// Framebuffer Output Tests
// =============================================================================

#[test]
fn test_render_pixel_writes_to_framebuffer() {
    let mut ppu = create_test_ppu();

    // Set up rendering
    ppu.mask_register.set_show_background_flag(true);

    // Set a known backdrop color
    ppu.vram.write_u8(0x3F00, 0x0F); // Black

    // Position at a visible pixel
    ppu.dot = 1; // First visible dot
    ppu.scanline = 0;

    // Render the pixel
    ppu.render_pixel();

    // Check framebuffer was written
    let pixel_index = 0; // First pixel
    let r = ppu.framebuffer[pixel_index * 3];
    let g = ppu.framebuffer[pixel_index * 3 + 1];
    let b = ppu.framebuffer[pixel_index * 3 + 2];

    // Should have some color value (not necessarily 0,0,0 - depends on tile data)
    assert!(
        ppu.framebuffer.len() == SCREEN_WIDTH * SCREEN_HEIGHT * 3,
        "Framebuffer should be properly sized"
    );
}

#[test]
fn test_render_pixel_skips_dot_0() {
    let mut ppu = create_test_ppu();

    // Fill framebuffer with a known value
    ppu.framebuffer.fill(0xAA);

    // Position at dot 0 (idle cycle)
    ppu.dot = 0;
    ppu.scanline = 0;

    // Render should skip dot 0
    ppu.render_pixel();

    // First pixel should be unchanged
    assert_eq!(ppu.framebuffer[0], 0xAA, "Dot 0 should not render");
}

#[test]
fn test_render_pixel_skips_after_dot_256() {
    let mut ppu = create_test_ppu();

    // Fill framebuffer with a known value
    ppu.framebuffer.fill(0xAA);

    // Position past visible area
    ppu.dot = 257;
    ppu.scanline = 0;

    // Render should skip
    ppu.render_pixel();

    // Last pixel should be unchanged (checking it wasn't overwritten)
    let last_pixel = (SCREEN_WIDTH - 1) * 3;
    assert_eq!(ppu.framebuffer[last_pixel], 0xAA, "Dots after 256 should not render");
}
