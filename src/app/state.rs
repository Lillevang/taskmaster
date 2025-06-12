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
    pub tag_temp: String,
    pub command_buffer: String,
}

pub struct TodoList {
    pub items: Vec<TodoItem>,
    pub state: ListState,
}
#[derive(PartialEq, Debug, Clone, Copy, Eq)]
pub enum Mode {
    TaskList,
    Editing,
    Creating,
    Command,
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
            tag_temp: String::new(),
            command_buffer: String::new(),
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
                tag_temp: String::new(),
                command_buffer: String::new(),
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

    pub fn create_new_task(&mut self) {
        // Switch to creating mode
        self.current_mode = Mode::Creating;
        self.current_editing_field = EditingField::TaskName;

        // Provide a blank template for the new task
        self.editing_task = Some(TodoItem {
            todo: String::new(),
            info: String::new(),
            status: Status::Todo,
            due_date: None,
            tags: vec![],
            due_date_temp: Some(String::new()),
        });
    }

    pub fn save_new_task(&mut self) {
        if let Some(new_task) = self.editing_task.take() {
            // Add the new task to the list
            self.todo_list.items.push(new_task);
            self.current_mode = Mode::TaskList;

            // Select the newly added task
            self.todo_list.state.select_last();
        }
    }

    pub fn delete_selected_task(&mut self) {
        if let Some(selected) = self.todo_list.state.selected() {
            // Remove the task from current state
            self.todo_list.items.remove(selected);

            // Reset the selected state to avaoid out-of-bounds selections
            self.todo_list.state.select_first();

            // Persist the updated state to the localfile
            if let Err(e) = self.save() {
                eprintln!("Failed to save the updated state: {}", e);
            }
        }
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
        if let Some(task) = &mut self.editing_task {
            match self.current_editing_field {
                EditingField::TaskName => {
                    task.todo.push(c); // Push characters to task name
                }
                EditingField::Description => {
                    task.info.push(c); // Push characters to description
                }
                EditingField::DueDate => {
                    task.due_date_temp.get_or_insert(String::new()).push(c); // Push characters to temporary due date
                }
                EditingField::Tags => {
                    if c == ' ' || c == ',' {
                        // Check for delimiter
                        if !self.tag_temp.trim().is_empty() {
                            task.tags.push(self.tag_temp.trim().to_string());
                        }
                        self.tag_temp.clear(); // Clear buffer for next tag
                    } else {
                        self.tag_temp.push(c); // Accumulate characters for the current tag
                    }
                }
            }
        }
    }

    pub fn backspace_field_input(&mut self) {
        if let Some(task) = &mut self.editing_task {
            match self.current_editing_field {
                EditingField::TaskName => {
                    task.todo.pop(); // Remove last character from task name
                }
                EditingField::Description => {
                    task.info.pop(); // Remove last character from description
                }
                EditingField::DueDate => {
                    if let Some(ref mut due_date_temp) = task.due_date_temp {
                        due_date_temp.pop(); // Remove last character from temporary due date
                    }
                }
                EditingField::Tags => {
                    // Remove the last character from the tag_temp buffer
                    self.tag_temp.pop();
                }
            }
        }
    }

    pub fn save_task(&mut self) {
        if self.current_mode == Mode::Editing {
            if let Some(selected) = self.todo_list.state.selected() {
                if let Some(mut editing_task) = self.editing_task.take() {
                    // Parse the due date string into NaiveDate
                    if let Some(due_date_str) = &editing_task.due_date_temp {
                        editing_task.due_date = Self::parse_due_date(due_date_str);
                    }
                    self.todo_list.items[selected] = editing_task.clone();
                }
            }

            // Persist the updated state to the localfile
            if let Err(e) = self.save() {
                eprintln!("Failed to save the updated state: {}", e);
            }

            // Switch back to TaskList mode
            self.current_mode = Mode::TaskList;
            self.current_editing_field = EditingField::TaskName;
            self.editing_task = None;
            self.tag_temp.clear();
        }
    }

    // Utility function to parse different date formats and keywords
    fn parse_due_date(input: &str) -> Option<NaiveDate> {
        match input.trim().to_lowercase().as_str() {
            "today" => Some(chrono::Local::now().naive_local().date()), // Parse "today"
            "tomorrow" => {
                Some(chrono::Local::now().naive_local().date() + chrono::Duration::days(1))
            } // Parse "tomorrow"
            _ => NaiveDate::parse_from_str(input, "%Y-%m-%d").ok(), // Try to parse as "YYYY-MM-DD"
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
        if self.todo_list.items.is_empty() {
            self.todo_list.state.select(None);
            return;
        }

        let next_index = match self.todo_list.state.selected() {
            Some(i) => {
                if i >= self.todo_list.items.len().saturating_sub(1) {
                    i
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.todo_list.state.select(Some(next_index));
    }
    pub fn select_previous(&mut self) {
        if self.todo_list.items.is_empty() {
            self.todo_list.state.select(None);
            return;
        }

        let prev_index = match self.todo_list.state.selected() {
            Some(i) => {
                if i == 0 {
                    0
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.todo_list.state.select(Some(prev_index));
    }

    pub fn select_first(&mut self) {
        if !self.todo_list.items.is_empty() {
            self.todo_list.state.select(Some(0));
        }
    }

    pub fn select_last(&mut self) {
        if !self.todo_list.items.is_empty() {
            self.todo_list
                .state
                .select(Some(self.todo_list.items.len() - 1));
        }
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
            tag_temp: String::new(),
            command_buffer: String::new(),
        }
    }

    pub fn enter_command_mode(&mut self) {
        self.current_mode = Mode::Command;
        self.command_buffer.clear();
    }

    pub fn exit_command_mode(&mut self) {
        self.current_mode = Mode::TaskList;
        self.command_buffer.clear();
    }

    pub fn handle_command(&mut self) {
        match self.command_buffer.as_str() {
            ":q" | ":quit" => self.should_exit = true,
            ":w" | ":write" => {
                if let Err(e) = self.save() {
                    eprintln!("Failed to save: {}", e);
                }
            }
            ":help" => {
                // TODO: Implement help display
            }
            _ => {
                // Unknown command
            }
        }
        self.exit_command_mode();
    }
}
