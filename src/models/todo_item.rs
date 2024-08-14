#[derive(Debug, Clone)]
pub struct TodoItem {
    pub todo: String,
    pub info: String,
    pub status: Status,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
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
        }
    }
}