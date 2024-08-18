use crate::models::TodoItem;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self};
use std::path::Path;

#[derive(Serialize, Deserialize, Debug)]
pub struct TodoData {
    pub items: Vec<TodoItem>,
}

// Save the TodoData to a file at the given path
pub fn save_to_file(path: &Path, todo_data: &TodoData) -> io::Result<()> {
    // Ensure the parent directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?; // Create the directory if it doesn't exist
    }

    // Open the file for writing
    let file = fs::File::create(path)?;

    // Write the TodoData to the file in pretty JSON format
    serde_json::to_writer_pretty(file, todo_data)?;

    Ok(())
}

// Load TodoData from the file at the given path
pub fn load_from_file(path: &Path) -> io::Result<TodoData> {
    // Check if the file exists
    if path.exists() {
        // Read the file content as a string
        let file_content = fs::read_to_string(path)?;

        // Deserialize the JSON string into TodoData
        let todo_data: TodoData = serde_json::from_str(&file_content)?;

        Ok(todo_data)
    } else {
        // Return an error if the file does not exist
        Err(io::Error::new(io::ErrorKind::NotFound, "File not found"))
    }
}
