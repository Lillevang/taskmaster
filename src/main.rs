mod app;
mod handlers;
mod models;
mod storage;
mod tui;
mod ui;

use app::App;
use crossterm::event::{self, Event};
use handlers::handle_key;
use std::error::Error;
use std::time::{Duration, Instant};

fn main() -> Result<(), Box<dyn Error>> {
    tui::init_error_hooks()?;
    let mut terminal = tui::init_terminal()?; // Make the terminal variable mutable

    let mut app = App::load_or_default();
    let blink_interval = Duration::from_millis(500); // Blink every 500ms
    let mut last_blink = Instant::now();

    // Run the application loop with cursor blinking
    loop {
        // Handle the cursor blinking independently of keypresses
        if last_blink.elapsed() >= blink_interval {
            app.toggle_cursor_visibility(); // Toggle cursor visibility
            last_blink = Instant::now();
            terminal.draw(|f| f.render_widget(&mut app, f.area()))?; // Redraw UI to update cursor
        }

        // Non-blocking event handling
        if event::poll(Duration::from_millis(10))? {
            if let Event::Key(key) = event::read()? {
                handle_key(&mut app, key);

                // Redraw UI after a key event
                terminal.draw(|f| f.render_widget(&mut app, f.area()))?;
            }
        }

        // Exit the loop if the app signals to exit
        if app.should_exit {
            break;
        }
    }

    // Saves tasks before exiting
    app.save()?;

    tui::restore_terminal()?;
    Ok(())
}
