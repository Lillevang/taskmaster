use crate::app::state::{EditingField, Mode, ConfirmationState};
use crate::app::App;
use crate::models::{Status, TodoItem};
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
            Mode::Command => self.render_selected_item(content_layout[1], buf), // Keep showing selected item in command mode
            Mode::Help => self.render_help(content_layout[1], buf), // Show help information
        }

        // Render footer, command input, or confirmation based on state
        match (self.current_mode, &self.confirmation_state) {
            (Mode::Command, _) => self.render_command_input(layout[1], buf),
            (_, ConfirmationState::Delete) => self.render_delete_confirmation(layout[1], buf),
            _ => App::render_footer(layout[1], buf),
        }
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
        let title = if self.show_archived {
            "ARCHIVED TASKS"
        } else {
            "ACTIVE TASKS"
        };

        let block = Block::new()
            .title(Line::raw(title).centered())
            .borders(Borders::ALL)
            .border_style(TODO_HEADER_STYLE)
            .style(Style::default().bg(NORMAL_ROW_BG));

        // Get visible tasks and their indices
        let visible_tasks: Vec<(usize, &TodoItem)> = if self.show_archived {
            self.todo_list.archived_items.iter().enumerate().collect()
        } else {
            self.todo_list.active_items.iter().enumerate().collect()
        };

        // Create list items from visible tasks
        let items: Vec<ListItem> = visible_tasks
            .iter()
            .map(|(i, todo_item)| {
                let color = alternate_colors(*i);
                let status_symbol = todo_item.status.symbol();
                let is_selected = self.todo_list.selected_indices.contains(i);
                let is_current = self.todo_list.state.selected() == Some(*i);

                let content = Line::styled(
                    format!("{} {}", status_symbol, todo_item.todo),
                    Style::default().fg(if todo_item.status == Status::Completed {
                        COMPLETED_TEXT_FG_COLOR
                    } else if todo_item.status == Status::Archived {
                        Color::DarkGray
                    } else {
                        TEXT_FG_COLOR
                    }),
                );

                let style = if is_selected {
                    Style::default()
                        .bg(Color::Blue)
                        .fg(Color::White)
                        .add_modifier(ratatui::style::Modifier::BOLD)
                } else if is_current {
                    Style::default()
                        .bg(color)
                        .add_modifier(ratatui::style::Modifier::BOLD)
                } else {
                    Style::default().bg(color)
                };

                ListItem::new(content).style(style)
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
            .style(Style::default().bg(NORMAL_ROW_BG))
            .padding(Padding::horizontal(1));

        let info = if let Some(i) = self.todo_list.state.selected() {
            let selected_task = if self.show_archived {
                &self.todo_list.archived_items[i]
            } else {
                &self.todo_list.active_items[i]
            };
            format!(
                "{}\n\nDescription:\n{}\n\n{}\n{}",
                if selected_task.status == Status::Completed {
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
                    Span::raw("> Task: "),
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
                    Span::raw("> Description: "),
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
                    Span::raw("> Due Date: "),
                    Span::raw(due_date_text),
                    Span::styled(cursor, cursor_style),
                ])
            } else {
                Line::from(vec![Span::raw("Due Date: "), Span::raw(due_date_text)])
            };

            // Tags field with real-time `tag_temp` rendering
            let tags_line = if self.current_editing_field == EditingField::Tags {
                let cursor = if self.cursor_visible { "|" } else { " " };

                // Combine existing tags with the ongoing input
                let combined_tags = if !self.tag_temp.is_empty() {
                    let mut all_tags = editing_task.tags.join(", ");
                    if !all_tags.is_empty() {
                        all_tags.push_str(", ");
                    }
                    all_tags.push_str(&self.tag_temp); // Include ongoing input
                    all_tags
                } else {
                    editing_task.tags.join(", ") // Only display existing tags if `tag_temp` is empty
                };

                Line::from(vec![
                    Span::raw("> Tags: "),
                    Span::raw(combined_tags),
                    Span::styled(cursor, cursor_style),
                ])
            } else {
                let tags_display = editing_task.tags.join(", ");
                Line::from(vec![Span::raw("Tags: "), Span::raw(tags_display)])
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

    fn render_command_input(&self, area: Rect, buf: &mut Buffer) {
        let cursor = if self.cursor_visible { "|" } else { " " };
        let command_text = format!(":{}", self.command_buffer);
        let text = format!("{}{}", command_text, cursor);

        Paragraph::new(text)
            .style(Style::default().fg(TEXT_FG_COLOR))
            .render(area, buf);
    }

    fn render_delete_confirmation(&self, area: Rect, buf: &mut Buffer) {
        let text = "Are you sure you want to delete this task? (y/n)";
        Paragraph::new(text)
            .style(Style::default().fg(Color::Red))
            .centered()
            .render(area, buf);
    }

    fn render_help(&self, area: Rect, buf: &mut Buffer) {
        let block = Block::new()
            .title(Line::raw("Help").centered())
            .borders(Borders::ALL)
            .border_style(TODO_HEADER_STYLE)
            .style(Style::default().bg(NORMAL_ROW_BG))
            .padding(Padding::horizontal(1));

        let help_text = vec![
            "Navigation:",
            "  ↑/↓ - Move selection",
            "  g/G - Go to top/bottom",
            "  Space - Toggle selection",
            "  Shift+Space - Toggle status",
            "  Enter - Toggle status",
            "",
            "Commands:",
            "  : - Enter command mode",
            "  add <task> - Add new task",
            "  delete <n> - Delete task n",
            "  complete <n> - Complete task n",
            "  archive <n> - Archive task n",
            "  unarchive <n> - Unarchive task n",
            "  archiveall - Archive all completed",
            "  view active/archived - Switch view",
            "  toggle - Toggle between views",
            "",
            "Other:",
            "  n - Create new task",
            "  e - Edit selected task",
            "  Ctrl+D - Delete selected task",
            "  q - Quit",
            "  Esc - Cancel/exit mode",
        ].join("\n");

        Paragraph::new(help_text)
            .block(block)
            .style(Style::default().fg(TEXT_FG_COLOR))
            .wrap(Wrap { trim: false })
            .render(area, buf);
    }
}
