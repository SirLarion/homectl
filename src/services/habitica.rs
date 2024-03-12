use clap::Subcommand;

use crate::error::AppError;

pub mod util;
use util::*; 

#[cfg(feature = "tui")]
pub mod tui;

#[derive(Subcommand)]
pub enum Operation {
  /// List all TODOs
  List,
  /// Create a new TODO item
  Task {
    /// Optionally define TODO item with a descriptor. Format:
    /// <name>,<difficulty>,<notes>,<due>,<checklist1>;<checklist2>;...
    descriptor: Option<String>
  }
}


pub fn run_service(operation: Option<Operation>) -> Result<(), AppError> {
  assert_service_installed()?;

  match operation {
    Some(Operation::List) => list_tasks()?,
    Some(Operation::Task { descriptor }) => create_task(descriptor)?,
    None => {
      #[cfg(feature = "tui")]   
      start_interactive()?;
    }
  }

  Ok(())
}
