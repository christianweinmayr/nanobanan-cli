use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Frame,
};

use super::app::{App, AppMode, SettingsField};

/// Main draw function
pub fn draw(frame: &mut Frame, app: &App) {
    match app.mode {
        AppMode::Main | AppMode::Input => draw_main(frame, app),
        AppMode::JobDetail => draw_job_detail(frame, app),
        AppMode::Settings => draw_settings(frame, app),
    }
}

/// Draw main view with job list
fn draw_main(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Title/input
            Constraint::Min(10),    // Job list
            Constraint::Length(3),  // Status bar
            Constraint::Length(2),  // Help line
        ])
        .split(frame.area());

    // Title or input
    if app.mode == AppMode::Input {
        draw_input(frame, app, chunks[0]);
    } else {
        draw_title(frame, chunks[0]);
    }

    // Job list
    draw_job_list(frame, app, chunks[1]);

    // Status bar
    draw_status(frame, app, chunks[2]);

    // Help line
    draw_help(frame, app, chunks[3]);
}

fn draw_title(frame: &mut Frame, area: Rect) {
    let title = Paragraph::new(vec![Line::from(vec![
        Span::styled("🍌 ", Style::default()),
        Span::styled(
            "Nano Banana Pro",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" - Gemini Image Generation", Style::default().fg(Color::Gray)),
    ])])
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow)),
    );
    frame.render_widget(title, area);
}

fn draw_input(frame: &mut Frame, app: &App, area: Rect) {
    let input = Paragraph::new(app.input.as_str())
        .style(Style::default().fg(Color::White))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan))
                .title("Enter prompt (Enter to generate, Esc to cancel)"),
        );
    frame.render_widget(input, area);

    // Show cursor
    frame.set_cursor_position((
        area.x + app.cursor_pos as u16 + 1,
        area.y + 1,
    ));
}

fn draw_job_list(frame: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = app
        .jobs
        .iter()
        .enumerate()
        .map(|(i, job)| {
            let status_style = match job.status_name() {
                "completed" => Style::default().fg(Color::Green),
                "failed" => Style::default().fg(Color::Red),
                "running" => Style::default().fg(Color::Yellow),
                "queued" => Style::default().fg(Color::Blue),
                _ => Style::default().fg(Color::Gray),
            };

            let content = Line::from(vec![
                Span::styled(
                    format!("{:<12}", job.id),
                    if i == app.selected_job {
                        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::White)
                    },
                ),
                Span::raw(" "),
                Span::styled(format!("{:<10}", job.status_name()), status_style),
                Span::raw(" "),
                Span::styled(
                    job.prompt_preview(50),
                    Style::default().fg(Color::White),
                ),
            ]);

            ListItem::new(content)
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!("Jobs ({})", app.jobs.len())),
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        );

    frame.render_widget(list, area);
}

fn draw_status(frame: &mut Frame, app: &App, area: Rect) {
    let (message, style) = if let Some(err) = &app.error_message {
        (err.as_str(), Style::default().fg(Color::Red))
    } else if let Some(status) = &app.status_message {
        (status.as_str(), Style::default().fg(Color::Green))
    } else if app.generating {
        ("Generating...", Style::default().fg(Color::Yellow))
    } else {
        ("Ready", Style::default().fg(Color::Gray))
    };

    let status = Paragraph::new(message)
        .style(style)
        .block(Block::default().borders(Borders::ALL).title("Status"));
    frame.render_widget(status, area);
}

fn draw_help(frame: &mut Frame, app: &App, area: Rect) {
    let help_text = match app.mode {
        AppMode::Input => "Enter: Generate | Esc: Cancel",
        AppMode::Main => "i: New prompt | Enter: View | s: Settings | d: Delete | r: Refresh | q: Quit",
        _ => "",
    };

    let help = Paragraph::new(help_text)
        .style(Style::default().fg(Color::DarkGray));
    frame.render_widget(help, area);
}

