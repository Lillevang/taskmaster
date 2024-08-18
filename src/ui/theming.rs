use ratatui::style::{Color, Modifier, Style};

pub const TODO_HEADER_STYLE: Style = Style::new()
    .fg(Color::Rgb(240, 240, 240)) // Soft, warm off-white for the header text
    .bg(Color::Rgb(34, 34, 34)) // Deep charcoal for the header background
    .add_modifier(Modifier::BOLD); // Bold for a clear, prominent header

pub const NORMAL_ROW_BG: Color = Color::Rgb(45, 45, 45); // Dark, muted gray for row background
pub const ALT_ROW_BG_COLOR: Color = Color::Rgb(50, 50, 50); // Slightly lighter gray for alternate rows

pub const SELECTED_STYLE: Style = Style::new()
    .fg(Color::Rgb(255, 255, 255)) // Brighter white for selected text
    .bg(Color::Rgb(80, 100, 120)) // Slightly darker slate blue for more contrast
    .add_modifier(Modifier::BOLD)
    .add_modifier(Modifier::ITALIC); // Bold and italic for emphasis without overpowering

pub const TEXT_FG_COLOR: Color = Color::Rgb(220, 220, 220); // Soft light gray for regular text
pub const COMPLETED_TEXT_FG_COLOR: Color = Color::Rgb(144, 238, 144); // Subtle, light green for completed tasks

pub const fn alternate_colors(i: usize) -> Color {
    if i % 2 == 0 {
        NORMAL_ROW_BG
    } else {
        ALT_ROW_BG_COLOR
    }
}
