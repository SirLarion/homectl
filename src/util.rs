use std::env;

use nix::unistd::Uid;
use dotenv::dotenv;

use crate::error::AppError;


pub const SYSTEMCTL_OPERATIONS: [&str; 6] = ["start", "stop", "restart", "enable", "disable", "status"];

const HOMECTL_CONFIG_DIR: &str = ".config/homectl";

pub fn load_env() -> Result<(), AppError> {
  let sudo_user_var = env::var("SUDO_USER");
  let home_var = env::var("HOME");
  let dir: String;

  match (sudo_user_var, home_var) {
    (Ok(user), _) => dir = format!("/home/{user}/{HOMECTL_CONFIG_DIR}"),
    (_, Ok(home)) => dir = format!("{home}/{HOMECTL_CONFIG_DIR}"),
    (Err(_), Err(e)) => return Err(e.into()),
  }

  // Go to config dir and pull .env contents
  if let Err(_) = env::set_current_dir(dir) {
    return Err(AppError::ServiceError("$HOME/.config/homectl not found".to_string()));
  }

  dotenv().ok();
  Ok(())
}

pub fn assert_root() -> Result<(), AppError> {
  if let Err(_) = env::var("HOMECTL_DEBUG") {
    if !Uid::effective().is_root() {
    return Err(
      AppError::AclError(
        "You cannot perform this operation unless you are root.".into()
    ))
    }
  }

  Ok(())
}
