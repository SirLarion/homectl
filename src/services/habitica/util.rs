use std::error::Error;
use std::{fmt, env};

use inquire::list_option::ListOption;
use inquire::validator::Validation;
use inquire::{Text, MultiSelect};
use log::debug;
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
use reqwest::blocking as req;
use serde::{Serialize, Deserialize};

use crate::error::AppError;

#[cfg(feature = "tui")]
use crate::services::habitica::tui;

#[derive(Serialize, Deserialize, Default)]
struct SubTask {
  text: String,
  completed: bool
}

#[derive(Serialize, Deserialize, Default)]
struct Task {
  text: String,
  #[serde(rename = "type")]
  task_type: String,
  priority: f32,
  notes: Option<String>,
  date: Option<String>,
  checklist: Option<Vec<SubTask>>,
}


impl fmt::Display for Task {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", &self.text)?;
    let _ = &self.notes.clone().map(|n| write!(f, "\n{}", n));
    let _ = &self.date.clone().map(|d| write!(f, "\nDue: {}", d));

    if let Some(subtasks) = &self.checklist {
      for SubTask { text, completed } in subtasks {
        let check = if *completed { "[x]" } else { "[ ]" };
        write!(f, "\n{check} {text}")?;
      }
    }
    write!(f, "\n")
  }
}

#[derive(Deserialize)]
struct ArrayRes<T> {
  data: Vec<T>,
}
#[derive(Deserialize)]
struct SingleRes<T> {
  data: T,
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

fn prompt_text_validator(input: &str) -> Result<Validation, Box<dyn Error + Send + Sync>> {
  if input.chars().count() == 0 {
    Ok(Validation::Invalid("Field cannot be empty.".into()))
  } else if input.chars().count() > 140 {
    Ok(Validation::Invalid("You're only allowed 60 characters.".into()))
  } else {
    Ok(Validation::Valid)
  }
}

fn prompt_multi_validator(selected: &[ListOption<&&str>]) -> Result<Validation, Box<dyn Error + Send + Sync>>  {
  if selected.len() != 1 {
    return Ok(Validation::Invalid("Select exactly one of the options.".into()));
  }

  Ok(Validation::Valid)
}

fn parse_difficulty(selected: Vec<&str>) -> Result<f32, AppError> {
  let head = selected[0];

  let parsed: f32 = match head {
    "Trivial" => 0.1,
    "Easy" => 1.0,
    "Medium" => 1.5,
    "Hard" => 2.0,
    _ => Err(AppError::CmdError("Incorrect difficulty value".into()))?
  };

  Ok(parsed)
}

fn parse_task_descriptor(descriptor: String) -> Result<Task, AppError> {
  let mut parts = descriptor.split(",");
  let parts = (parts.next(), parts.next(), parts.next(), parts.next(), parts.next());
  match parts {
    (Some(text), Some(priority), notes, date, check) => {
      return Ok(Task { 
        text: text.into(), 
        task_type: "todo".into(), 
        priority: priority.parse()?, 
        notes: notes.map(|n| n.into()), 
        date: date.map(|d| d.into()), 
        checklist: check.map(|c| c.split(";").map(|i| SubTask { text: i.into(), completed: false }).collect())
      }); 
    },
    (None, ..) => Err(AppError::CmdError("Incorrect input: <name> required".into()))?,
    (_, None, ..) => Err(AppError::CmdError("Incorrect input: <difficulty> required".into()))?
  }
}

fn prompt_for_task() -> Result<Task, AppError> {
  let name = Text::new("Task name:")
    .with_validator(prompt_text_validator)
    .prompt()?;

  let difficulty = MultiSelect::new("Difficulty:", vec!["Trivial", "Easy", "Medium", "Hard"])
    .with_validator(prompt_multi_validator)
    .with_vim_mode(true)
    .prompt()?;

  Ok(Task {
    text: name,
    task_type: "todo".into(),
    priority: parse_difficulty(difficulty)?,
    notes: None,
    date: None,
    checklist: None,
  })
}

pub fn create_task(descriptor: Option<String>) -> Result<(), AppError> {
  let task: Task; 
  if descriptor.is_some() {
    task = parse_task_descriptor(descriptor.unwrap())?; 
  } else {
    task = prompt_for_task()?;
  }
  debug!("Creating task: \n{task}");
  let client = req::Client::new();
  let headers = get_headers()?;
  let res = client
    .post(format!("{HABITICA_API_ENDPOINT}/tasks/user"))
    .json::<Task>(&task)
    .headers(headers)
    .send()?
    .error_for_status()?;

  let created = serde_json::from_str::<SingleRes<Task>>(&res.text()?)?;
  println!("Created: \n{}", created.data);
 
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
    println!("{task}");
  }

  Ok(())
}
