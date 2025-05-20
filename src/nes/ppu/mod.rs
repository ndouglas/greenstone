pub mod address;
pub use address::*;

pub mod registers;
pub use registers::*;

pub mod vram;
pub use vram::*;

#[cfg(test)]
mod tests;

pub const CONTROL_REGISTER_INDEX: u8 = 0;
pub const MASK_REGISTER_INDEX: u8 = 1;
pub const STATUS_REGISTER_INDEX: u8 = 2;
pub const OAM_ADDRESS_REGISTER_INDEX: u8 = 3;
pub const OAM_DATA_REGISTER_INDEX: u8 = 4;
pub const SCROLL_REGISTER_INDEX: u8 = 5;
pub const ADDRESS_REGISTER_INDEX: u8 = 6;
pub const DATA_REGISTER_INDEX: u8 = 7;

pub const OAM_RAM_SIZE: u16 = 256;

/// NES screen dimensions
pub const SCREEN_WIDTH: usize = 256;
pub const SCREEN_HEIGHT: usize = 240;

/// PPU timing constants
pub const DOTS_PER_SCANLINE: u16 = 341;
pub const SCANLINES_PER_FRAME: u16 = 262;
pub const VBLANK_SCANLINE: u16 = 241;
pub const PRE_RENDER_SCANLINE: u16 = 261;

/// NES Picture-Processing Unit
pub struct PPU {
  /// Control Register.
  pub control_register: ControlRegister,
  /// Mask Register.
  pub mask_register: MaskRegister,
  /// Status Register.
  pub status_register: StatusRegister,
  /// OAM Address Register.
  pub oam_address: u8,
  /// The PPU <-> CPU data bus, latching but constantly decaying...
  pub latching_bus: u8,
  /// Video RAM.
  pub vram: VRAM,
  /// OAM RAM.
  pub oam_ram: Vec<u8>,
  /// Whether we're forcing an NMI.
  pub force_nmi: bool,
  /// Whether we're currently latched.
  pub is_latched: bool,
  /// Loopy "t" address.
  pub t_address: Address,
  /// Loopy "v" address.
  pub v_address: Address,
  /// Fine "x" coordinate for scrolling, etc.
  pub fine_x: u8,
  /// Suppress vertical blank signals.
  pub suppress_vblanks: bool,
  /// Current dot (0-340) within the scanline.
  pub dot: u16,
  /// Current scanline (0-261).
  pub scanline: u16,
  /// Whether an NMI is pending (set at vblank, cleared when acknowledged).
  pub nmi_pending: bool,
  /// Frame counter for debugging/timing.
  pub frame_count: u64,
  /// Framebuffer: 256x240 pixels, RGB format (3 bytes per pixel).
  pub framebuffer: Vec<u8>,
  /// Whether a new frame is ready for display.
  pub frame_ready: bool,
  /// Secondary OAM: holds up to 8 sprites for the current scanline.
  /// Each sprite is 4 bytes (Y, tile, attributes, X), so 32 bytes total.
  pub secondary_oam: [u8; 32],
  /// Number of sprites in secondary OAM for the current scanline (0-8).
  pub sprites_on_scanline: u8,
  /// Whether sprite 0 is in secondary OAM (for sprite zero hit detection).
  pub sprite_zero_on_scanline: bool,
  /// Cached horizontal scroll values at the start of each scanline.
  /// These are captured when copy_x happens at dot 257 of the previous scanline.
  /// Using these instead of reading t_address directly prevents mid-frame
  /// PPUSCROLL writes from corrupting the current scanline's rendering.
  pub scanline_coarse_x: u8,
  pub scanline_nametable_x: u8,
}

/// Sprite pixel result containing color and metadata for compositing.
#[derive(Debug, Clone, Copy)]
struct SpritePixel {
  /// Color index from palette RAM (0x3F10-0x3F1F for sprites).
  color: u8,
  /// True if this is sprite 0 (for sprite zero hit detection).
  is_sprite_zero: bool,
  /// True if sprite has behind-background priority (attribute bit 5).
  behind_background: bool,
}

impl PPU {
  pub fn new() -> PPU {
    PPU {
      control_register: ControlRegister::new(),
      mask_register: MaskRegister::new(),
      status_register: StatusRegister::new(),
      oam_address: 0x00,
      latching_bus: 0x00,
      vram: VRAM::new(),
      oam_ram: vec![0; OAM_RAM_SIZE as usize],
      force_nmi: false,
      is_latched: false,
      t_address: Address(0),
      v_address: Address(0),
      fine_x: 0,
      suppress_vblanks: false,
      dot: 0,
      scanline: 0,
      nmi_pending: false,
      frame_count: 0,
      framebuffer: vec![0; SCREEN_WIDTH * SCREEN_HEIGHT * 3],
      frame_ready: false,
      secondary_oam: [0xFF; 32],
      sprites_on_scanline: 0,
      sprite_zero_on_scanline: false,
      scanline_coarse_x: 0,
      scanline_nametable_x: 0,
    }
  }

  #[named]
  #[inline]
  pub fn read_register(&mut self, index: u8) -> u8 {
    trace_enter!();
    trace_u8!(index);
    let result = match index {
      STATUS_REGISTER_INDEX => self.read_status_register(),
      OAM_DATA_REGISTER_INDEX => self.read_oam_data_register(),
      DATA_REGISTER_INDEX => self.read_data_register(),
      // By default, reads (including of write-only registers), return the
      // value on the decaying latching data bus.
      _ => self.latching_bus,
    };
    self.latching_bus = result;
    trace_result!(result);
    result
  }

  #[named]
  #[inline]
  pub fn write_register(&mut self, index: u8, value: u8) {
    trace_enter!();
    trace_u16!(index);
    trace_u8!(value);
    self.latching_bus = value;
    match index {
      CONTROL_REGISTER_INDEX => self.write_control_register(value),
      MASK_REGISTER_INDEX => self.write_mask_register(value),
      // Read-only!
      STATUS_REGISTER_INDEX => (),
      OAM_ADDRESS_REGISTER_INDEX => self.write_oam_address_register(value),
      OAM_DATA_REGISTER_INDEX => self.write_oam_data_register(value),
      SCROLL_REGISTER_INDEX => self.write_scroll_register(value),
      ADDRESS_REGISTER_INDEX => self.write_address_register(value),
      DATA_REGISTER_INDEX => self.write_data_register(value),
      _ => (),
    };
    trace_exit!();
  }

