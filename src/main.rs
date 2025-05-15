#![allow(unused_macros)]
#![allow(unused_imports)]

#[macro_use]
extern crate bitfield;
#[macro_use]
extern crate clap;
#[macro_use]
extern crate derivative;
#[macro_use]
extern crate function_name;
pub use ::function_name::named;
// extern crate iced;
// use iced::Application;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
extern crate pretty_env_logger;
#[macro_use]
extern crate serde;
#[macro_use]
extern crate serde_json;
extern crate tokio;
extern crate uuid;
#[macro_use]
extern crate warp;

pub use greenstone::*;

use clap::Parser;
use tokio::runtime::Runtime;

// Temporary.
use rand::Rng;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::pixels::PixelFormatEnum;
use sdl2::EventPump;

// Temporary
fn handle_user_input(cpu: &mut CPU, event_pump: &mut EventPump) {
  for event in event_pump.poll_iter() {
    match event {
      Event::Quit { .. }
      | Event::KeyDown {
        keycode: Some(Keycode::Escape),
        ..
      } => std::process::exit(0),
      Event::KeyDown {
        keycode: Some(Keycode::W), ..
      } => {
        cpu.write_u8(0xff, 0x77);
      }
      Event::KeyDown {
        keycode: Some(Keycode::S), ..
      } => {
        cpu.write_u8(0xff, 0x73);
      }
      Event::KeyDown {
        keycode: Some(Keycode::A), ..
      } => {
        cpu.write_u8(0xff, 0x61);
      }
      Event::KeyDown {
        keycode: Some(Keycode::D), ..
      } => {
        cpu.write_u8(0xff, 0x64);
      }
      _ => { /* do nothing */ }
    }
  }
}

// Temporary
fn color(byte: u8) -> Color {
  match byte {
    0 => sdl2::pixels::Color::BLACK,
    1 => sdl2::pixels::Color::WHITE,
    2 | 9 => sdl2::pixels::Color::GREY,
    3 | 10 => sdl2::pixels::Color::RED,
    4 | 11 => sdl2::pixels::Color::GREEN,
    5 | 12 => sdl2::pixels::Color::BLUE,
    6 | 13 => sdl2::pixels::Color::MAGENTA,
    7 | 14 => sdl2::pixels::Color::YELLOW,
    _ => sdl2::pixels::Color::CYAN,
  }
}

// Check if PPU has a new frame ready
fn check_ppu_frame_ready(cpu: &mut CPU) -> bool {
  cpu.take_frame_ready()
}

// Get the PPU framebuffer
fn get_ppu_framebuffer(cpu: &CPU) -> &[u8] {
  cpu.get_framebuffer()
}

// Temporary - for test ROMs that write to RAM
fn read_screen_state(cpu: &mut CPU, frame: &mut [u8; 32 * 3 * 32]) -> bool {
  let mut frame_idx = 0;
  let mut update = false;
  for i in 0x0200..0x0600 {
    let color_idx = cpu.unclocked_read_u8(i as u16);
    let (b1, b2, b3) = color(color_idx).rgb();
    if frame[frame_idx] != b1 || frame[frame_idx + 1] != b2 || frame[frame_idx + 2] != b3 {
      frame[frame_idx] = b1;
      frame[frame_idx + 1] = b2;
      frame[frame_idx + 2] = b3;
      update = true;
    }
    frame_idx += 3;
  }
  update
}
use crate::warp::Filter;

#[named]
#[tokio::main]
async fn main() {
  pretty_env_logger::init();
  trace!("main()");

  let args = Arguments::parse();
  let mut server_option = None;

  //
  // Server
  //
  if args.serve {
    println!("Serving!");
    let server_handle = tokio::spawn(async {
      start_server().await;
    });
    server_option = Some(server_handle);
  }

  // NES display constants
  const NES_WIDTH: u32 = 256;
  const NES_HEIGHT: u32 = 240;
  const SCALE: f32 = 3.0;

  let sdl_context = sdl2::init().unwrap();
  let video_subsystem = sdl_context.video().unwrap();
  let window = video_subsystem
    .window(
      "Greenstone NES Emulator",
      (NES_WIDTH as f32 * SCALE) as u32,
      (NES_HEIGHT as f32 * SCALE) as u32,
    )
    .position_centered()
    .build()
    .unwrap();

  let mut canvas = window.into_canvas().present_vsync().build().unwrap();
  let mut event_pump = sdl_context.event_pump().unwrap();
  canvas.set_scale(SCALE, SCALE).unwrap();
  let creator = canvas.texture_creator();
  let mut texture = creator
    .create_texture_target(PixelFormatEnum::RGB24, NES_WIDTH, NES_HEIGHT)
    .unwrap();

  // Load the ROM file
  let bytes: Vec<u8> = std::fs::read(args.file).unwrap();
  let mut bus = Bus::new();
  bus.load_cartridge_data(&bytes);

  // Initialize CPU with the bus
  let mut cpu = CPU::new_with_bus(Box::new(bus));
  cpu.handle_reset();

  // Run the emulation loop with frame rate limiting
  // NES runs at ~60.0988 FPS (NTSC), so ~16.64ms per frame
  let frame_duration = std::time::Duration::from_nanos(16_639_267); // 1/60.0988 seconds
  let mut last_frame_time = std::time::Instant::now();

  cpu.run_with_callback(move |cpu| {
    // Check if PPU has rendered a new frame
    if check_ppu_frame_ready(cpu) {
      // Only handle input once per frame (not every CPU cycle)
      handle_user_input(cpu, &mut event_pump);

      let framebuffer = get_ppu_framebuffer(cpu);
      if framebuffer.len() == (NES_WIDTH * NES_HEIGHT * 3) as usize {
        texture
          .update(None, framebuffer, (NES_WIDTH * 3) as usize)
          .unwrap();
        canvas.copy(&texture, None, None).unwrap();
        canvas.present();
      }

      // Frame rate limiting: sleep until next frame should start
      let elapsed = last_frame_time.elapsed();
      if elapsed < frame_duration {
        std::thread::sleep(frame_duration - elapsed);
      }
      last_frame_time = std::time::Instant::now();
    }
  });

  std::thread::sleep(std::time::Duration::from_millis(3600_000));
  if let Some(server_handle) = server_option {
    tokio::join!(server_handle);
  }
}
