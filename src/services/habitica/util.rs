use std::{fmt, env};

use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
use reqwest::blocking as req;
use serde::Deserialize;

use crate::error::AppError;

#[cfg(feature = "tui")]
use crate::services::habitica::tui;


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

const HABITICA_API_ENDPOINT: &str = "https://habitica.com/api/v3";

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
  // Test that env was loaded correctly
  get_env_vars()?;

  Ok(())
}

#[cfg(feature = "tui")]
pub fn start_interactive() -> Result<(), AppError> {
  tui::start()?;
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
