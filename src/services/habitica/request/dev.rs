use serde_json;
use std::fs::{self, File};
use std::io::Write;
use tokio::time::{sleep, Duration};

use crate::{
    error::AppError,
    services::habitica::{
        types::{Task, TaskId},
        util::{get_json_path, ArrayRes},
    },
};

pub async fn post_created_task(task: Task) -> Result<Task, AppError> {
    let path = get_json_path()?;
    let data = fs::read_to_string(&path)?;
    let mut tasks = serde_json::from_str::<ArrayRes<Task>>(data.as_str())?.data;

    tasks.insert(0, task.clone());

    let mut file = File::create(&path)?;
    let data = serde_json::to_string(&ArrayRes { data: tasks })?;
    file.write_all(data.as_bytes())?;

    Ok(task)
}

pub async fn edit_task(task: &Task) -> Result<&Task, AppError> {
    let path = get_json_path()?;
    let data = fs::read_to_string(&path)?;
    let mut tasks = serde_json::from_str::<ArrayRes<Task>>(data.as_str())?.data;

    let mut iter = tasks.iter_mut();
    let index_of = iter.position(|t| t.id == task.id);

    if let Some(index) = index_of {
        let _ = std::mem::replace(&mut tasks[index], task.clone());
    } else {
        tasks.insert(0, task.clone());
    }

    let mut file = File::create(&path)?;
    let data = serde_json::to_string(&ArrayRes { data: tasks })?;
    file.write_all(data.as_bytes())?;

    Ok(task)
}

pub async fn remove_task(task_id: TaskId) -> Result<Task, AppError> {
    let path = get_json_path()?;
    let data = fs::read_to_string(&path)?;
    let mut tasks = serde_json::from_str::<ArrayRes<Task>>(data.as_str())?.data;

    let mut iter = tasks.iter_mut();
    let task = iter
        .position(|t| t.id == task_id)
        .and_then(|i| Some(tasks.remove(i)))
        .ok_or(AppError::ServiceError(format!(
            "Task with ID: {task_id} not found"
        )))?;

    let mut file = File::create(&path)?;
    let data = serde_json::to_string(&ArrayRes { data: tasks })?;
    file.write_all(data.as_bytes())?;

    Ok(task)
}

pub async fn complete_task(task_id: TaskId) -> Result<(), AppError> {
    remove_task(task_id).await?;
    Ok(())
}

pub async fn reorder_task(task_id: TaskId, index: usize) -> Result<(), AppError> {
    let path = get_json_path()?;
    let data = fs::read_to_string(&path)?;
    let mut tasks = serde_json::from_str::<ArrayRes<Task>>(data.as_str())?.data;

    let mut iter = tasks.iter_mut();
    let i_old = iter.position(|t| t.id == task_id).unwrap();
    let task = tasks.remove(i_old);

    tasks.insert(index, task);
    let mut file = File::create(&path)?;
    let data = serde_json::to_string(&ArrayRes { data: tasks })?;
    file.write_all(data.as_bytes())?;

    Ok(())
}

/// Mock version of the fetch_tasks function to avoid unnecessary API calls.
/// Reads data from ~/.config/hutctl/habitica_tasks.json and will fail if such
/// a file does not exist
pub async fn fetch_tasks() -> Result<String, AppError> {
    let path = get_json_path()?;
    let data = fs::read_to_string(path)?;

    // Artificial delay
    sleep(Duration::from_millis(500)).await;

    Ok(data)
}
