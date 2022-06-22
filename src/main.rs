#![allow(unused_macros)]
#![allow(unused_imports)]

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

pub use greenstone::*;

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

// Temporary
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

#[named]
fn main() {
  pretty_env_logger::init();
  trace!("main()");

  // TEMPORARY

  let sdl_context = sdl2::init().unwrap();
  let video_subsystem = sdl_context.video().unwrap();
  let window = video_subsystem
    .window("snek game 🐍", (32.0 * 10.0) as u32, (32.0 * 10.0) as u32)
    .position_centered()
    .build()
    .unwrap();

  let mut canvas = window.into_canvas().present_vsync().build().unwrap();
  let mut event_pump = sdl_context.event_pump().unwrap();
  canvas.set_scale(10.0, 10.0).unwrap();
  let creator = canvas.texture_creator();
  let mut texture = creator.create_texture_target(PixelFormatEnum::RGB24, 32, 32).unwrap();

  let mut screen_state = [0 as u8; 32 * 3 * 32];
  let mut rng = rand::thread_rng();

  //load the game
  let bytes: Vec<u8> = std::fs::read("roms/snake.nes").unwrap();
  let mut bus = Bus::new();
  bus.load_cartridge_data(&bytes);

  //load the game
  let mut cpu = CPU::new_with_bus(Box::new(bus));
  cpu.handle_reset();

  // run the game cycle
  cpu.run_with_callback(move |cpu| {
    handle_user_input(cpu, &mut event_pump);
    cpu.write_u8(0xfe, rng.gen_range(1, 16));
    if read_screen_state(cpu, &mut screen_state) {
      texture.update(None, &screen_state, 32 * 3).unwrap();
      canvas.copy(&texture, None, None).unwrap();
      canvas.present();
    }
    ::std::thread::sleep(std::time::Duration::new(0, 70_000));
  });
  // /TEMPORARY

  // ui::UI::run(iced::Settings::default());
}
