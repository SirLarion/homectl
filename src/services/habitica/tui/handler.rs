use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use tui_textarea::CursorMove;

use crate::error::AppError;

use super::{
    app::{AppState, Habitui},
    util::Direction,
    widgets::editor::{EditorMode, EditorState},
};

/// Handles key events and updates the state of Habitui.
pub fn handle_key_events(key_event: KeyEvent, app: &mut Habitui) -> Result<(), AppError> {
    // Exit application on `Ctrl-C`
    if key_event.code == KeyCode::Char('c') || key_event.code == KeyCode::Char('C') {
        if key_event.modifiers == KeyModifiers::CONTROL {
            app.state = AppState::Exit;

            return Ok(());
        }
    }

    if app.state == AppState::Editor {
        let editor = app.editor_state.as_mut().unwrap();

        match editor.mode {
            EditorMode::Normal => match key_event.code {
                KeyCode::Esc | KeyCode::Char('q') => {
                    app.state = AppState::List;
                }
                KeyCode::Enter => {
                    if editor.is_modified {
                        let task = editor.clone_task();
                        app.handle_submit_task(task);

                        app.state = AppState::List;
                    }
                }
                KeyCode::Char('h') => editor.move_cursor(CursorMove::Back),
                KeyCode::Char('l') => {
                    if key_event.modifiers == KeyModifiers::CONTROL {
                        editor.mode = EditorMode::Calendar;
                    } else {
                        editor.move_cursor(CursorMove::Forward);
                    }
                }
                KeyCode::Char('w') => {
                    let mod_key = editor.pop_mod_key();
                    if let Some((textarea, i)) = editor.get_focused_mut() {
                        textarea.move_cursor(CursorMove::WordForward);
                        match mod_key {
                            Some(KeyEvent {
                                code: _c @ KeyCode::Char('c'),
                                ..
                            }) => {
                                textarea.delete_word();
                                editor.mark_modified(i);
                                editor.enter_insert_mode();
                            }
                            _ => {}
                        }
                    }
                }
                KeyCode::Char('b') => {
                    let mod_key = editor.pop_mod_key();
                    if let Some((textarea, i)) = editor.get_focused_mut() {
                        match mod_key {
                            Some(KeyEvent {
                                code: _c @ KeyCode::Char('c'),
                                ..
                            }) => {
                                textarea.delete_word();
                                editor.mark_modified(i);
                                editor.enter_insert_mode();
                            }
                            _ => textarea.move_cursor(CursorMove::WordBack),
                        }
                    }
                }
                KeyCode::Char('d') => {
                    let mod_key = editor.pop_mod_key();
                    match mod_key {
                        Some(KeyEvent {
                            code: _c @ KeyCode::Char('d'),
                            ..
                        }) => {
                            if let Some((textarea, i)) = editor.get_focused_mut() {
                                textarea.move_cursor(CursorMove::Head);
                                textarea.delete_line_by_end();
                                editor.mark_modified(i);
                                editor.sync_changes();
                            }
                        }
                        _ => {
                            editor.add_mod_key(key_event);
                        }
                    }
                }
                KeyCode::Char('c') => {
                    editor.add_mod_key(key_event);
                }
                KeyCode::Char('y') => {
                    editor.add_mod_key(key_event);
                }
                KeyCode::Char('j') | KeyCode::Tab => {
                    editor.focus = Some(editor.focus.map_or(0, |mut i| {
                        i += 1;
                        i.clamp(0, editor.fields.len() - 1)
                    }));
                }
                KeyCode::Char('k') => {
                    editor.focus = Some(editor.focus.map_or(0, |mut i| {
                        i = i.saturating_sub(1);
                        i.clamp(0, editor.fields.len() - 1)
                    }));
                }
                KeyCode::Char('i') => {
                    editor.enter_insert_mode();
                }
                KeyCode::Char('a') => {
                    editor.move_cursor(CursorMove::Forward);
                    editor.enter_insert_mode();
                }
                KeyCode::Char('o') | KeyCode::Char('O') => {
                    editor.insert_subtask();
                }
                KeyCode::Char('+') => {
                    editor.next_task_difficulty();
                }
                KeyCode::Char('-') => {
                    editor.prev_task_difficulty();
                }
                _ => {}
            },
            EditorMode::Insert => match key_event.code {
                KeyCode::Esc => editor.exit_insert_mode(),
                KeyCode::Enter => match editor.focus {
                    Some(i) if i < editor.fields.len() - 1 => {
                        editor.focus = Some(i + 1);
                    }
                    Some(_) => {
                        editor.insert_subtask();
                    }
                    None => {}
                },
                KeyCode::Tab => {
                    let mut i = *editor.focus.get_or_insert(0);
                    if key_event.modifiers == KeyModifiers::SHIFT {
                        i = i.saturating_sub(1);
                    } else {
                        i += 1;
                    }
                    editor.focus = Some(i.clamp(0, editor.fields.len() - 1));
                }
                c => {
                    if c == KeyCode::Char('j') {
                        match editor.mod_key {
                            Some((
                                KeyEvent {
                                    code: _c @ KeyCode::Char('j'),
                                    ..
                                },
                                _,
                            )) => editor.exit_insert_mode(),
                            _ => editor.add_mod_key(key_event),
                        }
                        return Ok(());
                    };
                    let mod_key = editor.pop_mod_key();

                    if let Some(index) = editor.focus {
                        if let Some(textarea) = editor.fields.get_mut(index) {
                            if let Some(pending) = mod_key {
                                textarea.input(pending);
                            }
                            textarea.input(key_event);
                            editor.dirty_fields.push(index);
                            editor.is_modified = true;
                        }
                    }
                }
            },
            EditorMode::Calendar => match key_event.code {
                KeyCode::Esc | KeyCode::Char('q') => {
                    app.state = AppState::List;
                }
                KeyCode::Char('h') => {
                    if key_event.modifiers == KeyModifiers::CONTROL {
                        editor.mode = EditorMode::Normal;
                    } else {
                        editor.move_date_cursor(CursorMove::Back);
                    }
                }
                KeyCode::Char('j') => editor.move_date_cursor(CursorMove::Down),
                KeyCode::Char('k') => editor.move_date_cursor(CursorMove::Up),
                KeyCode::Char('l') => editor.move_date_cursor(CursorMove::Forward),
                KeyCode::Char('x') => editor.remove_due_date(),
                KeyCode::Char(' ') => {
                    editor.task.date = editor.date_focus;
                    editor.is_modified = true;
                }
                KeyCode::Enter => {
                    if editor.is_modified {
                        let task = editor.clone_task();
                        app.handle_submit_task(task);

                        app.state = AppState::List;
                    }
                }
                KeyCode::Char('+') => {
                    editor.next_task_difficulty();
                }
                KeyCode::Char('-') => {
                    editor.prev_task_difficulty();
                }
                _ => {}
            },
        }

        return Ok(());
    }

    match key_event.code {
        // Exit application on `ESC` or `q`
        KeyCode::Esc | KeyCode::Char('q') => app.state = AppState::Exit,

        // Mark a task or subtask for completion
        KeyCode::Char(' ') => app.grid_state.mark_item_completed(),

        // Submit completed tasks or subtasks
        KeyCode::Enter => {
            if !app.grid_state.modifications.is_empty() {
                app.handle_submit_modifications();
            }
        }

        // Enter editor to create new task or edit an existing one
        KeyCode::Char('a') => {
            app.state = AppState::Editor;
            app.editor_state = Some(EditorState::new(None));
        }
        KeyCode::Char('e') => {
            let selected = app.grid_state.get_selected();
            if selected.is_some() {
                app.state = AppState::Editor;
                app.editor_state = Some(EditorState::new(selected));
            }
        }
        // Change selection with vim motions
        KeyCode::Char('h') => {
            match key_event.modifiers {
                KeyModifiers::ALT => app.grid_state.move_task(Direction::LEFT),
                _ => app.grid_state.select_next(Direction::LEFT),
            };
        }
        KeyCode::Char('j') => {
            match key_event.modifiers {
                KeyModifiers::ALT => app.grid_state.move_task(Direction::DOWN),
                KeyModifiers::CONTROL => app.grid_state.select_next_sub(),
                _ => app.grid_state.select_next(Direction::DOWN),
            };
        }
        KeyCode::Char('k') => {
            match key_event.modifiers {
                KeyModifiers::ALT => app.grid_state.move_task(Direction::UP),
                KeyModifiers::CONTROL => app.grid_state.select_prev_sub(),
                _ => app.grid_state.select_next(Direction::UP),
            };
        }
        KeyCode::Char('l') => match key_event.modifiers {
            KeyModifiers::ALT => app.grid_state.move_task(Direction::RIGHT),
            _ => app.grid_state.select_next(Direction::RIGHT),
        },

        // Remove task
        KeyCode::Char('d') => app.grid_state.mark_item_removed(),

        // Change page
        KeyCode::Char('J') => app.grid_state.next_page(),
        KeyCode::Char('K') => app.grid_state.prev_page(),

        // Shift-g and gg motions
        KeyCode::Char('g') => {
            if let Some(key) = app.grid_state.pop_mod_key() {
                match key.code {
                    KeyCode::Char('g') => {
                        app.grid_state.select_first();
                    }
                    _ => {}
                }
            } else {
                app.grid_state.add_mod_key(key_event);
            }
        }
        KeyCode::Char('G') => app.grid_state.select_last(),

        _ => {}
    }
    Ok(())
}