  #[named]
  #[inline]
  pub fn should_force_nmi(&self, new_control: u8) -> bool {
    trace_enter!();
    trace_u8!(new_control);
    let new_control_generates_nmi = (new_control & GENERATE_NMI_FLAG) > 0;
    trace_var!(new_control_generates_nmi);
    let current_control_generates_nmi = self.control_register.get_generate_nmi_flag();
    trace_var!(current_control_generates_nmi);
    let vblank_flag_set = self.status_register.get_vertical_blank_flag();
    trace_var!(vblank_flag_set);
    // NMI fires immediately if:
    // 1. NMI was disabled and is now being enabled
    // 2. AND the VBlank flag is currently set
    let result = !current_control_generates_nmi && new_control_generates_nmi && vblank_flag_set;
    trace_var!(result);
    trace_exit!();
    result
  }

  #[named]
  #[inline]
  pub fn write_control_register(&mut self, value: u8) {
    trace_enter!();
    trace_u8!(value);
    if self.should_force_nmi(value) {
      self.force_nmi = true;
    }
    self.control_register.write_u8(value);
    self.t_address.set_nametable(self.control_register.get_nametable());
    trace_exit!();
  }

  #[named]
  #[inline]
  pub fn write_mask_register(&mut self, value: u8) {
    trace_enter!();
    trace_u8!(value);
    self.mask_register.write_u8(value);
    trace_exit!();
  }

  #[named]
  #[inline]
  pub fn write_oam_address_register(&mut self, value: u8) {
    trace_enter!();
    trace_u8!(value);
    self.oam_address = value;
    trace_exit!();
  }

  #[named]
  #[inline]
  pub fn write_oam_data_register(&mut self, value: u8) {
    trace_enter!();
    trace_u8!(value);
    self.oam_ram[self.oam_address as usize] = value;
    self.oam_address = self.oam_address.wrapping_add(1);
    trace_exit!();
  }

  #[named]
  #[inline]
  pub fn read_oam_data_register(&mut self) -> u8 {
    trace_enter!();
    let result = self.oam_ram[self.oam_address as usize];
    trace_u8!(result);
    trace_exit!();
    result
  }

  #[named]
  #[inline]
  pub fn write_address_register(&mut self, value: u8) {
    trace_enter!();
    trace_u8!(value);
    if self.is_latched {
      self.t_address.set_low_byte(value);
      self.v_address = self.t_address.clone();
    } else {
      self.t_address.set_high_byte(value);
      // NES hardware clears bit 14 on the first PPUADDR write
      self.t_address.0 &= !0x4000;
    }
    trace_u16!(self.v_address.address());
    trace_u16!(self.t_address.address());
    self.is_latched = !self.is_latched;
    trace_var!(self.is_latched);
    trace_exit!();
  }

  #[named]
  #[inline]
  pub fn write_status_register(&mut self, value: u8) {
    trace_enter!();
    trace_u8!(value);
    self.status_register.write_u8(value);
    trace_exit!();
  }

  #[named]
  #[inline]
  pub fn write_data_register(&mut self, value: u8) {
    trace_enter!();
    trace_u8!(value);
    self.vram.write_u8(self.v_address.address(), value);
    self.v_address.increment(self.control_register.get_vram_address_increment() as u16);
    trace_exit!();
  }

  #[named]
  #[inline]
  pub fn write_scroll_register(&mut self, value: u8) {
    trace_enter!();
    trace_u8!(value);
    if self.is_latched {
      self.t_address.set_fine_y(value);
      self.t_address.set_coarse_y(value >> 3);
    } else {
      self.fine_x = value & 0b0000_0111;
      self.t_address.set_coarse_x(value >> 3);
    }
    trace_u16!(self.fine_x);
    trace_u16!(self.t_address.address());
    self.is_latched = !self.is_latched;
    trace_exit!();
  }

  #[named]
  #[inline]
  pub fn read_status_register(&mut self) -> u8 {
    trace_enter!();
    let mut result = self.status_register.read_u8();
    self.status_register.set_vertical_blank_flag(false);
    self.is_latched = false;
    // VBlank suppression race condition:
    // - Reading at dot 0 of scanline 241: Flag is read as 0, VBlank is suppressed
    // - Reading at dots 1-2 of scanline 241: Flag is read as set, but NMI is suppressed
    // This implements the NES "race condition" behavior.
    if self.scanline == VBLANK_SCANLINE && self.dot == 0 {
      // Reading at the exact cycle before VBlank is set - suppress the flag entirely
      self.suppress_vblanks = true;
    }
    if self.scanline == VBLANK_SCANLINE && self.dot >= 1 && self.dot <= 2 {
      // Reading shortly after VBlank starts - flag was set but we suppress NMI
      // This clears any pending NMI that was set when VBlank started
      self.nmi_pending = false;
    }
    // Status register includes some garbage from the bus.
    result = result | (self.latching_bus & 0b0001_1111);
    trace_result!(result);
    result
  }

  #[named]
  #[inline]
  pub fn read_data_register(&mut self) -> u8 {
    trace_enter!();
    let address = self.v_address.address();
    self.v_address.increment(self.control_register.get_vram_address_increment() as u16);
    let result = self.vram.buffered_read_u8(address);
    trace_result!(result);
    result
  }

  #[named]
  #[inline]
  pub fn write_oam_dma(&mut self, data: &[u8; 256]) {
    for x in data.iter() {
      self.oam_ram[self.oam_address as usize] = *x;
      self.oam_address = self.oam_address.wrapping_add(1);
    }
  }

