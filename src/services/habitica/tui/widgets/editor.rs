use std::cmp::max;use std::marker::PhantomData;
use crossterm::event::KeyEvent;
use ratatui::{
  widgets::{Clear, StatefulWidget, Widget, Block, Borders, Padding}, 
  widgets::{calendar::{Monthly, CalendarEventStore}, Paragraph, BorderType},
  layout::{Rect, Layout, Constraint}, 
  style::{Style, Color},
  buffer::Buffer
};
use time::{OffsetDateTime, Duration};
use tui_textarea::{TextArea, CursorMove};

use crate::services::habitica::{
  types::{Task, SubTask, Difficulty}, 
  tui::util::{Palette, MOD_KEY_TTL}
};

#[derive(PartialEq)]
pub enum EditorMode {
  Normal,
  Insert,
  Calendar
}

pub struct Editor<'e> {
  phantom: PhantomData<&'e u8>
}

impl<'e> Editor<'e> {
  pub fn new() -> Self {
    Self { phantom: PhantomData::default() }
  }
}

pub struct EditorState<'e> {
  pub task: Task,
  pub mode: EditorMode,
  pub focus: Option<usize>,
  pub date_focus: Option<OffsetDateTime>,
  pub fields: Vec<TextArea<'e>>,
  pub dirty_fields: Vec<usize>,
  pub mod_key: Option<(KeyEvent, u32)>,
  pub is_modified: bool,
}

fn set_default_styles<'e>(field: &mut TextArea<'e>, is_modified: bool) {
  let style = if is_modified { Style::default()
    .bg(Palette::YELLOW.into())
    .underline_color(Palette::YELLOW.into()) 
  } else {
    Style::default()
      .bg(Palette::BG.into())
      .underline_color(Palette::BG.into()) 
  };

  field.set_style(style);
  field.set_cursor_style(style);
  field.set_placeholder_style(style.fg(if is_modified { 
    Palette::YELLOW2.into() 
  } else { 
    Palette::BG2.into() 
  }));
}

fn build_input_field<'e>(lines: Vec<String>, is_sub: bool) -> TextArea<'e> {
  let mut field = TextArea::new(lines);
  if is_sub {
    field.set_block(Block::default().padding(Padding::horizontal(2)));
  } else {
    field.set_block(Block::default().borders(Borders::BOTTOM).border_style(Style::default().fg(Palette::BG2.into())));
  }
  set_default_styles(&mut field, false);

  field
}

impl<'e> EditorState<'e> {
  pub fn clone_task(&self) -> Task {
    self.task.clone()
  }

  pub fn get_focused_mut(&mut self) -> Option<(&mut TextArea<'e>, usize)> {
    if let Some(focus) = self.focus {
      return self.fields.get_mut(focus).map(|f| (f, focus));
    }
    None
  }

  pub fn move_cursor(&mut self, m: CursorMove) {
    if let Some((textarea, _)) = self.get_focused_mut() {
      textarea.move_cursor(m);
    }
  }

  pub fn move_date_cursor(&mut self, m: CursorMove) {
    let date = self.date_focus.get_or_insert(
      self.task.date.unwrap_or(
        OffsetDateTime::now_utc()
      ));

    match m {
      CursorMove::Up => *date -= Duration::WEEK,
      CursorMove::Down => *date += Duration::WEEK, 
      CursorMove::Forward => *date += Duration::DAY,
      CursorMove::Back => *date -= Duration::DAY,
      _ => {}
    }
    self.date_focus = Some(*date);
  }

  pub fn next_task_difficulty(&mut self) {
    self.task.difficulty = self.task.difficulty.next();
    self.is_modified = true;
  }

  pub fn prev_task_difficulty(&mut self) {
    self.task.difficulty = self.task.difficulty.prev();
    self.is_modified = true;
  }

  pub fn remove_due_date(&mut self) {
    if let Some(_) = self.task.date {
      self.is_modified = true;
    }
    self.task.date = None;
  }

  pub fn enter_insert_mode(&mut self) {
    self.focus.get_or_insert(0);
    self.mode = EditorMode::Insert;
  }

