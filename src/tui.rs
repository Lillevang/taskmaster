// The tui.rs file is responsible for setting up and restoring the terminal state, as well as initializing error hooks for handling panics and errors gracefully.
// It uses the ratatui crate for terminal UI management and color_eyre for enhanced error reporting.

use std::{io, io::stdout};

use color_eyre::config::HookBuilder;
use ratatui::{
    backend::{Backend, CrosstermBackend},
    crossterm::{
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
        ExecutableCommand,
    },
    Terminal,
};

pub fn init_error_hooks() -> color_eyre::Result<()> {
    let (panic, error) = HookBuilder::default().into_hooks();
    let panic = panic.into_panic_hook();
    let error = error.into_eyre_hook();
    color_eyre::eyre::set_hook(Box::new(move |e| {
        let _ = restore_terminal();
        error(e)
    }))?;
    std::panic::set_hook(Box::new(move |info| {
        let _ = restore_terminal();
        panic(info);
    }));
    Ok(())
}

pub fn init_terminal() -> io::Result<Terminal<impl Backend>> {
    stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;
    Terminal::new(CrosstermBackend::new(stdout()))
}

pub fn restore_terminal() -> io::Result<()> {
    stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_error_hooks() {
        let result = init_error_hooks();
        assert!(result.is_ok(), "Failed to initialize error hooks");
    }

    #[test]
    fn test_init_terminal() {
        if std::env::var("CI").is_ok() {
            eprintln!("Skipping terminal initialization test in CI environment.");
            return;
        }
        let result = init_terminal();
        assert!(result.is_ok(), "Failed to initialize terminal");
        // Normally, you would restore the terminal after this, but since we cannot directly observe
        // the effects of terminal initialization in a unit test, we'll focus on the success of the operation.
        let _ = restore_terminal();
    }

    #[test]
    fn test_restore_terminal() {
        let result = restore_terminal();
        assert!(result.is_ok(), "Failed to restore terminal");
    }
}
