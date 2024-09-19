use std::env;

use dotenv::dotenv;
use nix::unistd::Uid;

use crate::{error::AppError, types::Service};

#[cfg(feature = "git")]
use Service::Git;
#[cfg(feature = "minecraft")]
use Service::Minecraft;

pub const SYSTEMCTL_OPERATIONS: [&str; 6] =
    ["start", "stop", "restart", "enable", "disable", "status"];

const HUTCTL_CONFIG_DIR: &str = ".config/hutctl";

pub fn build_config_path() -> Result<String, AppError> {
    let sudo_user_var = env::var("SUDO_USER");
    let home_var = env::var("HOME");
    let dir: String;

    match (sudo_user_var, home_var) {
        (Ok(user), _) => dir = format!("/home/{user}/{HUTCTL_CONFIG_DIR}"),
        (_, Ok(home)) => dir = format!("{home}/{HUTCTL_CONFIG_DIR}"),
        (Err(_), Err(e)) => return Err(e.into()),
    }

    Ok(dir)
}

pub fn load_env() -> Result<(), AppError> {
    let cwd = env::current_dir()?;
    let dir = build_config_path()?;

    // Go to config dir and pull .env contents
    if let Err(_) = env::set_current_dir(dir) {
        return Err(AppError::ServiceError(
            "$HOME/.config/hutctl not found".to_string(),
        ));
    }

    dotenv().ok();

    env::set_current_dir(cwd)?;
    Ok(())
}

// Map service to checks for correct access permissions
pub fn assert_correct_permissions(service: &Service) -> Result<(), AppError> {
    // Return early if root or debug
    if Uid::effective().is_root() {
        return Ok(());
    }

    if let Ok(_) = env::var("HUTCTL_DEBUG") {
        return Ok(());
    }

    let user = env::var("USER")?;
    match service {
        #[cfg(feature = "minecraft")]
        Minecraft { .. } => {
            if user == "minecraft" {
                return Ok(());
            }
        }

        #[cfg(feature = "git")]
        Git { .. } => {
            if user == "git" {
                return Ok(());
            }
        }
    }
    Err(AppError::AclError(
        "Performing operation failed: Unauthorized access".into(),
    ))
}
