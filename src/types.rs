use clap::{Parser, Subcommand};

use crate::services::{minecraft, git};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(next_line_help = true)]
pub struct Cli {
  #[command(subcommand)]
  pub service: Option<Service> ,

  /// Run command verbosely
  #[arg(long, default_value_t = false)]
  pub verbose: bool,

  /// Turn debugging information on
  #[arg(short, long, default_value_t = false)]
  pub debug: bool,

}

#[derive(Subcommand)]
pub enum Service {
  /// operate on a Minecraft server
  Minecraft {
    #[command(subcommand)]
    operation: Option<minecraft::Operation>,
    
  },
  /// operate on the git server
  Git {
    #[command(subcommand)]
    operation: Option<git::Operation>
  }
}
