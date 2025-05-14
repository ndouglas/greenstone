//! Diagnostic tests for background rendering.
//!
//! These tests set up simple known patterns and verify the rendering pipeline.

use super::*;

/// Create a test PPU with writable CHR RAM (not ROM) for testing.
fn create_test_ppu_with_chr_ram() -> PPU {
    init();
    let mut ppu = PPU::new();

    // Create a cartridge with CHR RAM instead of ROM
    // Header: NES, 1x16KB PRG, 0x CHR (uses RAM), mapper 0
    let mut data = vec![
        0x4e, 0x45, 0x53, 0x1a, // NES header
        0x01, // 1x 16KB PRG ROM
        0x00, // 0x 8KB CHR ROM (will use CHR RAM)
        0x00, // Flags 6: horizontal mirroring, no battery, no trainer, mapper 0
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    ];
    // Add PRG ROM (16KB)
    data.extend_from_slice(&[0u8; 0x4000]);

    let cartridge = Rc::new(RefCell::new(Cartridge::new(&data)));
    ppu.vram.set_cartridge(cartridge);
    ppu
}

#[test]
fn test_debug_rendering_pipeline() {
    let mut ppu = create_test_ppu_with_chr_ram();

    // === Step 1: Set up a simple tile pattern in CHR RAM ===
    // Tile 0: All pixels are color 1 (solid tile)
    // Pattern table format: 8 bytes low plane, 8 bytes high plane
    // For color 1: low=0xFF, high=0x00 for each row
    for row in 0..8 {
        ppu.vram.write_u8(0x0000 + row, 0xFF); // Low plane: all 1s
        ppu.vram.write_u8(0x0008 + row, 0x00); // High plane: all 0s
    }

    // Tile 1: Checkerboard pattern (alternating colors 0 and 3)
    for row in 0..8 {
        let pattern = if row % 2 == 0 { 0xAA } else { 0x55 };
        ppu.vram.write_u8(0x0010 + row, pattern); // Low plane
        ppu.vram.write_u8(0x0018 + row, pattern); // High plane
    }

    // === Step 2: Set up nametable ===
    // Fill first row with tile 0, second row with tile 1
    for x in 0..32 {
        ppu.vram.write_u8(0x2000 + x, 0x00); // First row: tile 0
        ppu.vram.write_u8(0x2020 + x, 0x01); // Second row: tile 1
    }

    // === Step 3: Set up attributes (all use palette 0) ===
    for i in 0..64 {
        ppu.vram.write_u8(0x23C0 + i, 0x00);
    }

    // === Step 4: Set up palette ===
    ppu.vram.write_u8(0x3F00, 0x0F); // Backdrop: black
    ppu.vram.write_u8(0x3F01, 0x30); // Palette 0, color 1: white
    ppu.vram.write_u8(0x3F02, 0x10); // Palette 0, color 2: light gray
    ppu.vram.write_u8(0x3F03, 0x00); // Palette 0, color 3: dark gray

    // === Step 5: Enable rendering ===
    ppu.mask_register.set_show_background_flag(true);
    ppu.mask_register.set_show_background_left_flag(true);

    // === Step 6: Set scroll to 0,0 ===
    ppu.t_address.set_coarse_x(0);
    ppu.t_address.set_coarse_y(0);
    ppu.t_address.set_fine_y(0);
    ppu.t_address.set_nametable(0);
    ppu.v_address = ppu.t_address.clone();
    ppu.fine_x = 0;

    // === Step 7: Render one full frame ===
    println!("=== Starting frame render ===");

    // Tick through the frame
    let total_cycles = DOTS_PER_SCANLINE as u32 * SCANLINES_PER_FRAME as u32;
    tick_ppu_n_times(&mut ppu, total_cycles);

    // === Step 8: Check framebuffer ===
    println!("=== Checking framebuffer ===");

    // Check pixel (0, 0) - should be from tile 0, color 1 (white)
    let pixel_0_0 = get_framebuffer_pixel(&ppu, 0, 0);
    println!("Pixel (0,0): RGB({}, {}, {})", pixel_0_0.0, pixel_0_0.1, pixel_0_0.2);

    // Check pixel (0, 8) - should be from tile 1 (checkerboard)
    let pixel_0_8 = get_framebuffer_pixel(&ppu, 0, 8);
    println!("Pixel (0,8): RGB({}, {}, {})", pixel_0_8.0, pixel_0_8.1, pixel_0_8.2);

    // Expected white from NES palette 0x30: (236, 238, 236)
    let expected_white = (236, 238, 236);
    // Expected black from NES palette 0x0F: (0, 0, 0)
    let expected_black = (0, 0, 0);

    // First row should be white (tile 0, all color 1)
    assert_eq!(
        pixel_0_0, expected_white,
        "Pixel (0,0) should be white from tile 0"
    );
}

