use std::{env, fs, path::Path};
use std::process::Command;

use crate::error::AppError;

const GIT_BASE_PATH: &str = "/srv/git";

pub fn assert_service_installed() -> Result<(), AppError> {
  // Check that env vars are loaded
  env::var("FORGE_REMOTE")?;
  env::var("GH_REMOTE")?;

  if !Path::new(GIT_BASE_PATH).is_dir() {
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

fn chown_repo(target: &String) -> Result<(), AppError> {
  Command::new("chown")
    .args(["-R", "git:git", &target])
    .status()?;

  Ok(())
}


pub fn make_bare_repository(target: String) -> Result<(), AppError> {
  if Path::new(format!("{GIT_BASE_PATH}/{target}.git").as_str()).is_dir() {
    Err(AppError::ServiceError(format!("{target} already exists.")))?
  }
  env::set_current_dir(GIT_BASE_PATH)?;

  Command::new("git")
    .args(["init", "--bare", format!("{target}.git").as_str()])
    .status()?;
  
  chown_repo(&target)?;

  Ok(())
}

pub fn clone_mirror_repository(remote_target: String) -> Result<(), AppError> {
  // Parse <repo>.git from remote
  let Some(target) = remote_target.clone().split("/").last().map(|s| s.to_string()) else {
    Err(AppError::ServiceError(format!("Invalid remote_target: {remote_target}")))?
  };

  if Path::new(format!("{GIT_BASE_PATH}/{target}").as_str()).is_dir() {
    Err(AppError::ServiceError(format!("{target} already exists.")))?
  }
  env::set_current_dir(GIT_BASE_PATH)?;

  Command::new("git")
    .args(["clone", "--mirror", &remote_target])
    .status()?;

  chown_repo(&target)?;
  
  Ok(())
}

pub fn push_mirror_repository(target: String) -> Result<(), AppError> {
  let repo = format!("{GIT_BASE_PATH}/{target}.git");
  if !Path::new(repo.as_str()).is_dir() {
    Err(AppError::ServiceError(format!("{target} does not exist.")))?
  }
  env::set_current_dir(repo)?;

  let forge_remote = env::var("FORGE_REMOTE")?;
  let gh_remote = env::var("GH_REMOTE")?;

  // Push to Forgejo
  Command::new("git")
    .args(["push", "--mirror", format!("{}/{}.git", forge_remote, target).as_str()])
    .status()?;

  // Push to GH
  Command::new("git")
    .args(["push", "--mirror", format!("{}/{}.git", gh_remote, target).as_str()])
    .status()?;


  Ok(())
}
