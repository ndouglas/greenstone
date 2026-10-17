//! Play a log of APU register writes, one batch per NTSC frame, into samples.

use super::APU;

/// CPU cycles per NTSC frame, rounded: 1,789,773 Hz over 60.0988 frames a second.
pub const CYCLES_PER_FRAME: u32 = 29_781;
/// The APU's output rate (see `APU::new`).
pub const SAMPLE_RATE: u32 = 44_100;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RegisterWrite {
  pub frame: u32,
  pub address: u16,
  pub value: u8,
}

/// Parse `frames N` and then `frame address value` lines (hex address and value).
pub fn parse_log(text: &str) -> Result<(u32, Vec<RegisterWrite>), String> {
  let mut lines = text.lines().map(|l| l.split('#').next().unwrap().trim()).filter(|l| !l.is_empty());
  let header = lines.next().ok_or("empty log")?;
  let frames: u32 = header
    .strip_prefix("frames ")
    .ok_or_else(|| format!("expected 'frames N', got '{}'", header))?
    .trim()
    .parse()
    .map_err(|e| format!("bad frame count: {}", e))?;
  let mut writes = Vec::new();
  for line in lines {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() != 3 {
      return Err(format!("expected 'frame address value', got '{}'", line));
    }
    let frame: u32 = parts[0].parse().map_err(|e| format!("bad frame in '{}': {}", line, e))?;
    let address = u16::from_str_radix(parts[1], 16).map_err(|e| format!("bad address in '{}': {}", line, e))?;
    let value = u8::from_str_radix(parts[2], 16).map_err(|e| format!("bad value in '{}': {}", line, e))?;
    if frame >= frames {
      return Err(format!("write at frame {} is past the end ({})", frame, frames));
    }
    if writes.last().map_or(false, |w: &RegisterWrite| w.frame > frame) {
      return Err(format!("frames decrease at '{}'", line));
    }
    writes.push(RegisterWrite { frame, address, value });
  }
  Ok((frames, writes))
}

/// Play the writes into a fresh APU, each frame's writes before its cycles.
pub fn render_log(frames: u32, writes: &[RegisterWrite]) -> Vec<f32> {
  let mut apu = APU::new();
  let mut samples = Vec::new();
  let mut next = 0;
  for frame in 0..frames {
    while next < writes.len() && writes[next].frame == frame {
      apu.write_register(writes[next].address, writes[next].value);
      next += 1;
    }
    for _ in 0..CYCLES_PER_FRAME {
      apu.tick();
    }
    samples.extend(apu.take_samples());
  }
  samples
}

/// 16-bit PCM mono WAV at `SAMPLE_RATE`, samples scaled by 0.9 and clamped.
pub fn wav_bytes(samples: &[f32]) -> Vec<u8> {
  let data_len = (samples.len() * 2) as u32;
  let mut out = Vec::with_capacity(44 + data_len as usize);
  out.extend(b"RIFF");
  out.extend((36 + data_len).to_le_bytes());
  out.extend(b"WAVEfmt ");
  out.extend(16u32.to_le_bytes());
  out.extend(1u16.to_le_bytes());
  out.extend(1u16.to_le_bytes());
  out.extend(SAMPLE_RATE.to_le_bytes());
  out.extend((SAMPLE_RATE * 2).to_le_bytes());
  out.extend(2u16.to_le_bytes());
  out.extend(16u16.to_le_bytes());
  out.extend(b"data");
  out.extend(data_len.to_le_bytes());
  for &s in samples {
    let v = (s * 0.9).clamp(-0.9, 0.9) * i16::MAX as f32;
    out.extend((v as i16).to_le_bytes());
  }
  out
}

#[cfg(test)]
mod tests {
  use super::*;

  /// Strength of `hz` in `samples` by a single-bin DFT.
  fn strength(samples: &[f32], hz: f64) -> f64 {
    let (mut re, mut im) = (0.0, 0.0);
    for (i, &s) in samples.iter().enumerate() {
      let a = 2.0 * std::f64::consts::PI * hz * i as f64 / SAMPLE_RATE as f64;
      re += s as f64 * a.cos();
      im += s as f64 * a.sin();
    }
    (re * re + im * im).sqrt()
  }

  /// The frequency between `lo` and `hi` Hz, in 1 Hz steps, with the most energy.
  fn peak(samples: &[f32], lo: u32, hi: u32) -> u32 {
    (lo..=hi)
      .max_by(|&a, &b| strength(samples, a as f64).total_cmp(&strength(samples, b as f64)))
      .unwrap()
  }

  const LOG: &str = "frames 90
# all channels on, frame IRQ off
0 4015 0F
0 4017 40
# pulse 1, A440: duty 50%, constant volume 15, sweep off
0 4000 BF
0 4001 08
0 4002 FD
0 4003 08
# at frame 30, pulse off and triangle A220
30 4000 B0
30 4008 FF
30 400A FD
30 400B 08
# at frame 60, triangle off and noise on
60 4008 80
60 400C 3F
60 400E 04
60 400F 08
";

  #[test]
  fn parse_log_reads_the_header_and_hex_writes() {
    let (frames, writes) = parse_log(LOG).unwrap();
    assert_eq!(frames, 90);
    assert_eq!(writes.len(), 14);
    assert_eq!(
      writes[4],
      RegisterWrite {
        frame: 0,
        address: 0x4002,
        value: 0xFD
      }
    );
  }

  #[test]
  fn parse_log_rejects_bad_logs() {
    assert!(parse_log("0 4000 BF").is_err(), "no header");
    assert!(parse_log("frames 10\n5 4000 BF\n4 4000 BF").is_err(), "decreasing frames");
    assert!(parse_log("frames 10\n10 4000 BF").is_err(), "past the end");
    assert!(parse_log("frames 10\n1 4000 XYZ").is_err(), "bad hex");
  }

  #[test]
  fn render_log_plays_each_channel_at_its_pitch() {
    let (frames, writes) = parse_log(LOG).unwrap();
    let samples = render_log(frames, &writes);
    let per_frame = SAMPLE_RATE as f64 * CYCLES_PER_FRAME as f64 / 1_789_773.0;
    assert!((samples.len() as f64 - frames as f64 * per_frame).abs() < 2.0);
    let at = |f: f64| (f * per_frame) as usize;
    assert_eq!(peak(&samples[at(5.0)..at(25.0)], 400, 480), 440);
    assert_eq!(peak(&samples[at(35.0)..at(55.0)], 200, 240), 220);
    let noise = &samples[at(65.0)..at(85.0)];
    let mean = noise.iter().sum::<f32>() / noise.len() as f32;
    let rms = (noise.iter().map(|s| (s - mean) * (s - mean)).sum::<f32>() / noise.len() as f32).sqrt();
    assert!(rms > 0.02, "noise rms {}", rms);
  }

  #[test]
  fn wav_bytes_writes_a_canonical_header() {
    let wav = wav_bytes(&[0.0, 1.0, -1.0, 2.0]);
    assert_eq!(&wav[0..4], b"RIFF");
    assert_eq!(&wav[8..16], b"WAVEfmt ");
    assert_eq!(u32::from_le_bytes(wav[24..28].try_into().unwrap()), SAMPLE_RATE);
    assert_eq!(wav.len(), 44 + 4 * 2);
    let last = i16::from_le_bytes(wav[50..52].try_into().unwrap());
    assert_eq!(last, (0.9 * i16::MAX as f32) as i16, "clamped");
  }
}
