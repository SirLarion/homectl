use clap::Parser;

mod types;
use types::*;

fn main() {
  let Cli { .. } = Cli::parse();
  println!("Hello, world!");
}
