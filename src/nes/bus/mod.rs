use std::cell::RefCell;
use std::rc::Rc;

use serde::de::Deserializer;
use serde::ser::Serializer;

use crate::cartridge::Cartridge;
use crate::traits::Mappable;

use super::apu::APU;
use super::ppu::PPU;

const PROGRAM_CONTROL_ADDRESS: u16 = 0xFFFC;
const MAX_ADDRESS: u16 = 0xFFFF;

const RAM_START_ADDRESS: u16 = 0x0000;
const RAM_ACTUAL_END_ADDRESS: u16 = 0x07FF;
const RAM_RANGE_END_ADDRESS: u16 = 0x1FFF;

const PPU_REGISTER_START_ADDRESS: u16 = 0x2000;
const PPU_REGISTER_ACTUAL_END_ADDRESS: u16 = 0x2007;
const PPU_REGISTER_RANGE_END_ADDRESS: u16 = 0x3FFF;

const APU_IO_START_ADDRESS: u16 = 0x4000;
const APU_IO_END_ADDRESS: u16 = 0x4017;
const OAM_DMA_ADDRESS: u16 = 0x4014;

const CARTRIDGE_START_ADDRESS: u16 = 0x4018;
const CARTRIDGE_END_ADDRESS: u16 = MAX_ADDRESS;

mod addressable;

mod busable;

mod interruptible;

/// NES Controller button bits (Active-HIGH for internal state)
pub const BUTTON_A: u8 = 0b0000_0001;
pub const BUTTON_B: u8 = 0b0000_0010;
pub const BUTTON_SELECT: u8 = 0b0000_0100;
pub const BUTTON_START: u8 = 0b0000_1000;
pub const BUTTON_UP: u8 = 0b0001_0000;
pub const BUTTON_DOWN: u8 = 0b0010_0000;
pub const BUTTON_LEFT: u8 = 0b0100_0000;
pub const BUTTON_RIGHT: u8 = 0b1000_0000;

/// Serialize a fixed-size byte array as a byte sequence.
fn serialize_memory<S: Serializer>(memory: &[u8; (RAM_ACTUAL_END_ADDRESS + 1) as usize], serializer: S) -> Result<S::Ok, S::Error> {
  serializer.serialize_bytes(memory)
}

/// Deserialize a byte sequence into a fixed-size array.
fn deserialize_memory<'de, D: Deserializer<'de>>(deserializer: D) -> Result<[u8; (RAM_ACTUAL_END_ADDRESS + 1) as usize], D::Error> {
  let bytes: Vec<u8> = serde::Deserialize::deserialize(deserializer)?;
  if bytes.len() != (RAM_ACTUAL_END_ADDRESS + 1) as usize {
    return Err(serde::de::Error::custom(format!(
      "expected {} bytes for RAM, got {}",
      RAM_ACTUAL_END_ADDRESS + 1,
      bytes.len()
    )));
  }
  let mut memory = [0u8; (RAM_ACTUAL_END_ADDRESS + 1) as usize];
  memory.copy_from_slice(&bytes);
  Ok(memory)
}

/// Serialize Option<Rc<RefCell<Cartridge>>> by borrowing the inner Cartridge.
fn serialize_cartridge<S: Serializer>(cartridge: &Option<Rc<RefCell<Cartridge>>>, serializer: S) -> Result<S::Ok, S::Error> {
  match cartridge {
    Some(rc) => {
      let cart = rc.borrow();
      serde::Serialize::serialize(&Some(&*cart), serializer)
    }
    None => serde::Serialize::serialize(&None::<Cartridge>, serializer),
  }
}

/// Deserialize Option<Cartridge> and wrap in Rc<RefCell<>>.
/// After deserializing Bus, call `reconnect_cartridge()` to share with VRAM.
fn deserialize_cartridge<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Option<Rc<RefCell<Cartridge>>>, D::Error> {
  let opt: Option<Cartridge> = serde::Deserialize::deserialize(deserializer)?;
  Ok(opt.map(|c| Rc::new(RefCell::new(c))))
}

