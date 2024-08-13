use super::{
  widgets::grid::TaskGridState, util::Direction
};

use tokio::sync::mpsc;
use tokio::sync::mpsc::{Sender, Receiver};

use crate::services::habitica::util::get_task_list;
use crate::services::habitica::types::Task;

#[derive(PartialEq)]
pub enum AppState {
  List,
  Exit,
  CreateTask,
  EditTask
}

pub struct Habitui {
  pub state: AppState,
  pub grid_state: TaskGridState,
  pub tx: Sender<Vec<Task>>,
  pub rx: Receiver<Vec<Task>>,
  pub should_refresh_tasks: bool,
}

impl Default for Habitui {
  fn default() -> Self { 
    let (tx, rx) = mpsc::channel::<Vec<Task>>(1);
    Self {
      state: AppState::List,
      grid_state: TaskGridState::default(),
      tx,
      rx,
      should_refresh_tasks: true,
    }
  }
}

impl Habitui {
  fn handle_fetch_tasks(&mut self) {
    if self.should_refresh_tasks {
      let tx = self.tx.clone();

      tokio::spawn(async move {
        if let Ok(tasks_res) = get_task_list().await {
          if let Err(_) = tx.send(tasks_res).await {}
        }
      });
    }
    if let Ok(tasks) = self.rx.try_recv() {
      self.grid_state.set_items(tasks);
    }
  }

  pub fn is_running(&self) -> bool {
    self.state != AppState::Exit
  }

  pub fn tick(&mut self) {
    self.handle_fetch_tasks();
  }

  pub fn get_selected_task(&self) {
    let index = self.grid_state.selected;
  } 
}
