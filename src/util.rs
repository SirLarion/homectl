use std::env;

use nix::unistd::Uid;

use crate::error::AppError;


pub const SYSTEMCTL_OPERATIONS: [&str; 6] = ["start", "stop", "restart", "enable", "disable", "status"];

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
