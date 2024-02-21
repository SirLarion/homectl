use clap::Subcommand;

use crate::error::AppError;

mod util;
use util::*; 

#[derive(Subcommand)]
pub enum Operation {
  Init {
    target: String, 
  },
  Migrate {
    target: String, 
  }
}

fn init(target: String) -> Result<(), AppError> {
  println!("{target}");
  Ok(())
}

fn migrate(target: String) -> Result<(), AppError> {
  println!("{target}");
  Ok(())
}

pub fn run_service(operation: Option<Operation>) -> Result<(), AppError> {
  assert_service_installed()?;

  match operation {
    Some(Operation::Init { target })    => init(target)?,
    Some(Operation::Migrate { target }) => migrate(target)?,
    _ => {}
  }

  Ok(())
}
