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
    target: String, 
  },
  Start {
    target: String, 
  },
  Stop {
    target: Option<String>, 
  },
  Restart {
    target: Option<String>, 
  },
  Status {
    target: Option<String>, 
  },
  Backup {
    target: Option<String>, 
  },
  Update {
    target: Option<String>, 
  }
}
