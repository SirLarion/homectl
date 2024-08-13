use crate::error::AppError;

use super::{app::{Habitui, AppState}, util::Direction};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Handles key events and updates the state of Habitui.
pub fn handle_key_events(key_event: KeyEvent, app: &mut Habitui) -> Result<(), AppError> {
  match key_event.code {
    // Exit application on `ESC` or `q`
    KeyCode::Esc | KeyCode::Char('q') => {
      if app.state == AppState::CreateTask || app.state == AppState::EditTask {
        app.state = AppState::List;
      } else {
        app.state = AppState::Exit;
      }
    }
    // Exit application on `Ctrl-C`
    KeyCode::Char('c') | KeyCode::Char('C') => {
      if key_event.modifiers == KeyModifiers::CONTROL {
        app.state = AppState::Exit;
      }
    }
    KeyCode::Char('a') => {
      app.state = AppState::CreateTask;
    }
    KeyCode::Char('e') => {
      if let Some(_) = app.grid_state.get_selected() {
        app.state = AppState::EditTask;
      }
    }
    // Change selection with vim motions
    KeyCode::Char('h') => {
      app.grid_state.select_next(Direction::LEFT); 
    }
    KeyCode::Char('j') => {
      app.grid_state.select_next(Direction::DOWN); 
    }
    KeyCode::Char('k') => {
      app.grid_state.select_next(Direction::UP); 
    }
    KeyCode::Char('l') => {
      app.grid_state.select_next(Direction::RIGHT); 
    }

    // Change page
    KeyCode::Char('J') => {
      app.grid_state.next_page();
    }
    KeyCode::Char('K') => {
      app.grid_state.prev_page()
    }
    _ => {}
  }
  Ok(())
}
