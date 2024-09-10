use std::io::stdout;

use ratatui::prelude::{CrosstermBackend, Terminal};

use crate::error::AppError;

use app::Habitui;
use event::{Event, EventHandler};
use handler::handle_key_events;
use tui::Tui;

pub mod app;
pub mod event;
pub mod handler;
pub mod tui;
pub mod ui;
pub mod util;
pub mod widgets;

// Start interactive Habitui
pub async fn run() -> Result<(), AppError> {
    // Create an application.
    let mut app = Habitui::default();

    // Initialize the terminal user interface.
    let events = EventHandler::new(250);
    let mut tui = Tui::new(Terminal::new(CrosstermBackend::new(stdout()))?, events);
    tui.init()?;

    // Start the main loop.
    while app.is_running() {
        // Render the user interface.
        tui.draw(&mut app)?;

        // Handle events.
        match tui.events.next().await? {
            Event::Tick => app.tick(),
            Event::Key(key_event) => handle_key_events(key_event, &mut app)?,
            Event::Mouse(_) => {}
            Event::Resize(_, _) => {}
        }
    }

    // Exit the user interface.
    tui.exit()?;
    Ok(())
}