#[derive(Serialize, Deserialize)]
pub struct Bus {
  #[serde(serialize_with = "serialize_memory", deserialize_with = "deserialize_memory")]
  memory: [u8; (RAM_ACTUAL_END_ADDRESS + 1) as usize],
  #[serde(serialize_with = "serialize_cartridge", deserialize_with = "deserialize_cartridge")]
  cartridge: Option<Rc<RefCell<Cartridge>>>,
  clock_counter: u64,
  ppu: PPU,
  /// Controller 1 button state (bits: A, B, Select, Start, Up, Down, Left, Right)
  controller1_state: u8,
  /// Controller 2 button state
  controller2_state: u8,
  /// Controller 1 shift register (latched state being read out)
  controller1_shift: u8,
  /// Controller 2 shift register
  controller2_shift: u8,
  /// Controller strobe state (when high, continuously reload shift registers)
  controller_strobe: bool,
  /// APU (Audio Processing Unit)
  apu: APU,
}

impl Default for Bus {
  fn default() -> Self {
    Self::new()
  }
}

impl Bus {
  pub fn new() -> Bus {
    Bus {
      memory: [0; (RAM_ACTUAL_END_ADDRESS + 1) as usize],
      cartridge: None,
      clock_counter: 0,
      ppu: PPU::new(),
      controller1_state: 0,
      controller2_state: 0,
      controller1_shift: 0,
      controller2_shift: 0,
      controller_strobe: false,
      apu: APU::new(),
    }
  }

  /// Set the button state for controller 1.
  /// Use the BUTTON_* constants to set individual buttons.
  pub fn set_controller1(&mut self, state: u8) {
    self.controller1_state = state;
    // If strobe is high, immediately update shift register
    if self.controller_strobe {
      self.controller1_shift = state;
    }
  }

  /// Set the button state for controller 2.
  pub fn set_controller2(&mut self, state: u8) {
    self.controller2_state = state;
    if self.controller_strobe {
      self.controller2_shift = state;
    }
  }

  /// Press a button on controller 1.
  pub fn press_button1(&mut self, button: u8) {
    self.controller1_state |= button;
    if self.controller_strobe {
      self.controller1_shift = self.controller1_state;
    }
  }

  /// Release a button on controller 1.
  pub fn release_button1(&mut self, button: u8) {
    self.controller1_state &= !button;
    if self.controller_strobe {
      self.controller1_shift = self.controller1_state;
    }
  }

  #[named]
  pub fn inner_read_u8(&mut self, address: u16) -> u8 {
    trace_enter!();
    trace_u16!(address);
    let result = match address {
      RAM_START_ADDRESS..=RAM_RANGE_END_ADDRESS => {
        let actual_address = address & RAM_ACTUAL_END_ADDRESS;
        trace_u16!(actual_address);
        self.memory[actual_address as usize]
      }
      PPU_REGISTER_START_ADDRESS..=PPU_REGISTER_RANGE_END_ADDRESS => {
        let actual_address = address & PPU_REGISTER_ACTUAL_END_ADDRESS;
        trace_u16!(actual_address);
        let index = (actual_address % 8) as u8;
        self.ppu.read_register(index)
      }
      APU_IO_START_ADDRESS..=APU_IO_END_ADDRESS => {
        match address {
          0x4016 => {
            // Controller 1 read - return bit 0 of shift register, then shift
            // Upper bits return open bus (typically last value on bus)
            let result = (self.controller1_shift & 1) | 0x40;
            // Shift right (next read gets next button)
            // After all 8 buttons are read, subsequent reads return 1
            self.controller1_shift = (self.controller1_shift >> 1) | 0x80;
            result
          }
          0x4017 => {
            // Controller 2 read (not APU frame counter read!)
            let result = (self.controller2_shift & 1) | 0x40;
            self.controller2_shift = (self.controller2_shift >> 1) | 0x80;
            result
          }
          0x4015 => {
            // APU status register
            self.apu.read_register(address)
          }
          _ => {
            // Other APU registers return open bus (not readable)
            0
          }
        }
      }
      CARTRIDGE_START_ADDRESS..=CARTRIDGE_END_ADDRESS => {
        if let Some(ref cartridge) = self.cartridge {
          cartridge.borrow().read_prg_u8(address)
        } else {
          (address >> 8) as u8
        }
      }
    };
    trace_u8!(result);
    trace_exit!();
    result
  }

