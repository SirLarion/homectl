use std::cmp::max;
use std::marker::PhantomData;

use crossterm::event::KeyEvent;
use ratatui::{
  widgets::{Clear, StatefulWidget, Widget, Block, Borders, Padding}, 
  layout::{Rect, Layout, Constraint}, 
  style::{Style, Color},
  buffer::Buffer
};
use tui_textarea::{TextArea, CursorMove};

use crate::{services::habitica::{types::{Task, SubTask}, tui::util::Palette}};

pub enum EditorMode {
  Normal,
  Insert,
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
  task: Task,
  pub mode: EditorMode,
  pub focus: Option<usize>,
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
  field.set_placeholder_style(style);
  field.set_cursor_style(style);
}

fn build_input_field<'e>(lines: Vec<String>, padding: bool) -> TextArea<'e> {
  let mut field = TextArea::new(lines);
  if padding {
    field.set_block(Block::default().padding(Padding::horizontal(2)));
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

  pub fn enter_insert_mode(&mut self) {
    self.focus.get_or_insert(0);
    self.mode = EditorMode::Insert;
  }

  pub fn exit_insert_mode(&mut self) {
    self.sync_changes();
    self.mode = EditorMode::Normal;
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
    let (name, notes, mut subtasks, task) = match task_option {
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

        let mut name = build_input_field(vec![task.text.clone()], false);
        name.set_block(Block::default().borders(Borders::BOTTOM).border_style(Style::default().fg(Palette::BG2.into())));

        let mut notes = build_input_field(notes_content, false);
        notes.set_block(Block::default().borders(Borders::BOTTOM).border_style(Style::default().fg(Palette::BG2.into())));

        (name, notes, subtasks, task.clone())
      }, 
      None => {
        let mut name = build_input_field(Vec::new(), false);
        name.set_placeholder_text("New task");
        name.set_block(Block::default().title("Name"));

        let mut notes = build_input_field(Vec::new(), false);
        notes.set_placeholder_text("--- Notes ---");
        notes.set_block(Block::default().title("Notes"));

        let task = Task {
          id: "".into(),
          text: "".into(),
          notes: None,
          task_type: "todo".into(),
          priority: 1.0, 
          date: None,
          checklist: None,
        };

        (name, notes, Vec::new(), task)
      }
    };

    let mut fields = vec![name, notes];
    fields.append(&mut subtasks);


    Self {
      task,
      mode: EditorMode::Normal,
      focus: None, 
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
    let mut constraints: Vec<Constraint> = (2..state.fields.len())
      .map(|_| Constraint::Length(1))
      .collect();

    constraints.insert(0, Constraint::Length(2));
    constraints.insert(0, Constraint::Length(2));
    constraints.push(Constraint::Fill(1));

    let chunks = Layout::vertical(constraints)
      .vertical_margin(1)
      .horizontal_margin(3)
      .split(area);

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

    for (i, textarea) in state.fields.iter_mut().enumerate() {
      if let Some(s) = state.focus {
        set_default_styles(textarea, state.is_modified);
        if i == s {
          // let style = Style::default()
          //   .underline_color(Palette::BG2.into());
          // textarea.set_style(style);
          // textarea.set_placeholder_style(style);
          textarea.set_cursor_style(Style::default().bg(Palette::CURSOR.into()));
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
      height: chunks[chunks.len()-1].y - chunks[1].y,
    };
    Block::bordered().border_style(border_bg).render(checklist_area, buf);
  }
}
