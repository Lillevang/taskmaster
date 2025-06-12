pub fn render_task_list(f: &mut Frame, app: &App, area: Rect) {
    let tasks: Vec<ListItem> = app.get_visible_tasks()
        .iter()
        .map(|item| {
            let status = match item.status {
                Status::Todo => "□",
                Status::Completed => "■",
                Status::Archived => "📦",
            };
            let style = match item.status {
                Status::Todo => Style::default(),
                Status::Completed => Style::default().fg(Color::Green),
                Status::Archived => Style::default().fg(Color::Gray),
            };
            ListItem::new(format!("{} {}", status, item.task))
                .style(style)
        })
        .collect();

    let list = List::new(tasks)
        .block(Block::default().title("Tasks").borders(Borders::ALL))
        .highlight_style(Style::default().add_modifier(Modifier::BOLD))
        .highlight_symbol("> ");

    f.render_stateful_widget(list, area, &mut app.todo_list.state.clone());
}

pub fn render_command_input(f: &mut Frame, app: &App, area: Rect) {
    let input = Paragraph::new(app.command_buffer.as_str())
        .style(Style::default())
        .block(Block::default()
            .borders(Borders::ALL)
            .title("Command")
            .border_type(BorderType::Double));

    f.render_widget(input, area);
    if app.current_mode == Mode::Command {
        f.set_cursor(
            area.x + app.command_buffer.len() as u16 + 1,
            area.y + 1,
        );
    }
}

pub fn render_status_bar(f: &mut Frame, app: &App, area: Rect) {
    let mode = match app.current_mode {
        Mode::TaskList => "NORMAL",
        Mode::Command => "COMMAND",
        Mode::Edit => "EDIT",
    };

    let archive_status = if app.show_archived { "ARCHIVED" } else { "ACTIVE" };

    let status = format!("{} | {} | {} tasks", mode, archive_status, app.get_visible_tasks().len());
    let status_bar = Paragraph::new(status)
        .style(Style::default())
        .block(Block::default().borders(Borders::ALL));

    f.render_widget(status_bar, area);
} 