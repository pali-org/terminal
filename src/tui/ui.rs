//! TUI rendering and layout logic

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Frame,
};

use crate::tui::app::{App, AppScreen};
use crate::ID_DISPLAY_LENGTH;

use chrono::{Local, TimeZone, Utc};

/// Formats due date timestamp for display in TUI
fn format_due_date(due_ts: i64) -> Option<(String, Color)> {
    let due_dt = Utc.timestamp_opt(due_ts, 0).latest()?;
    let local_due = due_dt.with_timezone(&Local);
    let now = Local::now();

    let today = now.date_naive();
    let due_date = local_due.date_naive();

    if due_date == today {
        Some(("Today".to_string(), Color::Yellow))
    } else if due_date == today + chrono::Days::new(1) {
        Some(("Tomorrow".to_string(), Color::Cyan))
    } else if local_due < now {
        Some((local_due.format("%Y-%m-%d").to_string(), Color::Red))
    } else {
        Some((local_due.format("%Y-%m-%d").to_string(), Color::White))
    }
}

pub fn render(frame: &mut Frame, app: &mut App) {
    let size = frame.area();

    // Create main layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(4), // Header (with status bar)
            Constraint::Min(0),    // Main content
            Constraint::Length(3), // Footer (fixed size)
        ])
        .split(size);

    // Render header
    render_header(frame, chunks[0], app);

    // Render main content based on current screen
    match app.current_screen {
        AppScreen::TodoList => render_todo_list(frame, chunks[1], app),
        AppScreen::AddTodo => render_add_todo(frame, chunks[1], app),
        AppScreen::EditTodo => render_edit_todo(frame, chunks[1], app),
        AppScreen::Help => render_help(frame, chunks[1]),
        AppScreen::Settings => render_settings(frame, chunks[1], app),
        AppScreen::Search => render_search(frame, chunks[1], app),
        AppScreen::TodoDetail => render_todo_detail(frame, chunks[1], app),
    }

    // Render footer
    render_footer(frame, chunks[2], app);

    // Render loading overlay if needed
    if app.loading {
        render_loading_overlay(frame, size, app);
    }

    // Render toast notifications
    if app.error_message.is_some() || app.success_message.is_some() {
        render_toast_notification(frame, size, app);
    }
}

fn render_header(frame: &mut Frame, area: Rect, app: &App) {
    // Split header into title and status bar
    let header_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Length(1), // Status bar
        ])
        .split(area);

    let title_text = match app.current_screen {
        AppScreen::TodoList => {
            let completed = app.todos.iter().filter(|t| t.completed).count();
            let pending = app.todos.len() - completed;
            let filter_info = if app.show_all_todos { "all" } else { "pending" };
            let priority_filter = match app.filter_priority {
                Some(1) => " (low priority)",
                Some(2) => " (medium priority)",
                Some(3) => " (high priority)",
                _ => "",
            };
            format!(
                "Pali Todo Manager - {pending} pending, {completed} completed (showing {filter_info}{priority_filter})"
            )
        }
        AppScreen::AddTodo => "Pali Todo Manager - Add New Todo".to_string(),
        AppScreen::EditTodo => {
            if let Some(index) = app.selected_todo {
                if let Some(todo) = app.filtered_todos.get(index) {
                    format!(
                        "Pali Todo Manager - Edit: {}",
                        if todo.title.len() > 30 {
                            format!("{title}...", title = &todo.title[..27])
                        } else {
                            todo.title.clone()
                        }
                    )
                } else {
                    "Pali Todo Manager - Edit Todo".to_string()
                }
            } else {
                "Pali Todo Manager - Edit Todo".to_string()
            }
        }
        AppScreen::Help => "Pali Todo Manager - Help & Keyboard Shortcuts".to_string(),
        AppScreen::Settings => "Pali Todo Manager - Configuration".to_string(),
        AppScreen::Search => "Pali Todo Manager - Search Todos".to_string(),
        AppScreen::TodoDetail => "Pali Todo Manager - Todo Details".to_string(),
    };

    let title = Paragraph::new(title_text)
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(title, header_chunks[0]);

    // Render status bar
    render_status_bar(frame, header_chunks[1], app);
}

