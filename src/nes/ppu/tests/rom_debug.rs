//! Debug tests for ROM loading and PPU state inspection.

use super::*;

/// Load a ROM file and inspect its CHR data via a PPU with cartridge
#[test]
fn test_accuracycoin_chr_data() {
    init();

    // Try to load the ROM
    let rom_path = "roms/AccuracyCoin.nes";
    let rom_data = match std::fs::read(rom_path) {
        Ok(data) => data,
        Err(e) => {
            println!("Could not load ROM {}: {}", rom_path, e);
            println!("Skipping test - ROM not found");
            return;
        }
    };

    println!("ROM loaded: {} bytes", rom_data.len());
    println!("Header: {:02X} {:02X} {:02X} {:02X}",
             rom_data[0], rom_data[1], rom_data[2], rom_data[3]);
    println!("PRG ROM: {} x 16KB", rom_data[4]);
    println!("CHR ROM: {} x 8KB", rom_data[5]);

    // Create a PPU and attach the cartridge directly
    let mut ppu = PPU::new();
    let cartridge = Rc::new(RefCell::new(Cartridge::new(&rom_data)));
    ppu.vram.set_cartridge(cartridge);

    // The PPU should now have access to CHR ROM through the cartridge
    // CHR ROM is at PPU addresses 0x0000-0x1FFF

    // Read first few bytes of CHR ROM through PPU VRAM
    println!("\n=== CHR ROM Data (first 32 bytes) ===");
    for i in 0..32u16 {
        let byte = ppu.vram.read_u8(i);
        if i % 16 == 0 {
            print!("{:04X}: ", i);
        }
        print!("{:02X} ", byte);
        if i % 16 == 15 {
            println!();
        }
    }

    // Check if there's any non-zero data (indicates CHR is loaded)
    let mut has_chr_data = false;
    let mut first_nonzero_addr = 0u16;
    for i in 0..0x2000u16 {
        if ppu.vram.read_u8(i) != 0 {
            has_chr_data = true;
            first_nonzero_addr = i;
            break;
        }
    }
    println!("\nCHR ROM has data: {} (first non-zero at 0x{:04X})", has_chr_data, first_nonzero_addr);

    // Show some non-zero CHR data if found
    if has_chr_data {
        println!("\n=== CHR data around first non-zero byte ===");
        let start = (first_nonzero_addr / 16) * 16;
        for i in start..(start + 32).min(0x2000) {
            let byte = ppu.vram.read_u8(i);
            if (i - start) % 16 == 0 {
                print!("{:04X}: ", i);
            }
            print!("{:02X} ", byte);
            if (i - start) % 16 == 15 {
                println!();
            }
        }
    }

    assert!(has_chr_data, "CHR ROM should have non-zero data");

    // Show the first tile's pattern data (16 bytes)
    println!("\n=== First tile pattern (tile 0) ===");
    println!("Low plane (0x0000-0x0007):");
    for i in 0..8u16 {
        let byte = ppu.vram.read_u8(i);
        print!("{:02X} ({:08b}) ", byte, byte);
    }
    println!("\nHigh plane (0x0008-0x000F):");
    for i in 8..16u16 {
        let byte = ppu.vram.read_u8(i);
        print!("{:02X} ({:08b}) ", byte, byte);
    }
    println!();
}

/// Test that we can read palette RAM
#[test]
fn test_palette_ram_access() {
    init();
    let mut ppu = PPU::new();

    // Write to palette
    ppu.vram.write_u8(0x3F00, 0x0F);
    ppu.vram.write_u8(0x3F01, 0x30);
    ppu.vram.write_u8(0x3F02, 0x10);

    // Read back
    let p0 = ppu.vram.read_u8(0x3F00);
    let p1 = ppu.vram.read_u8(0x3F01);
    let p2 = ppu.vram.read_u8(0x3F02);

    println!("Palette: {:02X} {:02X} {:02X}", p0, p1, p2);
    assert_eq!(p0, 0x0F);
    assert_eq!(p1, 0x30);
    assert_eq!(p2, 0x10);
}

