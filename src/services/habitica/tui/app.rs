use crossterm::event::{Event, KeyEventKind, KeyCode, KeyEvent, self};
// use ratatui::text::Text;
use ratatui::Frame;
use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Rect};
use ratatui::symbols::border;
// use ratatui::text::Line;
use ratatui::widgets::block::{Position, Title};
use ratatui::widgets::{Paragraph, Borders, Block, Widget};

use crate::services::habitica::tui;
use crate::error::AppError;

#[derive(PartialEq)]
pub enum TuiState {
  List,
  Create,
  Exit
  // Edit? More?
}

pub struct Habitui {
  state: TuiState,
}

impl Default for Habitui {
  fn default() -> Self { 
    Self { state: TuiState::List } 
  }
}

impl Habitui {
  pub fn run(&mut self, terminal: &mut tui::Tui) -> Result<(), AppError> {
    while self.state != TuiState::Exit {
      terminal.draw(|frame| self.render_frame(frame))?;
      self.handle_events()?;
    }
    Ok(())
  }

  fn exit(&mut self) {
    self.state = TuiState::Exit
  }

  fn render_frame(&self, frame: &mut Frame) {
    frame.render_widget(self, frame.size());
  }

  fn handle_events(&mut self) -> Result<(), AppError> {
    match event::read()? {
      // it's important to check that the event is a key press event as
      // crossterm also emits key release and repeat events on Windows.
      Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
        self.handle_key_event(key_event);
      },
      _ => {}
    };
    Ok(())
  }

  fn handle_key_event(&mut self, key_event: KeyEvent) {
    match key_event.code {
      KeyCode::Char('q') => self.exit(),
      _ => {}
    }
  }
}

impl Widget for &Habitui {
  fn render(self, area: Rect, buf: &mut Buffer) {
    let title = Title::from(" HabiTUI ");
    let block = Block::default()
      .title(title.alignment(Alignment::Center))
      .borders(Borders::ALL)
      .border_set(border::THICK);

    Paragraph::new("shbubbi")
      .centered()
      .block(block)
      .render(area, buf);
  }
}
