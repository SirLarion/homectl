use std::{fmt, env};

use inquire::{Text, Select, DateSelect, min_length, max_length};
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


fn parse_difficulty(selected: &str) -> Result<f32, AppError> {
  let parsed: f32 = match selected {
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

fn checklist_item_formatter(i: &str) -> String { format!("[] {i}")}

fn prompt_for_checklist() -> Result<Option<Vec<SubTask>>, AppError> {
  let mut list: Vec<SubTask> = Vec::new();
  let mut finished = false;
  let mut i = 1;

  while !finished {
    let item = Text::new(format!("Checlist item #{i}:").as_str())
      .with_help_message("Press ESC to skip")
      .with_formatter(&checklist_item_formatter)
      .prompt_skippable()?;

    if item.is_none() { 
      finished = true; 
    } else {
      list.push(SubTask { text: item.unwrap(), completed: false })
    }

    i += 1;
  }

  Ok(if list.is_empty() { None } else { Some(list) })
}

fn prompt_for_task() -> Result<Task, AppError> {
  let name = Text::new("Task name:")
    .with_validator(min_length!(1, "Task name cannot be empty."))
    .with_validator(max_length!(60, "Task name must be at most 60 characters."))
    .prompt()?;

  let difficulty = Select::new("Difficulty:", vec!["Trivial", "Easy", "Medium", "Hard"])
    .with_vim_mode(true)
    .prompt()?;

  let notes = Text::new("Extra notes:")
    .with_validator(max_length!(60, "Notes must be at most 60 characters."))
    .prompt()?;

  let date = DateSelect::new("Due date:")
    .with_help_message("Press ESC to skip")
    .prompt_skippable()?
    .map(|d| d.format("%F").to_string());

  let checklist = prompt_for_checklist()?;

  Ok(Task {
    text: name,
    task_type: "todo".into(),
    priority: parse_difficulty(difficulty)?,
    notes: if notes.is_empty() { None } else { Some(notes) },
    date,
    checklist,
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
