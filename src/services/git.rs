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
  },
  Mirror {
    target: String
  }
}

pub fn run_service(operation: Option<Operation>) -> Result<(), AppError> {
  assert_service_installed()?;

  match operation {
    Some(Operation::Init { target })    => make_bare_repository(target)?,
    Some(Operation::Migrate { target }) => clone_mirror_repository(target)?,
    Some(Operation::Mirror { target }) => push_mirror_repository(target)?,
    _ => {}
  }

  Ok(())
}
