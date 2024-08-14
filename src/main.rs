mod models;
mod app;
mod ui;
mod tui;

use app::App;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    tui::init_error_hooks()?;
    let terminal = tui::init_terminal()?;

    let mut app = App::default();
    app.run(terminal)?;

    tui::restore_terminal()?;
    Ok(())
}
