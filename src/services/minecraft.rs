use std::{fs, env, path::Path};
use std::process::Command;

use log::info;
use serde::{Deserialize, Deserializer};

use crate::error::AppError;
use crate::types::Operation;
use crate::util::SYSTEMCTL_OPERATIONS;


#[derive(Deserialize, Debug)]
struct MCDownloadsIndex {
    #[serde(rename = "downloads", deserialize_with = "lift_nested_server_info")]
    url_and_sha: (String, String),
}

fn lift_nested_server_info<'de, D>(deserializer: D) -> Result<(String, String), D::Error>
    where D: Deserializer<'de>
{
  #[derive(Deserialize)]
  struct Downloads {
    server: Server,
  }
  #[derive(Deserialize)]
  struct Server {
    sha1: String,
    url: String
  }

  Downloads::deserialize(deserializer).map(|dl| { let s = dl.server; (s.url, s.sha1)})
}

#[derive(Deserialize, Debug)]
struct MCVersion {
  #[serde(rename = "type")]
  kind: String,
  url: String,

} 

#[derive(Deserialize, Debug)]
struct MCVersionManifest {
  versions: Vec<MCVersion>
}

const MC_MANIFEST_URL: &str = "https://launchermeta.mojang.com/mc/game/version_manifest.json";
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
    .status()?;

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
  if instance.is_empty() {
    return Err(AppError::ServiceError("no service enabled.".into()));
  }
  Ok(instance)
}

fn get_target_or_enabled(target: Option<String>) -> Result<String, AppError> {
  match target {
    Some(t) => Ok(t),
    None    => get_enabled_instance()
  }
}

fn get_backup_dir() -> Result<String, AppError> {
  let sudo_user_var = env::var("SUDO_USER");
  let backup_dir_var = env::var("BACKUP_DIR");

  match (sudo_user_var, backup_dir_var) {
    (_, Ok(bak_dir)) => Ok(format!("{bak_dir}/minecraft")),
    (Ok(user), _) => Ok(format!("/home/{user}/minecraft")),
    _ => Err(AppError::ServiceError("backup directory not defined.".into())),
  }
}

fn download_mc_server() -> Result<(), AppError> {
  println!("Downloading server.jar...");

  let manifest = reqwest::blocking::get(MC_MANIFEST_URL)?
    .json::<MCVersionManifest>()?;

  let Some(version_url) = manifest.versions.into_iter().find(|mcv| {
    mcv.kind == "release"
  }).map(|mcv| { mcv.url }) else {
    Err(AppError::ServiceError("no stable release of Minecraft found.".into()))?
  };

  let (url, sha) = reqwest::blocking::get(version_url)?
    .json::<MCDownloadsIndex>()?.url_and_sha;

  println!("this may take a while.");
  Command::new("wget")
    .arg(url)
    .status()?;

  println!("Verifying server.jar integrity");
  let sha_bytes = Command::new("sha1sum")
    .arg("server.jar")
    .output()?.stdout;

  let sha_output = String::from_utf8_lossy(&sha_bytes);
  let server_sha = sha_output.split(" ").next();

  if server_sha != Some(&sha) {
    fs::remove_file("server.jar")?;
    Err(AppError::ServiceError("server.jar SHA checksum could not be verified.".into()))?
  }

  Ok(())
}

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

fn restart(target: Option<String>) -> Result<(), AppError> {
  let target = get_target_or_enabled(target)?;
  assert_target_exists(&target)?;

  run_systemctl("restart", &target).and_then(|()| {
    println!("Restarting minecraft@{target}...");
    Ok(())
  })
}

fn status(target: Option<String>) -> Result<(), AppError> {
  let target = get_target_or_enabled(target)?;
  assert_target_exists(&target)?;

  run_systemctl("status", &target)
}

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

pub fn run_service(operation: Option<Operation>) -> Result<(), AppError> {
  assert_service_installed()?;

  match operation {
    Some(Operation::Init { target })    => init(target)?,
    Some(Operation::Start { target })   => start(target)?,
    Some(Operation::Stop { target })    => stop(target)?,
    Some(Operation::Restart { target }) => restart(target)?,
    Some(Operation::Status { target })  => status(target)?,
    Some(Operation::Backup { target })  => backup(target)?,
    _ => {}
  }

  Ok(())
}