  #[named]
  pub fn inner_write_u8(&mut self, address: u16, value: u8) {
    trace_enter!();
    trace_u16!(address);
    trace_u8!(value);
    match address {
      RAM_START_ADDRESS..=RAM_RANGE_END_ADDRESS => {
        let actual_address = address & RAM_ACTUAL_END_ADDRESS;
        trace_u16!(actual_address);
        self.memory[actual_address as usize] = value;
      }
      PPU_REGISTER_START_ADDRESS..=PPU_REGISTER_RANGE_END_ADDRESS => {
        let actual_address = address & PPU_REGISTER_ACTUAL_END_ADDRESS;
        trace_u16!(actual_address);
        let index = (actual_address % 8) as u8;
        self.ppu.write_register(index, value);
      }
      APU_IO_START_ADDRESS..=APU_IO_END_ADDRESS => {
        if address == OAM_DMA_ADDRESS {
          // OAM DMA: copy 256 bytes from CPU page XX00-XXFF to OAM
          let source_page = (value as u16) << 8;
          let mut oam_data = [0u8; 256];
          for i in 0..256u16 {
            let src_addr = source_page | i;
            oam_data[i as usize] = self.inner_read_u8(src_addr);
          }
          self.ppu.write_oam_dma(&oam_data);
          // Note: Real OAM DMA takes 513-514 CPU cycles
          // TODO: Add cycle-accurate DMA timing
        } else if address == 0x4016 {
          // Controller strobe
          let new_strobe = (value & 1) != 0;
          // When strobe goes from high to low, latch current button state
          if self.controller_strobe && !new_strobe {
            self.controller1_shift = self.controller1_state;
            self.controller2_shift = self.controller2_state;
          }
          self.controller_strobe = new_strobe;
          // While strobe is high, continuously reload
          if self.controller_strobe {
            self.controller1_shift = self.controller1_state;
            self.controller2_shift = self.controller2_state;
          }
        } else {
          // All other APU registers ($4000-$4013, $4015, $4017)
          self.apu.write_register(address, value);
        }
      }
      CARTRIDGE_START_ADDRESS..=CARTRIDGE_END_ADDRESS => {
        if let Some(ref cartridge) = self.cartridge {
          cartridge.borrow_mut().write_prg_u8(address, value);
        }
      }
    }
    trace_exit!();
  }

  #[named]
  pub fn load_cartridge_data(&mut self, data: &[u8]) {
    trace_enter!();
    info!("Loading cartridge from data...");
    let cartridge = Rc::new(RefCell::new(Cartridge::new(data)));
    self.ppu.vram.set_cartridge(cartridge.clone());
    self.cartridge = Some(cartridge);
    trace_exit!();
  }

  /// Get a reference to the PPU's framebuffer.
  pub fn get_framebuffer(&self) -> &[u8] {
    self.ppu.get_framebuffer()
  }

  /// Check if a new frame is ready and clear the flag.
  pub fn take_frame_ready(&mut self) -> bool {
    self.ppu.take_frame_ready()
  }

  /// Get the current PPU scanline (0-261).
  pub fn get_ppu_scanline(&self) -> u16 {
    self.ppu.scanline
  }

  /// Get the current PPU dot (0-340).
  pub fn get_ppu_dot(&self) -> u16 {
    self.ppu.dot
  }

