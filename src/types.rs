use clap::{Parser, Subcommand};

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
    operation: Option<Operation>,
    
  },
  /// operate on the git server
  Git
}

#[derive(Subcommand)]
pub enum Operation {
  Init {
    /// the selected server instance
    target: String, 
  },
  Start {
    /// the selected server instance
    target: String, 
  },
  Stop {
    /// optional targeted server instance
    target: Option<String>, 
  },
  Restart {
    /// optional targeted server instance
    target: Option<String>, 
  },
  Status {
    /// optional targeted server instance
    target: Option<String>, 
  },
  Backup {
    /// optional targeted server instance
    target: Option<String>, 
  }
}
