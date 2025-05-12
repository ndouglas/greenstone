//! PPU cycle-accurate test suite.
//!
//! This module contains comprehensive tests for verifying cycle-accurate
//! PPU behavior, organized by functionality.

mod timing;
mod vblank;
mod registers;
mod scrolling;
mod background;
mod sprites;
mod integration;

use super::*;
use crate::test::init;
use crate::Cartridge;
use std::cell::RefCell;
use std::rc::Rc;

/// Helper to create a test PPU with a cartridge attached for CHR ROM access.
pub fn create_test_ppu() -> PPU {
    init();
    let mut ppu = PPU::new();
    let cartridge = build_test_cartridge(false);
    ppu.vram.set_cartridge(cartridge);
    ppu
}

/// Helper to create a test PPU with vertical mirroring.
pub fn create_test_ppu_vertical() -> PPU {
    init();
    let mut ppu = PPU::new();
    let cartridge = build_test_cartridge(true);
    ppu.vram.set_cartridge(cartridge);
    ppu
}

/// Build a test cartridge with optional vertical mirroring.
fn build_test_cartridge(vertical: bool) -> Rc<RefCell<Cartridge>> {
    let mut data = vec![
        0x4e, 0x45, 0x53, 0x1a, // NES header
        0x02, // 2x 16KB PRG ROM
        0x01, // 1x 8KB CHR ROM
        0x00, // Flags 6 (will be modified for mirroring)
        0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    ];
    if vertical {
        data[6] = 0x01; // Vertical mirroring bit
    }
    // Add PRG ROM (2x 16KB = 32KB)
    data.extend_from_slice(&[0u8; 2 * 0x4000]);
    // Add CHR ROM (8KB) with pattern data
    for i in 0..0x2000u16 {
        data.push(i as u8);
    }
    Rc::new(RefCell::new(Cartridge::new(&data)))
}

/// Advance the PPU by N cycles.
pub fn tick_ppu_n_times(ppu: &mut PPU, n: u32) {
    for _ in 0..n {
        ppu.tick();
    }
}

/// Advance the PPU to a specific dot and scanline.
/// This is useful for testing behavior at specific timing points.
pub fn advance_ppu_to(ppu: &mut PPU, target_scanline: u16, target_dot: u16) {
    let start_frame = ppu.frame_count;
    // Calculate how many ticks needed to reach the target
    loop {
        if ppu.scanline == target_scanline && ppu.dot == target_dot {
            break;
        }
        ppu.tick();
        // Safety check to avoid infinite loops (allow up to 2 frames from start)
        if ppu.frame_count > start_frame + 2 {
            panic!(
                "Failed to reach target ({}, {}), currently at ({}, {})",
                target_scanline, target_dot, ppu.scanline, ppu.dot
            );
        }
    }
}

/// Calculate the total PPU cycles for a given position in the frame.
pub fn cycles_to_position(scanline: u16, dot: u16) -> u32 {
    (scanline as u32 * DOTS_PER_SCANLINE as u32) + dot as u32
}
