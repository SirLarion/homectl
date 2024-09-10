use super::widgets::editor::EditorState;
use super::widgets::grid::TaskGridState;

use tokio::sync::mpsc;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::task::JoinSet;

use crate::services::habitica::request::{
    complete_task, edit_task, post_created_task, remove_task, reorder_task,
};
use crate::services::habitica::types::{Action, Task};
use crate::services::habitica::util::get_task_list;

#[derive(PartialEq)]
pub enum AppState {
    List,
    Exit,
    Editor,
}

pub struct Habitui<'e> {
    pub state: AppState,
    pub grid_state: TaskGridState,
    pub editor_state: Option<EditorState<'e>>,
    pub tx: Sender<Vec<(Task, Action)>>,
    pub rx: Receiver<Vec<(Task, Action)>>,
    pub should_refresh_tasks: bool,
    pub log_debug: Option<(String, u32)>,
}

impl Default for Habitui<'_> {
    fn default() -> Self {
        let (tx, rx) = mpsc::channel::<Vec<(Task, Action)>>(1);
        Self {
            state: AppState::List,
            grid_state: TaskGridState::default(),
            tx,
            rx,
            should_refresh_tasks: true,
            editor_state: None,
            log_debug: None,
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
                    let tasks_msg = tasks_res.into_iter().map(|t| (t, Action::Create)).collect();
                    if let Err(_) = tx.send(tasks_msg).await {}
                }
            });
        }
        if let Ok(tasks) = self.rx.try_recv() {
            if self.grid_state.task_items.len() == 0 {
                self.grid_state.task_items = tasks.into_iter().map(|(t, _)| t).collect();
            } else {
                let mut updates: Vec<(Option<usize>, (Task, Action))> = Vec::new();
                for task in tasks {
                    let i_task = self
                        .grid_state
                        .task_items
                        .iter_mut()
                        .position(|t| t.id == task.0.id);

                    updates.push((i_task, task));
                }
                for (index_of, (task, action)) in updates {
                    let exists = index_of.is_some();
                    match action {
                        Action::Create => {
                            self.grid_state.task_items.insert(0, task);
                        }
                        Action::ToggleComplete | Action::Remove if exists => {
                            self.grid_state.task_items.remove(index_of.unwrap());
                        }
                        Action::Edit(_) if exists => {
                            let _ = std::mem::replace(
                                &mut self.grid_state.task_items[index_of.unwrap()],
                                task,
                            );
                        }
                        _ => {}
                    }
                }
                self.grid_state.modifications.clear();
            }
        }
    }

    pub fn handle_submit_task(&mut self, task: Task) {
        let tx = self.tx.clone();

        tokio::spawn(async move {
            if task.id.is_empty() {
                if let Ok(create_res) = post_created_task(task).await {
                    if let Err(_) = tx.send(vec![(create_res, Action::Create)]).await {}
                }
            } else {
                if let Ok(update_res) = edit_task(&task).await {
                    if let Err(_) = tx
                        .send(vec![(update_res.clone(), Action::Edit(task))])
                        .await
                    {}
                }
            }
        });
    }

    pub fn handle_submit_modifications(&mut self) {
        let tx = self.tx.clone();
        let tasks = self.grid_state.task_items.clone();
        let task_edits = self.grid_state.modifications.clone();

        tokio::spawn(async move {
            let mut handle_set: JoinSet<(Task, Vec<Action>)> = JoinSet::new();
            for (id, mods) in task_edits {
                let task = tasks.iter().find(|t| t.id == id).unwrap().clone();
                handle_set.spawn(async move {
                    let mut updates: (Task, Vec<Action>) = (task, Vec::new());
                    let mut destructive_update: Option<Action> = None;
                    for m in mods {
                        match m {
                            Action::Edit(m_task) => {
                                let _ = edit_task(&m_task).await.and_then(|res| {
                                    updates.0 = res.clone();
                                    updates.1.push(Action::Edit(res.clone()));
                                    Ok(())
                                });
                            }
                            Action::ToggleComplete | Action::Remove => {
                                destructive_update = Some(m);
                            }
                            Action::Reorder(o) => {
                                let _ = reorder_task(id.clone(), o.1).await;
                                updates.1.push(Action::Reorder(o));
                            }
                            _ => {}
                        }
                    }
                    if let Some(u) = destructive_update {
                        if u == Action::Remove {
                            let _ = remove_task(id.clone()).await;
                        } else {
                            let _ = complete_task(id.clone()).await;
                        }
                        updates.1.push(u)
                    }
                    updates
                });
            }
            let mut updates: Vec<(Task, Action)> = Vec::new();
            while let Some(res) = handle_set.join_next().await {
                if let Ok((task, actions)) = res {
                    for a in actions {
                        updates.push((task.clone(), a));
                    }
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
