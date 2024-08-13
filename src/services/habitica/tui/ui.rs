use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::Style,
    widgets::{Clear, Block, BorderType, Paragraph, Padding},
    Frame,
};

use crate::services::habitica::types::Task;

use super::{
  app::{Habitui, AppState}, 
  widgets::grid::TaskGrid, 
  util::Palette
};

const TITLE_STR: &str = "╻ ╻┏━┓┏┓ ╻╺┳╸╻ ╻╻\n┣━┫┣━┫┣┻┓┃ ┃ ┃ ┃┃\n╹ ╹╹ ╹┗━┛╹ ╹ ┗━┛╹";

fn render_task_grid(f: &mut Frame, area: Rect, app: &mut Habitui) {
  let state = &mut app.grid_state;
  let widget = TaskGrid {};
  f.render_stateful_widget(widget, area, state);
}

fn render_footer(f: &mut Frame, area: Rect) {
  let style = Style::default().fg(Palette::BG2.into());
  let block = Block::bordered()
    .border_style(style)
    .border_type(BorderType::Rounded)
    .padding(Padding::horizontal(2));

  f.render_widget(Paragraph::new("q: quit | hjkl: navigate | b: bluupi")
    .block(block), area);
}

fn render_editor(f: &mut Frame, area: Rect, task: Option<&Task>) {
  let popup_area = Rect {
      x: area.width / 4,
      y: area.height / 3,
      width: area.width / 2,
      height: area.height / 3,
  };
  f.render_widget(Clear::default(), popup_area);
  f.render_widget(Paragraph::new("Edit time loll").block(Block::bordered()), popup_area);
}

pub fn render(frame: &mut Frame, app: &mut Habitui) {
  let [title_area, main_area, _, footer_area] = Layout::vertical([
    Constraint::Length(5), 
    Constraint::Fill(1), 
    Constraint::Length(1),
    Constraint::Length(3)
  ])
    .areas(frame.size());


  frame.render_widget(Paragraph::new(TITLE_STR)
    .centered()
    .block(Block::default()), title_area);

  render_task_grid(frame, main_area, app);

  if app.state == AppState::CreateTask { 
    render_editor(frame, main_area, None);
  } else if app.state == AppState::EditTask {
    render_editor(frame, main_area, app.grid_state.get_selected());
  }

  render_footer(frame, footer_area);
}