/// Test nametable access
#[test]
fn test_nametable_access() {
    init();
    let mut ppu = PPU::new();

    // Need a cartridge for mirroring mode
    let cartridge = build_test_cartridge(false);
    ppu.vram.set_cartridge(cartridge);

    // Write to nametable
    ppu.vram.write_u8(0x2000, 0x42);
    ppu.vram.write_u8(0x2001, 0x43);

    // Read back
    let n0 = ppu.vram.read_u8(0x2000);
    let n1 = ppu.vram.read_u8(0x2001);

    println!("Nametable: {:02X} {:02X}", n0, n1);
    assert_eq!(n0, 0x42);
    assert_eq!(n1, 0x43);
}

/// Test AccuracyCoin rendering after enabling background
#[test]
fn test_accuracycoin_rendering_with_manual_setup() {
    init();

    // Load ROM
    let rom_path = "roms/AccuracyCoin.nes";
    let rom_data = match std::fs::read(rom_path) {
        Ok(data) => data,
        Err(e) => {
            println!("Could not load ROM {}: {}", rom_path, e);
            println!("Skipping test - ROM not found");
            return;
        }
    };

    // Create PPU with cartridge
    let mut ppu = PPU::new();
    let cartridge = Rc::new(RefCell::new(Cartridge::new(&rom_data)));
    ppu.vram.set_cartridge(cartridge);

    // Manually enable rendering (simulating what the game would do)
    ppu.mask_register.set_show_background_flag(true);
    ppu.mask_register.set_show_background_left_flag(true);
    ppu.mask_register.set_show_sprites_flag(true);
    ppu.mask_register.set_show_sprites_left_flag(true);

    // Set up a simple nametable: fill with tile 0 (the "0" digit)
    for i in 0..960 {
        ppu.vram.write_u8(0x2000 + i, 0x00);
    }
    // Write "12345" at position (10, 15)
    let text_offset = 15 * 32 + 10;
    ppu.vram.write_u8(0x2000 + text_offset, 0x01); // '1'
    ppu.vram.write_u8(0x2000 + text_offset + 1, 0x02); // '2'
    ppu.vram.write_u8(0x2000 + text_offset + 2, 0x03); // '3'
    ppu.vram.write_u8(0x2000 + text_offset + 3, 0x04); // '4'
    ppu.vram.write_u8(0x2000 + text_offset + 4, 0x05); // '5'

    // Set up attributes (all palette 0)
    for i in 0..64 {
        ppu.vram.write_u8(0x23C0 + i, 0x00);
    }

    // Set up palette
    ppu.vram.write_u8(0x3F00, 0x0F); // Background: black
    ppu.vram.write_u8(0x3F01, 0x30); // Color 1: white
    ppu.vram.write_u8(0x3F02, 0x10); // Color 2: light gray
    ppu.vram.write_u8(0x3F03, 0x00); // Color 3: dark gray

    // Reset scroll
    ppu.t_address.0 = 0;
    ppu.v_address.0 = 0;
    ppu.fine_x = 0;

    // Render one full frame
    let total_cycles = 341 * 262;
    tick_ppu_n_times(&mut ppu, total_cycles);

    // Check framebuffer for non-black pixels
    let mut non_black_count = 0;
    let mut sample_pixels: Vec<(usize, usize, u8, u8, u8)> = Vec::new();
    for y in 0..240 {
        for x in 0..256 {
            let idx = (y * 256 + x) * 3;
            let r = ppu.framebuffer[idx];
            let g = ppu.framebuffer[idx + 1];
            let b = ppu.framebuffer[idx + 2];
            if r != 0 || g != 0 || b != 0 {
                non_black_count += 1;
                if sample_pixels.len() < 10 {
                    sample_pixels.push((x, y, r, g, b));
                }
            }
        }
    }

    println!("Non-black pixels: {}", non_black_count);
    for (x, y, r, g, b) in &sample_pixels {
        println!("  Pixel ({}, {}): RGB({}, {}, {})", x, y, r, g, b);
    }

    assert!(non_black_count > 0, "Should have rendered some non-black pixels");
}

