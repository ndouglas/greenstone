//! Games whose memory answers "is the player in the air?", with every
//! address cited to its RAM map and verified here on the exact ROM file.
//! See `docs/series/games/001.md`.

use crate::nes::bus::{BUTTON_A, BUTTON_RIGHT, BUTTON_START, BUTTON_UP};
use crate::nes::gym::reservoir::demo::{smb_dead, smb_start_state};
use crate::nes::gym::NesEmulator;

/// A game, the ROM its addresses were checked on, and where it keeps the
/// player's height and whether he's in the air.
pub struct Game {
  pub name: &'static str,
  pub rom: &'static str,
  pub sha1: &'static str,
  /// The answer key: the byte that says whether the player is in the air.
  pub airborne_byte: u16,
  /// The player's height.
  pub y: u16,
  /// The player's vertical speed, if the game keeps one.
  pub vy: Option<u16>,
  pub(crate) is_airborne: fn(u8) -> bool,
  start: fn(&[u8]) -> NesEmulator,
  died: fn(&mut NesEmulator) -> bool,
}

impl Game {
  /// An emulator at the first frame of play, after checking the ROM.
  pub fn boot(&self, rom: &[u8]) -> Result<NesEmulator, String> {
    verify_rom(self, rom)?;
    Ok((self.start)(rom))
  }

  /// Whether the player has just died (so a recorder should start over).
  pub fn died(&self, emu: &mut NesEmulator) -> bool {
    (self.died)(emu)
  }

  /// What the answer key says this frame.
  pub fn airborne(&self, emu: &mut NesEmulator) -> bool {
    (self.is_airborne)(emu.ram_read(self.airborne_byte))
  }
}

/// Refuse a ROM other than the one the addresses were verified on.
pub fn verify_rom(game: &Game, rom: &[u8]) -> Result<(), String> {
  let digest = sha1_smol::Sha1::from(rom).digest().to_string();
  if digest == game.sha1 {
    Ok(())
  } else {
    Err(format!(
      "{} is SHA-1 {}, but {}'s addresses were verified on {}",
      game.rom, digest, game.name, game.sha1
    ))
  }
}

/// Run `frames` frames, pressing `buttons` during each (start, length) window.
fn play_script(emu: &mut NesEmulator, frames: usize, script: &[(usize, usize, u8)]) {
  for f in 0..frames {
    let held = script.iter().filter(|(s, l, _)| f >= *s && f < s + l).fold(0, |b, (_, _, x)| b | x);
    emu.step_frame(held);
  }
}

/// Super Mario Bros., per Data Crystal's "Super Mario Bros./RAM map".
pub mod smb1 {
  /// Float state: 0 standing, 1 airborne by jumping, 2 off a ledge, 3 flagpole.
  pub const FLOAT_STATE: u16 = 0x001D;
  /// Y position on screen.
  pub const Y: u16 = 0x00CE;
  /// Vertical velocity in whole pixels, signed.
  pub const VY: u16 = 0x009F;
  /// Six BCD digits, one per byte, millions down to tens.
  pub const SCORE_DIGITS: u16 = 0x07DD;
}

/// Super Mario Bros. 3, per Data Crystal's "Super Mario Bros. 3/RAM map".
pub mod smb3 {
  /// "In air flag": 1 in the air, 0 on the ground (values seen in Greenstone).
  pub const IN_AIR: u16 = 0x00D8;
  /// Vertical position in the level.
  pub const Y: u16 = 0x00A2;
  /// Vertical velocity, signed.
  pub const VY: u16 = 0x00CF;
  /// Mario's lives.
  pub const LIVES: u16 = 0x0736;
  /// Non-zero while Mario is dying (found by experiment; see the games doc).
  pub const DYING: u16 = 0x058C;
}

/// Castlevania, per Data Crystal's "Castlevania (NES, Famicom Disk System)/RAM map".
pub mod castlevania {
  /// Simon's state: 01 jumping, 05 knocked back, 07 dropping, 00 walking, …
  pub const STATE: u16 = 0x046C;
  /// Simon's real y coordinate.
  pub const Y: u16 = 0x003F;
}

pub const SMB1: Game = Game {
  name: "Super Mario Bros.",
  rom: "roms2/super_mario_bros.nes",
  sha1: "ea343f4e445a9050d4b4fbac2c77d0693b1d0922",
  airborne_byte: smb1::FLOAT_STATE,
  y: smb1::Y,
  vy: Some(smb1::VY),
  is_airborne: |v| matches!(v, 1 | 2),
  died: smb_dead,
  start: |rom| {
    let mut emu = NesEmulator::new(rom);
    emu.load_state(&smb_start_state(rom));
    // That state is a few frames before play: wait for the player state
    // ($000E) to read 08, normal play.
    for _ in 0..120 {
      if emu.ram_read(0x000E) == 0x08 {
        break;
      }
      emu.step_frame(0);
    }
    emu.step_frame(0);
    emu
  },
};

