use crate::models::{Status, TodoItem};
use crate::storage::{get_default_storage_path, load_from_file, save_to_file, TodoData};

use chrono::NaiveDate;
use ratatui::{
    backend::Backend,
    crossterm::event::{self, Event, KeyEvent},
    widgets::ListState,
    Terminal,
};
use std::io;

#[derive(Debug, PartialEq, Eq)]
pub enum EditingField {
    TaskName,
    Description,
    DueDate,
    Tags,
}

pub struct App {
    pub cursor_visible: bool,
    pub should_exit: bool,
    pub todo_list: TodoList,
    pub current_mode: Mode,
    pub editing_task: Option<TodoItem>,
    pub current_editing_field: EditingField,
}

pub struct TodoList {
    pub items: Vec<TodoItem>,
    pub state: ListState,
}
#[derive(PartialEq, Debug)]
pub enum Mode {
    TaskList,
    Editing,
}

impl Default for App {
    fn default() -> Self {
        Self {
            cursor_visible: true,
            should_exit: false,
            todo_list: TodoList::from_iter([
                (Status::Completed, "Order Sodastreamer", "Find cheapest on pricerunner"),
                (Status::Completed, "Order Kitchen Aid", "Do some research first"),
                (Status::Completed, "Write to Helene regarding financing of Electrical Vehicle", "Reference the mail from Brian"),
                (Status::Todo, "Analyze discrepencies between invoices and payments received, reach out to GT for right contact", "Go through all payments GT made and the corresponding invoice to see how big a difference it is."),
                (Status::Todo, "Security approval", "Send security approval docs to Zacharias"),
                (Status::Completed, "Register time ATP", "Make sure it is 1:1 with GT registration"),
                (Status::Todo, "Calculate private financing for company", "Pay out what the company owe me for MasterCard payments"),
                (Status::Todo, "Insurance: answer the email.", "Don't forget!"),
                (Status::Completed, "Bestil Sæbe/Deo", "Proshave")
            ]),
            current_mode: Mode::TaskList,
            editing_task: None,
            current_editing_field: EditingField::TaskName,
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
    pub fn toggle_cursor_visibility(&mut self) {
        self.cursor_visible = !self.cursor_visible;
    }

    pub fn load_or_default() -> Self {
        let storage_path = get_default_storage_path();
        match load_from_file(&storage_path) {
            Ok(todo_data) => Self {
                cursor_visible: true,
                should_exit: false,
                todo_list: TodoList {
                    items: todo_data.items,
                    state: ListState::default(),
                },
                current_mode: Mode::TaskList,
                editing_task: None,
                current_editing_field: EditingField::TaskName,
            },
            Err(_) => Self::default(),
        }
    }

    pub fn save(&self) -> io::Result<()> {
        let storage_path = get_default_storage_path();
        let todo_data = TodoData {
            items: self.todo_list.items.clone(),
        };
        save_to_file(&storage_path, &todo_data)
    }

    pub fn enter_editing_mode(&mut self) {
        if let Some(selected) = self.todo_list.state.selected() {
            self.current_mode = Mode::Editing;
            let mut task = self.todo_list.items[selected].clone();
            // Initialize due_date_temp with the existing due_date if present
            task.due_date_temp = task
                .due_date
                .map(|date| date.format("%Y-%m-%d").to_string());
            self.editing_task = Some(task);
            self.current_editing_field = EditingField::TaskName;
        }
    }

    pub fn switch_editing_field(&mut self) {
        self.current_editing_field = match self.current_editing_field {
            EditingField::TaskName => EditingField::Description,
            EditingField::Description => EditingField::DueDate,
            EditingField::DueDate => EditingField::Tags,
            EditingField::Tags => EditingField::TaskName,
        };
    }

    pub fn editing_field_input(&mut self, c: char) {
        if let Some(editing_task) = &mut self.editing_task {
            match self.current_editing_field {
                EditingField::TaskName => editing_task.todo.push(c),
                EditingField::Description => editing_task.info.push(c),
                EditingField::DueDate => {
                    if let Some(due_date) = &mut editing_task.due_date_temp {
                        if due_date.len() < 10 && (c.is_ascii_digit() || c == '-') {
                            due_date.push(c);
                        }
                    } else {
                        editing_task.due_date_temp = Some(c.to_string());
                    }
                }
                EditingField::Tags => editing_task.tags.push(c.to_string()),
            }
        }
    }

    pub fn backspace_field_input(&mut self) {
        if let Some(editing_task) = &mut self.editing_task {
            match self.current_editing_field {
                EditingField::TaskName => {
                    editing_task.todo.pop();
                }
                EditingField::Description => {
                    editing_task.info.pop();
                }
                EditingField::DueDate => {
                    if let Some(due_date_temp) = &mut editing_task.due_date_temp {
                        due_date_temp.pop();
                    }
                }
                EditingField::Tags => {
                    editing_task.tags.pop();
                }
            }
        }
    }

    pub fn save_task(&mut self) {
        if self.current_mode == Mode::Editing {
            if let Some(selected) = self.todo_list.state.selected() {
                if let Some(editing_task) = &self.editing_task {
                    // Parse the due date string into NaiveDate
                    if let Some(due_date_str) = &editing_task.due_date_temp {
                        match NaiveDate::parse_from_str(due_date_str, "%Y-%m-%d") {
                            Ok(due_date) => {
                                self.todo_list.items[selected].due_date = Some(due_date);
                            }
                            Err(_) => {
                                // Handle invalid date format
                                self.todo_list.items[selected].due_date = None;
                            }
                        }
                    }
                    self.todo_list.items[selected] = editing_task.clone();
                }
            }
            self.current_mode = Mode::TaskList;
            self.editing_task = None;
        }
    }

    pub fn cancel_editing(&mut self) {
        self.current_mode = Mode::TaskList;
        self.editing_task = None;
    }

    pub fn run_with_handler<F>(
        &mut self,
        mut terminal: Terminal<impl Backend>,
        handler: F,
    ) -> io::Result<()>
    where
        F: Fn(&mut App, KeyEvent),
    {
        while !self.should_exit {
            terminal.draw(|f| f.render_widget(&mut *self, f.area()))?;
            if let Event::Key(key) = event::read()? {
                handler(self, key);
            };
        }
        Ok(())
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

    // TEST UTILITY FUNCTIONS

    pub fn load_test_data() -> Self {
        // Create some mock todo items for testing purposes
        let test_items = vec![
            TodoItem::new(Status::Todo, "Test Task 1", "Some info about Test Task 1"),
            TodoItem::new(Status::Todo, "Test Task 2", "Some info about Test Task 2"),
            TodoItem::new(
                Status::Completed,
                "Test Task 3",
                "Some info about Test Task 3",
            ),
        ];

        // Initialize the TodoList with the mock items
        let todo_list = TodoList {
            items: test_items,
            state: ListState::default(),
        };

        // Return the App with a test state
        Self {
            cursor_visible: true,
            should_exit: false,
            todo_list,
            current_mode: Mode::TaskList,
            editing_task: None,
            current_editing_field: EditingField::TaskName,
        }
    }
}
