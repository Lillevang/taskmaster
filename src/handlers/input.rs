use crate::app::state::{Mode, ConfirmationState};
use crate::app::App;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub fn handle_key(app: &mut App, key: KeyEvent) {
    match app.current_mode {
        Mode::TaskList => handle_task_list_input(app, key),
        Mode::Editing => handle_editing_input(app, key),
        Mode::Creating => handle_creation_input(app, key),
        Mode::Command => handle_command_input(app, key),
        Mode::Help => handle_help_input(app, key),
    }
}

fn handle_task_list_input(app: &mut App, key: KeyEvent) {
    match app.confirmation_state {
        ConfirmationState::Delete => {
            match key.code {
                KeyCode::Char('y') => app.confirm_delete(),
                KeyCode::Char('n') | KeyCode::Esc => app.cancel_delete(),
                _ => {}
            }
            return;
        }
        _ => {}
    }

    match key.code {
        KeyCode::Char(':') => app.enter_command_mode(),
        KeyCode::Char('n') => app.create_new_task(),
        KeyCode::Char('e') => app.enter_editing_mode(),
        KeyCode::Char('q') | KeyCode::Esc => app.should_exit = true,
        KeyCode::Up => app.select_previous(),
        KeyCode::Down => app.select_next(),
        KeyCode::Char('g') => app.select_first(),
        KeyCode::Char('G') => app.select_last(),
        KeyCode::Char(' ') => {
            if key.modifiers.contains(KeyModifiers::SHIFT) {
                app.toggle_status();
            } else {
                app.toggle_selection();
            }
        },
        KeyCode::Char('E') => app.clear_selection(),
        KeyCode::Char('l') | KeyCode::Enter => app.toggle_status(),
        KeyCode::Delete if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.request_delete_confirmation()
        },
        KeyCode::Char('a') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.create_new_task();
        },
        KeyCode::Char('h') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.current_mode = Mode::Help;
        },
        KeyCode::Char('t') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.todo_list.toggle_archive_view();
        },
        KeyCode::Char('>') => {
            if let Some(selected) = app.todo_list.state.selected() {
                app.todo_list.archive_task(selected);
            }
        },
        KeyCode::Char('<') => {
            if let Some(selected) = app.todo_list.state.selected() {
                app.todo_list.unarchive_task(selected);
            }
        },
        KeyCode::Char('A') if key.modifiers.contains(KeyModifiers::SHIFT) => {
            app.todo_list.archive_all_completed();
        },
        _ => {}
    }
}

fn handle_help_input(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char('q') | KeyCode::Esc => app.current_mode = Mode::TaskList,
        _ => {}
    }
}

fn handle_command_input(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Enter => {
            let cmd = app.command_buffer.clone();
            if let Err(e) = app.handle_command(&cmd) {
                // Show error in the command line
                app.command_buffer = format!("Error: {}", e);
            } else {
                app.exit_command_mode();
            }
        }
        KeyCode::Esc => app.exit_command_mode(),
        KeyCode::Char(c) => {
            // Handle special command shortcuts
            match c {
                'q' => {
                    app.should_exit = true;
                    return;
                }
                'h' => {
                    app.current_mode = Mode::Help;
                    return;
                }
                't' => {
                    app.todo_list.toggle_archive_view();
                    app.exit_command_mode();
                    return;
                }
                'a' => {
                    app.create_new_task();
                    return;
                }
                'e' => {
                    app.enter_editing_mode();
                    return;
                }
                '>' => {
                    if let Some(selected) = app.todo_list.state.selected() {
                        app.todo_list.archive_task(selected);
                        app.exit_command_mode();
                    }
                    return;
                }
                '<' => {
                    if let Some(selected) = app.todo_list.state.selected() {
                        app.todo_list.unarchive_task(selected);
                        app.exit_command_mode();
                    }
                    return;
                }
                'A' => {
                    app.todo_list.archive_all_completed();
                    app.exit_command_mode();
                    return;
                }
                _ => app.command_buffer.push(c),
            }
        }
        KeyCode::Backspace => {
            app.command_buffer.pop();
        }
        _ => {}
    }
}

fn handle_editing_input(app: &mut App, key: KeyEvent) {
    if let Some(_editing_task) = &mut app.editing_task {
        match key.code {
            KeyCode::Esc => app.cancel_editing(),
            KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => app.save_task(),
            KeyCode::Char(c) => {
                // Handle text input for the currently selected field
                app.editing_field_input(c);
            }
            KeyCode::Backspace => {
                // Handle backspace to remove characters
                app.backspace_field_input();
            }
            KeyCode::Tab => {
                // Switch between fields on Tab
                app.switch_editing_field();
            }
            _ => {}
        }
    }
}

fn handle_creation_input(app: &mut App, key: KeyEvent) {
    // Ensure we are in the creation mode and have an active editing task
    if app.current_mode == Mode::Creating {
        match key.code {
            KeyCode::Esc => app.cancel_editing(),
            KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                app.save_new_task(); // Save the new task
            }
            KeyCode::Char(c) => {
                // Handle text input for the currently selected field
                app.editing_field_input(c);
            }
            KeyCode::Backspace => {
                // Handle backspace to remove characters
                app.backspace_field_input();
            }
            KeyCode::Tab => {
                // Switch between fields on Tab
                app.switch_editing_field();
            }
            _ => {}
        }
    }
}
