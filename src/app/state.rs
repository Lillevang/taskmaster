use crate::models::{Status, TodoItem};
use std::io;
use ratatui::{
    backend::Backend,
    crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    widgets::ListState,
    Terminal
};

pub struct App {
    pub should_exit: bool,
    pub todo_list: TodoList,
}

pub struct TodoList {
    pub items: Vec<TodoItem>,
    pub state: ListState,
}

impl Default for App {
    fn default() -> Self {
        Self {
            should_exit: false,
            todo_list: TodoList::from_iter([
                (Status::Todo, "Rewrite everything with Rust!", "I can't hold my inner voice. He tells me to rewrite the complete universe with Rust"),
                (Status::Completed, "Rewrite all of your tui apps with Ratatui", "Yes, you heard that right. Go and replace your tui with Ratatui."),
                (Status::Todo, "Pet your cat", "Minnak loves to be pet by you! Don't forget to pet and give some treats!"),
                (Status::Todo, "Walk with your dog", "Max is bored, go walk with him!"),
                (Status::Completed, "Refactor list example", "If you see this info that means I completed this task!"),
                (Status::Completed, "Pay the bills", "Pay the train subscription!!!"),
            ]),
        }
    }
}

impl FromIterator<(Status, &'static str, &'static str)> for TodoList {
    fn from_iter<I: IntoIterator<Item = (Status, &'static str, &'static str)>>(iter: I) -> Self {
        let items = iter
            .into_iter()
            .map(|(status, todo, info)| TodoItem::new(status, todo, info))
            .collect();
        let state = ListState::default();
        Self { items, state }
    }
}

/// This struct holds the current state of the app. In particular, it has the `todo_list` field
/// which is a wrapper around `ListState`. Keeping track of the state lets us render the
/// associated widget with its state and have access to features such as natural scrolling.
///
/// Check the event handling at the bottom to see how to change the state on incoming events. Check
/// the drawing logic for items on how to specify the highlighting style for selected items.
impl App {

    pub fn run(&mut self, mut terminal: Terminal<impl Backend>) -> io::Result<()> {
        while !self.should_exit {
            terminal.draw(|f| f.render_widget(&mut *self, f.area()))?;
            if let Event::Key(key) = event::read()? {
                self.handle_key(key);
            };
        }
        Ok(())
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        if key.kind != KeyEventKind::Press {
            return;
        }
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => self.should_exit = true,
            KeyCode::Char('h') | KeyCode::Left => self.select_none(),
            KeyCode::Char('j') | KeyCode::Down => self.select_next(),
            KeyCode::Char('k') | KeyCode::Up => self.select_previous(),
            KeyCode::Char('g') | KeyCode::Home => self.select_first(),
            KeyCode::Char('G') | KeyCode::End => self.select_last(),
            KeyCode::Char('l') | KeyCode::Right | KeyCode::Enter => {
                self.toggle_status();
            }
            _ => {}
        }
    }

    pub fn select_none(&mut self) {
        self.todo_list.state.select(None);
    }

    pub fn select_next(&mut self) {
        self.todo_list.state.select_next();
    }
    pub fn select_previous(&mut self) {
        self.todo_list.state.select_previous();
    }

    pub fn select_first(&mut self) {
        self.todo_list.state.select_first();
    }

    pub fn select_last(&mut self) {
        self.todo_list.state.select_last();
    }

    /// Changes the status of the selected list item
    pub fn toggle_status(&mut self) {
        if let Some(i) = self.todo_list.state.selected() {
            self.todo_list.items[i].status = match self.todo_list.items[i].status {
                Status::Completed => Status::Todo,
                Status::Todo => Status::Completed,
            }
        }
    }
}