fn render_status_bar(frame: &mut Frame, area: Rect, app: &App) {
    let (status_text, status_style) = if let Some(error) = &app.error_message {
        (
            error.as_str(),
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        )
    } else if let Some(success) = &app.success_message {
        (
            success.as_str(),
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )
    } else {
        ("Ready", Style::default().fg(Color::Gray))
    };

    let status_bar = Paragraph::new(status_text).style(status_style);
    frame.render_widget(status_bar, area);
}

fn render_todo_list(frame: &mut Frame, area: Rect, app: &mut App) {
    let todos: Vec<ListItem> = app
        .filtered_todos
        .iter()
        .enumerate()
        .map(|(i, todo)| {
            let status = if todo.completed { "✓" } else { "○" };
            let id_short = if todo.id.len() > ID_DISPLAY_LENGTH {
                &todo.id[..ID_DISPLAY_LENGTH]
            } else {
                &todo.id
            };

            let priority_indicator = match todo.priority {
                1 => "!",
                2 => "!!",
                3 => "!!!",
                _ => "?",
            };

            let mut style = Style::default();
            if todo.completed {
                style = style.fg(Color::Green).add_modifier(Modifier::CROSSED_OUT);
            } else {
                style = style.fg(Color::White);
                if todo.priority == 3 {
                    style = style.fg(Color::Red).add_modifier(Modifier::BOLD);
                } else if todo.priority == 1 {
                    style = style.fg(Color::Gray);
                }
            }

            if Some(i) == app.selected_todo {
                style = style.bg(Color::Blue);
            }

            // Build the line with due date if present
            let mut line = format!(
                "{} [{}] {} {}",
                status, id_short, todo.title, priority_indicator
            );

            if let Some(due_ts) = todo.due_date {
                if let Some((due_str, due_color)) = format_due_date(due_ts) {
                    line.push_str(&format!(" [Due: {due_str}]"));
                    // Update style to show due date color if not completed
                    if !todo.completed {
                        style = match due_color {
                            Color::Red => style.fg(Color::Red),       // Overdue
                            Color::Yellow => style.fg(Color::Yellow), // Today
                            Color::Cyan => style.fg(Color::Cyan),     // Tomorrow
                            _ => style, // Keep original style for future dates
                        };
                    }
                }
            }

            ListItem::new(line).style(style)
        })
        .collect();

    let title = if app.filtered_todos.is_empty() {
        if app.todos.is_empty() {
            "📝 Welcome to Pali! Press 'n' to add your first todo"
        } else {
            "🔍 No todos match your current filters - press 'f' to toggle, '0' to clear priority filter, or '/' to search"
        }
    } else {
        "📋 Your Todos (↑↓ select, Enter toggle, d delete, e edit, n add, / search, f filter)"
    };

    // Render different UI based on whether there are todos to show
    if app.filtered_todos.is_empty() && app.todos.is_empty() {
        // First-time user empty state with helpful tips
        render_empty_state_welcome(frame, area);
    } else if app.filtered_todos.is_empty() {
        // Filtered empty state
        render_empty_state_filtered(frame, area, app);
    } else {
        // Normal todo list
        let todos_list = List::new(todos)
            .block(Block::default().title(title).borders(Borders::ALL))
            .highlight_style(Style::default().bg(Color::Blue));

        // Use app's persistent list_state instead of creating new one each time
        frame.render_stateful_widget(todos_list, area, &mut app.list_state);
    }
}

fn render_add_todo(frame: &mut Frame, area: Rect, app: &App) {
    app.input_form.render(frame, area);
}

fn render_edit_todo(frame: &mut Frame, area: Rect, app: &App) {
    // Use the same input form but with different title
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Title
            Constraint::Min(0),    // Form
        ])
        .split(area);

    let title = Paragraph::new("Edit Todo")
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .block(Block::default().borders(Borders::TOP | Borders::LEFT | Borders::RIGHT));
    frame.render_widget(title, chunks[0]);

    // Render the input form in the remaining space
    let form_area = Rect {
        x: chunks[1].x,
        y: chunks[1].y,
        width: chunks[1].width,
        height: chunks[1].height + 1, // Extend to connect with title border
    };
    app.input_form.render(frame, form_area);
}

