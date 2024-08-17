use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use crate::app::App;
use crate::app::state::Mode;

pub fn handle_key(app: &mut App, key: KeyEvent) {
    match app.current_mode {
        Mode::TaskList => handle_task_list_input(app, key),
        Mode::Editing => handle_editing_input(app, key),
    }
}

fn handle_task_list_input(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char('e') => app.enter_editing_mode(),
        KeyCode::Char('q') | KeyCode::Esc => app.should_exit = true,
        KeyCode::Up => app.select_previous(),
        KeyCode::Down => app.select_next(),
        KeyCode::Char('g') => app.select_first(),
        KeyCode::Char('G') => app.select_last(),
        KeyCode::Char(' ') | KeyCode::Char('l') | KeyCode::Enter => app.toggle_status(),
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