/// Test AccuracyCoin running through full CPU + Bus system
#[test]
fn test_accuracycoin_full_system() {
    use crate::nes::{Bus, CPU};
    use crate::traits::{Addressable, Interruptible};

    init();

    // Load ROM
    let rom_path = "roms/AccuracyCoin.nes";
    let rom_data = match std::fs::read(rom_path) {
        Ok(data) => data,
        Err(e) => {
            println!("Could not load ROM {}: {}", rom_path, e);
            println!("Skipping test - ROM not found");
            return;
        }
    };

    // Create full system
    let mut bus = Bus::new();
    bus.load_cartridge_data(&rom_data);

    // Print initial PPU state
    println!("Initial PPU state:");
    println!("{}", bus.get_ppu_debug_info());

    let mut cpu = CPU::new_with_bus(Box::new(bus));
    cpu.handle_reset();

    println!("\nInitial PC: 0x{:04X}", cpu.program_counter);

    // Run for several frames worth of CPU cycles
    // 1 frame = ~29780 CPU cycles (89342 PPU cycles / 3)
    let cycles_per_frame = 29780u32;
    let total_cycles = cycles_per_frame * 30; // Run for 30 frames (about 0.5 seconds)

    for i in 0..total_cycles {
        cpu.clock();

        // Check for frame completion every frame
        if i > 0 && i % cycles_per_frame == 0 {
            let fb = cpu.get_framebuffer();
            let non_black = fb.chunks(3).filter(|p| p[0] != 0 || p[1] != 0 || p[2] != 0).count();
            println!("Frame {}: {} non-black pixels", i / cycles_per_frame, non_black);
        }
    }

    // Check final framebuffer
    let fb = cpu.get_framebuffer();
    let non_black_count = fb.chunks(3).filter(|p| p[0] != 0 || p[1] != 0 || p[2] != 0).count();

    println!("\nFinal framebuffer: {} non-black pixels out of {} total", non_black_count, fb.len() / 3);

    // Show sample of pixel colors at various locations
    println!("\nSample pixels:");
    let sample_coords = [(0, 0), (128, 0), (255, 0), (0, 120), (128, 120), (255, 239)];
    for (x, y) in sample_coords {
        let idx = (y * 256 + x) * 3;
        println!("  ({}, {}): RGB({}, {}, {})", x, y, fb[idx], fb[idx + 1], fb[idx + 2]);
    }

    // Count unique colors
    use std::collections::HashMap;
    let mut color_counts: HashMap<(u8, u8, u8), usize> = HashMap::new();
    for pixel in fb.chunks(3) {
        let color = (pixel[0], pixel[1], pixel[2]);
        *color_counts.entry(color).or_insert(0) += 1;
    }
    println!("\nUnique colors: {}", color_counts.len());
    let mut sorted_colors: Vec<_> = color_counts.iter().collect();
    sorted_colors.sort_by(|a, b| b.1.cmp(a.1));
    for (color, count) in sorted_colors.iter().take(10) {
        println!("  RGB{:?}: {} pixels", color, count);
    }

    println!("\nFinal PC: 0x{:04X}", cpu.program_counter);

    // This test is informational - we want to see if anything got rendered
    // Don't assert yet, just report
    if non_black_count == 0 {
        println!("WARNING: No pixels rendered - game may not be initializing PPU correctly");
    }
}

