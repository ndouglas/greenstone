use clap::Parser;
use std::path::PathBuf;

/// Command-line arguments
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Arguments {
  /// The path to the ROM file to load
  #[clap(short, long, parse(from_os_str))]
  pub file: PathBuf,

  /// Whether to start the WebSocket server
  #[clap(short, long)]
  pub serve: bool,

  /// The port the server should use
  #[clap(long, value_parser, default_value_t = 44553)]
  pub server_port: u16,
}
