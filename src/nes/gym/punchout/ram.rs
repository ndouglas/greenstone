//! Punch-Out!!'s RAM: where the game keeps what the fight loop needs.
//!
//! Every address was found by the probes in `examples/punchout.rs` (`probe`)
//! and is pinned by a behavioural test below, not by its value.

use crate::nes::gym::NesEmulator;

/// The fight number, 0 for Glass Joe; read when the "vs" screen is drawn, so
/// writing it before then picks the opponent (poked 1, 3, 6 and saw Von
/// Kaiser, Don Flamenco, Bald Bull walk out; steps 0 -> 1 after beating Joe).
pub const FIGHT_NUMBER: u16 = 0x0001;
/// 255 while the ring is on screen; leaves 255 when the game cuts to the
/// result screen, after a KO, a TKO or a round's bell (probes: 3738, 2778, 4594).
pub const SCREEN: u16 = 0x0004;
/// Who is down: 0 nobody, 1 the opponent, 2 Mac, 3 the round is over (the
/// idle probe goes 0 -> 2 when Mac's health hits 0; jabs at poked-low health
/// give 0 -> 1 and back when the opponent gets up; the bell gives 3).
pub const DOWN: u16 = 0x0005;
/// 1 while the fighters are out of play (intro, knockdowns, count-outs), 0
/// once the referee steps away (0 from probe frame 627 to the knockdown).
pub const OUT_OF_PLAY: u16 = 0x0022;
/// 1 once the round's clock has started (probe frame 692, with the clock's
/// sub-second byte), 0 again after the bell.
pub const CLOCK_RUNNING: u16 = 0x0300;
/// The clock's minutes; 3 is the bell (the round-break probe).
pub const CLOCK_MINUTES: u16 = 0x0302;
/// Stars: poking 2 and pressing Start twice threw two uppercuts, 2 -> 1 -> 0,
/// each taking 18 health where a jab takes 5.
pub const STARS: u16 = 0x0342;
/// Mac's health, 96 when full: monotone in the idle probe, 11 per hit, each
/// drop a hit in the frames.
pub const MAC_HEALTH: u16 = 0x0391;
/// The opponent's health, 96 when full: 5 per jab in the punch probe, and it
/// recovers when Mac is hit.
pub const OPPONENT_HEALTH: u16 = 0x0398;

pub fn mac_health(emu: &mut NesEmulator) -> u8 {
  emu.ram_read(MAC_HEALTH)
}

pub fn opponent_health(emu: &mut NesEmulator) -> u8 {
  emu.ram_read(OPPONENT_HEALTH)
}

/// In a round, the clock running, both fighters up and in play.
pub fn mac_can_act(emu: &mut NesEmulator) -> bool {
  emu.ram_read(SCREEN) == 0xFF
    && emu.ram_read(OUT_OF_PLAY) == 0
    && emu.ram_read(CLOCK_RUNNING) == 1
    && emu.ram_read(CLOCK_MINUTES) < 3
    && emu.ram_read(DOWN) == 0
}

/// A KO or TKO: the game has left the ring with someone down. (The bell also
/// leaves the ring, with `DOWN` at 3: that is a round break, not the end.)
pub fn fight_over(emu: &mut NesEmulator) -> bool {
  emu.ram_read(SCREEN) != 0xFF && matches!(emu.ram_read(DOWN), 1 | 2)
}

/// Between rounds: the bell has gone and the game waits for Start.
pub fn round_over(emu: &mut NesEmulator) -> bool {
  emu.ram_read(DOWN) == 3
}

pub fn mac_down(emu: &mut NesEmulator) -> bool {
  emu.ram_read(DOWN) == 2
}

pub fn opponent_down(emu: &mut NesEmulator) -> bool {
  emu.ram_read(DOWN) == 1
}

pub fn stars(emu: &mut NesEmulator) -> u8 {
  emu.ram_read(STARS)
}