  #[named]
  #[inline]
  pub fn tick(&mut self) {
    trace_enter!();

    // Check if rendering is enabled
    let rendering_enabled =
      self.mask_register.get_show_background_flag() || self.mask_register.get_show_sprites_flag();

    // Sprite evaluation: evaluate sprites for current scanline at dot 0
    // Real hardware does this incrementally during dots 65-256 of the previous scanline,
    // but for correctness we evaluate at the start of each visible scanline.
    if self.scanline < 240 && self.dot == 0 {
      self.evaluate_sprites();
    }

    // Handle rendering for visible scanlines (0-239)
    if self.scanline < 240 {
      self.render_pixel();
    }

    // Advance dot counter first, then process events for the new dot
    self.dot += 1;

    // NTSC odd frame cycle skip:
    // On odd frames with rendering enabled, the pre-render scanline is one cycle shorter.
    // The PPU skips from dot 339 directly to dot 0 of the next frame (skipping dot 340).
    let is_odd_frame = self.frame_count % 2 == 1;
    if rendering_enabled
      && is_odd_frame
      && self.scanline == PRE_RENDER_SCANLINE
      && self.dot == 340
    {
      // Skip dot 340 - go directly to the next frame
      self.dot = 0;
      self.scanline = 0;
      self.frame_count = self.frame_count.wrapping_add(1);
      self.frame_ready = true;
    } else if self.dot >= DOTS_PER_SCANLINE {
      // Normal end of scanline handling
      self.dot = 0;
      self.scanline += 1;

      // Handle end of frame
      if self.scanline >= SCANLINES_PER_FRAME {
        self.scanline = 0;
        self.frame_count = self.frame_count.wrapping_add(1);
        self.frame_ready = true;
      }
    }

    // Scroll register updates during rendering
    // These happen on visible scanlines (0-239) and pre-render scanline (261)
    let is_render_scanline = self.scanline < 240 || self.scanline == PRE_RENDER_SCANLINE;

    if rendering_enabled && is_render_scanline {
      // Coarse X increment every 8 dots during visible portion
      // On real hardware this happens at dots 8, 16, ..., 256, but since we check after
      // dot increment and need scroll_x to happen AFTER the tile's last pixel is rendered,
      // we check for dots 9, 17, ..., 257 (i.e., dot % 8 == 1 and dot >= 9 and dot <= 257)
      // Also at dots 328 and 336 for next scanline tile prefetch (check 329, 337)
      if (self.dot >= 9 && self.dot <= 257 && self.dot % 8 == 1)
        || self.dot == 329
        || self.dot == 337
      {
        self.v_address.scroll_x();
      }

      // Fine Y increment at dot 256 (we check for 257 because this runs after dot increment,
      // and we need scroll_y to happen AFTER pixel 255 is rendered at dot 256)
      if self.dot == 257 {
        self.v_address.scroll_y();
      }

      // Horizontal bits reload from t to v at dot 257
      if self.dot == 257 {
        self.v_address.copy_x(self.t_address);
        // Cache horizontal scroll values for the next scanline's rendering.
        // This prevents mid-frame PPUSCROLL writes from affecting the current scanline.
        self.scanline_coarse_x = self.v_address.coarse_x();
        self.scanline_nametable_x = self.v_address.nametable() & 1;
      }

      // Vertical bits reload from t to v during pre-render scanline dots 280-304
      if self.scanline == PRE_RENDER_SCANLINE && self.dot >= 280 && self.dot <= 304 {
        self.v_address.copy_y(self.t_address);
      }
    }

    // Vblank start: scanline 241, dot 1
    if self.scanline == VBLANK_SCANLINE && self.dot == 1 {
      if !self.suppress_vblanks {
        self.status_register.set_vertical_blank_flag(true);
        if self.control_register.get_generate_nmi_flag() {
          self.nmi_pending = true;
        }
      }
      self.suppress_vblanks = false;
    }

    // Handle force_nmi (set when NMI enable is turned on during vblank)
    if self.force_nmi {
      self.nmi_pending = true;
      self.force_nmi = false;
    }

    // Pre-render line: clear flags at dot 1
    if self.scanline == PRE_RENDER_SCANLINE && self.dot == 1 {
      self.status_register.set_vertical_blank_flag(false);
      self.status_register.set_sprite_zero_hit_flag(false);
      self.status_register.set_sprite_overflow_flag(false);
      self.frame_ready = false;
    }

    trace_exit!();
  }

  /// Render a single pixel to the framebuffer.
  /// This is called during visible scanlines (0-239) at dots 1-256.
  #[named]
  #[inline]
  fn render_pixel(&mut self) {
    // Only render during the visible portion (dots 1-256)
    if self.dot == 0 || self.dot > 256 {
      return;
    }

    let x = (self.dot - 1) as usize;
    let y = self.scanline as usize;

    // Get background pixel info
    let bg_color = self.get_background_pixel(x, y);
    let bg_is_opaque = self.is_background_pixel_opaque(x, y);

    // Get sprite pixel info
    let sprite_pixel = self.get_sprite_pixel(x, y);

    // Sprite zero hit detection
    // Conditions for sprite zero hit:
    // 1. Both background and sprite rendering are enabled
    // 2. Both background and sprite 0 have opaque pixels at this location
    // 3. x != 255 (hardware quirk)
    // 4. Not in the left 8 pixels if either left clipping is enabled
    if let Some(ref sp) = sprite_pixel {
      if sp.is_sprite_zero
        && bg_is_opaque
        && x != 255
        && !self.status_register.get_sprite_zero_hit_flag()
      {
        // Additional clipping check for sprite zero hit
        let bg_clipped = x < 8 && !self.mask_register.get_show_background_left_flag();
        let sp_clipped = x < 8 && !self.mask_register.get_show_sprites_left_flag();

        if !bg_clipped && !sp_clipped {
          self.status_register.set_sprite_zero_hit_flag(true);
        }
      }
    }

    // Composite sprite and background based on priority
    let final_color = match sprite_pixel {
      Some(sp) => {
        if sp.behind_background && bg_is_opaque {
          // Sprite is behind background, and background is opaque - show background
          bg_color
        } else {
          // Sprite is in front, or background is transparent - show sprite
          sp.color
        }
      }
      None => {
        // No sprite at this pixel - show background
        bg_color
      }
    };

    // Write to framebuffer (RGB format)
    let pixel_index = (y * SCREEN_WIDTH + x) * 3;
    if pixel_index + 2 < self.framebuffer.len() {
      let (r, g, b) = self.palette_to_rgb(final_color);
      self.framebuffer[pixel_index] = r;
      self.framebuffer[pixel_index + 1] = g;
      self.framebuffer[pixel_index + 2] = b;
    }
  }

