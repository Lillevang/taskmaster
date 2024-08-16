use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::Stylize,
    text::Line,
    widgets::{Block, Borders, HighlightSpacing, List, ListItem, Padding, Paragraph, StatefulWidget, Widget, Wrap},
};
use crate::app::App;
use crate::models;
use crate::ui::theming::{
    TODO_HEADER_STYLE, NORMAL_ROW_BG, SELECTED_STYLE, TEXT_FG_COLOR, COMPLETED_TEXT_FG_COLOR, alternate_colors,
};

impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(3)].as_ref())
            .split(area);

        let content_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(30), Constraint::Percentage(70)].as_ref())
            .split(layout[0]);

        self.render_list(content_layout[0], buf); // Left pane for task list
        self.render_selected_item(content_layout[1], buf); // Right pane for task details
        App::render_footer(layout[1], buf); // Footer section at the bottom
    }
}

impl App {
    fn render_footer(area: Rect, buf: &mut Buffer) {
        Paragraph::new("Use ↓↑ to move, ← to unselect, → to change status, g/G to go top/bottom.")
            .centered()
            .render(area, buf);
    }

    fn render_list(&mut self, area: Rect, buf: &mut Buffer) {
        let block = Block::new()
            .title(Line::raw("TODO List").centered())
            .borders(Borders::ALL)
            .border_style(TODO_HEADER_STYLE)
            .bg(NORMAL_ROW_BG);

        let items: Vec<ListItem> = self
            .todo_list
            .items
            .iter()
            .enumerate()
            .map(|(i, todo_item)| {
                let color = alternate_colors(i);
                ListItem::from(todo_item).bg(color)
            })
            .collect();

        let list = List::new(items)
            .block(block)
            .highlight_style(SELECTED_STYLE)
            .highlight_symbol(">")
            .highlight_spacing(HighlightSpacing::Always);

        StatefulWidget::render(list, area, buf, &mut self.todo_list.state);
    }

    fn render_selected_item(&self, area: Rect, buf: &mut Buffer) {
        let block = Block::new()
            .title(Line::raw("Task Details").centered())
            .borders(Borders::ALL)
            .border_style(TODO_HEADER_STYLE)
            .bg(NORMAL_ROW_BG)
            .padding(Padding::horizontal(1));

        let info = if let Some(i) = self.todo_list.state.selected() {
            match self.todo_list.items[i].status {
                models::Status::Completed => format!(
                    "✓ DONE: {}\n\nDescription:\n{}",
                    self.todo_list.items[i].todo, // Use the task title
                    self.todo_list.items[i].info // Use the task info for description
                ),
                models::Status::Todo => format!(
                    "☐ TODO: {}\n\nDescription:\n{}",
                    self.todo_list.items[i].todo,
                    self.todo_list.items[i].info
                ),
            }
        } else {
            "No task selected...".to_string()
        };

        Paragraph::new(info)
            .block(block)
            .fg(TEXT_FG_COLOR)
            .wrap(Wrap { trim: false })
            .render(area, buf);
    }
}

impl From<&models::TodoItem> for ListItem<'_> {
    fn from(value: &models::TodoItem) -> Self {
        let line = match value.status {
            models::Status::Todo => Line::styled(format!(" ☐ {}", value.todo), TEXT_FG_COLOR),
            models::Status::Completed => {
                Line::styled(format!(" ✓ {}", value.todo), COMPLETED_TEXT_FG_COLOR)
            }
        };
        ListItem::new(line)
    }
}