pub fn opponent_id(emu: &mut NesEmulator) -> u8 {
  emu.ram_read(FIGHT_NUMBER)
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::nes::bus::{BUTTON_A, BUTTON_LEFT, BUTTON_START, BUTTON_UP};
  use crate::nes::gym::punchout::start::{fight_start, Opponent, HELD_OUT, TRAIN};
  use crate::nes::gym::punchout::PUNCH_OUT;
  use crate::nes::gym::reservoir::rom_or_skip;

  fn glass_joe(rom: &[u8]) -> NesEmulator {
    let (state, _) = fight_start(rom, Opponent::GlassJoe).unwrap();
    let mut emu = NesEmulator::new(rom);
    emu.load_state(&state);
    emu
  }

  #[test]
  fn idle_mac_loses_health_and_never_gains_it() {
    let Some(rom) = rom_or_skip(PUNCH_OUT) else { return };
    let mut emu = glass_joe(&rom);
    let start = mac_health(&mut emu);
    let mut last = start;
    let mut drops = 0;
    for _ in 0..3600 {
      emu.step_frames(0, 1);
      if !mac_can_act(&mut emu) {
        break;
      }
      let h = mac_health(&mut emu);
      assert!(h <= last, "health rose from {} to {}", last, h);
      if h < last {
        drops += 1;
      }
      last = h;
    }
    assert!(drops >= 2, "Glass Joe hit an idle Mac only {} times", drops);
  }

  #[test]
  fn jabbing_lowers_the_opponents_health() {
    let Some(rom) = rom_or_skip(PUNCH_OUT) else { return };
    let mut emu = glass_joe(&rom);
    let start = opponent_health(&mut emu);
    for i in 0..600 {
      emu.step_frames(if i % 16 < 4 { BUTTON_UP | BUTTON_A } else { 0 }, 1);
    }
    assert!(opponent_health(&mut emu) < start);
  }

  #[test]
  fn every_opponent_starts_with_mac_able_to_act_and_its_own_id() {
    let Some(rom) = rom_or_skip(PUNCH_OUT) else { return };
    let mut ids = std::collections::BTreeSet::new();
    for o in TRAIN.iter().chain(HELD_OUT.iter()) {
      let (state, frame) = fight_start(&rom, *o).unwrap();
      let mut emu = NesEmulator::new(&rom);
      emu.load_state(&state);
      assert!(mac_can_act(&mut emu), "{} start: Mac cannot act", o.name());
      assert!(frame > 0);
      ids.insert(opponent_id(&mut emu));
    }
    assert_eq!(ids.len(), 5, "five opponents, five ids");
  }

  #[test]
  fn save_step_load_step_gives_the_same_frames() {
    let Some(rom) = rom_or_skip(PUNCH_OUT) else { return };
    let mut emu = glass_joe(&rom);
    emu.step_frames(0, 100);
    let mid = emu.save_state();
    emu.step_frames(BUTTON_LEFT, 40);
    let first = emu.framebuffer().to_vec();
    emu.load_state(&mid);
    emu.step_frames(BUTTON_LEFT, 40);
    assert_eq!(emu.framebuffer(), &first[..], "MMC2 state survives a save state");
  }

  /// No idle fight earns a star, so this one is given: a star turns Start into
  /// an uppercut that spends it and hurts more than a jab.
  #[test]
  fn a_star_is_spent_by_an_uppercut_that_outhits_a_jab() {
    let Some(rom) = rom_or_skip(PUNCH_OUT) else { return };
    let mut emu = glass_joe(&rom);
    emu.step_frames(0, 60);
    let before = emu.save_state();
    let damage = |emu: &mut NesEmulator, buttons: u8| {
      let start = opponent_health(emu);
      emu.step_frames(buttons, 4);
      emu.step_frames(0, 80);
      start - opponent_health(emu)
    };
    let jab = damage(&mut emu, BUTTON_UP | BUTTON_A);
    emu.load_state(&before);
    assert_eq!(stars(&mut emu), 0, "a fight starts with no stars");
    emu.ram_write(STARS, 1);
    let uppercut = damage(&mut emu, BUTTON_START);
    assert_eq!(stars(&mut emu), 0, "the uppercut spent the star");
    assert!(uppercut > jab, "uppercut {} vs jab {}", uppercut, jab);
  }

  #[test]
  fn idle_mac_goes_down_and_the_fight_ends_in_a_loss() {
    let Some(rom) = rom_or_skip(PUNCH_OUT) else { return };
    let mut emu = glass_joe(&rom);
    let mut went_down = false;
    for _ in 0..6000 {
      emu.step_frames(0, 1);
      went_down |= mac_down(&mut emu);
      assert!(!opponent_down(&mut emu), "nobody hit Glass Joe");
      if fight_over(&mut emu) {
        break;
      }
    }
    assert!(went_down, "Mac never went down");
    assert!(fight_over(&mut emu), "the fight never ended");
    assert!(mac_down(&mut emu) && !mac_can_act(&mut emu));
  }

  #[test]
  fn three_knockdowns_end_the_fight_with_the_opponent_down() {
    let Some(rom) = rom_or_skip(PUNCH_OUT) else { return };
    let mut emu = glass_joe(&rom);
    let mut knockdowns = 0;
    let mut was_down = false;
    for i in 0..6000 {
      if mac_can_act(&mut emu) && opponent_health(&mut emu) > 1 {
        emu.ram_write(OPPONENT_HEALTH, 1);
      }
      emu.step_frames(if i % 16 < 8 { BUTTON_UP | BUTTON_A } else { 0 }, 1);
      let down = opponent_down(&mut emu);
      if down && !was_down {
        knockdowns += 1;
        assert!(!mac_can_act(&mut emu), "Mac cannot act while the opponent is down");
      }
      was_down = down;
      if fight_over(&mut emu) {
        break;
      }
    }
    assert_eq!(knockdowns, 3, "a TKO is three knockdowns");
    assert!(fight_over(&mut emu) && opponent_down(&mut emu));
  }
}
