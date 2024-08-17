use std::fs;
use std::fs::File;
use std::io::Write;
use std::{fmt, env};

use time::{
  OffsetDateTime, 
  format_description::well_known::Iso8601
};
use tokio::time::{sleep, Duration};
use inquire::{Text, Select, DateSelect, min_length, max_length};
use log::debug;
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
use serde::{Serialize, Deserialize};

#[cfg(not(debug_assertions))]
use reqwest as req;

use crate::error::AppError;
use crate::util::build_config_path;
use super::types::{Task, SubTask};

pub const ISO8601: Iso8601 = Iso8601::DEFAULT;

impl fmt::Display for Task {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", &self.text)?;
    let _ = &self.notes.clone().map(|n| write!(f, "\n{}", n));
    let _ = &self.date.clone().map(|d| write!(f, "\n{}", d.format(&Iso8601::DATE).unwrap()));

    if let Some(subtasks) = &self.checklist {
      for SubTask { text, completed } in subtasks {
        let check = if *completed { "✅" } else { 
          if cfg!(feature = "dark-mode") {
            "⬛"
          } else {
            "⬜" 
          }
        };
        write!(f, "\n{check} {text}")?;
      }
    }
    write!(f, "\n")
  }
}

#[derive(Serialize, Deserialize)]
struct ArrayRes<T> {
  data: Vec<T>,
}
#[derive(Serialize, Deserialize)]
struct SingleRes<T> {
  data: T,
}

const HABITICA_API_ENDPOINT: &str = "https://habitica.com/api/v3";

fn get_json_path() -> Result<String, AppError> {
  let dir = build_config_path()?;
  Ok(format!("{dir}/habitica_tasks.json"))
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
  // Test that env was loaded correctly
  get_env_vars()?;

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
        id: "".into(),
        text: text.into(), 
        task_type: "todo".into(), 
        priority: priority.parse()?, 
        notes: notes.map(|n| n.into()), 
        date: date.map(|d| OffsetDateTime::parse(d.into(), &ISO8601).unwrap()), 
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
    .map(|d| OffsetDateTime::parse(&d.format("%F").to_string(), &ISO8601).unwrap());

  let checklist = prompt_for_checklist()?;

  Ok(Task {
    id: "".into(),
    text: name,
    task_type: "todo".into(),
    priority: parse_difficulty(difficulty)?,
    notes: if notes.is_empty() { None } else { Some(notes) },
    date,
    checklist,
  })
}

pub async fn create_task(descriptor: Option<String>) -> Result<(), AppError> {
  let task: Task; 
  if descriptor.is_some() {
    task = parse_task_descriptor(descriptor.unwrap())?; 
  } else {
    task = prompt_for_task()?;
  }
  debug!("Creating task: \n{task}");

  let created = post_created_task(task).await?;

  println!("Created: \n{}", created);
 
  Ok(())
}

#[cfg(not(debug_assertions))]
pub async fn post_created_task(task: Task) -> Result<Task, AppError> {
  let client = req::Client::new();
  let headers = get_headers()?;
  let res = client
    .post(format!("{HABITICA_API_ENDPOINT}/tasks/user"))
    .json::<Task>(&task)
    .headers(headers)
    .send().await?
    .error_for_status()?;

  let created = serde_json::from_str::<SingleRes<Task>>(&res.text().await?)?;

  Ok(created.data)
}

#[cfg(debug_assertions)]
pub async fn post_created_task(task: Task) -> Result<Task, AppError> {
  let path = get_json_path()?;
  let data = fs::read_to_string(path)?;
  let mut tasks = serde_json::from_str::<ArrayRes<Task>>(data.as_str())?.data;

  tasks.insert(0, task.clone());

  let mut file = File::create(get_json_path()?)?; 
  let data = serde_json::to_string(&ArrayRes { data: tasks })?;
  file.write_all(data.as_bytes())?;

  Ok(task)

}

#[cfg(debug_assertions)]
pub async fn edit_task(task: Task) -> Result<Task, AppError> {
  let path = get_json_path()?;
  let data = fs::read_to_string(path)?;
  let mut tasks = serde_json::from_str::<ArrayRes<Task>>(data.as_str())?.data;

  let mut iter = tasks.iter_mut();
  let index_of = iter.position(|t| t.id == task.id);

  if let Some(index) = index_of {
    let _ = std::mem::replace(&mut tasks[index], task.clone());
  } else {
    tasks.insert(0, task.clone());
  }

  let mut file = File::create(get_json_path()?)?; 
  let data = serde_json::to_string(&ArrayRes { data: tasks })?;
  file.write_all(data.as_bytes())?;

  Ok(task)
}

#[cfg(not(debug_assertions))]
pub async fn edit_task(task: Task) -> Result<Task, AppError> {
  let client = req::Client::new();
  let headers = get_headers()?;
  let res = client
    .put(format!("{HABITICA_API_ENDPOINT}/tasks/{}", task.id))
    .json::<Task>(&task)
    .headers(headers)
    .send().await?
    .error_for_status()?;

  let created = serde_json::from_str::<SingleRes<Task>>(&res.text().await?)?;
  Ok(created.data)
}

/// Mock version of the fetch_tasks function to avoid unnecessary API calls.
/// Reads data from ~/.config/hutctl/habitica_tasks.json and will fail if such
/// a file does not exist
#[cfg(debug_assertions)]
async fn fetch_tasks() -> Result<ArrayRes<Task>, AppError> {
  let path = get_json_path()?;
  let data = fs::read_to_string(path)?;
  let tasks = serde_json::from_str::<ArrayRes<Task>>(data.as_str())?;

  // Artificial delay
  sleep(Duration::from_millis(500)).await;

  Ok(tasks)
}

/// Fetch all tasks of type: todo from Habitica API. For our purposes a "todo"
/// task is the same as a task in general
#[cfg(not(debug_assertions))]
async fn fetch_tasks() -> Result<ArrayRes<Task>, AppError> {
  let client = req::Client::new();
  let headers = get_headers()?;
  let res = client
    .get(format!("{HABITICA_API_ENDPOINT}/tasks/user?type=todos"))
    .headers(headers)
    .send()
    .await?;

  let tasks = serde_json::from_str::<ArrayRes<Task>>(&res.text().await?)?;

  Ok(tasks)
}

pub async fn get_task_list() -> Result<Vec<Task>, AppError> {
  let deserialized = fetch_tasks().await?;
  Ok(deserialized.data)
}

pub async fn list_tasks(save_json: bool) -> Result<(), AppError> {
  let deserialized = fetch_tasks().await?;

  for task in &deserialized.data {
    println!("{task}");
  }

  if save_json {
    let mut file = File::create(get_json_path()?)?; 
    let data = serde_json::to_string(&deserialized)?;
    file.write_all(data.as_bytes())?;
    println!("\nSaved list to ~/.config/habitica_tasks.json");
  }

  Ok(())
}
