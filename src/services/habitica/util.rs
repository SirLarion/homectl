use std::{fmt, env};

use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
use reqwest::blocking as req;
use dotenv::dotenv;
use serde::Deserialize;

use crate::error::AppError;

#[derive(Deserialize)]
struct Task {
  text: String,
  notes: String
}


impl fmt::Display for Task {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}\n  {}\n", &self.text, &self.notes)
  }
}

#[derive(Deserialize)]
struct ArrayRes<T> {
  data: Vec<T>,
}

const HABITICA_CONFIG_DIR: &str = ".config/habitica";
const HABITICA_API_ENDPOINT: &str = "https://habitica.com/api/v3";

fn get_habitica_env() -> Result<(), AppError> {
  let sudo_user_var = env::var("SUDO_USER");
  let home_var = env::var("HOME");
  let dir: String;

  match (sudo_user_var, home_var) {
    (Ok(user), _) => dir = format!("/home/{user}/{HABITICA_CONFIG_DIR}"),
    (_, Ok(home)) => dir = format!("{home}/{HABITICA_CONFIG_DIR}"),
    (Err(_), Err(e)) => return Err(e.into()),
  }

  // Go to config dir and pull .env contents
  if let Err(_) = env::set_current_dir(dir) {
    return Err(AppError::ServiceError("$HOME/.config/habitica not found".to_string()));
  }

  dotenv().ok();
  Ok(())
}

fn get_env_vars() -> Result<(String, String, String), AppError> {
  Ok((
    env::var("HABITICA_USER_ID")?,
    env::var("HABITICA_TOKEN")?,
    env::var("HABITICA_XCLIENT")?
  ))
}

fn get_headers() -> Result<HeaderMap, AppError> {
  let (id, token, xclient) = get_env_vars()?;

  let mut headers = HeaderMap::new();
  headers.insert("x-api-user", HeaderValue::from_str(id.as_str())?);
  headers.insert("x-api-key", HeaderValue::from_str(token.as_str())?);
  headers.insert("x-client", HeaderValue::from_str(xclient.as_str())?);
  headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

  Ok(headers)
}

pub fn assert_service_installed() -> Result<(), AppError> {
  get_habitica_env()?;

  // Test that env was loaded correctly
  get_env_vars()?;

  Ok(())
}

pub fn start_interactive() -> Result<(), AppError> {
  Ok(())
}

pub fn list_tasks() -> Result<(), AppError> {
  let client = req::Client::new();
  let headers = get_headers()?;
  let res = client
    .get(format!("{HABITICA_API_ENDPOINT}/tasks/user?type=todos"))
    .headers(headers)
    .send()?;

  let tasks = serde_json::from_str::<ArrayRes<Task>>(&res.text()?)?;
  for task in tasks.data {
    println!("{}", task);
  }

  Ok(())
}
