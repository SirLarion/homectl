use std::cmp::max;

use crossterm::event::KeyEvent;
use ratatui::{
  widgets::{Widget, StatefulWidget, Paragraph, Block, Borders, Padding}, 
  layout::{Rect, Layout, Constraint, Margin}, 
  buffer::Buffer,
  style::{Style, Color, Styled}, text::Line
};

use crate::services::habitica::{
  types::Task,
  tui::util::{Direction, Palette}
};

const GRID_WIDTH: usize = 3;
const GRID_HEIGHT: usize = 3;
const GRID_SIZE: usize = GRID_WIDTH * GRID_HEIGHT;

pub struct TaskGridState {
  pub page: usize,
  pub selected: Option<usize>,
  pub selected_sub: Option<usize>,
  pub task_items: Vec<Task>,
  pub loading: bool,
  pub modified_items: Option<Vec<Task>>,
  pub mod_key: Option<(KeyEvent, u32)>,
}

impl Default for TaskGridState {
  fn default() -> Self {
    Self { page: 0, selected: None, selected_sub: None, task_items: Vec::new(), modified_items: None, loading: false, mod_key: None }
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

  pub fn select_next_sub(&mut self) {
    if let Some(selected) = self.selected {
      let task = self.task_items.get(usize::from(selected)).unwrap();
      if let Some(checklist) = &task.checklist {
        if !checklist.is_empty() {
          if let Some(selected_sub) = self.selected_sub {
            self.selected_sub = Some((selected_sub + 1) % checklist.len())
          } else {
            self.selected_sub = Some(0);
          }
        }
      }
    }
  }

  pub fn select_prev_sub(&mut self) {
    if let Some(selected) = self.selected {
      let task = self.task_items.get(usize::from(selected)).unwrap();
      if let Some(checklist) = &task.checklist {
        if !checklist.is_empty() {
          if let Some(selected_sub) = self.selected_sub {
            self.selected_sub = Some(selected_sub.saturating_sub(1))
          } else {
            self.selected_sub = Some(0);
          }
        }
      }
    }
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

  pub fn find_modified(&self, task: &Task) -> Option<&Task> {
    if let Some(modified) = &self.modified_items {
      modified.iter().find(|t| t.id == task.id)
    } else {
      None
    }
  }

  pub fn get_selected(&self) -> Option<&Task> {
    self.selected.map(|i| self.task_items.get(i).unwrap())
  }

  pub fn add_mod_key(&mut self, key: KeyEvent) {
    self.mod_key = Some((key, 100));
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

        if let Some(task) = state.task_items.get(index) {
          let mod_task_opt = state.find_modified(task);
          let is_modified = mod_task_opt.is_some();
          let is_selected = Some(index) == state.selected;

          let style = if is_selected {
            Style::default().bg(Palette::GREEN.into())
          } else {
            if is_modified {
              Style::default().bg(Palette::YELLOW.into())
            } else {
              Style::default().bg(Palette::BG2.into())
            }
          };

          let rendered_task = if let Some(mod_task) = mod_task_opt {
            mod_task
          }
          else {
            task
          };

          let block = Block::default()
              .padding(Padding::proportional(1))
              .style(style);

          let inner = block.inner(cell);
          let max_y = inner.y + inner.height;
          block.render(cell, buf);

          for (i, line_str) in rendered_task.to_string().split("\n").enumerate() {
            let mut line = Line::from(line_str).style(style);
            let y = inner.y + i as u16;
            if let Some(sub) = state.selected_sub {
              if is_selected && i.wrapping_sub(2) == sub {
                line = line.style(Style::default().bg(Palette::GREEN2.into()));
              }
            }
            if y < max_y {
              buf.set_line(inner.x, y, &line, inner.width);
            }
          }
        }
      }
    }
  }
}

