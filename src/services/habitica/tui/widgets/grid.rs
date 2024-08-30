use std::{  mem, 
  collections::{HashSet, HashMap},
  cmp::max,
};

use crossterm::event::KeyEvent;
use ratatui::{
  widgets::{Widget, StatefulWidget, Paragraph, Block, Padding}, 
  layout::{Rect, Layout, Constraint, Margin}, 
  buffer::Buffer,
  style::Style
};

use crate::services::habitica::{
  types::{Task, SubTask, TaskId, Action},
  tui::util::{Direction, Palette, MOD_KEY_TTL}
};

const GRID_WIDTH: usize = 3;
const GRID_HEIGHT: usize = 3;
const GRID_SIZE: usize = GRID_WIDTH * GRID_HEIGHT;


type Diff = HashSet<Action>;

pub struct TaskGridState {
  pub page: usize,
  pub selected: Option<usize>,
  pub selected_sub: Option<usize>,
  pub task_items: Vec<Task>,
  pub loading: bool,
  pub modifications: HashMap<TaskId, Diff>,
  pub mod_key: Option<(KeyEvent, u32)>,
}

impl Default for TaskGridState {
  fn default() -> Self {
    Self { page: 0, selected: None, selected_sub: None, task_items: Vec::new(), modifications: HashMap::new(), loading: false, mod_key: None }
  }
}
impl TaskGridState {
  pub fn select_first(&mut self) {
    self.selected_sub = None;
    self.selected = Some(0);
    self.page = 0;
  }

  pub fn select_last(&mut self) {
    self.selected_sub = None;
    let len = self.task_items.len();
    self.selected = Some(len - 1);
    self.page = len / GRID_SIZE;
  }

  pub fn select_next(&mut self, direction: Direction) {
    self.selected_sub = None;

    if let None = self.selected {
      self.selected = Some(self.page * GRID_SIZE);
      return;
    }

    let mut selection = self.selected.unwrap() as i32;
    let w = GRID_WIDTH as i32;
    let h = GRID_HEIGHT as i32;

    selection = match direction {
      Direction::UP    => selection - w,
      Direction::DOWN  => selection + w,
      Direction::LEFT  => selection - 1,
      Direction::RIGHT => selection + 1
    };

    // Clamp selection between 0 and items.len
    selection = selection.clamp(0, max(0, self.task_items.len() as i32 -1));

    // Selection is on previous page
    if selection < (self.page as i32) * w * h {
      self.page -= 1;
    }
    // Selection is on next page
    else if selection >= ((self.page as i32) + 1) * w * h {
      self.page += 1;
    }

    match usize::try_from(selection) {
      Ok(s) => self.selected = Some(s),
      Err(_) => self.selected = Some(0)
    }
  } 

  fn get_all_items(&self) -> Vec<&Task> {
    self.task_items.iter().map(|t| {
      self.modifications.get(&t.id).map_or(t, |mods| {
        let mut task = t;
        for m in mods {
          match m {
            Action::Edit(m_task) => task = m_task,
            _ => {}
          }
        }
        task
      })
    }).collect()
  }

  pub fn select_next_sub(&mut self) {
    let Some(checklist) = self.get_selected_checklist() else {
      return;
    };
    if let Some(selected_sub) = self.selected_sub {
      self.selected_sub = Some((selected_sub + 1) % checklist.len())
    } else {
      self.selected_sub = Some(0);
    }
  }

  pub fn select_prev_sub(&mut self) {
    let Some(checklist) = self.get_selected_checklist() else {
      return;
    };
    if let Some(selected_sub) = self.selected_sub {
      if let Some(valid_i) = selected_sub.checked_sub(1) {
        self.selected_sub = Some(valid_i);
      } else {
        self.selected_sub = Some(checklist.len() - 1);
      }
    } else {
      self.selected_sub = Some(0);
    }
  }

  pub fn move_task(&mut self, direction: Direction) {
    let Some(selected) = self.selected else {
      return; 
    };

    self.select_next(direction);
    let next_selected = self.selected.unwrap();
    let task = self.task_items.remove(selected);

    self.upsert_modified(task.id.clone(), Action::Reorder((selected, next_selected))); 
    self.task_items.insert(next_selected, task);

  }

  pub fn mark_item_completed(&mut self) {
    let Some(mut task) = self.get_selected().cloned() else {
      return;
    };
    let id = task.id.clone();
    if let Some(selected_sub) = self.selected_sub {
      let Some(checklist) = task.checklist.as_mut() else {
        return;
      };
      let subtask_mut = checklist.get_mut(selected_sub).unwrap(); 
      let subtask = subtask_mut.clone();
      let _ = mem::replace(subtask_mut, SubTask { completed: !subtask.completed, ..subtask });
      self.upsert_modified(id, Action::Edit(Task { checklist: Some(checklist.clone()), ..task }));
    } else {
      self.upsert_modified(id, Action::ToggleComplete);
    }
  }

  pub fn mark_item_removed(&mut self) {
    let Some(task) = self.get_selected().cloned() else {
      return;
    };
    self.upsert_modified(task.id, Action::Remove);
  }

  pub fn next_page(&mut self) {
    if self.task_items.len() > ((self.page + 1) * GRID_SIZE).into() {
      self.selected_sub = None;
      self.selected = self.selected.map(|s| s + GRID_SIZE);

      self.page += 1;
    } 
  }

  pub fn prev_page(&mut self) {
    if self.page != 0 {
      self.selected_sub = None;
      self.selected = self.selected.map(|s| s - GRID_SIZE);

      self.page -= 1;
    }
  }

