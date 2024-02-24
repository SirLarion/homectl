use clap::Subcommand;

use crate::error::AppError;

mod util;
use util::*; 

// #[derive(Clone)]
// enum Task {
//   Habit,
//   Daily,
//   Todo
// }

#[derive(Subcommand)]
pub enum Operation {
  List
}

fn start_interactive() -> Result<(), AppError> {
  Ok(())
}

fn list_tasks() -> Result<(), AppError> {
  Ok(())
}

pub fn run_service(operation: Option<Operation>) -> Result<(), AppError> {
  assert_service_installed()?;

  match operation {
    Some(Operation::List)    => list_tasks()?,
    None => start_interactive()?,
  }

  Ok(())
}
