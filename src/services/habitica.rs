use clap::Subcommand;

use crate::error::AppError;

pub mod util;
use util::*; 

#[cfg(feature = "tui")]
pub mod tui;

#[derive(Subcommand)]
pub enum Operation {
  List
}


pub fn run_service(operation: Option<Operation>) -> Result<(), AppError> {
  assert_service_installed()?;

  match operation {
    Some(Operation::List) => list_tasks()?,
    None => {
      #[cfg(feature = "tui")]   
      start_interactive()?;
    }
  }

  Ok(())
}
