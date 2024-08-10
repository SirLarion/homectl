use std::io::{stdout, Stdout};

use crossterm::{
  execute, 
  terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen,
    LeaveAlternateScreen,
  },
};
use ratatui::prelude::{CrosstermBackend, Terminal};

use crate::error::AppError;

use self::app::Habitui;

mod app;

pub type Tui = Terminal<CrosstermBackend<Stdout>>;

// Initialize the terminal
fn init() -> Result<Tui, AppError> {
    execute!(stdout(), EnterAlternateScreen)?;
    enable_raw_mode()?;
    Ok(Terminal::new(CrosstermBackend::new(stdout()))?)
}

// Restore the terminal to its original state
fn restore() -> Result<(), AppError> {
    execute!(stdout(), LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}

// Start interactive Habitui
pub fn start() -> Result<(), AppError> {
  let mut terminal = init()?;

  Habitui::default().run(&mut terminal)?;

  restore()?;
  Ok(())
}
