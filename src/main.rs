mod app;
mod models;
mod handlers;
mod ui;
mod tui;

use app::App;
use handlers::handle_key;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    tui::init_error_hooks()?;
    let terminal = tui::init_terminal()?;

    let mut app = App::default();
    app.run_with_handler(terminal, handle_key)?;

    tui::restore_terminal()?;
    Ok(())
}
