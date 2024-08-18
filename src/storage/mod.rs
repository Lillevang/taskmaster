pub mod file;

pub use file::{load_from_file, save_to_file, TodoData};

use directories::BaseDirs;
use std::path::PathBuf;

pub fn get_default_storage_path() -> PathBuf {
    if let Some(base_dirs) = BaseDirs::new() {
        let mut path = base_dirs.home_dir().to_path_buf();
        path.push(".taskmaster");
        path.push("tasks.json");
        path
    } else {
        panic!("Could not determine home directory");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Status, TodoItem};
    use std::fs;
    use std::io::ErrorKind::NotFound;
    use tempfile::tempdir;

    #[test]
    fn test_save_to_file() {
        // Arrange
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("todo_data.json");
        let todo_item = TodoItem::new(Status::Todo, "Test task", "Testing save functionality");
        let todo_data = TodoData {
            items: vec![todo_item.clone()],
        };

        // Act
        let result = save_to_file(&file_path, &todo_data);

        // Assert
        assert!(result.is_ok());
        assert!(file_path.exists());

        // Verify content
        let saved_content = fs::read_to_string(&file_path).unwrap();
        let loaded_data: TodoData = serde_json::from_str(&saved_content).unwrap();
        assert_eq!(loaded_data.items.len(), 1);
        assert_eq!(loaded_data.items[0].todo, todo_item.todo);
    }

    #[test]
    fn test_load_from_file() {
        // Arrange
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("todo_data.json");
        let todo_item = TodoItem::new(Status::Todo, "Test task", "Testing load functionality");
        let todo_data = TodoData {
            items: vec![todo_item.clone()],
        };

        // Save the data first
        save_to_file(&file_path, &todo_data).unwrap();

        // Act
        let result = load_from_file(&file_path);

        // Assert
        assert!(result.is_ok());
        let loaded_data = result.unwrap();
        assert_eq!(loaded_data.items.len(), 1);
        assert_eq!(loaded_data.items[0].todo, todo_item.todo);
    }

    #[test]
    fn test_load_from_non_existent_file() {
        // Arrange
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("non_existent.json");

        // Act
        let result = load_from_file(&file_path);

        // Assert
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), NotFound);
    }
}