  /// Check if the background pixel at (x, y) is opaque (non-zero).
  /// Used for sprite zero hit detection and priority.
  fn is_background_pixel_opaque(&mut self, x: usize, _y: usize) -> bool {
    // If background is disabled, treat as transparent
    if !self.mask_register.get_show_background_flag() {
      return false;
    }

    // If left-edge clipping, treat as transparent
    if x < 8 && !self.mask_register.get_show_background_left_flag() {
      return false;
    }

    // Get the actual pixel value (0 = transparent, 1-3 = opaque)
    // This duplicates some logic from get_background_pixel but avoids the palette lookup
    let coarse_y = self.v_address.coarse_y();
    let fine_y = self.v_address.fine_y();
    let nametable = self.v_address.nametable();

    // Use cached horizontal scroll values (captured at dot 257 of previous scanline)
    let start_coarse_x = self.scanline_coarse_x as usize;
    let start_nametable_x = self.scanline_nametable_x as usize;
    let total_x = start_coarse_x * 8 + self.fine_x as usize + x;
    let tile_x = (total_x / 8) % 32;
    let nametable_x = ((total_x / 256) + start_nametable_x) % 2;

    let nametable_y = (nametable >> 1) & 1;
    let current_nametable = (nametable_y << 1) | nametable_x as u8;

    let nametable_base = 0x2000u16 + (current_nametable as u16 * 0x400);
    let nametable_addr = nametable_base + (coarse_y as u16 * 32) + tile_x as u16;

    let tile_index = self.vram.read_u8(nametable_addr);

    let pattern_base: u16 = if self.control_register.get_background_address_flag() {
      0x1000
    } else {
      0x0000
    };

    let pattern_addr = pattern_base + (tile_index as u16 * 16) + fine_y as u16;
    let pattern_low = self.vram.read_u8(pattern_addr);
    let pattern_high = self.vram.read_u8(pattern_addr + 8);

    let bit_position = 7 - ((self.fine_x as usize + x) % 8);
    let pixel_low = (pattern_low >> bit_position) & 1;
    let pixel_high = (pattern_high >> bit_position) & 1;
    let pixel_value = (pixel_high << 1) | pixel_low;

    pixel_value != 0
  }

  /// Get the background pixel color index at the given screen position.
  ///
  /// Uses v_address for vertical scroll (updated per-scanline) and calculates
  /// horizontal position from fine_x and the screen x coordinate.
  #[named]
  fn get_background_pixel(&mut self, x: usize, _y: usize) -> u8 {
    // Check if background rendering is enabled
    if !self.mask_register.get_show_background_flag() {
      return self.vram.read_u8(0x3F00);
    }

    // Check left-edge clipping
    if x < 8 && !self.mask_register.get_show_background_left_flag() {
      return self.vram.read_u8(0x3F00);
    }

    // Use v_address for vertical position (updated per-scanline by scroll logic)
    // Use t_address for horizontal starting position (restored at dot 257)
    let coarse_y = self.v_address.coarse_y();
    let fine_y = self.v_address.fine_y();
    let nametable = self.v_address.nametable();

    // Calculate horizontal tile position
    // Use cached horizontal scroll values (captured at dot 257 of previous scanline)
    // This prevents mid-frame PPUSCROLL writes from affecting the current scanline.
    // For screen pixel x: total_x = coarse_x * 8 + fine_x + x
    // tile_x = total_x / 8 = coarse_x + (fine_x + x) / 8
    let start_coarse_x = self.scanline_coarse_x as usize;
    let start_nametable_x = self.scanline_nametable_x as usize;
    let total_x = start_coarse_x * 8 + self.fine_x as usize + x;
    let tile_x = (total_x / 8) % 32;
    let nametable_x = ((total_x / 256) + start_nametable_x) % 2;

    // Build nametable index
    let nametable_y = (nametable >> 1) & 1;
    let current_nametable = (nametable_y << 1) | nametable_x as u8;

    // Calculate nametable address
    let nametable_base = 0x2000u16 + (current_nametable as u16 * 0x400);
    let nametable_addr = nametable_base + (coarse_y as u16 * 32) + tile_x as u16;

    // Read tile index from nametable
    let tile_index = self.vram.read_u8(nametable_addr);

    // Get pattern table base from control register
    let pattern_base: u16 = if self.control_register.get_background_address_flag() {
      0x1000
    } else {
      0x0000
    };

    // Read pattern data (two bit planes)
    let pattern_addr = pattern_base + (tile_index as u16 * 16) + fine_y as u16;
    let pattern_low = self.vram.read_u8(pattern_addr);
    let pattern_high = self.vram.read_u8(pattern_addr + 8);

    // Extract the 2-bit pixel value
    let bit_position = 7 - ((self.fine_x as usize + x) % 8);
    let pixel_low = (pattern_low >> bit_position) & 1;
    let pixel_high = (pattern_high >> bit_position) & 1;
    let pixel_value = (pixel_high << 1) | pixel_low;

    // Transparent pixel (0) shows backdrop color
    if pixel_value == 0 {
      return self.vram.read_u8(0x3F00);
    }

    // Get attribute byte for palette selection
    let attr_x = tile_x / 4;
    let attr_y = coarse_y as usize / 4;
    let attribute_addr = nametable_base + 0x3C0 + (attr_y as u16 * 8) + attr_x as u16;
    let attribute = self.vram.read_u8(attribute_addr);

    // Determine which 2-bit palette from the attribute byte
    // Each attribute byte covers 4x4 tiles, divided into 2x2 quadrants
    let quadrant_x = (tile_x % 4) / 2;
    let quadrant_y = (coarse_y as usize % 4) / 2;
    let palette_shift = (quadrant_y * 2 + quadrant_x) * 2;
    let palette_index = (attribute >> palette_shift) & 0x03;

    // Get final color from palette RAM
    let palette_addr = 0x3F00 + (palette_index as u16 * 4) + pixel_value as u16;
    self.vram.read_u8(palette_addr)
  }

