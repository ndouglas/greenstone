//! Pointing 1-1's level-pointer table entries somewhere else in our copy of the ROM.

/// File offset of PRG ROM (after the iNES header). SMB is NROM-256: 32 KB at $8000.
pub const PRG_START: usize = 16;
const PRG_LEN: usize = 0x8000;

/// 1-1 is ground area 5: layout entry index 3 + 5 = 8 in SMB's tables at
/// $9D2C (low) and $9D4E (high); enemy entry index 6 + 5 = 11 in the tables
/// at $9CE4 (low) and $9D06 (high).
const AREA_LOW: u16 = 0x9D34;
const AREA_HIGH: u16 = 0x9D56;
const ENEMY_LOW: u16 = 0x9CEF;
const ENEMY_HIGH: u16 = 0x9D11;

pub const ONE_ONE_AREA: u16 = 0xA68E;
pub const ONE_ONE_ENEMIES: u16 = 0x9F01;

fn at(addr: u16) -> usize {
  PRG_START + (addr - 0x8000) as usize
}

fn pointer(rom: &[u8], low: u16, high: u16) -> u16 {
  rom[at(low)] as u16 | (rom[at(high)] as u16) << 8
}

fn set_pointer(rom: &mut [u8], low: u16, high: u16, value: u16) {
  rom[at(low)] = value as u8;
  rom[at(high)] = (value >> 8) as u8;
}

pub fn area_pointer(rom: &[u8]) -> u16 {
  pointer(rom, AREA_LOW, AREA_HIGH)
}

pub fn enemy_pointer(rom: &[u8]) -> u16 {
  pointer(rom, ENEMY_LOW, ENEMY_HIGH)
}

/// The first $FF byte in PRG. $FF ends an enemy list, so pointing there
/// gives a level no enemies.
pub fn empty_enemy_list(rom: &[u8]) -> u16 {
  let i = rom[PRG_START..PRG_START + PRG_LEN]
    .iter()
    .position(|&b| b == 0xFF)
    .expect("SMB's PRG has an $FF byte");
  0x8000 + i as u16
}

pub fn patch_area(rom: &[u8], area: u16) -> Vec<u8> {
  let mut out = rom.to_vec();
  set_pointer(&mut out, AREA_LOW, AREA_HIGH, area);
  out
}

pub fn without_enemies(rom: &[u8]) -> Vec<u8> {
  let mut out = rom.to_vec();
  set_pointer(&mut out, ENEMY_LOW, ENEMY_HIGH, empty_enemy_list(rom));
  out
}

/// 1-1's layout at `area`, with no enemies.
pub fn patch_rom(rom: &[u8], area: u16) -> Vec<u8> {
  without_enemies(&patch_area(rom, area))
}

/// The Prospector only knows this SMB's table addresses.
pub fn check_rom(rom: &[u8]) -> Result<(), String> {
  if rom.len() < PRG_START + PRG_LEN {
    return Err(format!("ROM is too short ({} bytes) to be SMB", rom.len()));
  }
  if area_pointer(rom) != ONE_ONE_AREA || enemy_pointer(rom) != ONE_ONE_ENEMIES {
    return Err(format!(
      "this ROM does not keep 1-1 at ${:04X}/${:04X} (found ${:04X}/${:04X}); the Prospector needs the original Super Mario Bros.",
      ONE_ONE_AREA,
      ONE_ONE_ENEMIES,
      area_pointer(rom),
      enemy_pointer(rom)
    ));
  }
  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::nes::gym::reservoir::rom_or_skip;

  fn rom() -> Option<Vec<u8>> {
    rom_or_skip("roms2/super_mario_bros.nes")
  }

  #[test]
  fn the_real_rom_points_1_1_where_smb_keeps_it() {
    let Some(rom) = rom() else { return };
    assert_eq!(area_pointer(&rom), ONE_ONE_AREA);
    assert_eq!(enemy_pointer(&rom), ONE_ONE_ENEMIES);
    assert_eq!(check_rom(&rom), Ok(()));
  }

  #[test]
  fn patch_area_at_1_1_changes_nothing() {
    let Some(rom) = rom() else { return };
    assert_eq!(patch_area(&rom, ONE_ONE_AREA), rom);
  }

  #[test]
  fn patch_area_round_trips_the_top_offset() {
    let Some(rom) = rom() else { return };
    for area in [0x8000, 0xC4A0, 0xFFFF] {
      let patched = patch_area(&rom, area);
      assert_eq!(area_pointer(&patched), area);
      assert_eq!(enemy_pointer(&patched), ONE_ONE_ENEMIES);
      let differing = rom.iter().zip(&patched).filter(|(a, b)| a != b).count();
      assert!(differing <= 2, "only the two pointer bytes may change, {} did", differing);
    }
  }

  #[test]
  fn without_enemies_points_at_an_ff_byte() {
    let Some(rom) = rom() else { return };
    let patched = patch_rom(&rom, 0xC4A0);
    let empty = enemy_pointer(&patched);
    assert_eq!(empty, empty_enemy_list(&rom));
    assert_eq!(patched[PRG_START + (empty - 0x8000) as usize], 0xFF);
    assert_eq!(area_pointer(&patched), 0xC4A0);
  }

  #[test]
  fn check_rom_rejects_a_rom_without_1_1_where_expected() {
    let mut fake = vec![0u8; PRG_START + 0x8000];
    fake[..4].copy_from_slice(b"NES\x1a");
    assert!(check_rom(&fake).unwrap_err().contains("1-1"));
    assert!(check_rom(&[0u8; 10]).unwrap_err().contains("too short"));
  }
}
