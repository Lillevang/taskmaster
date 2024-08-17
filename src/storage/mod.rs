pub mod file;

pub use file::{load_from_file, save_to_file, TodoData};

use std::path::PathBuf;
use directories::BaseDirs;

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