  fn upsert_modified(&mut self, id: TaskId, modification: Action) {
    if let Some(diff) = self.modifications.get_mut(&id) {
      // Allow only one destructive modification
      if modification.is_destructive() {
        let destructive_set = HashSet::from([Action::ToggleComplete, Action::Remove]);
        let diff_clone = diff.clone();

        let m = diff_clone.intersection(&destructive_set).next();
        if let Some(m) = m {
          diff.remove(m);
        } 
        if m.is_none() || m != Some(&modification) {
          diff.insert(modification);
        }
      } else {
        match (&modification, diff.clone().get(&modification)) {
          (Action::Reorder((_, i)), Some(Action::Reorder((original, _)))) => { 
            if *original == *i {
              diff.remove(&modification);
            } else {
              diff.replace(Action::Reorder((*original, *i)));
            }
            return;
          },
          (_, Some(Action::Edit(t))) => {
            if self.task_items.contains(t) {
              diff.remove(&modification); 
              return;
            }
          },
          _ => {}
        }
        diff.replace(modification);
      }
      if diff.is_empty() {
        self.modifications.remove(&id);
      }
    } else {
      self.modifications.insert(id, HashSet::from([modification]));
    }
  }

  pub fn get_selected(&self) -> Option<&Task> {
    self.selected.map(|i| *self.get_all_items().get(i).unwrap())
  }

  fn get_selected_checklist(&self) -> Option<&Vec<SubTask>> {
    self.get_selected().and_then(|task| {
      task.checklist.as_ref().filter(|l| !l.is_empty())
    })
  }

  fn get_line_offset_selected_sub(&self) -> Option<usize> {
    self.selected_sub.map(|index| {
      let mut i = index + 1;
      let task = self.get_selected();
      task.map(|t| {
        t.notes.clone().map(|_| i += 1);
        t.date.map(|_| i += 1);
      });
      i
    })
  }

  pub fn add_mod_key(&mut self, key: KeyEvent) {
    self.mod_key = Some((key, MOD_KEY_TTL));
  }
 
  pub fn pop_mod_key(&mut self) -> Option<KeyEvent> {
    self.mod_key.take().map(|k| k.0)
  }

  // Decay mod key press TTL
  pub fn decay_mod_key(&mut self) {
    if let Some(mut mk) = self.mod_key.clone() {
      mk.1 -= 1;
      if mk.1 == 0 {
        self.mod_key = None;
      } else {
        self.mod_key = Some(mk);
      }
    }
  }
}

pub struct TaskGrid {}

/// TaskGrid widget definition
impl StatefulWidget for TaskGrid {
  type State = TaskGridState;
  fn render(self, area: Rect, buf: &mut Buffer, state: &mut TaskGridState) {
    let cols: Vec<[Rect; GRID_HEIGHT]> =
      Layout::horizontal([Constraint::Fill(1), Constraint::Fill(1), Constraint::Fill(1)])
        .areas::<GRID_HEIGHT>(area)
        .into_iter()
        .map(|r: Rect| {
          Layout::vertical([Constraint::Fill(1), Constraint::Fill(1), Constraint::Fill(1)])
            .horizontal_margin(1)
            .areas(r)
        })
        .collect();

    for j in 0..GRID_HEIGHT {
      for i in 0..GRID_WIDTH {
        let col = cols.get(i).unwrap();
        let cell_full = col.get(j).unwrap();
        let cell = cell_full.inner(Margin::new(0, j as u16 % 2));

        let index = state.page * GRID_WIDTH * GRID_HEIGHT + j * GRID_WIDTH + i;

        if let Some(task) = state.get_all_items().get(index).as_deref() {
          let mod_task_opt = state.modifications.get(&task.id);
          let is_selected = Some(index) == state.selected;
          let is_modified = mod_task_opt.is_some();
          let is_remove = mod_task_opt.is_some_and(|set| set.contains(&Action::Remove));

          let style = match (is_selected, is_modified, is_remove) {
            (true, _ , _)    => Style::default().bg(Palette::GREEN.into()),
            (_, _, true)     => Style::default().bg(Palette::RED.into()),
            (_, true, _)     => Style::default().bg(Palette::YELLOW.into()),
                           _ => Style::default().bg(Palette::BG2.into())
          };

          let (rendered_task, completed) = mod_task_opt.map_or((*task, false), |set| {
            let mut t = *task;
            let mut completed = false;
            for m in set.iter() {
              match m {
                Action::Edit(m_task) => t = m_task,
                Action::ToggleComplete => completed = true,
                _ => {}
              }
            }
            (t, completed)
          });

          let block = Block::default()
              .padding(Padding::proportional(1))
              .style(style);

          let inner = block.inner(cell);
          let max_y = inner.y + inner.height;
          block.render(cell, buf);

          for (i, line_str) in rendered_task.to_string().split("\n").enumerate() {
            let mut line_style = style;
            let y = inner.y + i as u16;
            if let Some(subtask_i) = state.get_line_offset_selected_sub() {
              if is_selected && i == subtask_i {
                line_style = Style::default().bg(Palette::GREEN2.into());
              }
            }
            if y < max_y {
              Paragraph::new(line_str)
                .style(line_style)
                .render(Rect { x: inner.x, y, width: inner.width - 2, height: 1 }, buf);
            }
            if i == 0 && completed {
              buf.set_string(inner.x + rendered_task.text.len() as u16 + 1, y, "✅", line_style);
            }
          }
        }
      }
    }
  }
}