pub const SMB3: Game = Game {
  name: "Super Mario Bros. 3",
  rom: "roms2/super_mario_bros_3.nes",
  sha1: "a03e7e526e79df222e048ae22214bca2bc49c449",
  airborne_byte: smb3::IN_AIR,
  y: smb3::Y,
  vy: Some(smb3::VY),
  is_airborne: |v| v != 0,
  // Not on Data Crystal's map: found by experiment. It turns on the frame a
  // death begins and stays on through the death animation (about 200 frames),
  // until the lives counter drops.
  died: |emu| emu.ram_read(smb3::DYING) != 0,
  // Title, one player, then on the map: Right one square, Up to level 1, A.
  start: |rom| {
    let mut emu = NesEmulator::new(rom);
    let script = [
      (300, 4, BUTTON_START),
      (600, 4, BUTTON_START),
      (1500, 60, BUTTON_RIGHT),
      (1700, 60, BUTTON_UP),
      (1900, 6, BUTTON_A),
    ];
    play_script(&mut emu, 1990, &script);
    emu
  },
};

pub const CASTLEVANIA: Game = Game {
  name: "Castlevania",
  rom: "roms2/castlevania.nes",
  sha1: "7a20c44f302fb2f1b7adffa6b619e3e1cae7b546",
  airborne_byte: castlevania::STATE,
  y: castlevania::Y,
  vy: None,
  // Jumping, knocked back (he flies backwards through the air), dropping.
  is_airborne: |v| matches!(v, 0x01 | 0x05 | 0x07),
  died: |emu| emu.ram_read(castlevania::STATE) == 0x08,
  // Start at the title, Start again; a third Start would pause the game.
  start: |rom| {
    let mut emu = NesEmulator::new(rom);
    play_script(&mut emu, 800, &[(200, 4, BUTTON_START), (500, 4, BUTTON_START)]);
    emu
  },
};

pub const GAMES: [&Game; 3] = [&SMB1, &SMB3, &CASTLEVANIA];

/// Mario's score from its six BCD digit bytes (the ones digit is always 0).
pub fn smb1_score(emu: &mut NesEmulator) -> u32 {
  (0..6).fold(0, |s, i| s * 10 + (emu.ram_read(smb1::SCORE_DIGITS + i) & 0x0F) as u32) * 10
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::nes::bus::BUTTON_A;
  use crate::nes::gym::reservoir::rom_or_skip;

  fn jump_and_land(game: &Game) {
    let Some(rom) = rom_or_skip(game.rom) else { return };
    let mut emu = game.boot(&rom).unwrap();
    assert!(!game.airborne(&mut emu), "{} starts in the air", game.name);
    let ground_y = emu.ram_read(game.y);
    let mut rose = false;
    for _ in 0..12 {
      emu.step_frame(BUTTON_A);
      rose |= game.airborne(&mut emu);
    }
    assert!(rose, "{} never left the ground", game.name);
    assert_ne!(emu.ram_read(game.y), ground_y, "{}'s height didn't change", game.name);
    let landed = (0..180).any(|_| {
      emu.step_frame(0);
      !game.airborne(&mut emu)
    });
    assert!(landed, "{} never landed", game.name);
  }

  #[test]
  fn rom_is_the_one_the_addresses_were_checked_on() {
    for game in GAMES {
      let Some(mut rom) = rom_or_skip(game.rom) else { continue };
      verify_rom(game, &rom).unwrap();
      rom[100] ^= 0xFF;
      assert!(verify_rom(game, &rom).is_err(), "{} accepted a different file", game.name);
    }
  }

  #[test]
  fn smb1_airborne_is_zero_on_the_ground_and_not_after_a_jump() {
    jump_and_land(&SMB1);
  }

  #[test]
  fn smb3_airborne_is_zero_on_the_ground_and_not_after_a_jump() {
    jump_and_land(&SMB3);
  }

  #[test]
  fn castlevania_airborne_is_zero_on_the_ground_and_not_after_a_jump() {
    jump_and_land(&CASTLEVANIA);
  }

  #[test]
  fn castlevania_knockback_counts_as_in_the_air() {
    let mut emu_state = 0x05u8;
    assert!((CASTLEVANIA.is_airborne)(emu_state), "knocked back (05) is in the air");
    emu_state = 0x00;
    assert!(!(CASTLEVANIA.is_airborne)(emu_state));
  }

  #[test]
  fn smb1_score_digits_read_as_the_score() {
    let Some(rom) = rom_or_skip(SMB1.rom) else { return };
    let mut emu = SMB1.boot(&rom).unwrap();
    assert_eq!(smb1_score(&mut emu), 0);
    for (i, d) in [0u8, 0, 1, 2, 3, 4].iter().enumerate() {
      emu.ram_write(smb1::SCORE_DIGITS + i as u16, *d);
    }
    assert_eq!(smb1_score(&mut emu), 12_340);
  }
}
