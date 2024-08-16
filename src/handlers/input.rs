use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use crate::app::App;

pub fn handle_key(app: &mut App, key: KeyEvent) {
    if key.kind != KeyEventKind::Press {
        return;
    }
    match key.code {
        KeyCode::Char('q') | KeyCode::Esc => app.should_exit = true,
        KeyCode::Char('h') | KeyCode::Left => app.select_none(),
        KeyCode::Char('j') | KeyCode::Down => app.select_next(),
        KeyCode::Char('k') | KeyCode::Up => app.select_previous(),
        KeyCode::Char('g') | KeyCode::Home => app.select_first(),
        KeyCode::Char('G') | KeyCode::End => app.select_last(),
        KeyCode::Char('l') | KeyCode::Right | KeyCode::Enter => {
            app.toggle_status();
        }
        _ => {}
    }
}