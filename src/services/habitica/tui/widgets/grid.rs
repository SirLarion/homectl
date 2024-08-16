use std::cmp::max;

use ratatui::{
  widgets::{Widget, StatefulWidget, Paragraph, Block, Borders, Padding}, 
  layout::{Rect, Layout, Constraint, Margin}, 
  buffer::Buffer,
  style::{Style, Color}
};

use crate::services::habitica::{
  types::Task,
  tui::util::{Direction, Palette}
};

const GRID_WIDTH: usize = 3;
const GRID_HEIGHT: usize = 3;
const GRID_SIZE: u8 = (GRID_WIDTH * GRID_HEIGHT) as u8;

pub struct TaskGridState {
  pub page: u8,
  pub selected: Option<u8>,
  pub task_items: Vec<Task>,
  pub loading: bool
}

impl Default for TaskGridState {
  fn default() -> Self {
    Self { page: 0, selected: None, task_items: Vec::new(), loading: false }
  }
}
impl TaskGridState {
  pub fn select_next(&mut self, direction: Direction) {
    if let None = self.selected {
      self.selected = Some(self.page * GRID_SIZE);
      return;
    }

    let mut selection = i32::from(self.selected.unwrap());
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
    if selection < i32::from(self.page) * w * h {
      self.page -= 1;
    }
    // Selection is on next page
    else if selection >= (i32::from(self.page) + 1) * w * h {
      self.page += 1;
    }

    match u8::try_from(selection) {
      Ok(s) => self.selected = Some(s),
      Err(_) => self.selected = Some(0)
    }
  } 

  pub fn next_page(&mut self) {
    if self.task_items.len() > ((self.page + 1) * GRID_SIZE).into() {
      self.selected = self.selected.map(|s| s + GRID_SIZE);

      self.page += 1;
    } 
  }

  pub fn prev_page(&mut self) {
    if self.page != 0 {
      self.selected = self.selected.map(|s| s - GRID_SIZE);

      self.page -= 1;
    }
  }

  pub fn get_selected(&self) -> Option<&Task> {
    self.selected.map(|i| self.task_items.get(usize::from(i)).unwrap())
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

        let index = usize::from(state.page) * GRID_WIDTH * GRID_HEIGHT + j * GRID_WIDTH + i;

        if let Some(task) = state.task_items.get(index) {
          let style = if Some(index as u8) == state.selected {
            Style::default().bg(Palette::GREEN.into())
          } else {
            Style::default().bg(Palette::BG2.into())
          };

          Paragraph::new(task.to_string())
            .block(Block::default()
              .padding(Padding::proportional(1))
              .style(style)
            )
            .render(cell, buf);
        }
      }
    }
  }
}