  pub fn exit_insert_mode(&mut self) {
    self.sync_changes();
    self.mode = EditorMode::Normal;
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
        if let Some(focus) = self.focus {
          if let Some(textarea) = self.fields.get_mut(focus) {
            textarea.input(mk.0);
          }
        }
        self.mod_key = None;
      } else {
        self.mod_key = Some(mk);
      }
    }
  }

  pub fn mark_modified(&mut self, i: usize) {
    self.dirty_fields.push(i);
    self.is_modified = true;
  }
  
  pub fn sync_changes(&mut self) {
    self.dirty_fields.sort();
    self.dirty_fields.reverse();

    for i in &self.dirty_fields {
      match i {
        0 => self.task.text = self.fields[0].lines().join("\n"),
        1 => {
          let content = self.fields[1].lines().join("\n");
          if content.len() > 0 {
            self.task.notes = Some(content);
          } else {
            self.task.notes = None;
          }
        }
        n if *n < self.fields.len() => {
          let list = self.task.checklist.clone().or_else(|| Some(Vec::new())).map(|mut v| {
            let content = self.fields[*n].lines().join("\n");
            if content.len() > 0 {
              v[(*n)-2].text = content;
            } else {
              v.remove((*n)-2);
              self.fields.remove(*n);
            }
            v
          }); 
          self.task.checklist = list;
        },
        _ => {}
      }
    }
  }

  pub fn insert_subtask(&mut self) {
    let field = build_input_field(Vec::new(), true);
    let index: usize;
    if let Some(i) = self.focus {
      index = max(2, i+1);
    } else {
      index = 2;
    }
    self.focus = Some(index);
    self.fields.insert(index, field);
    self.dirty_fields.push(index);
    let list = self.task.checklist.clone().or_else(|| Some(Vec::new())).map(|mut v| {
      v.insert(index-2, SubTask { text: "".into(), completed: false });
      v
    });

    self.mode = EditorMode::Insert;
    self.task.checklist = list;
  }

  pub fn new(task_option: Option<&Task>) -> Self {
    let (name, notes, mut subtasks, task, mode) = match task_option {
      Some(task) => {
        let notes_content = if let Some(n) = &task.notes { 
          n.split("\n").map(|n| n.to_string()).collect() 
        } else { 
          Vec::new() 
        };
        let subtasks = if let Some(st) = &task.checklist {
          st.iter().map(|sub| {
            build_input_field(vec![sub.text.clone()], true)
          }).collect()
        } else {
          Vec::new()
        };

        let name = build_input_field(vec![task.text.clone()], false);
        let notes = build_input_field(notes_content, false);

        (name, notes, subtasks, task.clone(), EditorMode::Normal)
      }, 
      None => {
        let mut name = build_input_field(Vec::new(), false);
        name.set_placeholder_text("Task name");

        let mut notes = build_input_field(Vec::new(), false);
        notes.set_placeholder_text("--- Notes ---");

        (name, notes, Vec::new(), Task::default(), EditorMode::Insert)
      }
    };

    let mut fields = vec![name, notes];
    fields.append(&mut subtasks);


    Self {
      task,
      mode,
      focus: Some(0), 
      date_focus: None,
      fields,
      dirty_fields: Vec::new(),
      mod_key: None,
      is_modified: false,
    }
  }
}

impl<'e> StatefulWidget for Editor<'e> {
  type State = EditorState<'e>;
  fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
    let [left_col, right_col] = Layout::horizontal([Constraint::Fill(2), Constraint::Fill(1)])
      .vertical_margin(1)
      .horizontal_margin(3)
      .areas(area);

    let mut constraints: Vec<Constraint> = (2..state.fields.len())
      .map(|_| Constraint::Length(1))
      .collect();

    constraints.insert(0, Constraint::Length(2));
    constraints.insert(0, Constraint::Length(2));
    constraints.push(Constraint::Fill(1));

    let chunks = Layout::vertical(constraints)
      .split(left_col);

    Clear::default().render(area, buf);

    // Popup card
    let card_bg: Color = if state.is_modified {
      Palette::YELLOW.into() 
    } else {
      Palette::BG.into()
    };
    let border_bg: Color = if state.is_modified {
      Palette::YELLOW2.into() 
    } else {
      Palette::BG2.into()
    };
    Block::default().style(Style::default().bg(card_bg)).render(area, buf);

    let cursor_style = Style::default().bg(Palette::CURSOR.into());

    for (i, textarea) in state.fields.iter_mut().enumerate() {
      if let Some(s) = state.focus {
        set_default_styles(textarea, state.is_modified);
        if i == s && state.mode != EditorMode::Calendar {
          textarea.set_cursor_style(cursor_style);
        };
      }
      if let Some(block) = textarea.block().cloned() {
        textarea.set_block(block.border_style(border_bg));
      };
      textarea.render(chunks[i], buf);
    }

    let checklist_area = Rect {
      x: chunks[0].x,
      y: chunks[1].y + 1,
      width: chunks[0].width,
      height: chunks[chunks.len()-1].y + chunks[chunks.len()-1].height - chunks[1].y - 1,
    };
    Block::bordered().border_style(border_bg).render(checklist_area, buf);

    let [cal_area, diff_area] = Layout::vertical([Constraint::Length(7), Constraint::Fill(1)]).areas(right_col);


    let mut event_store = CalendarEventStore::default();

    state.task.date.map(|d| event_store.add(d.date(), Style::default().bg(border_bg)));

    let date = if let Some(focus) = state.date_focus {
      focus.date()
    } else {
      if let Some(due) = state.task.date {
        due.date()
      } else {
        OffsetDateTime::now_utc().date()
      }
    };

    if state.mode == EditorMode::Calendar {
      event_store.add(date, cursor_style);
    }

    Monthly::new(date, event_store)
      .block(Block::default().borders(Borders::BOTTOM).border_style(Style::default().fg(border_bg)))
      .show_surrounding(Style::default())
      .show_month_header(Style::default())
      .render(cal_area, buf);

    let diff_chunks: Vec<Rect> = Layout::vertical([Constraint::Length(3), Constraint::Length(3)])
      .horizontal_margin(2)
      .areas::<2>(diff_area)
      .into_iter()
      .flat_map(|r: Rect| {
        Layout::horizontal([Constraint::Fill(1), Constraint::Fill(1)])
          .spacing(1)
          .areas::<2>(r)
      })
      .collect();

    for (i, a) in diff_chunks.iter().enumerate() {
      let diff = match i {
        0 => Difficulty::TRIVIAL,
        1 => Difficulty::EASY,
        2 => Difficulty::MEDIUM,
        3 => Difficulty::HARD,
        _ => Difficulty::EASY
      }; 
      let style = if diff == state.task.difficulty {
        Style::default().fg(Palette::FG.into())
      } else {
        Style::default().fg(border_bg)
      };

      Paragraph::new(diff.to_string())
        .block(Block::bordered()
          .style(style)
          .border_type(BorderType::Rounded)
          .border_style(style)
          // .padding()
      ).render(*a, buf);
    }
  }
}
