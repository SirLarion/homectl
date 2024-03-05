use clap::Subcommand;

use crate::error::AppError;

mod util;
use util::*; 

#[derive(Subcommand)]
pub enum Operation {
  List
}


pub fn run_service(operation: Option<Operation>) -> Result<(), AppError> {
  assert_service_installed()?;

  match operation {
    Some(Operation::List)    => list_tasks()?,
    None => start_interactive()?,
  }

  Ok(())
}
