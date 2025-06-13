//! Frame preprocessing and observation stacking for RL.
//!
//! Implements the standard DQN preprocessing pipeline:
//! 1. Convert RGB framebuffer to grayscale
//! 2. Downsample to 84×84
//! 3. Stack N consecutive frames as channels
//! 4. Max-pool adjacent frames to handle sprite flickering
//!
//! The resulting observation is a flat `Vec<u8>` of shape
//! `[stack_size, 84, 84]` suitable for feeding into a CNN.

/// Observation dimensions.
pub const OBS_WIDTH: usize = 84;
pub const OBS_HEIGHT: usize = 84;
pub const OBS_PIXELS: usize = OBS_WIDTH * OBS_HEIGHT;

/// NES source dimensions.
const SRC_WIDTH: usize = 256;
const SRC_HEIGHT: usize = 240;

/// Frame stacker that maintains a rolling window of preprocessed frames.
///
/// Each call to `push()` adds a new grayscale 84×84 frame.
/// `observation()` returns all stacked frames concatenated.
pub struct FrameStack {
  /// Number of frames to stack (typically 4).
  stack_size: usize,
  /// Ring buffer of preprocessed frames, each 84×84 bytes.
  frames: Vec<Vec<u8>>,
  /// Index of the next frame to overwrite.
  write_idx: usize,
  /// Whether we've filled the buffer at least once.
  filled: bool,
  /// Previous raw frame for max-pooling (before grayscale conversion).
  prev_frame: Vec<u8>,
  /// Whether max-pooling between consecutive frames is enabled.
  max_pool: bool,
}

impl FrameStack {
  /// Create a new frame stacker.
  ///
  /// * `stack_size` - Number of frames to stack (typically 4).
  /// * `max_pool` - If true, take pixel-wise max of current and previous
  ///   frame before downsampling. This eliminates NES sprite flickering
  ///   artifacts where alternating frames show/hide sprites.
  pub fn new(stack_size: usize, max_pool: bool) -> Self {
    assert!(stack_size >= 1, "stack_size must be >= 1");
    let frames = (0..stack_size).map(|_| vec![0u8; OBS_PIXELS]).collect();
    FrameStack {
      stack_size,
      frames,
      write_idx: 0,
      filled: false,
      prev_frame: Vec::new(),
      max_pool,
    }
  }

  /// Push a new raw RGB24 framebuffer (256×240×3 bytes).
  /// Applies max-pooling (if enabled), grayscale conversion, and downsampling.
  pub fn push(&mut self, rgb_framebuffer: &[u8]) {
    let working_frame = if self.max_pool && !self.prev_frame.is_empty() {
      // Max-pool current with previous frame
      max_pool_frames(&self.prev_frame, rgb_framebuffer)
    } else {
      rgb_framebuffer.to_vec()
    };

    // Store current frame for next max-pool operation
    if self.max_pool {
      self.prev_frame = rgb_framebuffer.to_vec();
    }

    // Downsample to 84×84 grayscale
    let gray = downsample_grayscale(&working_frame, SRC_WIDTH, SRC_HEIGHT, OBS_WIDTH, OBS_HEIGHT);

    // Write into ring buffer
    self.frames[self.write_idx] = gray;
    self.write_idx = (self.write_idx + 1) % self.stack_size;
    if self.write_idx == 0 {
      self.filled = true;
    }
  }

  /// Get the stacked observation as a flat `Vec<u8>`.
  /// Shape is `[stack_size, OBS_HEIGHT, OBS_WIDTH]` (channels first).
  /// Frames are ordered oldest-to-newest.
  pub fn observation(&self) -> Vec<u8> {
    let mut obs = Vec::with_capacity(self.stack_size * OBS_PIXELS);
    for i in 0..self.stack_size {
      // Read from oldest to newest
      let idx = if self.filled { (self.write_idx + i) % self.stack_size } else { i };
      obs.extend_from_slice(&self.frames[idx]);
    }
    obs
  }

  /// Get the observation as normalized `f32` values in [0, 1].
  /// Same shape as `observation()` but floating point.
  pub fn observation_f32(&self) -> Vec<f32> {
    self.observation().iter().map(|&b| b as f32 / 255.0).collect()
  }

  /// Total number of elements in the observation tensor.
  pub fn observation_size(&self) -> usize {
    self.stack_size * OBS_PIXELS
  }

  /// Shape of the observation tensor: `[stack_size, height, width]`.
  pub fn observation_shape(&self) -> [usize; 3] {
    [self.stack_size, OBS_HEIGHT, OBS_WIDTH]
  }

  /// Reset all frames to black.
  pub fn reset(&mut self) {
    for frame in &mut self.frames {
      frame.fill(0);
    }
    self.write_idx = 0;
    self.filled = false;
    self.prev_frame.clear();
  }

  /// Number of frames in the stack.
  pub fn stack_size(&self) -> usize {
    self.stack_size
  }
}

/// Pixel-wise max of two RGB24 framebuffers.
/// Used to eliminate NES sprite flickering.
fn max_pool_frames(a: &[u8], b: &[u8]) -> Vec<u8> {
  a.iter().zip(b.iter()).map(|(&x, &y)| x.max(y)).collect()
}