  /// Convert a NES palette index to RGB values.
  fn palette_to_rgb(&self, palette_index: u8) -> (u8, u8, u8) {
    // NES palette - 64 colors (using a standard palette)
    const NES_PALETTE: [(u8, u8, u8); 64] = [
      (84, 84, 84),    // 0x00
      (0, 30, 116),    // 0x01
      (8, 16, 144),    // 0x02
      (48, 0, 136),    // 0x03
      (68, 0, 100),    // 0x04
      (92, 0, 48),     // 0x05
      (84, 4, 0),      // 0x06
      (60, 24, 0),     // 0x07
      (32, 42, 0),     // 0x08
      (8, 58, 0),      // 0x09
      (0, 64, 0),      // 0x0A
      (0, 60, 0),      // 0x0B
      (0, 50, 60),     // 0x0C
      (0, 0, 0),       // 0x0D
      (0, 0, 0),       // 0x0E
      (0, 0, 0),       // 0x0F
      (152, 150, 152), // 0x10
      (8, 76, 196),    // 0x11
      (48, 50, 236),   // 0x12
      (92, 30, 228),   // 0x13
      (136, 20, 176),  // 0x14
      (160, 20, 100),  // 0x15
      (152, 34, 32),   // 0x16
      (120, 60, 0),    // 0x17
      (84, 90, 0),     // 0x18
      (40, 114, 0),    // 0x19
      (8, 124, 0),     // 0x1A
      (0, 118, 40),    // 0x1B
      (0, 102, 120),   // 0x1C
      (0, 0, 0),       // 0x1D
      (0, 0, 0),       // 0x1E
      (0, 0, 0),       // 0x1F
      (236, 238, 236), // 0x20
      (76, 154, 236),  // 0x21
      (120, 124, 236), // 0x22
      (176, 98, 236),  // 0x23
      (228, 84, 236),  // 0x24
      (236, 88, 180),  // 0x25
      (236, 106, 100), // 0x26
      (212, 136, 32),  // 0x27
      (160, 170, 0),   // 0x28
      (116, 196, 0),   // 0x29
      (76, 208, 32),   // 0x2A
      (56, 204, 108),  // 0x2B
      (56, 180, 204),  // 0x2C
      (60, 60, 60),    // 0x2D
      (0, 0, 0),       // 0x2E
      (0, 0, 0),       // 0x2F
      (236, 238, 236), // 0x30
      (168, 204, 236), // 0x31
      (188, 188, 236), // 0x32
      (212, 178, 236), // 0x33
      (236, 174, 236), // 0x34
      (236, 174, 212), // 0x35
      (236, 180, 176), // 0x36
      (228, 196, 144), // 0x37
      (204, 210, 120), // 0x38
      (180, 222, 120), // 0x39
      (168, 226, 144), // 0x3A
      (152, 226, 180), // 0x3B
      (160, 214, 228), // 0x3C
      (160, 162, 160), // 0x3D
      (0, 0, 0),       // 0x3E
      (0, 0, 0),       // 0x3F
    ];

    let index = (palette_index & 0x3F) as usize;
    NES_PALETTE[index]
  }

  /// Get the sprite pixel at the given screen position.
  ///
  /// Returns Some(SpritePixel) if a sprite has an opaque pixel at this position,
  /// None if no sprite covers this pixel or all sprites are transparent here.
  ///
  /// Uses secondary OAM which contains only sprites on the current scanline (up to 8).
  /// Sprites are checked in secondary OAM order (preserves priority from primary OAM).
  fn get_sprite_pixel(&mut self, x: usize, y: usize) -> Option<SpritePixel> {
    // Check if sprite rendering is enabled
    if !self.mask_register.get_show_sprites_flag() {
      return None;
    }

    // Check left-edge clipping for sprites
    if x < 8 && !self.mask_register.get_show_sprites_left_flag() {
      return None;
    }

    // Get sprite size from control register
    let sprite_height: usize = if self.control_register.get_sprite_size_flag() {
      16 // 8x16 sprites
    } else {
      8 // 8x8 sprites
    };

    // Scan secondary OAM (only sprites on current scanline, up to 8)
    for sprite_index in 0..self.sprites_on_scanline as usize {
      let oam_offset = sprite_index * 4;

      // Read sprite data from secondary OAM
      let sprite_y = self.secondary_oam[oam_offset] as usize;
      let tile_index = self.secondary_oam[oam_offset + 1];
      let attributes = self.secondary_oam[oam_offset + 2];
      let sprite_x = self.secondary_oam[oam_offset + 3] as usize;

      // Check if this pixel is within the sprite's horizontal bounds
      if x < sprite_x || x >= sprite_x + 8 {
        continue;
      }

      // Calculate pixel position within the sprite
      // Note: OAM Y is the scanline BEFORE the sprite, so actual sprite top is Y+1
      let mut pixel_x = x - sprite_x;
      let mut pixel_y = y - sprite_y - 1;

      // Handle horizontal flip (attribute bit 6)
      if attributes & 0x40 != 0 {
        pixel_x = 7 - pixel_x;
      }

      // Handle vertical flip (attribute bit 7)
      if attributes & 0x80 != 0 {
        pixel_y = sprite_height - 1 - pixel_y;
      }

      // Get pattern table address
      let pattern_addr = if sprite_height == 16 {
        // 8x16 sprites: bit 0 of tile index selects pattern table
        let pattern_table = (tile_index & 1) as u16 * 0x1000;
        let tile = (tile_index & 0xFE) as u16;
        // Top half or bottom half of the 8x16 sprite
        let tile_offset = if pixel_y < 8 { tile } else { tile + 1 };
        let row = (pixel_y % 8) as u16;
        pattern_table + tile_offset * 16 + row
      } else {
        // 8x8 sprites: control register selects pattern table
        let pattern_table: u16 = if self.control_register.get_sprite_address_flag() {
          0x1000
        } else {
          0x0000
        };
        pattern_table + (tile_index as u16 * 16) + pixel_y as u16
      };

      // Read pattern data
      let pattern_low = self.vram.read_u8(pattern_addr);
      let pattern_high = self.vram.read_u8(pattern_addr + 8);

      // Extract pixel value (2 bits)
      let bit_position = 7 - pixel_x;
      let pixel_low = (pattern_low >> bit_position) & 1;
      let pixel_high = (pattern_high >> bit_position) & 1;
      let pixel_value = (pixel_high << 1) | pixel_low;

      // Skip transparent pixels (value 0)
      if pixel_value == 0 {
        continue;
      }

      // Get palette index from attributes (bits 0-1)
      let palette_index = attributes & 0x03;

      // Sprite palettes are at 0x3F10-0x3F1F (palettes 4-7)
      let palette_addr = 0x3F10 + (palette_index as u16 * 4) + pixel_value as u16;
      let color = self.vram.read_u8(palette_addr);

      // Check if this is sprite 0 (only sprite_index 0 in secondary OAM
      // if sprite 0 was found during evaluation)
      let is_sprite_zero = sprite_index == 0 && self.sprite_zero_on_scanline;

      return Some(SpritePixel {
        color,
        is_sprite_zero,
        behind_background: attributes & 0x20 != 0,
      });
    }

    None
  }

