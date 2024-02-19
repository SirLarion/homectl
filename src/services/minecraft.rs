use std::fs;
use std::process::Command;
use std::path::Path;

use crate::error::AppError;
use crate::types::Operation;
use crate::util::SYSTEMCTL_OPERATIONS;

const MC_USER_DIR: &str = "/opt/minecraft";
const ENABLED_INSTANCE_FILE: &str = "/opt/minecraft/enabled";

fn assert_service_installed() -> Result<(), AppError> {
  if !Path::new(&format!("/etc/systemd/system/minecraft@.service")).is_file() {
    return Err(
      AppError::ServiceError(
        format!("Minecraft template service does not exist.")
      )
    );
  }

  Ok(())
}


fn assert_target_exists(target: &String) -> Result<(), AppError> {
  if !Path::new(&format!("{MC_USER_DIR}/{target}")).is_dir() {
    return Err(
      AppError::ServiceError(
        format!("target: {target} does not exist. Did you mean to call 'homectl minecraft init {target}'?")
      )
    );
  }

  Ok(())
}

fn run_systemctl(op: &str, target: &String) -> Result<(), AppError> {
  if !SYSTEMCTL_OPERATIONS.contains(&op) {
    return Err(AppError::CmdError(format!("invalid systemctl operation: {op}.")));
  }

  Command::new("systemctl")
    .args([op, format!("minecraft@{target}").as_str()])
    .output()?;

  Ok(())
}

fn enable_instance(target: &String) -> Result<(), AppError> {
  if let Ok(enabled) = get_enabled_instance() {
    disable_instance(&enabled)?;
  }

  run_systemctl("enable", target).and_then(|()| {
    fs::write(ENABLED_INSTANCE_FILE, target)?;
    Ok(())
  })
}

fn disable_instance(target: &String) -> Result<(), AppError> {
  let enabled = get_enabled_instance()?;
  if &enabled != target {
    return Err(AppError::ServiceError(format!("cannot disable target: {target}, target is not enabled.")));
  }

  run_systemctl("disable", target).and_then(|()| {
    fs::write(ENABLED_INSTANCE_FILE, "")?;
    Ok(())
  })
}

fn get_enabled_instance() -> Result<String, AppError> {
  let instance = fs::read_to_string(ENABLED_INSTANCE_FILE)?;
  Ok(instance)
}

fn init(target: String) -> Result<(), AppError> {
  println!("{target}");
  Ok(())
}

fn start(target: String) -> Result<(), AppError> {
  assert_target_exists(&target)?;
  enable_instance(&target).and_then(|()| {
    run_systemctl("start", &target)
  })
}

fn stop(target: Option<String>) -> Result<(), AppError> {
  let target = match target {
    Some(t) => t,
    None    => get_enabled_instance()?
  };
  assert_target_exists(&target)?;
  disable_instance(&target).and_then(|()| {
    run_systemctl("stop", &target)
  })
}

fn restart(target: Option<String>) -> Result<(), AppError> {
  let target = match target {
    Some(t) => t,
    None    => get_enabled_instance()?
  };
  assert_target_exists(&target)?;

  run_systemctl("restart", &target)
}

fn status(target: Option<String>) -> Result<(), AppError> {
  let target = match target {
    Some(t) => t,
    None    => get_enabled_instance()?
  };
  assert_target_exists(&target)?;
  run_systemctl("status", &target)
}


pub fn run_service(operation: Option<Operation>, target: Option<String>) -> Result<(), AppError> {
  assert_service_installed()?;
  let err_not_specified = Err(AppError::ServiceError("target not specified.".to_string()));

  match (operation, target.clone()) {
    (Some(Operation::Init),    Some(t)) => init(t)?,
    (Some(Operation::Init),    None)    => err_not_specified?,
    (Some(Operation::Start),   Some(t)) => start(t)?,
    (Some(Operation::Start),   None)    => err_not_specified?,
    (Some(Operation::Stop),    _)       => stop(target)?,
    (Some(Operation::Restart), _)       => restart(target)?,
    (Some(Operation::Status),  _)       => status(target)?,
    _ => {}
  }

  Ok(())
}
