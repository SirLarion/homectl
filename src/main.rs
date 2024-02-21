use clap::Parser;
use nix::unistd::Uid;

mod logger;
mod types;
mod error;
mod services;
mod util;

use types::{Cli, Service::*};
use error::*;
use logger::*;

fn main() -> Result<(), AppError> {
  let Cli { service, verbose, debug } = Cli::parse();
  let _ = logger::init(LoggerFlags { verbose, debug });

  if !debug && !Uid::effective().is_root() {
    return Err(
      AppError::AclError(
        "You cannot perform this operation unless you are root.".into()
    ))
  }

  match service {
    Some(Minecraft { operation }) => {
      services::minecraft::run_service(operation)?
    }
    Some(Git { operation }) => {
      services::git::run_service(operation)?
    },
    None => {}
  }

  Ok(())
}