fn render_help(frame: &mut Frame, area: Rect) {
    let help_text = vec![
        Line::from(vec![Span::styled(
            "Pali TUI Help",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Navigation:",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from("  ↑/k        - Move up"),
        Line::from("  ↓/j        - Move down"),
        Line::from("  q/Esc      - Quit"),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Todo Management:",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from("  n/a        - Add new todo"),
        Line::from("  e          - Edit selected todo"),
        Line::from("  Enter/Space- Toggle completion"),
        Line::from("  d          - Delete selected todo"),
        Line::from("  v          - View todo details"),
        Line::from("  r          - Refresh todo list"),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Search & Filtering:",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from("  /          - Search todos"),
        Line::from("  f          - Toggle show all/pending"),
        Line::from("  1/2/3      - Filter by priority"),
        Line::from("  0          - Clear priority filter"),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Other:",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from("  h/?        - Show this help"),
        Line::from("  s          - Settings"),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Priority Indicators:",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::raw("  "),
            Span::styled("!", Style::default().fg(Color::Gray)),
            Span::raw("   - Low priority"),
        ]),
        Line::from(vec![
            Span::raw("  "),
            Span::styled("!!", Style::default().fg(Color::White)),
            Span::raw("  - Medium priority"),
        ]),
        Line::from(vec![
            Span::raw("  "),
            Span::styled(
                "!!!",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ),
            Span::raw(" - High priority"),
        ]),
    ];

    let help = Paragraph::new(help_text)
        .block(Block::default().title("Help").borders(Borders::ALL))
        .wrap(Wrap { trim: true });

    frame.render_widget(help, area);
}

fn render_settings(frame: &mut Frame, area: Rect, app: &App) {
    let key_status = if app.config.api_key.is_some() {
        (
            Span::styled("✓ Configured", Style::default().fg(Color::Green)),
            Color::Green,
        )
    } else {
        (
            Span::styled("✗ Not set", Style::default().fg(Color::Red)),
            Color::Red,
        )
    };

    let settings_text = vec![
        Line::from(vec![Span::styled(
            "Current Configuration",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from(vec![
            Span::styled("API Endpoint: ", Style::default().fg(Color::Yellow)),
            Span::styled(&app.config.api_endpoint, Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("API Key: ", Style::default().fg(Color::Yellow)),
            key_status.0,
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Configuration File: ", Style::default().fg(Color::Yellow)),
            Span::styled(
                "~/.config/pali/config.json",
                Style::default().fg(Color::Gray),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "💡 Tip: ",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("Use 'pacli config' to modify settings from the command line"),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Press ", Style::default().fg(Color::Gray)),
            Span::styled("Esc", Style::default().fg(Color::Yellow)),
            Span::styled(" to return to todo list", Style::default().fg(Color::Gray)),
        ]),
    ];

    let settings = Paragraph::new(settings_text)
        .block(
            Block::default()
                .title("Configuration")
                .borders(Borders::ALL),
        )
        .wrap(Wrap { trim: true });

    frame.render_widget(settings, area);
}

fn render_search(frame: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Search input
            Constraint::Min(0),    // Instructions
        ])
        .split(area);

    // Search input field
    let search_input = Paragraph::new(app.search_query.as_str())
        .style(Style::default().fg(Color::Yellow))
        .block(Block::default().title("Search Todos").borders(Borders::ALL));
    frame.render_widget(search_input, chunks[0]);

    // Instructions
    let instructions_text = vec![
        Line::from(vec![Span::styled(
            "Search Tips:",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from("• Search matches both todo titles and descriptions"),
        Line::from("• Search is case-insensitive"),
        Line::from("• Empty search returns to regular todo list"),
        Line::from(""),
        Line::from(vec![
            Span::styled("Press ", Style::default().fg(Color::Gray)),
            Span::styled("Enter", Style::default().fg(Color::Yellow)),
            Span::styled(" to search or ", Style::default().fg(Color::Gray)),
            Span::styled("Esc", Style::default().fg(Color::Yellow)),
            Span::styled(" to cancel", Style::default().fg(Color::Gray)),
        ]),
    ];

    let instructions = Paragraph::new(instructions_text)
        .block(Block::default().title("Instructions").borders(Borders::ALL))
        .style(Style::default().fg(Color::Gray));
    frame.render_widget(instructions, chunks[1]);

    // Show cursor in search field
    let cursor_x = chunks[0].x + u16::try_from(app.search_query.len()).unwrap_or(0) + 1;
    frame.set_cursor_position((cursor_x, chunks[0].y + 1));
}

fn render_todo_detail(frame: &mut Frame, area: Rect, app: &App) {
    if let Some(index) = app.selected_todo {
        if let Some(todo) = app.filtered_todos.get(index) {
            // Pre-format dates to avoid lifetime issues
            let created_str = chrono::DateTime::from_timestamp(todo.created_at, 0)
                .map(|dt| {
                    dt.with_timezone(&chrono::Local)
                        .format("%Y-%m-%d %H:%M:%S")
                        .to_string()
                })
                .unwrap_or_else(|| "Invalid date".to_string());

            let updated_str = chrono::DateTime::from_timestamp(todo.updated_at, 0)
                .map(|dt| {
                    dt.with_timezone(&chrono::Local)
                        .format("%Y-%m-%d %H:%M:%S")
                        .to_string()
                })
                .unwrap_or_else(|| "Invalid date".to_string());

            let due_date_str = if let Some(due_ts) = todo.due_date {
                chrono::DateTime::from_timestamp(due_ts, 0)
                    .map(|dt| {
                        dt.with_timezone(&chrono::Local)
                            .format("%Y-%m-%d %H:%M:%S")
                            .to_string()
                    })
                    .unwrap_or_else(|| "Invalid date".to_string())
            } else {
                "Not set".to_string()
            };

            let due_date_color = if let Some(due_ts) = todo.due_date {
                format_due_date(due_ts)
                    .map(|(_, color)| color)
                    .unwrap_or(Color::White)
            } else {
                Color::Gray
            };

            let detail_text = vec![
                Line::from(vec![Span::styled(
                    "Todo Details",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                )]),
                Line::from(""),
                Line::from(vec![
                    Span::styled("ID: ", Style::default().fg(Color::Yellow)),
                    Span::styled(&todo.id, Style::default().fg(Color::Gray)),
                ]),
                Line::from(vec![
                    Span::styled("Title: ", Style::default().fg(Color::Yellow)),
                    Span::styled(
                        &todo.title,
                        Style::default()
                            .fg(Color::White)
                            .add_modifier(Modifier::BOLD),
                    ),
                ]),
                Line::from(""),
                Line::from(vec![Span::styled(
                    "Description:",
                    Style::default().fg(Color::Yellow),
                )]),
                Line::from(match &todo.description {
                    Some(desc) => desc.as_str(),
                    None => "(no description)",
                }),
                Line::from(""),
                Line::from(vec![
                    Span::styled("Status: ", Style::default().fg(Color::Yellow)),
                    Span::styled(
                        if todo.completed {
                            "Completed"
                        } else {
                            "Pending"
                        },
                        if todo.completed {
                            Style::default()
                                .fg(Color::Green)
                                .add_modifier(Modifier::BOLD)
                        } else {
                            Style::default().fg(Color::Yellow)
                        },
                    ),
                ]),
                Line::from(vec![
                    Span::styled("Priority: ", Style::default().fg(Color::Yellow)),
                    Span::styled(
                        match todo.priority {
                            1 => "Low (!)",
                            2 => "Medium (!!)",
                            3 => "High (!!!)",
                            _ => "Unknown (?)",
                        },
                        match todo.priority {
                            1 => Style::default().fg(Color::Gray),
                            2 => Style::default().fg(Color::White),
                            3 => Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                            _ => Style::default().fg(Color::Gray),
                        },
                    ),
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::styled("Due Date: ", Style::default().fg(Color::Yellow)),
                    Span::styled(
                        &due_date_str,
                        Style::default().fg(due_date_color).add_modifier(
                            if due_date_color == Color::Red {
                                Modifier::BOLD
                            } else {
                                Modifier::empty()
                            },
                        ),
                    ),
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::styled("Created: ", Style::default().fg(Color::Yellow)),
                    Span::styled(&created_str, Style::default().fg(Color::Gray)),
                ]),
                Line::from(vec![
                    Span::styled("Updated: ", Style::default().fg(Color::Yellow)),
                    Span::styled(&updated_str, Style::default().fg(Color::Gray)),
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::styled("Press ", Style::default().fg(Color::Gray)),
                    Span::styled("Esc", Style::default().fg(Color::Yellow)),
                    Span::styled(" to return to todo list", Style::default().fg(Color::Gray)),
                ]),
            ];

            let detail = Paragraph::new(detail_text)
                .block(Block::default().title("Todo Details").borders(Borders::ALL))
                .wrap(Wrap { trim: true });

            frame.render_widget(detail, area);
        }
    }
}

fn render_footer(frame: &mut Frame, area: Rect, app: &App) {
    // Footer only shows help text now - messages moved to header status bar

    // Render help text based on current screen
    let help_text = match app.current_screen {
        AppScreen::TodoList => vec![
            Span::styled("q", Style::default().fg(Color::Yellow)),
            Span::raw(" quit │ "),
            Span::styled("n", Style::default().fg(Color::Yellow)),
            Span::raw(" add │ "),
            Span::styled("e", Style::default().fg(Color::Yellow)),
            Span::raw(" edit │ "),
            Span::styled("/", Style::default().fg(Color::Yellow)),
            Span::raw(" search │ "),
            Span::styled("f", Style::default().fg(Color::Yellow)),
            Span::raw(" filter │ "),
            Span::styled("?", Style::default().fg(Color::Yellow)),
            Span::raw(" help"),
        ],
        AppScreen::AddTodo => vec![
            Span::styled("Tab/↓", Style::default().fg(Color::Yellow)),
            Span::raw(" next │ "),
            Span::styled("Shift+Tab/↑", Style::default().fg(Color::Yellow)),
            Span::raw(" prev │ "),
            Span::styled("Enter", Style::default().fg(Color::Yellow)),
            Span::raw(" create │ "),
            Span::styled("Esc", Style::default().fg(Color::Yellow)),
            Span::raw(" cancel"),
        ],
        AppScreen::EditTodo => vec![
            Span::styled("Tab/↓", Style::default().fg(Color::Yellow)),
            Span::raw(" next │ "),
            Span::styled("Shift+Tab/↑", Style::default().fg(Color::Yellow)),
            Span::raw(" prev │ "),
            Span::styled("Enter", Style::default().fg(Color::Yellow)),
            Span::raw(" save │ "),
            Span::styled("Esc", Style::default().fg(Color::Yellow)),
            Span::raw(" cancel"),
        ],
        AppScreen::Help | AppScreen::Settings | AppScreen::TodoDetail => vec![
            Span::styled("Esc", Style::default().fg(Color::Yellow)),
            Span::raw(" back to todos │ "),
            Span::styled("q", Style::default().fg(Color::Yellow)),
            Span::raw(" quit"),
        ],
        AppScreen::Search => vec![
            Span::styled("Enter", Style::default().fg(Color::Yellow)),
            Span::raw(" search │ "),
            Span::styled("Esc", Style::default().fg(Color::Yellow)),
            Span::raw(" cancel"),
        ],
    };

    let help = Paragraph::new(Line::from(help_text))
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default().fg(Color::Gray));
    frame.render_widget(help, area);
}

fn render_loading_overlay(frame: &mut Frame, area: Rect, app: &App) {
    let spinner_chars = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
    let spinner_char = spinner_chars[app.loading_spinner_state % spinner_chars.len()];

    let loading_text = vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            format!("{spinner_char} Loading..."),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Please wait...",
            Style::default().fg(Color::Gray),
        )]),
    ];

    let loading_dialog = Paragraph::new(loading_text)
        .block(
            Block::default()
                .title(" Processing ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .wrap(Wrap { trim: true });

    // Calculate center position for loading dialog
    let popup_area = centered_rect(40, 25, area);

    frame.render_widget(Clear, popup_area);
    frame.render_widget(loading_dialog, popup_area);
}

fn render_toast_notification(frame: &mut Frame, area: Rect, app: &App) {
    let (message, icon, color) = if let Some(error) = &app.error_message {
        (error.as_str(), "❌", Color::Red)
    } else if let Some(success) = &app.success_message {
        (success.as_str(), "✅", Color::Green)
    } else {
        return;
    };

    // Simple single line toast - no padding, no borders, minimal intrusion
    let toast_text = Line::from(vec![Span::styled(
        format!("{icon} {message}"),
        Style::default().fg(color).add_modifier(Modifier::BOLD),
    )]);

    // Position toast at top-right corner, single line only
    let toast_width = (message.len() + 3).min(area.width.saturating_sub(2) as usize) as u16;
    let popup_area = Rect {
        x: area.width.saturating_sub(toast_width + 2),
        y: 1,
        width: toast_width + 2,
        height: 1,
    };

    // Render directly without borders or background clearing
    let toast_paragraph = Paragraph::new(toast_text);
    frame.render_widget(toast_paragraph, popup_area);
}

fn render_empty_state_welcome(frame: &mut Frame, area: Rect) {
    let welcome_text = vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            "🎉 Welcome to Pali Todo Manager!",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from("Get started with your first todo:"),
        Line::from(""),
        Line::from(vec![
            Span::styled("Press ", Style::default().fg(Color::Gray)),
            Span::styled(
                "n",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" to add a new todo", Style::default().fg(Color::Gray)),
        ]),
        Line::from(vec![
            Span::styled("Press ", Style::default().fg(Color::Gray)),
            Span::styled(
                "?",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                " for help and keyboard shortcuts",
                Style::default().fg(Color::Gray),
            ),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "💡 Pro tips:",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from("• Use priorities: ! (low), !! (medium), !!! (high)"),
        Line::from("• Set due dates for better organization"),
        Line::from("• Use search (/) to quickly find todos"),
        Line::from("• Filter by priority (1/2/3) or status (f)"),
    ];

    let welcome_widget = Paragraph::new(welcome_text)
        .block(
            Block::default()
                .title("📝 Welcome to Pali!")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .style(Style::default().fg(Color::White))
        .wrap(Wrap { trim: true });

    frame.render_widget(welcome_widget, area);
}

fn render_empty_state_filtered(frame: &mut Frame, area: Rect, app: &App) {
    let mut filter_info = Vec::new();

    if !app.show_all_todos {
        filter_info.push("• Showing pending todos only".to_string());
    } else {
        filter_info.push("• Showing all todos".to_string());
    }

    if let Some(priority) = app.filter_priority {
        filter_info.push(format!("• Priority filter: {priority}"));
    }

    if !app.search_query.is_empty() {
        filter_info.push(format!("• Search query: '{}'", app.search_query));
    }

    let mut filtered_text = vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            "🔍 No todos match your current filters",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Active filters:",
            Style::default().fg(Color::Gray),
        )]),
    ];

    for info in &filter_info {
        filtered_text.push(Line::from(info.as_str()));
    }

    filtered_text.extend(vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            "Try:",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::styled(
                "f",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                " - Toggle showing all/pending todos",
                Style::default().fg(Color::Gray),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                "0",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" - Clear priority filter", Style::default().fg(Color::Gray)),
        ]),
        Line::from(vec![
            Span::styled(
                "/",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" - Search all todos", Style::default().fg(Color::Gray)),
        ]),
        Line::from(vec![
            Span::styled(
                "r",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" - Refresh todo list", Style::default().fg(Color::Gray)),
        ]),
    ]);

    let filtered_widget = Paragraph::new(filtered_text)
        .block(
            Block::default()
                .title("🔍 No Matching Todos")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow)),
        )
        .style(Style::default().fg(Color::White))
        .wrap(Wrap { trim: true });

    frame.render_widget(filtered_widget, area);
}

// Helper function to center a rectangle within another rectangle
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
