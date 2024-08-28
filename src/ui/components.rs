use crate::app::state::{EditingField, Mode};
use crate::app::App;
use crate::models;
use crate::ui::theming::{
    alternate_colors, COMPLETED_TEXT_FG_COLOR, NORMAL_ROW_BG, SELECTED_STYLE, TEXT_FG_COLOR,
    TODO_HEADER_STYLE,
};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span, Text},
    widgets::{
        Block, Borders, HighlightSpacing, List, ListItem, Padding, Paragraph, StatefulWidget,
        Widget, Wrap,
    },
};

impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(3)].as_ref())
            .split(area);

        let content_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(25), Constraint::Percentage(75)].as_ref())
            .split(layout[0]);

        self.render_list(content_layout[0], buf); // Left pane for task list

        match self.current_mode {
            Mode::TaskList => self.render_selected_item(content_layout[1], buf), // Right pane for task details
            Mode::Editing => self.render_editing_item(content_layout[1], buf), // Right pane for editing
            Mode::Creating => self.render_editing_item(content_layout[1], buf), // Right pane for creating new task
        }

        App::render_footer(layout[1], buf); // Footer section at the bottom
    }
}

impl App {
    fn render_footer(area: Rect, buf: &mut Buffer) {
        Paragraph::new("Use ↓↑ to move, ← to unselect, → to change status, g/G to go top/bottom. Press 'e' to edit, 'q' to quit. Use Tab to switch fields, Ctrl+S to save.")
            .style(Style::default().fg(TEXT_FG_COLOR))
            .centered()
            .render(area, buf);
    }

    fn render_list(&mut self, area: Rect, buf: &mut Buffer) {
        let block = Block::new()
            .title(Line::raw("TODO List").centered())
            .borders(Borders::ALL)
            .border_style(TODO_HEADER_STYLE)
            .style(Style::default().bg(NORMAL_ROW_BG));

        // Iterate through all elements in the `items` and stylize them.
        let items: Vec<ListItem> = self
            .todo_list
            .items
            .iter()
            .enumerate()
            .map(|(i, todo_item)| {
                let color = alternate_colors(i);
                let status_symbol = if todo_item.status == models::Status::Completed {
                    "✓"
                } else {
                    "☐"
                };

                let content = Line::styled(
                    format!("{} {}", status_symbol, todo_item.todo),
                    Style::default().fg(if todo_item.status == models::Status::Completed {
                        COMPLETED_TEXT_FG_COLOR // Color for completed tasks
                    } else {
                        TEXT_FG_COLOR
                    }),
                );

                ListItem::new(content).style(Style::default().bg(color))
            })
            .collect();

        let list = List::new(items)
            .block(block)
            .highlight_style(SELECTED_STYLE)
            .highlight_symbol(">")
            .highlight_spacing(HighlightSpacing::Always);

        // We need to disambiguate this trait method as both `Widget` and `StatefulWidget` share the
        // same method name `render`.
        StatefulWidget::render(list, area, buf, &mut self.todo_list.state);
    }

    fn render_selected_item(&self, area: Rect, buf: &mut Buffer) {
        let block = Block::new()
            .title(Line::raw("Task Details").centered())
            .borders(Borders::ALL)
            .border_style(TODO_HEADER_STYLE)
            .style(Style::default().bg(NORMAL_ROW_BG))
            .padding(Padding::horizontal(1));

        let info = if let Some(i) = self.todo_list.state.selected() {
            let selected_task = &self.todo_list.items[i];
            format!(
                "{}\n\nDescription:\n{}\n\n{}\n{}",
                if selected_task.status == models::Status::Completed {
                    format!("✓ DONE: {}", selected_task.todo)
                } else {
                    format!("☐ TODO: {}", selected_task.todo)
                },
                selected_task.info,
                selected_task
                    .due_date
                    .map_or("No due date".to_string(), |d| format!("Due: {}", d)),
                if !selected_task.tags.is_empty() {
                    format!("Tags: {}", selected_task.tags.join(", "))
                } else {
                    "No tags".to_string()
                }
            )
        } else {
            "No task selected...".to_string()
        };

        Paragraph::new(info)
            .block(block)
            .style(Style::default().fg(TEXT_FG_COLOR))
            .wrap(Wrap { trim: false })
            .render(area, buf);
    }

    fn render_editing_item(&self, area: Rect, buf: &mut Buffer) {
        if let Some(editing_task) = &self.editing_task {
            let block = Block::new()
                .title(Line::raw("Edit Task").centered())
                .borders(Borders::ALL)
                .border_style(TODO_HEADER_STYLE)
                .style(Style::default().bg(NORMAL_ROW_BG))
                .padding(Padding::horizontal(1));

            let cursor_style = Style::default().fg(Color::White); // Set cursor color

            // Task name field with cursor if active
            let task_name_line = if self.current_editing_field == EditingField::TaskName {
                let cursor = if self.cursor_visible { "|" } else { " " };
                Line::from(vec![
                    Span::raw("Task: "),
                    Span::raw(&editing_task.todo),
                    Span::styled(cursor, cursor_style),
                ])
            } else {
                Line::from(vec![Span::raw("Task: "), Span::raw(&editing_task.todo)])
            };

            // Description field with cursor if active
            let description_line = if self.current_editing_field == EditingField::Description {
                let cursor = if self.cursor_visible { "|" } else { " " };
                Line::from(vec![
                    Span::raw("Description: "),
                    Span::raw(&editing_task.info),
                    Span::styled(cursor, cursor_style),
                ])
            } else {
                Line::from(vec![
                    Span::raw("Description: "),
                    Span::raw(&editing_task.info),
                ])
            };

            // Due date field with cursor if active
            let due_date_text = editing_task
                .due_date_temp
                .clone()
                .unwrap_or_else(|| "No due date".to_string());
            let due_date_line = if self.current_editing_field == EditingField::DueDate {
                let cursor = if self.cursor_visible { "|" } else { " " };
                Line::from(vec![
                    Span::raw("Due Date: "),
                    Span::raw(due_date_text),
                    Span::styled(cursor, cursor_style),
                ])
            } else {
                Line::from(vec![Span::raw("Due Date: "), Span::raw(due_date_text)])
            };

            // Tags field with cursor if active
            let tags_line = if self.current_editing_field == EditingField::Tags {
                let cursor = if self.cursor_visible { "|" } else { " " };
                Line::from(vec![
                    Span::raw("Tags: "),
                    Span::raw(editing_task.tags.join(", ")),
                    Span::styled(cursor, cursor_style),
                ])
            } else {
                Line::from(vec![
                    Span::raw("Tags: "),
                    Span::raw(editing_task.tags.join(", ")),
                ])
            };

            // Combine all lines into a Text object
            let info = Text::from(vec![
                task_name_line,
                description_line,
                due_date_line,
                tags_line,
            ]);

            Paragraph::new(info)
                .block(block)
                .style(Style::default().fg(TEXT_FG_COLOR))
                .wrap(Wrap { trim: false })
                .render(area, buf);
        }
    }
}