  #[named]
  pub fn reset(&mut self) {
    trace_enter!();
    self.control_register = ControlRegister::new();
    self.mask_register = MaskRegister::new();
    self.status_register = StatusRegister::new();
    self.vram.reset();
    self.oam_ram = vec![0; OAM_RAM_SIZE as usize];
    self.dot = 0;
    self.scanline = 0;
    self.nmi_pending = false;
    self.frame_count = 0;
    self.framebuffer = vec![0; SCREEN_WIDTH * SCREEN_HEIGHT * 3];
    self.frame_ready = false;
    self.secondary_oam = [0xFF; 32];
    self.sprites_on_scanline = 0;
    self.sprite_zero_on_scanline = false;
    self.scanline_coarse_x = 0;
    self.scanline_nametable_x = 0;
    trace_exit!();
  }

  /// Evaluate which sprites are on the current scanline.
  ///
  /// Scans primary OAM and copies up to 8 sprites to secondary OAM.
  /// Sets the sprite overflow flag if more than 8 sprites are found.
  fn evaluate_sprites(&mut self) {
    // Clear secondary OAM
    self.secondary_oam = [0xFF; 32];
    self.sprites_on_scanline = 0;
    self.sprite_zero_on_scanline = false;

    // If sprite rendering is disabled, don't evaluate
    if !self.mask_register.get_show_sprites_flag() {
      return;
    }

    // Get sprite height from control register
    let sprite_height: u16 = if self.control_register.get_sprite_size_flag() {
      16 // 8x16 sprites
    } else {
      8 // 8x8 sprites
    };

    let current_scanline = self.scanline;

    // Scan all 64 sprites in OAM
    for sprite_index in 0..64 {
      let oam_offset = sprite_index * 4;

      // Read sprite Y position
      // Note: OAM Y is the scanline BEFORE the sprite appears (off by 1)
      // So Y=0 means sprite top is at scanline 1, Y=1 means scanline 2, etc.
      let sprite_y = self.oam_ram[oam_offset] as u16 + 1;

      // Check if sprite is on current scanline
      if current_scanline >= sprite_y && current_scanline < sprite_y + sprite_height {
        if self.sprites_on_scanline < 8 {
          // Copy sprite to secondary OAM
          let secondary_offset = self.sprites_on_scanline as usize * 4;
          self.secondary_oam[secondary_offset] = self.oam_ram[oam_offset];
          self.secondary_oam[secondary_offset + 1] = self.oam_ram[oam_offset + 1];
          self.secondary_oam[secondary_offset + 2] = self.oam_ram[oam_offset + 2];
          self.secondary_oam[secondary_offset + 3] = self.oam_ram[oam_offset + 3];

          // Track if sprite 0 is on this scanline
          if sprite_index == 0 {
            self.sprite_zero_on_scanline = true;
          }

          self.sprites_on_scanline += 1;
        } else {
          // More than 8 sprites - set overflow flag
          // Note: Real hardware has a bug here (diagonal evaluation),
          // but we implement the correct behavior for now
          self.status_register.set_sprite_overflow_flag(true);
          break;
        }
      }
    }
  }

  /// Check if an NMI is pending and should be sent to the CPU.
  pub fn is_nmi_pending(&self) -> bool {
    self.nmi_pending
  }

  /// Acknowledge the NMI (called after CPU handles it).
  pub fn acknowledge_nmi(&mut self) {
    self.nmi_pending = false;
  }

  /// Get a reference to the framebuffer.
  pub fn get_framebuffer(&self) -> &[u8] {
    &self.framebuffer
  }

  /// Check if a new frame is ready and clear the flag.
  pub fn take_frame_ready(&mut self) -> bool {
    let ready = self.frame_ready;
    self.frame_ready = false;
    ready
  }
}

// The following test cases come from Starr Horne's `nes-rust`
// and Rafael Bagmanov's NES eBook
// See https://github.com/starrhorne/nes-rust/blob/master/src/ppu/registers.rs
// and
// https://github.com/bugzmanov/nes_ebook/blob/master/code/ch6.1/src/ppu/mod.rs
#[cfg(test)]
mod test {
  use super::*;
  use crate::test::init;
  use crate::Cartridge;
  use std::cell::RefCell;
  use std::rc::Rc;

  #[test]
  fn test_write_control_register() {
    init();
    let mut ppu = PPU::new();
    ppu.write_register(CONTROL_REGISTER_INDEX, 0b1010_1010);
    assert_eq!(ppu.control_register.read_u8(), 0b1010_1010);
  }

  #[test]
  fn test_write_mask_register() {
    init();
    let mut ppu = PPU::new();
    ppu.write_register(MASK_REGISTER_INDEX, 0b1010_1010);
    assert_eq!(ppu.mask_register.read_u8(), 0b1010_1010);
  }

  #[test]
  fn test_write_oam_address_register() {
    init();
    let mut ppu = PPU::new();
    ppu.write_register(OAM_ADDRESS_REGISTER_INDEX, 0xF0);
    assert_eq!(ppu.oam_address, 0xF0);
  }

  #[test]
  fn test_write_oam_data_register() {
    init();
    let mut ppu = PPU::new();
    ppu.oam_address = 5;
    ppu.write_register(OAM_DATA_REGISTER_INDEX, 0xF0);
    assert_eq!(ppu.oam_ram[5], 0xF0);
    assert_eq!(ppu.oam_address, 6);
  }

