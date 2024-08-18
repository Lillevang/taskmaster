pub mod input;

pub use input::handle_key;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::state::Mode;
    use crate::app::App;
    use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    #[test]
    fn test_handle_key_task_list_mode() {
        let mut app = App::load_test_data();
        app.current_mode = Mode::TaskList;

        // Ensure a task is selected
        app.todo_list.state.select(Some(0));

        // Test entering editing mode
        handle_key(
            &mut app,
            KeyEvent::new(KeyCode::Char('e'), KeyModifiers::NONE),
        );
        assert_eq!(app.current_mode, Mode::Editing);

        // Test quitting the app
        app.current_mode = Mode::TaskList;
        handle_key(
            &mut app,
            KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE),
        );
        assert!(app.should_exit);

        // Test selecting next task
        handle_key(&mut app, KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
        assert_eq!(
            app.todo_list.state.selected(),
            Some(1),
            "Expected selection to move to the second task"
        );

        // Test selecting previous task
        handle_key(&mut app, KeyEvent::new(KeyCode::Up, KeyModifiers::NONE));
        assert_eq!(
            app.todo_list.state.selected(),
            Some(0),
            "Expected selection to move back to the first task"
        );

        // Test boundary condition: Press Up at the first task
        handle_key(&mut app, KeyEvent::new(KeyCode::Up, KeyModifiers::NONE));
        assert_eq!(
            app.todo_list.state.selected(),
            Some(0),
            "Expected selection to remain at the first task"
        );

        // Move to the last task
        handle_key(&mut app, KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
        handle_key(&mut app, KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
        assert_eq!(
            app.todo_list.state.selected(),
            Some(2),
            "Expected selection to move to the last task"
        );

        // Test boundary condition: Press Down at the last task
        handle_key(&mut app, KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
        assert_eq!(
            app.todo_list.state.selected(),
            Some(3),
            "Expected selection to remain at the last task"
        );
    }
}
