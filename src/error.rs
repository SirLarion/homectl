use std::{io, env, num};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
  #[error(transparent)]
  IoError(#[from] io::Error),

  #[error(transparent)]
  EnvError(#[from] env::VarError),

  #[error(transparent)]
  HTTPError(#[from] reqwest::Error),

  #[error(transparent)]
  SerdeError(#[from] serde_json::Error),

  #[error(transparent)]
  HeaderError(#[from] reqwest::header::InvalidHeaderValue),

  #[error(transparent)]
  PromptError(#[from] inquire::InquireError),

  #[error(transparent)]
  ParseFloatError(#[from] num::ParseFloatError),

  #[error("incorrect rights for the requested operation")]
  AclError(String),

  #[error("executing command failed: {0}")]
  CmdError(String),

  #[error("running operation for service failed: {0}")]
  ServiceError(String)
}
