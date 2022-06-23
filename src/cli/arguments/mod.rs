use clap::Parser;

/// Command-line arguments
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Arguments {
  /// The path to the ROM file to load
  #[clap(short, long, parse(from_os_str))]
  pub file: std::path::PathBuf,
}