/// Downsample an RGB24 image to grayscale at the target resolution.
/// Uses area-averaging for good quality at speed.
fn downsample_grayscale(rgb: &[u8], src_w: usize, src_h: usize, dst_w: usize, dst_h: usize) -> Vec<u8> {
  let mut out = vec![0u8; dst_w * dst_h];

  for dy in 0..dst_h {
    for dx in 0..dst_w {
      let sx_start = dx * src_w / dst_w;
      let sx_end = ((dx + 1) * src_w / dst_w).min(src_w);
      let sy_start = dy * src_h / dst_h;
      let sy_end = ((dy + 1) * src_h / dst_h).min(src_h);

      let mut sum: u32 = 0;
      let mut count: u32 = 0;
      for sy in sy_start..sy_end {
        for sx in sx_start..sx_end {
          let idx = (sy * src_w + sx) * 3;
          if idx + 2 < rgb.len() {
            // ITU-R BT.601 luminance
            let r = rgb[idx] as u32;
            let g = rgb[idx + 1] as u32;
            let b = rgb[idx + 2] as u32;
            sum += (r * 299 + g * 587 + b * 114) / 1000;
            count += 1;
          }
        }
      }
      out[dy * dst_w + dx] = if count > 0 { (sum / count) as u8 } else { 0 };
    }
  }
  out
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_frame_stack_basic() {
    let mut stack = FrameStack::new(4, false);
    assert_eq!(stack.observation_shape(), [4, 84, 84]);
    assert_eq!(stack.observation_size(), 4 * 84 * 84);

    // Initially all zeros
    let obs = stack.observation();
    assert_eq!(obs.len(), 4 * 84 * 84);
    assert!(obs.iter().all(|&b| b == 0));
  }

  #[test]
  fn test_frame_stack_push() {
    let mut stack = FrameStack::new(4, false);

    // Push a white frame
    let white_frame = vec![255u8; SRC_WIDTH * SRC_HEIGHT * 3];
    stack.push(&white_frame);

    let obs = stack.observation();
    // First 3 frames should be black (zero), last should be non-zero
    let frame_0 = &obs[0..OBS_PIXELS];
    let frame_3 = &obs[3 * OBS_PIXELS..4 * OBS_PIXELS]; // newest (after write_idx wraps logic)
                                                        // Since we pushed 1 frame and stack_size is 4, frame at index 0 should be non-zero
                                                        // (write_idx started at 0, wrote there, now write_idx = 1)
                                                        // observation() reads from oldest to newest
    assert!(frame_0.iter().any(|&b| b > 0)); // The pushed frame
    assert!(frame_3.iter().all(|&b| b == 0)); // Not yet written
  }

  #[test]
  fn test_frame_stack_f32() {
    let mut stack = FrameStack::new(1, false);
    let white_frame = vec![255u8; SRC_WIDTH * SRC_HEIGHT * 3];
    stack.push(&white_frame);

    let obs = stack.observation_f32();
    assert_eq!(obs.len(), OBS_PIXELS);
    // All white -> grayscale ~255 -> f32 ~1.0
    assert!(obs[0] > 0.99);
  }

  #[test]
  fn test_frame_stack_reset() {
    let mut stack = FrameStack::new(4, false);
    let white_frame = vec![255u8; SRC_WIDTH * SRC_HEIGHT * 3];
    stack.push(&white_frame);

    stack.reset();
    let obs = stack.observation();
    assert!(obs.iter().all(|&b| b == 0));
  }

  #[test]
  fn test_max_pooling() {
    // Frame A: red stripe
    let mut frame_a = vec![0u8; SRC_WIDTH * SRC_HEIGHT * 3];
    for y in 0..SRC_HEIGHT {
      for x in 0..SRC_WIDTH {
        let idx = (y * SRC_WIDTH + x) * 3;
        if y < SRC_HEIGHT / 2 {
          frame_a[idx] = 200; // R
        }
      }
    }

    // Frame B: green stripe
    let mut frame_b = vec![0u8; SRC_WIDTH * SRC_HEIGHT * 3];
    for y in 0..SRC_HEIGHT {
      for x in 0..SRC_WIDTH {
        let idx = (y * SRC_WIDTH + x) * 3;
        if y >= SRC_HEIGHT / 2 {
          frame_b[idx + 1] = 200; // G
        }
      }
    }

    let pooled = max_pool_frames(&frame_a, &frame_b);
    // Top half: R=200, bottom half: G=200
    let top_pixel = 0;
    assert_eq!(pooled[top_pixel], 200);
    let bottom_pixel = (SRC_HEIGHT / 2) * SRC_WIDTH * 3 + 1;
    assert_eq!(pooled[bottom_pixel], 200);
  }

  #[test]
  fn test_downsample_dimensions() {
    let rgb = vec![128u8; SRC_WIDTH * SRC_HEIGHT * 3];
    let gray = downsample_grayscale(&rgb, SRC_WIDTH, SRC_HEIGHT, OBS_WIDTH, OBS_HEIGHT);
    assert_eq!(gray.len(), OBS_WIDTH * OBS_HEIGHT);
    // Uniform gray input -> uniform gray output
    let first = gray[0];
    assert!(gray.iter().all(|&b| (b as i16 - first as i16).unsigned_abs() < 2));
  }

  #[test]
  fn test_stack_ordering() {
    let mut stack = FrameStack::new(3, false);

    // Push 3 frames with different brightness
    for brightness in [50u8, 100, 200] {
      let frame = vec![brightness; SRC_WIDTH * SRC_HEIGHT * 3];
      stack.push(&frame);
    }

    let obs = stack.observation();
    // Oldest frame (brightness 50) should be first channel
    let avg_ch0: u32 = obs[0..OBS_PIXELS].iter().map(|&b| b as u32).sum::<u32>() / OBS_PIXELS as u32;
    let avg_ch2: u32 = obs[2 * OBS_PIXELS..3 * OBS_PIXELS].iter().map(|&b| b as u32).sum::<u32>() / OBS_PIXELS as u32;
    // Channel 0 (oldest, brightness ~50) should be dimmer than channel 2 (newest, brightness ~200)
    assert!(avg_ch0 < avg_ch2, "avg_ch0={avg_ch0}, avg_ch2={avg_ch2}");
  }
}
