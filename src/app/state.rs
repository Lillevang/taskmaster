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

#[derive(Debug, PartialEq, Eq)]
pub enum ConfirmationState {
    None,
    Delete,
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
    pub confirmation_state: ConfirmationState,
    pub show_archived: bool,
}

pub struct TodoList {
    pub active_items: Vec<TodoItem>,
    pub archived_items: Vec<TodoItem>,
    pub state: ListState,
    pub selected_indices: Vec<usize>,
    pub show_archived: bool,
}

#[derive(PartialEq, Debug, Clone, Copy, Eq)]
pub enum Mode {
    TaskList,
    Editing,
    Creating,
    Command,
    Help,
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
            confirmation_state: ConfirmationState::None,
            show_archived: false,
        }
    }
}

impl Default for TodoList {
    fn default() -> Self {
        Self {
            active_items: Vec::new(),
            archived_items: Vec::new(),
            state: ListState::default(),
            selected_indices: Vec::new(),
            show_archived: false,
        }
    }
}

impl FromIterator<(Status, &'static str, &'static str)> for TodoList {
    fn from_iter<I: IntoIterator<Item = (Status, &'static str, &'static str)>>(iter: I) -> Self {
        let mut active_items = Vec::new();
        let mut archived_items = Vec::new();

        for (status, todo, info) in iter {
            let item = TodoItem::new(status, todo, info);
            match item.status {
                Status::Archived => archived_items.push(item),
                _ => active_items.push(item),
            }
        }

        Self {
            active_items,
            archived_items,
            state: ListState::default(),
            selected_indices: Vec::new(),
            show_archived: false,
        }
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
            Ok(todo_data) => {
                let mut active_items = Vec::new();
                let mut archived_items = Vec::new();

                for item in todo_data.items {
                    match item.status {
                        Status::Archived => archived_items.push(item),
                        _ => active_items.push(item),
                    }
                }

                Self {
                    cursor_visible: true,
                    should_exit: false,
                    todo_list: TodoList {
                        active_items,
                        archived_items,
                        state: ListState::default(),
                        selected_indices: Vec::new(),
                        show_archived: false,
                    },
                    current_mode: Mode::TaskList,
                    editing_task: None,
                    current_editing_field: EditingField::TaskName,
                    tag_temp: String::new(),
                    command_buffer: String::new(),
                    confirmation_state: ConfirmationState::None,
                    show_archived: false,
                }
            }
            Err(_) => Self::default(),
        }
    }

    pub fn save(&self) -> io::Result<()> {
        let storage_path = get_default_storage_path();
        let todo_data = TodoData {
            items: [&self.todo_list.active_items[..], &self.todo_list.archived_items[..]].concat(),
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
            self.todo_list.active_items.push(new_task);
            self.current_mode = Mode::TaskList;

            // Select the newly added task
            self.todo_list.state.select_last();
        }
    }

    pub fn delete_selected_task(&mut self) {
        if let Some(selected) = self.todo_list.state.selected() {
            if self.todo_list.show_archived {
                if selected < self.todo_list.archived_items.len() {
                    self.todo_list.archived_items.remove(selected);
                }
            } else {
                if selected < self.todo_list.active_items.len() {
                    self.todo_list.active_items.remove(selected);
                }
            }

            // Reset the selected state to avoid out-of-bounds selections
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
            let mut task = self.todo_list.active_items[selected].clone();
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
                    self.todo_list.active_items[selected] = editing_task.clone();
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
            self.todo_list.active_items[i].status = match self.todo_list.active_items[i].status {
                Status::Todo => Status::Completed,
                Status::Completed => Status::Todo,
                Status::Archived => Status::Todo,
            }
        }
    }

    pub fn toggle_archive_visibility(&mut self) {
        self.show_archived = !self.show_archived;
        // Reset selection when switching lists
        self.clear_selection();
        self.todo_list.state.select_first();
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
            active_items: test_items,
            archived_items: Vec::new(),
            state: ListState::default(),
            selected_indices: Vec::new(),
            show_archived: false,
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
            confirmation_state: ConfirmationState::None,
            show_archived: false,
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

    pub fn archive_selected_task(&mut self) {
        if let Some(selected) = self.todo_list.state.selected() {
            if !self.todo_list.show_archived {
                if selected < self.todo_list.active_items.len() {
                    let item = self.todo_list.active_items.remove(selected);
                    self.todo_list.archived_items.push(item);
                    if let Err(e) = self.save() {
                        eprintln!("Failed to save after archiving: {}", e);
                    }
                }
            }
        }
    }

    pub fn unarchive_selected_task(&mut self) {
        if let Some(selected) = self.todo_list.state.selected() {
            if self.todo_list.show_archived {
                if selected < self.todo_list.archived_items.len() {
                    let item = self.todo_list.archived_items.remove(selected);
                    self.todo_list.active_items.push(item);
                    if let Err(e) = self.save() {
                        eprintln!("Failed to save after unarchiving: {}", e);
                    }
                }
            }
        }
    }

    pub fn request_delete_confirmation(&mut self) {
        self.confirmation_state = ConfirmationState::Delete;
    }

    pub fn confirm_delete(&mut self) {
        if self.confirmation_state == ConfirmationState::Delete {
            self.delete_selected_task();
            self.confirmation_state = ConfirmationState::None;
        }
    }

    pub fn cancel_delete(&mut self) {
        self.confirmation_state = ConfirmationState::None;
    }

    pub fn toggle_selection(&mut self) {
        if let Some(selected) = self.todo_list.state.selected() {
            if self.todo_list.selected_indices.contains(&selected) {
                // Remove from selection
                self.todo_list.selected_indices.retain(|&i| i != selected);
            } else {
                // Add to selection
                self.todo_list.selected_indices.push(selected);
            }
        }
    }

    pub fn clear_selection(&mut self) {
        self.todo_list.selected_indices.clear();
    }

    pub fn archive_selected_tasks(&mut self) {
        let indices = if self.todo_list.selected_indices.is_empty() {
            if let Some(selected) = self.todo_list.state.selected() {
                vec![selected]
            } else {
                vec![]
            }
        } else {
            self.todo_list.selected_indices.clone()
        };

        // Move selected completed items to archived list
        let mut items_to_move = Vec::new();
        for &i in &indices {
            if i < self.todo_list.active_items.len() {
                if self.todo_list.active_items[i].status == Status::Completed {
                    items_to_move.push(i);
                }
            }
        }

        // Remove from active list and add to archived list
        for &i in items_to_move.iter().rev() {
            let item = self.todo_list.active_items.remove(i);
            self.todo_list.archived_items.push(item);
        }

        // Clear selection after archiving
        self.clear_selection();
        
        // Update list state
        if let Some(selected) = self.todo_list.state.selected() {
            if selected >= self.get_visible_tasks().len() {
                self.todo_list.state.select(Some(self.get_visible_tasks().len().saturating_sub(1)));
            }
        }
        
        if let Err(e) = self.save() {
            eprintln!("Failed to save after archiving: {}", e);
        }
    }

    pub fn archive_all_completed(&mut self) {
        self.todo_list.archive_all_completed();
        if let Err(e) = self.save() {
            eprintln!("Failed to save after archiving all completed: {}", e);
        }
    }

    pub fn handle_command(&mut self, cmd: &str) -> Result<(), String> {
        let parts: Vec<&str> = cmd.split_whitespace().collect();
        if parts.is_empty() {
            return Ok(());
        }

        match parts[0].to_lowercase().as_str() {
            "add" | "a" | "+" => {
                if parts.len() < 2 {
                    return Err("Usage: add <task description>".to_string());
                }
                let description = parts[1..].join(" ");
                self.todo_list.add_task(description);
                Ok(())
            }
            "delete" | "d" | "del" | "-" => {
                if parts.len() < 2 {
                    return Err("Usage: delete <task number>".to_string());
                }
                if let Ok(index) = parts[1].parse::<usize>() {
                    if index == 0 {
                        return Err("Task numbers start from 1".to_string());
                    }
                    self.todo_list.delete_task(index - 1);
                    Ok(())
                } else {
                    Err("Invalid task number".to_string())
                }
            }
            "complete" | "c" | "done" | "✓" => {
                if parts.len() < 2 {
                    return Err("Usage: complete <task number>".to_string());
                }
                if let Ok(index) = parts[1].parse::<usize>() {
                    if index == 0 {
                        return Err("Task numbers start from 1".to_string());
                    }
                    self.todo_list.toggle_complete(index - 1);
                    Ok(())
                } else {
                    Err("Invalid task number".to_string())
                }
            }
            "archive" | "ar" | ">" => {
                if parts.len() < 2 {
                    return Err("Usage: archive <task number>".to_string());
                }
                if let Ok(index) = parts[1].parse::<usize>() {
                    if index == 0 {
                        return Err("Task numbers start from 1".to_string());
                    }
                    if index > self.todo_list.active_items.len() {
                        return Err(format!("Task number {} is out of range", index));
                    }
                    self.todo_list.archive_task(index - 1);
                    Ok(())
                } else {
                    Err("Invalid task number".to_string())
                }
            }
            "unarchive" | "uar" | "<" => {
                if parts.len() < 2 {
                    return Err("Usage: unarchive <task number>".to_string());
                }
                if let Ok(index) = parts[1].parse::<usize>() {
                    if index == 0 {
                        return Err("Task numbers start from 1".to_string());
                    }
                    if index > self.todo_list.archived_items.len() {
                        return Err(format!("Task number {} is out of range", index));
                    }
                    self.todo_list.unarchive_task(index - 1);
                    Ok(())
                } else {
                    Err("Invalid task number".to_string())
                }
            }
            "archiveall" | "aa" | ">>" => {
                self.todo_list.archive_all_completed();
                Ok(())
            }
            "view" | "v" => {
                if parts.len() < 2 {
                    return Err("Usage: view <active|archived>".to_string());
                }
                match parts[1].to_lowercase().as_str() {
                    "active" | "a" => {
                        self.todo_list.show_archived = false;
                        Ok(())
                    }
                    "archived" | "ar" => {
                        self.todo_list.show_archived = true;
                        Ok(())
                    }
                    _ => Err("Invalid view. Use 'active' or 'archived'".to_string()),
                }
            }
            "toggle" | "t" | "~" => {
                self.todo_list.toggle_archive_view();
                Ok(())
            }
            "help" | "h" | "?" => {
                self.current_mode = Mode::Help;
                Ok(())
            }
            "quit" | "q" | "exit" => {
                self.should_exit = true;
                Ok(())
            }
            _ => Err(format!("Unknown command: {}. Type 'h' for help", parts[0])),
        }
    }

    pub fn get_visible_tasks(&self) -> Vec<&TodoItem> {
        if self.show_archived {
            self.todo_list.archived_items.iter().collect()
        } else {
            self.todo_list.active_items.iter().collect()
        }
    }

    pub fn get_visible_tasks_mut(&mut self) -> Vec<&mut TodoItem> {
        if self.show_archived {
            self.todo_list.archived_items.iter_mut().collect()
        } else {
            self.todo_list.active_items.iter_mut().collect()
        }
    }
}

