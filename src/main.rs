use std::env;

use clap::Parser;

mod error;
mod logger;
mod services;
mod types;
mod util;

use error::*;
use logger::*;
use types::{Cli, Service::*};
use util::{assert_correct_permissions, load_env};

fn main() -> Result<(), AppError> {
    let Cli {
        service,
        verbose,
        debug,
    } = Cli::parse();
    let _ = logger::init(LoggerFlags { verbose, debug });

    if debug {
        env::set_var("HUTCTL_DEBUG", "true");
    }

    load_env()?;

    match service {
        Some(s) => {
            assert_correct_permissions(&s)?;
            match s {
                #[cfg(feature = "minecraft")]
                Minecraft { operation } => services::minecraft::run_service(operation)?,

                #[cfg(feature = "git")]
                Git { operation } => services::git::run_service(operation)?,

                #[cfg(feature = "habitica")]
                Habitica { operation } => {
                    services::habitica::run_service(operation)?;
                }
            }
        }
        None => {}
    }

    Ok(())
}
