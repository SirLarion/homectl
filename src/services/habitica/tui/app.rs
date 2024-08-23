
use super::widgets::editor::EditorState;
use super::widgets::grid::TaskGridState;

use tokio::join;
use tokio::sync::mpsc;
use tokio::sync::mpsc::{Sender, Receiver};
use tokio::task::{JoinHandle, JoinSet};

use crate::services::habitica::util::{get_task_list, edit_task, post_created_task};
use crate::services::habitica::types::Task;

#[derive(PartialEq)]
pub enum AppState {
  List,
  Exit,
  Editor
}

pub struct Habitui<'e> {
  pub state: AppState,
  pub grid_state: TaskGridState,
  pub editor_state: Option<EditorState<'e>>,
  pub tx: Sender<Vec<Task>>,
  pub rx: Receiver<Vec<Task>>,
  pub should_refresh_tasks: bool,
  pub log_debug: Option<(String, u32)>,
}

impl Default for Habitui<'_> {
  fn default() -> Self { 
    let (tx, rx) = mpsc::channel::<Vec<Task>>(1);
    Self {
      state: AppState::List,
      grid_state: TaskGridState::default(),
      tx,
      rx,
      should_refresh_tasks: true,
      editor_state: None,
      log_debug: None

    }
  }
}

impl Habitui<'_> {
  fn handle_fetch_tasks(&mut self) {
    if self.should_refresh_tasks {
      self.should_refresh_tasks = false;
      let tx = self.tx.clone();

      tokio::spawn(async move {
        if let Ok(tasks_res) = get_task_list().await {
          if let Err(_) = tx.send(tasks_res).await {}
        }
      });
    }
    if let Ok(tasks) = self.rx.try_recv() {
      if self.grid_state.task_items.len() == 0 {
        self.grid_state.task_items = tasks;
      } else {
        let mut iter = self.grid_state.task_items.iter_mut();
        let mut updates: Vec<(Option<usize>, Task)> = Vec::new();
        for task in tasks {
          updates.push((iter.position(|t| t.id == task.id), task));
        }
        for (index_of, task) in updates {
          if let Some(index) = index_of {
            let _ = std::mem::replace(&mut self.grid_state.task_items[index], task);
          } else {
            self.grid_state.task_items.insert(0, task);
          }
        }
        self.grid_state.modified_items = None;
      }
    }
  }

  pub fn handle_submit_task(&mut self, task: Task) {
    let tx = self.tx.clone();

    tokio::spawn(async move {
      if task.id.is_empty() {
        if let Ok(create_res) = post_created_task(task).await {
          if let Err(_) = tx.send(vec![create_res]).await {}
        }
      } else {
        if let Ok(update_res) = edit_task(task).await {
          if let Err(_) = tx.send(vec![update_res]).await {}
        }
      }
    });
  }

  pub fn handle_submit_edits(&mut self) {
    let tx = self.tx.clone();
    let tasks = self.grid_state.task_items.clone();
    let Some(task_edits) = self.grid_state.modified_items.clone() else {
      return;
    };
    tokio::spawn(async move {
      let mut handle_set: JoinSet<Task> = JoinSet::new();
      for (task, completed) in task_edits {
        let is_modified = tasks.iter().find(|t| *t == &task).is_none();
        handle_set.spawn(async move {
          let mut update: Task = task.clone();
          if is_modified {
            if let Ok(res) = edit_task(task).await {
              update = res;
            }
          }
          if completed {}
          update
        });
      }
      let mut updates: Vec<Task> = Vec::new();
      while let Some(res) = handle_set.join_next().await {
        if let Ok(task) = res {
          updates.push(task)
        }
      }
      if let Err(_) = tx.send(updates).await {}
    });
  }

  pub fn is_running(&self) -> bool {
    self.state != AppState::Exit
  }

  // Decay log message TTL
  fn decay_debug_msg(&mut self) {
    if let Some(mut dbg) = self.log_debug.clone() {
      dbg.1 -= 1;
      if dbg.1 == 0 {
        self.log_debug = None;
      } else {
        self.log_debug = Some(dbg);
      }
    }
  }


  pub fn tick(&mut self) {
    self.handle_fetch_tasks();
    
    self.editor_state.as_mut().map(|s| {
      s.decay_mod_key();
    });

    self.grid_state.decay_mod_key();

    self.decay_debug_msg();
  }
}