/// Draw job detail view
fn draw_job_detail(frame: &mut Frame, app: &App) {
    let area = frame.area();

    let Some(job) = &app.current_job else {
        return;
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Header
            Constraint::Min(10),    // Details
            Constraint::Length(2),  // Help
        ])
        .split(area);

    // Header
    let header = Paragraph::new(vec![Line::from(vec![
        Span::styled("Job: ", Style::default().fg(Color::Gray)),
        Span::styled(&job.id, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
    ])])
    .block(Block::default().borders(Borders::ALL));
    frame.render_widget(header, chunks[0]);

    // Details
    let status_color = match job.status_name() {
        "completed" => Color::Green,
        "failed" => Color::Red,
        "running" => Color::Yellow,
        _ => Color::Gray,
    };

    let mut lines = vec![
        Line::from(vec![
            Span::styled("Status: ", Style::default().fg(Color::Gray)),
            Span::styled(job.status.to_string(), Style::default().fg(status_color)),
        ]),
        Line::from(vec![
            Span::styled("Action: ", Style::default().fg(Color::Gray)),
            Span::styled(job.action.to_string(), Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("Model: ", Style::default().fg(Color::Gray)),
            Span::styled(&job.model, Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("Created: ", Style::default().fg(Color::Gray)),
            Span::styled(
                job.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
                Style::default().fg(Color::White),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Prompt:", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled(&job.params.prompt, Style::default().fg(Color::White)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Parameters:", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled(
                format!("  Aspect Ratio: {}", job.params.aspect_ratio),
                Style::default().fg(Color::White),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                format!("  Size: {}", job.params.size),
                Style::default().fg(Color::White),
            ),
        ]),
    ];

    if !job.images.is_empty() {
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled(
                format!("Images ({}):", job.images.len()),
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            ),
        ]));
        for img in &job.images {
            let path_text = img
                .path
                .as_deref()
                .unwrap_or("(not downloaded)");
            lines.push(Line::from(vec![
                Span::styled(format!("  [{}] {}", img.index, path_text), Style::default().fg(Color::White)),
            ]));
        }
    }

    let details = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).title("Details"))
        .wrap(Wrap { trim: true });
    frame.render_widget(details, chunks[1]);

    // Help
    let help = Paragraph::new("Esc/q: Back")
        .style(Style::default().fg(Color::DarkGray));
    frame.render_widget(help, chunks[2]);
}

/// Draw settings screen
fn draw_settings(frame: &mut Frame, app: &App) {
    let area = frame.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Header
            Constraint::Min(10),    // Settings list
            Constraint::Length(3),  // Status
            Constraint::Length(2),  // Help
        ])
        .split(area);

    // Header
    let header = Paragraph::new("Settings")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(header, chunks[0]);

    // Settings list
    let fields = SettingsField::all();
    let items: Vec<ListItem> = fields
        .iter()
        .enumerate()
        .map(|(i, field)| {
            let is_selected = i == app.settings_selected;
            let value = if app.settings_editing && is_selected {
                format!("{}▏", app.settings_edit_buffer)
            } else {
                app.get_settings_value(field)
            };

            let has_options = app.get_settings_options(field).is_some();
            let hint = if has_options { " [←→]" } else { "" };

            let content = Line::from(vec![
                Span::styled(
                    format!("{:<20}", field.label()),
                    if is_selected {
                        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::White)
                    },
                ),
                Span::styled(
                    format!("{}{}", value, hint),
                    if is_selected && app.settings_editing {
                        Style::default().fg(Color::Yellow)
                    } else {
                        Style::default().fg(Color::Gray)
                    },
                ),
            ]);

            ListItem::new(content)
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL))
        .highlight_style(Style::default().bg(Color::DarkGray));
    frame.render_widget(list, chunks[1]);

    // Status
    draw_status(frame, app, chunks[2]);

    // Help
    let help_text = if app.settings_editing {
        "Enter: Save | Esc: Cancel"
    } else {
        "↑↓: Navigate | Enter/Space: Edit/Toggle | Esc/q: Back"
    };
    let help = Paragraph::new(help_text)
        .style(Style::default().fg(Color::DarkGray));
    frame.render_widget(help, chunks[3]);
}
