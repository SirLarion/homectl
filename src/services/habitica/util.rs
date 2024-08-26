use std::fs;
use std::fs::File;
use std::io::Write;
use std::env;

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
use super::types::{Task, SubTask, Difficulty, TaskId};

pub const ISO8601: Iso8601 = Iso8601::DEFAULT;

#[derive(Serialize, Deserialize)]
struct ArrayRes<T> {
  data: Vec<T>,
}
#[derive(Serialize, Deserialize)]
struct SingleRes<T> {
  data: T,
}

#[cfg(not(debug_assertions))]
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

#[cfg(not(debug_assertions))]
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

fn parse_difficulty(selected: &str) -> Result<Difficulty, AppError> {
  let parsed: Difficulty = match selected {
    "Trivial" => Difficulty::TRIVIAL,
    "Easy"    => Difficulty::EASY,
    "Medium"  => Difficulty::MEDIUM,
    "Hard"    => Difficulty::HARD,
    _ => Err(AppError::CmdError("Incorrect difficulty value".into()))?
  };

  Ok(parsed)
}

fn parse_task_descriptor(descriptor: String) -> Result<Task, AppError> {
  let mut parts = descriptor.split(",");
  let parts = (parts.next(), parts.next(), parts.next(), parts.next(), parts.next());
  match parts {
    (Some(text), Some(difficulty), notes, date, check) => {
      return Ok(Task { 
        id: TaskId::empty(),
        text: text.into(), 
        task_type: "todo".into(), 
        difficulty: parse_difficulty(difficulty)?, 
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

  let difficulty = Select::new("Difficulty:", vec![Difficulty::TRIVIAL, Difficulty::EASY, Difficulty::MEDIUM, Difficulty::HARD])
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
    id: TaskId::empty(),
    text: name,
    task_type: "todo".into(),
    difficulty,
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
pub async fn edit_task(task: &Task) -> Result<&Task, AppError> {
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
pub async fn edit_task(task: &Task) -> Result<Task, AppError> {
  let client = req::Client::new();
  let headers = get_headers()?;
  let res = client
    .put(format!("{HABITICA_API_ENDPOINT}/tasks/{}", task.id))
    .json::<Task>(task)
    .headers(headers)
    .send().await?
    .error_for_status()?;

  let created = serde_json::from_str::<SingleRes<Task>>(&res.text().await?)?;
  Ok(created.data)
}

#[cfg(debug_assertions)]
pub async fn remove_task(task_id: TaskId) -> Result<Task, AppError> {
  let path = get_json_path()?;
  let data = fs::read_to_string(path)?;
  let mut tasks = serde_json::from_str::<ArrayRes<Task>>(data.as_str())?.data;

  let mut iter = tasks.iter_mut();
  let task = iter.position(|t| t.id == task_id)
    .and_then(|i| Some(tasks.remove(i)))
    .ok_or(AppError::ServiceError(format!("Task with ID: {task_id} not found")))?;

  let mut file = File::create(get_json_path()?)?; 
  let data = serde_json::to_string(&ArrayRes { data: tasks })?;
  file.write_all(data.as_bytes())?;

  Ok(task)
}

#[cfg(not(debug_assertions))]
pub async fn remove_task(task_id: TaskId) -> Result<Task, AppError> {
  let client = req::Client::new();
  let headers = get_headers()?;
  let res = client
    .delete(format!("{HABITICA_API_ENDPOINT}/tasks/{}", task_id))
    .headers(headers)
    .send().await?
    .error_for_status()?;

  let removed = serde_json::from_str::<SingleRes<Task>>(&res.text().await?)?;
  Ok(removed.data)
}

#[cfg(debug_assertions)]
pub async fn complete_task(task_id: TaskId) -> Result<(), AppError> {
  remove_task(task_id).await?;
  Ok(())
}

#[cfg(not(debug_assertions))]
pub async fn complete_task(task_id: TaskId) -> Result<(), AppError> {
  let client = req::Client::new();
  let headers = get_headers()?;
  let res = client
    .post(format!("{HABITICA_API_ENDPOINT}/tasks/{}/score/up", task_id))
    .headers(headers)
    .send().await?
    .error_for_status()?;

  Ok(())
}

#[cfg(debug_assertions)]
pub async fn reorder_task(task_id: TaskId, index: usize) -> Result<(), AppError> {
  let path = get_json_path()?;
  let data = fs::read_to_string(path)?;
  let mut tasks = serde_json::from_str::<ArrayRes<Task>>(data.as_str())?.data;

  let mut iter = tasks.iter_mut();
  let task = iter.position(|t| t.id == task_id)
    .and_then(|i| Some(tasks.remove(i)))
    .ok_or(AppError::ServiceError(format!("Task with ID: {task_id} not found")))?;

  tasks.remove(index);
  tasks.insert(index, task);

  let mut file = File::create(get_json_path()?)?; 
  let data = serde_json::to_string(&ArrayRes { data: tasks })?;
  file.write_all(data.as_bytes())?;

  Ok(())
}

#[cfg(not(debug_assertions))]
pub async fn reorder_task(task_id: TaskId, index: usize) -> Result<(), AppError> {
  let client = req::Client::new();
  let headers = get_headers()?;
  let res = client
    .post(format!("{HABITICA_API_ENDPOINT}/tasks/{}/move/to/{}", task_id, index))
    .headers(headers)
    .send().await?
    .error_for_status()?;

  Ok(())
}

/// Mock version of the fetch_tasks function to avoid unnecessary API calls.
/// Reads data from ~/.config/hutctl/habitica_tasks.json and will fail if such
/// a file does not exist
#[cfg(debug_assertions)]
async fn fetch_tasks() -> Result<String, AppError> {
  let path = get_json_path()?;
  let data = fs::read_to_string(path)?;

  // Artificial delay
  sleep(Duration::from_millis(500)).await;

  Ok(data)
}

/// Fetch all tasks of type: todo from Habitica API. For our purposes a "todo"
/// task is the same as a task in general
#[cfg(not(debug_assertions))]
async fn fetch_tasks() -> Result<String, AppError> {
  let client = req::Client::new();
  let headers = get_headers()?;
  let res = client
    .get(format!("{HABITICA_API_ENDPOINT}/tasks/user?type=todos"))
    .headers(headers)
    .send()
    .await?;

  Ok(res.text().await?)
}

pub async fn get_task_list() -> Result<Vec<Task>, AppError> {
  let raw_tasks = fetch_tasks().await?;
  let tasks = serde_json::from_str::<ArrayRes<Task>>(raw_tasks.as_str())?.data;
  Ok(tasks)
}

pub async fn list_tasks(save_json: bool) -> Result<(), AppError> {
  let raw_tasks = fetch_tasks().await?;
  let tasks = serde_json::from_str::<ArrayRes<Task>>(raw_tasks.as_str())?.data;

  for task in tasks {
    println!("{task}");
  }

  if save_json {
    let mut file = File::create(get_json_path()?)?; 
    file.write_all(raw_tasks.as_bytes())?;
    println!("\nSaved list to ~/.config/habitica_tasks.json");
  }

  Ok(())
}