/// Debug test to inspect what the game writes to PPU registers
#[test]
fn test_accuracycoin_ppu_writes() {
    use crate::nes::Bus;
    use crate::traits::Addressable;

    init();

    // Load ROM
    let rom_path = "roms/AccuracyCoin.nes";
    let rom_data = match std::fs::read(rom_path) {
        Ok(data) => data,
        Err(e) => {
            println!("Could not load ROM {}: {}", rom_path, e);
            println!("Skipping test - ROM not found");
            return;
        }
    };

    // Create bus with cartridge
    let mut bus = Bus::new();
    bus.load_cartridge_data(&rom_data);

    // Check initial PPU state
    println!("Initial state:");
    println!("{}", bus.get_ppu_debug_info());

    // Simulate writing to PPU registers like the game would
    // PPUMASK = 0x2001 - enable background rendering
    println!("\nWriting 0x1E to PPUMASK (enable all rendering):");
    bus.unclocked_write_u8(0x2001, 0x1E);
    println!("{}", bus.get_ppu_debug_info());

    // Write palette data
    println!("\nWriting palette data...");
    // PPUADDR = 0x3F00 (palette start)
    bus.unclocked_write_u8(0x2006, 0x3F);
    bus.unclocked_write_u8(0x2006, 0x00);
    // Write some palette colors
    bus.unclocked_write_u8(0x2007, 0x0F); // Backdrop: black
    bus.unclocked_write_u8(0x2007, 0x30); // Color 1: white
    bus.unclocked_write_u8(0x2007, 0x10); // Color 2: light gray
    bus.unclocked_write_u8(0x2007, 0x00); // Color 3: dark gray

    // Write to nametable - put some non-zero tiles
    println!("Writing nametable data...");
    bus.unclocked_write_u8(0x2006, 0x20);
    bus.unclocked_write_u8(0x2006, 0x00);
    for i in 0..32 {
        bus.unclocked_write_u8(0x2007, 0x01); // Fill first row with tile 1
    }

    // Reset scroll/address
    bus.unclocked_write_u8(0x2005, 0x00); // Scroll X
    bus.unclocked_write_u8(0x2005, 0x00); // Scroll Y

    // Tick through one frame
    let total_cycles = 89342;
    for _ in 0..total_cycles {
        bus.tick();
    }

    // Check final state
    println!("\nAfter one frame:");
    println!("{}", bus.get_ppu_debug_info());

    // Check framebuffer
    let fb = bus.get_framebuffer();
    let non_black = fb.chunks(3).filter(|p| p[0] != 0 || p[1] != 0 || p[2] != 0).count();
    println!("\nNon-black pixels: {}", non_black);

    // Count unique colors
    use std::collections::HashMap;
    let mut color_counts: HashMap<(u8, u8, u8), usize> = HashMap::new();
    for pixel in fb.chunks(3) {
        let color = (pixel[0], pixel[1], pixel[2]);
        *color_counts.entry(color).or_insert(0) += 1;
    }
    println!("Unique colors: {}", color_counts.len());
    for (color, count) in &color_counts {
        println!("  RGB{:?}: {} pixels", color, count);
    }
}

/// Test to dump CHR ROM and nametable contents
#[test]
fn test_accuracycoin_memory_dump() {
    init();

    // Load ROM
    let rom_path = "roms/AccuracyCoin.nes";
    let rom_data = match std::fs::read(rom_path) {
        Ok(data) => data,
        Err(e) => {
            println!("Could not load ROM {}: {}", rom_path, e);
            println!("Skipping test - ROM not found");
            return;
        }
    };

    // Create PPU with cartridge
    let mut ppu = PPU::new();
    let cartridge = Rc::new(RefCell::new(Cartridge::new(&rom_data)));
    ppu.vram.set_cartridge(cartridge);

    // Dump first few tiles of CHR ROM (each tile is 16 bytes)
    println!("=== CHR ROM - First 4 tiles ===");
    for tile in 0..4 {
        let base = tile * 16;
        println!("Tile {}: ", tile);
        print!("  Low:  ");
        for i in 0..8 {
            print!("{:02X} ", ppu.vram.read_u8(base + i));
        }
        println!();
        print!("  High: ");
        for i in 8..16 {
            print!("{:02X} ", ppu.vram.read_u8(base + i));
        }
        println!();
    }

    // Dump nametable 0 (first 64 bytes to see what tiles are there)
    println!("\n=== Nametable 0 (0x2000) - first 64 bytes ===");
    for row in 0..2 {
        print!("{:04X}: ", 0x2000 + row * 32);
        for col in 0..32 {
            let addr = 0x2000 + row * 32 + col;
            print!("{:02X} ", ppu.vram.read_u8(addr));
        }
        println!();
    }

    // Dump palette
    println!("\n=== Palette (0x3F00-0x3F1F) ===");
    print!("BG: ");
    for i in 0..16 {
        print!("{:02X} ", ppu.vram.read_u8(0x3F00 + i));
    }
    println!();
    print!("SP: ");
    for i in 0..16 {
        print!("{:02X} ", ppu.vram.read_u8(0x3F10 + i));
    }
    println!();

    // Check if tile 0 has any non-zero data
    let mut tile0_has_data = false;
    for i in 0..16 {
        if ppu.vram.read_u8(i) != 0 {
            tile0_has_data = true;
            break;
        }
    }
    println!("\nTile 0 has data: {}", tile0_has_data);
}

