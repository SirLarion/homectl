use std::{env, fs, path::Path};
use std::process::Command;

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

fn chown_repo(target: &String)-> Result<(), AppError> {
  Command::new("chown")
    .args(["-R", "git:git", &target])
    .status()?;

  Ok(())
}


pub fn make_bare_repository(target: String) -> Result<(), AppError> {
  if Path::new(format!("/srv/git/{target}").as_str()).is_dir() {
    Err(AppError::ServiceError(format!("{target} already exists.")))?
  }

  Command::new("git")
    .args(["init", "--bare", &target])
    .status()?;
  
  chown_repo(&target)?;

  Ok(())
}

pub fn clone_mirror_repository(target: String) -> Result<(), AppError> {
  if Path::new(format!("/srv/git/{target}").as_str()).is_dir() {
    Err(AppError::ServiceError(format!("{target} already exists.")))?
  }

  Command::new("git")
    .args(["clone", "--mirror", &target])
    .status()?;

  chown_repo(&target)?;
  
  Ok(())
}

pub fn push_mirror_repository(target: String) -> Result<(), AppError> {
  if !Path::new(format!("/srv/git/{target}").as_str()).is_dir() {
    Err(AppError::ServiceError(format!("{target} does not exist.")))?
  }
  let forge_url = env::var("FORGE_URL")?;
  let gh_url = env::var("GH_URL")?;

  // Push to Forgejo
  Command::new("git")
    .args(["push", "--mirror", &target, forge_url.as_str()])
    .status()?;

  // Push to GH
  Command::new("git")
    .args(["push", "--mirror", &target, gh_url.as_str()])
    .status()?;


  Ok(())
}
