use std::{fs, env};
use std::process::Command;

use log::info;
use clap::Subcommand;

use crate::error::AppError;

mod util;
use util::*; 

#[derive(Subcommand)]
pub enum Operation {
  Init {
    target: String, 
  },
  Start {
    target: String, 
  },
  Stop {
    target: Option<String>, 
  },
  Restart {
    target: Option<String>, 
  },
  Status {
    target: Option<String>, 
  },
  Backup {
    target: Option<String>, 
  },
  Update {
    target: Option<String>, 
  }
}

// ------------------------------
fn init(target: String) -> Result<(), AppError> {
  let Err(_) = assert_target_exists(&target) else {
    Err(AppError::ServiceError(format!("initialization failed: {target} already exists")))?
  };

  println!("Creating {target}...");
  env::set_current_dir(MC_USER_DIR)?;
  Command::new("cp")
    .args(["-rf", "template", &target])
    .status()?;

  env::set_current_dir(format!("{MC_USER_DIR}/{target}"))?;

  if let Err(e) = download_mc_server() {
    println!("Download failed. Cleaning up...");
    fs::remove_dir_all(&target)?;
    Err(e)?
  };

  println!("Finalizing...");
  env::set_current_dir(MC_USER_DIR)?;

  Command::new("chown")
    .args(["-R", "minecraft", &target])
    .status()?;

  Command::new("chgrp")
    .args(["-R", "minecraft", &target])
    .status()?;

  println!("All done!");
  Ok(())
}

// ------------------------------
fn start(target: String) -> Result<(), AppError> {
  assert_target_exists(&target)?;

  // Stop any current instance
  let _ = stop(None);

  enable_instance(&target).and_then(|()| {
    run_systemctl("start", &target).and_then(|()| {
      println!("Starting minecraft@{target}...");
      Ok(())
    })
  })
}

// ------------------------------
fn stop(target: Option<String>) -> Result<(), AppError> {
  let target = get_target_or_enabled(target)?;
  assert_target_exists(&target)?;

  disable_instance(&target).and_then(|()| {
    run_systemctl("stop", &target).and_then(|()| {
      println!("Stopping minecraft@{target}...");
      Ok(())
    })
  })
}

// ------------------------------
fn restart(target: Option<String>) -> Result<(), AppError> {
  let target = get_target_or_enabled(target)?;
  assert_target_exists(&target)?;

  run_systemctl("restart", &target).and_then(|()| {
    println!("Restarting minecraft@{target}...");
    Ok(())
  })
}

// ------------------------------
fn status(target: Option<String>) -> Result<(), AppError> {
  let target = get_target_or_enabled(target)?;
  assert_target_exists(&target)?;

  run_systemctl("status", &target)
}

// ------------------------------
fn backup(target: Option<String>) -> Result<(), AppError> {
  let target = get_target_or_enabled(target)?;
  assert_target_exists(&target)?; 

  let dir = get_backup_dir()?;

  info!("Backing up {MC_USER_DIR}/{target} to {dir}");

  Command::new("cp")
    .args(["-rf", format!("{MC_USER_DIR}/{target}").as_str(), dir.as_str()])
    .output()?;

  Ok(())
}

// ------------------------------
fn update(target: Option<String>) -> Result<(), AppError> {
  println!("{:?}", target);
  Ok(())
}

pub fn run_service(operation: Option<Operation>) -> Result<(), AppError> {
  assert_service_installed()?;

  match operation {
    Some(Operation::Init { target })    => init(target)?,
    Some(Operation::Start { target })   => start(target)?,
    Some(Operation::Stop { target })    => stop(target)?,
    Some(Operation::Restart { target }) => restart(target)?,
    Some(Operation::Status { target })  => status(target)?,
    Some(Operation::Backup { target })  => backup(target)?,
    Some(Operation::Update { target })  => update(target)?,
    _ => {}
  }

  Ok(())
}
