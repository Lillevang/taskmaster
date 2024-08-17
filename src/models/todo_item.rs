use chrono::{NaiveDate, ParseError};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TodoItem {
    pub todo: String,
    pub info: String,
    pub status: Status,
    pub due_date: Option<NaiveDate>,
    pub due_date_temp: Option<String>,
    pub tags: Vec<String>
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Status {
    Todo,
    Completed,
}

impl TodoItem {
    pub fn new(status: Status, todo: &str, info: &str) -> Self {
        Self {
            status,
            todo: todo.to_string(),
            info: info.to_string(),
            due_date: None,
            due_date_temp: None,
            tags: Vec::new()
        }
    }

    pub fn set_due_date(&mut self, date_str: &str) -> Result<(), ParseError> {
        self.due_date = Some(NaiveDate::parse_from_str(date_str, "%Y-%m-%d")?);
        Ok(())
    }

    pub fn add_tag(&mut self, tag: &str) {
        if !self.tags.contains(&tag.to_string()) {
            self.tags.push(tag.to_string());
        }
    }
}