  /// Get debug info about PPU state
  #[cfg(test)]
  pub fn get_ppu_debug_info(&self) -> String {
    format!(
      "PPU Debug:\n  CTRL: 0x{:02X}\n  MASK: 0x{:02X}\n  STATUS: 0x{:02X}\n  show_bg: {}\n  show_sprites: {}\n  scanline: {}\n  dot: {}",
      self.ppu.control_register.read_u8(),
      self.ppu.mask_register.read_u8(),
      self.ppu.status_register.read_u8(),
      self.ppu.mask_register.get_show_background_flag(),
      self.ppu.mask_register.get_show_sprites_flag(),
      self.ppu.scanline,
      self.ppu.dot
    )
  }

  /// Print comprehensive debug dump to stderr.
  pub fn debug_dump(&self) {
    eprintln!("=== GREENSTONE DEBUG DUMP ===");

    // PPU registers
    let ctrl = self.ppu.control_register.read_u8();
    let mask = self.ppu.mask_register.read_u8();
    let status = self.ppu.status_register.read_u8();
    eprintln!(
      "PPU CTRL:   0x{:02X}  (NMI={}, BG_table={}, SPR_table={})",
      ctrl,
      if ctrl & 0x80 != 0 { "ON" } else { "off" },
      if ctrl & 0x10 != 0 { "$1000" } else { "$0000" },
      if ctrl & 0x08 != 0 { "$1000" } else { "$0000" }
    );
    eprintln!(
      "PPU MASK:   0x{:02X}  (show_bg={}, show_spr={}, left_bg={}, left_spr={})",
      mask,
      self.ppu.mask_register.get_show_background_flag(),
      self.ppu.mask_register.get_show_sprites_flag(),
      self.ppu.mask_register.get_show_background_left_flag(),
      self.ppu.mask_register.get_show_sprites_left_flag()
    );
    eprintln!(
      "PPU STATUS: 0x{:02X}  (vblank={}, spr0_hit={}, overflow={})",
      status,
      self.ppu.status_register.get_vertical_blank_flag(),
      self.ppu.status_register.get_sprite_zero_hit_flag(),
      self.ppu.status_register.get_sprite_overflow_flag()
    );
    eprintln!("PPU scan/dot: {}/{}", self.ppu.scanline, self.ppu.dot);

    // Palette
    eprint!("Palette BG: ");
    for i in 0..16 {
      eprint!("{:02X} ", self.ppu.vram.palettes[i]);
    }
    eprintln!();
    eprint!("Palette SP: ");
    for i in 16..32 {
      eprint!("{:02X} ", self.ppu.vram.palettes[i]);
    }
    eprintln!();

    // Nametable 0, first 4 rows (to see if text was written)
    eprintln!("Nametable 0 (first 4 rows, non-zero tiles shown):");
    for row in 0..4 {
      let mut any_nonzero = false;
      for col in 0..32 {
        let tile = self.ppu.vram.nametables[row * 32 + col];
        if tile != 0 {
          any_nonzero = true;
        }
      }
      if any_nonzero {
        eprint!("  row {:2}: ", row);
        for col in 0..32 {
          let tile = self.ppu.vram.nametables[row * 32 + col];
          if tile != 0 {
            eprint!("{:02X} ", tile);
          } else {
            eprint!(".. ");
          }
        }
        eprintln!();
      }
    }

    // Check rows 10 and 14 specifically (title screen text positions)
    eprintln!("Nametable 0 (rows 10, 14 — title screen text):");
    for row in [10, 14] {
      eprint!("  row {:2}: ", row);
      for col in 0..32 {
        let tile = self.ppu.vram.nametables[row * 32 + col];
        if tile != 0 {
          eprint!("{:02X} ", tile);
        } else {
          eprint!(".. ");
        }
      }
      eprintln!();
    }

    // Count total non-zero nametable entries
    let nonzero_count = self
      .ppu
      .vram
      .nametables
      .iter()
      .take(960) // 32*30 = 960 tiles in one nametable
      .filter(|&&t| t != 0)
      .count();
    eprintln!("Nametable 0: {}/960 non-zero tiles", nonzero_count);

    // Pattern table spot check (first few tiles to verify CHR loaded)
    eprintln!("CHR spot check (tile $20='space', $53='S'):");
    if let Some(ref cartridge) = self.cartridge {
      let cart = cartridge.borrow();
      // Tile $20 (space) - should be blank
      let t20_byte0 = cart.read_chr_u8(0x20 * 16);
      let t20_byte1 = cart.read_chr_u8(0x20 * 16 + 8);
      eprintln!("  Tile $20 plane0[0]: {:02X}  plane1[0]: {:02X}", t20_byte0, t20_byte1);
      // Tile $53 ('S') - should have pixel data
      let t53_byte0 = cart.read_chr_u8(0x53 * 16);
      let t53_byte1 = cart.read_chr_u8(0x53 * 16 + 8);
      eprintln!("  Tile $53 plane0[0]: {:02X}  plane1[0]: {:02X}", t53_byte0, t53_byte1);
    } else {
      eprintln!("  No cartridge loaded!");
    }

    // CPU RAM spot check: ZP $02-$03 (cc65 sp), first few BSS bytes
    eprintln!("CPU RAM:");
    eprint!("  ZP $00-$1F: ");
    for i in 0..0x20 {
      eprint!("{:02X} ", self.memory[i]);
    }
    eprintln!();
    eprint!("  $0300-$031F: ");
    for i in 0x0300..0x0320 {
      eprint!("{:02X} ", self.memory[i]);
    }
    eprintln!();

    // Framebuffer sample (check what's actually being displayed)
    let fb = self.ppu.get_framebuffer();
    if fb.len() >= 3 {
      // Sample a few pixels: top-left, center of row 10, center of row 14
      let tl = (fb[0], fb[1], fb[2]);
      // Sample at title text: 'S' at row 10, col 6 → pixel (52, 83) (middle of 8px tall tile)
      let text_s = if fb.len() > (83 * 256 + 52) * 3 + 2 {
        let i = (83 * 256 + 52) * 3;
        (fb[i], fb[i + 1], fb[i + 2])
      } else {
        (0, 0, 0)
      };
      // Sample at 'P' of PRESS START: row 14, col 10 → pixel (84, 115)
      let text_p = if fb.len() > (115 * 256 + 84) * 3 + 2 {
        let i = (115 * 256 + 84) * 3;
        (fb[i], fb[i + 1], fb[i + 2])
      } else {
        (0, 0, 0)
      };
      eprintln!("Framebuffer samples:");
      eprintln!("  (0,0): RGB({},{},{})", tl.0, tl.1, tl.2);
      eprintln!("  S@(52,83): RGB({},{},{})", text_s.0, text_s.1, text_s.2);
      eprintln!("  P@(84,115): RGB({},{},{})", text_p.0, text_p.1, text_p.2);
      // Count unique colors
      let mut black_count = 0u32;
      let mut nonblack_count = 0u32;
      for pixel in fb.chunks(3) {
        if pixel[0] == 0 && pixel[1] == 0 && pixel[2] == 0 {
          black_count += 1;
        } else {
          nonblack_count += 1;
        }
      }
      eprintln!("  Black pixels: {}, Non-black: {}", black_count, nonblack_count);
    }

    eprintln!("=== END DEBUG DUMP ===");
  }

  /// Take accumulated audio samples from the APU.
  pub fn take_audio_samples(&mut self) -> Vec<f32> {
    self.apu.take_samples()
  }

  /// Check if the APU is requesting an IRQ.
  pub fn apu_irq_pending(&self) -> bool {
    self.apu.irq_pending()
  }

  /// Reconnect the shared Rc<RefCell<Cartridge>> to the PPU's VRAM.
  /// Must be called after deserialization to restore the shared reference.
  pub fn reconnect_cartridge(&mut self) {
    if let Some(ref cartridge) = self.cartridge {
      self.ppu.vram.set_cartridge(cartridge.clone());
    }
  }
}