/// Trace CPU execution to see what the game is doing
#[test]
fn test_accuracycoin_cpu_trace() {
    use crate::nes::{Bus, CPU};
    use crate::traits::{Addressable, Interruptible};

    init();

    // Load ROM
    let rom_path = "roms/AccuracyCoin.nes";
    let rom_data = match std::fs::read(rom_path) {
        Ok(data) => data,
        Err(e) => {
            println!("Could not load ROM {}: {}", rom_path, e);
            println!("Skipping test - ROM not found");
            return;
        }
    };

    // Create full system
    let mut bus = Bus::new();
    bus.load_cartridge_data(&rom_data);

    // First, dump the ROM around the suspected loop
    println!("=== ROM Disassembly around 0x8040 ===");
    for addr in (0x8040..=0x8050).step_by(1) {
        let byte = bus.unclocked_read_u8(addr);
        print!("{:04X}: {:02X}  ", addr, byte);
        // Simple disassembly hints
        match byte {
            0xAD => println!("LDA abs"),
            0x10 => println!("BPL rel"),
            0x30 => println!("BMI rel"),
            0x4C => println!("JMP abs"),
            0xD0 => println!("BNE rel"),
            0xF0 => println!("BEQ rel"),
            0x29 => println!("AND #imm"),
            0x2C => println!("BIT abs"),
            _ => println!(""),
        }
    }

    let mut cpu = CPU::new_with_bus(Box::new(bus));
    cpu.handle_reset();

    println!("\nReset vector: 0x{:04X}", cpu.program_counter);

    // Track PPU register writes
    let mut last_pc = cpu.program_counter;

    // Run for enough cycles to complete at least 2 frames
    // One frame = ~29780 CPU cycles (89342 PPU cycles / 3)
    let total_cycles = 29780 * 3; // Run for 3 frames

    for i in 0..total_cycles {
        cpu.clock();

        // Log major events
        if i < 30 || (i > 29700 && i < 29850) || (i % 10000 == 0) {
            println!("Cycle {}: PC=0x{:04X} A=0x{:02X}", i, cpu.program_counter, cpu.a);
        }

        last_pc = cpu.program_counter;
    }

    println!("\nFinal state after 5000 cycles:");
    println!("  PC: 0x{:04X}", cpu.program_counter);
    println!("  A: 0x{:02X}, X: 0x{:02X}, Y: 0x{:02X}", cpu.a, cpu.x, cpu.y);
    println!("  SP: 0x{:02X}, Status: 0x{:02X}", cpu.stack_pointer, cpu.status);

    // Check if any pixels were rendered
    let fb = cpu.get_framebuffer();
    let non_black = fb.chunks(3).filter(|p| p[0] != 0 || p[1] != 0 || p[2] != 0).count();
    println!("\nNon-black pixels: {}", non_black);
}