  #[test]
  fn test_read_status_register() {
    init();
    let mut ppu = PPU::new();
    ppu.is_latched = true;
    ppu.write_status_register(0b1110_0000);
    let status = ppu.read_register(STATUS_REGISTER_INDEX);
    assert_eq!(status, 0b1110_0000);
    assert_eq!(ppu.is_latched, false);
    assert_eq!(ppu.status_register.get_vertical_blank_flag(), false);
  }

  #[test]
  fn test_read_ghost_bits() {
    init();
    let mut ppu = PPU::new();
    ppu.write_register(STATUS_REGISTER_INDEX, 0b1111_1111);
    ppu.write_status_register(0b0000_0000);
    assert_eq!(ppu.read_register(STATUS_REGISTER_INDEX), 0b0001_1111);
    assert_eq!(ppu.read_register(CONTROL_REGISTER_INDEX), 0b0001_1111);
    assert_eq!(ppu.read_register(MASK_REGISTER_INDEX), 0b0001_1111);
    assert_eq!(ppu.read_register(OAM_ADDRESS_REGISTER_INDEX), 0b0001_1111);
    assert_eq!(ppu.read_register(SCROLL_REGISTER_INDEX), 0b0001_1111);
    assert_eq!(ppu.read_register(ADDRESS_REGISTER_INDEX), 0b0001_1111);
  }

  #[test]
  fn test_read_oam_data() {
    init();
    let mut ppu = PPU::new();
    ppu.oam_ram[5] = 0x0F;
    ppu.oam_address = 5;
    assert_eq!(ppu.read_register(OAM_DATA_REGISTER_INDEX), 0x0F);
    assert_eq!(ppu.oam_address, 5);
  }

  #[test]
  fn test_read_data_delayed() {
    init();
    let mut ppu = PPU::new();
    ppu.vram.write_u8(0x2001, 1);
    ppu.vram.write_u8(0x2002, 2);
    ppu.vram.write_u8(0x2003, 3);
    ppu.v_address.0 = 0x2001;
    ppu.read_register(DATA_REGISTER_INDEX);
    assert_eq!(ppu.read_register(DATA_REGISTER_INDEX), 1);
    assert_eq!(ppu.read_register(DATA_REGISTER_INDEX), 2);
    assert_eq!(ppu.read_register(DATA_REGISTER_INDEX), 3);
  }
  #[test]
  fn test_write_scroll_register() {
    init();
    let mut ppu = PPU::new();
    ppu.write_register(SCROLL_REGISTER_INDEX, 0b10101_010);
    assert_eq!(ppu.fine_x, 0b010);
    assert_eq!(ppu.t_address.coarse_x(), 0b10101);
    assert_eq!(ppu.is_latched, true);

    ppu.write_register(SCROLL_REGISTER_INDEX, 0b01010_101);
    assert_eq!(ppu.t_address.fine_y(), 0b101);
    assert_eq!(ppu.t_address.coarse_y(), 0b01010);
    assert_eq!(ppu.is_latched, false);
  }

  #[test]
  fn test_write_address_register() {
    init();
    let mut ppu = PPU::new();
    ppu.write_register(ADDRESS_REGISTER_INDEX, 0b11_101010);
    assert_eq!(ppu.t_address.high_byte(), 0b00_101010);
    assert_ne!(ppu.t_address, ppu.v_address);
    assert_eq!(ppu.is_latched, true);
    ppu.write_register(ADDRESS_REGISTER_INDEX, 0b1010_1010);
    assert_eq!(ppu.t_address.0, 0b0010_1010_1010_1010);
    assert_eq!(ppu.t_address, ppu.v_address);
    assert_eq!(ppu.is_latched, false);
  }

  #[test]
  fn test_write_data_register() {
    init();
    let mut ppu = PPU::new();
    ppu.v_address.0 = 0x2000;
    ppu.write_register(DATA_REGISTER_INDEX, 0xF0);
    assert_eq!(ppu.vram.read_u8(0x2000), 0xF0);
    assert_eq!(ppu.v_address.0, 0x2001);
    // Use vertical increment.
    ppu.write_control_register(0b0000_0100);
    ppu.write_register(DATA_REGISTER_INDEX, 0x0F);
    assert_eq!(ppu.vram.read_u8(0x2001), 0x0F);
    assert_eq!(ppu.v_address.0, 0x2001 + 32);
  }

  fn build_cartridge(vertical: bool) -> Rc<RefCell<Cartridge>> {
    init();
    let mut data = vec![
      0x4e, 0x45, 0x53, 0x1a, 0x02, 0x01, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    ];
    if vertical {
      data[6] = 0x01;
    }
    data.extend_from_slice(&[0u8; 2 * 0x4000]);
    for i in 0..0x2000u16 {
      data.push(i as u8);
    }
    Rc::new(RefCell::new(Cartridge::new(&data)))
  }

  #[test]
  fn test_ppu_vram_writes() {
    init();
    let mut ppu = PPU::new();
    let cartridge = build_cartridge(false);
    ppu.vram.set_cartridge(cartridge);
    ppu.write_address_register(0x23);
    ppu.write_address_register(0x05);
    ppu.write_data_register(0x66);
    assert_eq!(ppu.vram.read_u8(0x2305), 0x66);
  }

  #[test]
  fn test_ppu_vram_reads() {
    init();
    let mut ppu = PPU::new();
    let cartridge = build_cartridge(false);
    ppu.vram.set_cartridge(cartridge);
    ppu.write_control_register(0x00);
    ppu.vram.write_u8(0x2305, 0x66);
    ppu.write_address_register(0x23);
    ppu.write_address_register(0x05);
    // Dummy read.
    ppu.read_data_register();
    assert_eq!(ppu.v_address.address(), 0x2306);
    assert_eq!(ppu.read_data_register(), 0x66);
  }

  #[test]
  fn test_ppu_vram_reads_cross_page() {
    init();
    let mut ppu = PPU::new();
    let cartridge = build_cartridge(true);
    ppu.vram.set_cartridge(cartridge);
    ppu.write_control_register(0x00);
    ppu.vram.write_u8(0x21FF, 0x66);
    ppu.vram.write_u8(0x2200, 0x77);
    ppu.write_address_register(0x21);
    ppu.write_address_register(0xFF);
    assert_eq!(ppu.v_address.address(), 0x21FF);
    // Dummy read.
    ppu.read_data_register();
    assert_eq!(ppu.v_address.address(), 0x2200);
    assert_eq!(ppu.read_data_register(), 0x66);
    assert_eq!(ppu.read_data_register(), 0x77);
  }

