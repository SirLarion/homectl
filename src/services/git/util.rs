use std::fs;
use std::path::Path;

use crate::error::AppError;

pub fn assert_service_installed() -> Result<(), AppError> {
  if !Path::new("/srv/git").is_dir() {
    Err(AppError::ServiceError("git root directory not found.".into()))?
  }

  let f_users = fs::read_to_string("/etc/passwd")?;

  for row in f_users.split("\n") {
    if row.starts_with("git") && row.contains("/home/git") {
      return Ok(());
    }
  }

  Err(AppError::ServiceError("invalid git user.".into()))
}