impl TodoList {
    pub fn toggle_archive_view(&mut self) {
        self.show_archived = !self.show_archived;
        // Reset selection when switching views
        self.state.select(None);
        self.selected_indices.clear();
    }

    pub fn add_task(&mut self, description: String) {
        let new_task = TodoItem {
            todo: description,
            info: String::new(),
            status: Status::Todo,
            due_date: None,
            tags: Vec::new(),
            due_date_temp: None,
        };
        self.active_items.push(new_task);
    }

    pub fn delete_task(&mut self, index: usize) {
        if index < self.active_items.len() {
            self.active_items.remove(index);
        }
    }

    pub fn toggle_complete(&mut self, index: usize) {
        if index < self.active_items.len() {
            self.active_items[index].status = match self.active_items[index].status {
                Status::Todo => Status::Completed,
                Status::Completed => Status::Todo,
                Status::Archived => Status::Todo,
            };
        }
    }

    pub fn archive_task(&mut self, index: usize) {
        if index < self.active_items.len() {
            let item = self.active_items.remove(index);
            self.archived_items.push(item);
        }
    }

    pub fn unarchive_task(&mut self, index: usize) {
        if index < self.archived_items.len() {
            let item = self.archived_items.remove(index);
            self.active_items.push(item);
        }
    }

    pub fn archive_all_completed(&mut self) {
        let mut i = 0;
        while i < self.active_items.len() {
            if self.active_items[i].status == Status::Completed {
                let item = self.active_items.remove(i);
                self.archived_items.push(item);
            } else {
                i += 1;
            }
        }
    }
}