  #[test]
  fn test_ppu_vram_reads_step_32() {
    init();
    let mut ppu = PPU::new();
    let cartridge = build_cartridge(true);
    ppu.vram.set_cartridge(cartridge);
    ppu.write_control_register(0x00);
    ppu.control_register.set_vram_increment_size_flag(true);
    ppu.vram.write_u8(0x21FF, 0x66);
    ppu.vram.write_u8(0x21FF + 32, 0x77);
    ppu.vram.write_u8(0x21FF + 64, 0x88);
    ppu.write_address_register(0x21);
    ppu.write_address_register(0xFF);
    // Dummy read.
    ppu.read_data_register();
    assert_eq!(ppu.v_address.address(), 0x221F);
    assert_eq!(ppu.read_data_register(), 0x66);
    assert_eq!(ppu.read_data_register(), 0x77);
    assert_eq!(ppu.read_data_register(), 0x88);
  }

  // Horizontal: https://wiki.nesdev.com/w/index.php/Mirroring
  //   [0x2000 A ] [0x2400 a ]
  //   [0x2800 B ] [0x2C00 b ]
  #[test]
  fn test_vram_horizontal_mirror() {
    init();
    let mut ppu = PPU::new();
    let cartridge = build_cartridge(false);
    ppu.vram.set_cartridge(cartridge);
    ppu.write_control_register(0x00);
    ppu.write_address_register(0x24);
    ppu.write_address_register(0x05);
    ppu.write_data_register(0x66);
    ppu.write_address_register(0x28);
    ppu.write_address_register(0x05);
    ppu.write_data_register(0x77);
    ppu.write_address_register(0x20);
    ppu.write_address_register(0x05);
    // Dummy read.
    ppu.read_data_register();
    assert_eq!(ppu.read_data_register(), 0x66);
    ppu.write_address_register(0x2C);
    ppu.write_address_register(0x05);
    // Dummy read.
    ppu.read_data_register();
    assert_eq!(ppu.read_data_register(), 0x77);
  }
  // Vertical: https://wiki.nesdev.com/w/index.php/Mirroring
  //   [0x2000 A ] [0x2400 B ]
  //   [0x2800 a ] [0x2C00 b ]
  #[test]
  fn test_vram_vertical_mirror() {
    init();
    let mut ppu = PPU::new();
    let cartridge = build_cartridge(true);
    ppu.vram.set_cartridge(cartridge);
    ppu.write_control_register(0x00);
    ppu.write_address_register(0x20);
    ppu.write_address_register(0x05);
    ppu.write_data_register(0x66);
    ppu.write_address_register(0x2C);
    ppu.write_address_register(0x05);
    ppu.write_data_register(0x77);
    ppu.write_address_register(0x28);
    ppu.write_address_register(0x05);
    // Dummy read.
    ppu.read_data_register();
    assert_eq!(ppu.read_data_register(), 0x66);
    ppu.write_address_register(0x24);
    ppu.write_address_register(0x05);
    // Dummy read.
    ppu.read_data_register();
    assert_eq!(ppu.read_data_register(), 0x77);
  }

  #[test]
  fn test_read_status_resets_latch() {
    init();
    let mut ppu = PPU::new();
    let cartridge = build_cartridge(true);
    ppu.vram.set_cartridge(cartridge);
    ppu.vram.write_u8(0x2305, 0x66);
    ppu.write_address_register(0x21);
    ppu.write_address_register(0x23);
    ppu.write_address_register(0x05);
    // Dummy read.
    ppu.read_data_register();
    assert_ne!(ppu.read_data_register(), 0x66);
    ppu.read_status_register();
    ppu.write_address_register(0x23);
    ppu.write_address_register(0x05);
    // Dummy read.
    ppu.read_data_register();
    assert_eq!(ppu.read_data_register(), 0x66);
  }

  #[test]
  fn test_ppu_vram_mirroring() {
    init();
    let mut ppu = PPU::new();
    let cartridge = build_cartridge(true);
    ppu.vram.set_cartridge(cartridge);
    ppu.write_control_register(0x00);
    ppu.vram.write_u8(0x2305, 0x66);
    ppu.write_address_register(0x63);
    ppu.write_address_register(0x05);
    // Dummy read.
    ppu.read_data_register();
    assert_eq!(ppu.read_data_register(), 0x66);
  }

  #[test]
  fn test_read_status_resets_vblank() {
    init();
    let mut ppu = PPU::new();
    let cartridge = build_cartridge(true);
    ppu.vram.set_cartridge(cartridge);
    ppu.status_register.set_vertical_blank_flag(true);
    let status = ppu.read_status_register();
    assert_eq!(status >> 7, 1);
    assert_eq!(ppu.read_status_register() >> 7, 0);
  }

  #[test]
  fn test_oam_read_write() {
    init();
    let mut ppu = PPU::new();
    let cartridge = build_cartridge(true);
    ppu.vram.set_cartridge(cartridge);
    ppu.write_oam_address_register(0x10);
    ppu.write_oam_data_register(0x66);
    ppu.write_oam_data_register(0x77);
    ppu.write_oam_address_register(0x10);
    assert_eq!(ppu.read_oam_data_register(), 0x66);
    ppu.write_oam_address_register(0x11);
    assert_eq!(ppu.read_oam_data_register(), 0x77);
  }

  #[test]
  fn test_oam_dma() {
    init();
    let mut ppu = PPU::new();
    let cartridge = build_cartridge(true);
    ppu.vram.set_cartridge(cartridge);
    let mut data = [0x66; 256];
    data[0] = 0x77;
    data[255] = 0x88;
    ppu.write_oam_address_register(0x10);
    ppu.write_oam_dma(&data);
    ppu.write_oam_address_register(0x0F);
    assert_eq!(ppu.read_oam_data_register(), 0x88);
    ppu.write_oam_address_register(0x10);
    assert_eq!(ppu.read_oam_data_register(), 0x77);
    ppu.write_oam_address_register(0x11);
    assert_eq!(ppu.read_oam_data_register(), 0x66);
  }
}
