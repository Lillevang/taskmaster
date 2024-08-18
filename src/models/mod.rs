pub mod todo_item;

pub use todo_item::{Status, TodoItem};

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    fn create_todo_item() -> TodoItem {
        TodoItem::new(Status::Todo, "Test Task", "This is a test task")
    }

    #[test]
    fn test_todo_item_creation() {
        let todo_item = create_todo_item();
        assert_eq!(todo_item.todo, "Test Task");
        assert_eq!(todo_item.info, "This is a test task");
        assert_eq!(todo_item.status, Status::Todo);
        assert!(todo_item.due_date.is_none());
        assert!(todo_item.tags.is_empty());
    }

    #[test]
    fn test_set_due_date() {
        let mut todo_item = create_todo_item();
        let date_str = "2024-08-18";
        todo_item.set_due_date(date_str).unwrap();
        assert_eq!(
            todo_item.due_date,
            Some(NaiveDate::from_ymd_opt(2024, 8, 18).unwrap())
        );
    }

    #[test]
    fn test_set_invalid_due_date() {
        let mut todo_item = TodoItem::new(Status::Todo, "Test Task", "This is a test task");
        let date_str = "Invalid date";
        let result = todo_item.set_due_date(date_str);
        assert!(result.is_err());
    }

    #[test]
    fn test_add_tag() {
        let mut todo_item = create_todo_item();
        todo_item.add_tag("urgent");
        assert_eq!(todo_item.tags.len(), 1);
        assert!(todo_item.tags.contains(&"urgent".to_string()));

        // Adding the same tag again should not duplicate it
        todo_item.add_tag("urgent");
        assert_eq!(todo_item.tags.len(), 1);
    }

    #[test]
    fn test_add_multiple_tags() {
        let mut todo_item = create_todo_item();
        todo_item.add_tag("urgent");
        todo_item.add_tag("work");
        assert_eq!(todo_item.tags.len(), 2);
        assert!(todo_item.tags.contains(&"urgent".to_string()));
        assert!(todo_item.tags.contains(&"work".to_string()));
    }
}