#[test]
fn test_debug_single_pixel_trace() {
    let mut ppu = create_test_ppu_with_chr_ram();

    // Set up minimal rendering: one tile with a known pattern
    // Tile 0 at pattern table 0x0000: vertical stripes
    // Color pattern: 1,0,1,0,1,0,1,0 for each row
    for row in 0..8 {
        ppu.vram.write_u8(0x0000 + row, 0xAA); // Low plane: 10101010
        ppu.vram.write_u8(0x0008 + row, 0x00); // High plane: 00000000
    }

    // Nametable: tile 0 at position (0,0)
    ppu.vram.write_u8(0x2000, 0x00);

    // Attributes: palette 0
    ppu.vram.write_u8(0x23C0, 0x00);

    // Palette
    ppu.vram.write_u8(0x3F00, 0x0F); // Backdrop: black (should not appear)
    ppu.vram.write_u8(0x3F01, 0x30); // Color 1: white

    // Enable rendering
    ppu.mask_register.set_show_background_flag(true);
    ppu.mask_register.set_show_background_left_flag(true);

    // Reset scroll
    ppu.t_address.0 = 0;
    ppu.v_address.0 = 0;
    ppu.fine_x = 0;

    println!("=== Tracing single pixel render ===");
    println!("t_address: 0x{:04X}", ppu.t_address.0);
    println!("v_address: 0x{:04X}", ppu.v_address.0);
    println!("fine_x: {}", ppu.fine_x);
    println!("coarse_x: {}, coarse_y: {}", ppu.v_address.coarse_x(), ppu.v_address.coarse_y());
    println!("fine_y: {}, nametable: {}", ppu.v_address.fine_y(), ppu.v_address.nametable());

    // Manually call get_background_pixel for x=0, y=0
    let color_index = ppu.get_background_pixel(0, 0);
    println!("get_background_pixel(0, 0) returned color index: 0x{:02X}", color_index);

    // The pattern 0xAA = 10101010, so bit 7 (pixel 0) = 1
    // Color should be 1, which maps to palette entry 0x3F01 = 0x30
    assert_eq!(color_index, 0x30, "Pixel 0 should be color index 0x30 (white)");

    // Check pixel 1 (bit 6 = 0, should be backdrop)
    let color_index_1 = ppu.get_background_pixel(1, 0);
    println!("get_background_pixel(1, 0) returned color index: 0x{:02X}", color_index_1);
    assert_eq!(color_index_1, 0x0F, "Pixel 1 should be backdrop 0x0F (black)");
}

#[test]
fn test_debug_vram_reads() {
    let mut ppu = create_test_ppu_with_chr_ram();

    // Write test data
    ppu.vram.write_u8(0x0000, 0xAB); // CHR RAM
    ppu.vram.write_u8(0x2000, 0xCD); // Nametable
    ppu.vram.write_u8(0x3F00, 0x12); // Palette

    // Read it back
    let chr = ppu.vram.read_u8(0x0000);
    let nt = ppu.vram.read_u8(0x2000);
    let pal = ppu.vram.read_u8(0x3F00);

    println!("CHR read: 0x{:02X} (expected 0xAB)", chr);
    println!("Nametable read: 0x{:02X} (expected 0xCD)", nt);
    println!("Palette read: 0x{:02X} (expected 0x12)", pal);

    assert_eq!(chr, 0xAB, "CHR RAM read failed");
    assert_eq!(nt, 0xCD, "Nametable read failed");
    assert_eq!(pal, 0x12, "Palette read failed");
}

#[test]
fn test_debug_tile_fetch_calculation() {
    let mut ppu = create_test_ppu_with_chr_ram();

    // Set up scroll at (0, 0)
    ppu.t_address.0 = 0;
    ppu.v_address.0 = 0;
    ppu.fine_x = 0;

    println!("=== Testing tile fetch calculations ===");

    // For screen pixel (0, 0):
    // - coarse_x = 0, coarse_y = 0, fine_x = 0, fine_y = 0
    // - nametable address = 0x2000 + 0*32 + 0 = 0x2000
    // - Should fetch tile index from 0x2000

    let coarse_y = ppu.v_address.coarse_y();
    let fine_y = ppu.v_address.fine_y();
    let nametable = ppu.v_address.nametable();

    let start_coarse_x = ppu.t_address.coarse_x() as usize;
    let start_nametable_x = (ppu.t_address.nametable() & 1) as usize;

    let x = 0usize;
    let total_x = start_coarse_x * 8 + ppu.fine_x as usize + x;
    let tile_x = (total_x / 8) % 32;
    let nametable_x = ((total_x / 256) + start_nametable_x) % 2;

    let nametable_y = (nametable >> 1) & 1;
    let current_nametable = (nametable_y << 1) | nametable_x as u8;

    let nametable_base = 0x2000u16 + (current_nametable as u16 * 0x400);
    let nametable_addr = nametable_base + (coarse_y as u16 * 32) + tile_x as u16;

    println!("coarse_y: {}, fine_y: {}, nametable: {}", coarse_y, fine_y, nametable);
    println!("start_coarse_x: {}, start_nametable_x: {}", start_coarse_x, start_nametable_x);
    println!("total_x: {}, tile_x: {}, nametable_x: {}", total_x, tile_x, nametable_x);
    println!("current_nametable: {}", current_nametable);
    println!("nametable_base: 0x{:04X}", nametable_base);
    println!("nametable_addr: 0x{:04X}", nametable_addr);

    assert_eq!(nametable_addr, 0x2000, "Nametable address calculation is wrong");
}

/// Helper to get RGB pixel from framebuffer
fn get_framebuffer_pixel(ppu: &PPU, x: usize, y: usize) -> (u8, u8, u8) {
    let index = (y * SCREEN_WIDTH + x) * 3;
    (
        ppu.framebuffer[index],
        ppu.framebuffer[index + 1],
        ppu.framebuffer[index + 2],
    )
}
