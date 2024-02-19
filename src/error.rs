use std::{io, env};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
  #[error(transparent)]
  IoError(#[from] io::Error),

  #[error(transparent)]
  EnvError(#[from] env::VarError),

  #[error("incorrect rights for the requested operation")]
  AclError(String),

  #[error("executing command failed: {0}")]
  CmdError(String),

  #[error("running operation for service failed: {0}")]
  ServiceError(String)
}
