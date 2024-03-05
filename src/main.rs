use std::env;

use clap::Parser;

mod logger;
mod types;
mod error;
mod services;
mod util;

use types::{Cli, Service::*};
use error::*;
use logger::*;
use util::assert_root;

fn main() -> Result<(), AppError> {
  let Cli { service, verbose, debug } = Cli::parse();
  let _ = logger::init(LoggerFlags { verbose, debug });

  if debug {
    env::set_var("HOMECTL_DEBUG", "true");
  }

  match service {
    Some(Minecraft { operation }) => {
      assert_root()?;
      services::minecraft::run_service(operation)?
    }
    Some(Git { operation }) => {
      assert_root()?;
      services::git::run_service(operation)?
    },
    Some(Habitica { operation }) => {
      services::habitica::run_service(operation)?
    }
    None => {}
  }

  Ok(())
}
