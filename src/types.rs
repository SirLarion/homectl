use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(next_line_help = true)]
pub struct Cli {
  #[command(subcommand)]
  service: Option<Service> 
}

#[derive(Subcommand)]
enum Service {
  /// operate on a Minecraft server
  Minecraft {
    /// the selected server instance
    target: String, 

    #[command(subcommand)]
    operation: Option<Operation>
  },
  /// operate on the git server
  Git {
    #[command(subcommand)]
    operation: Option<Operation>
  }
}

#[derive(Subcommand)]
enum Operation {
  Start,
  Stop,
  Restart,
}
