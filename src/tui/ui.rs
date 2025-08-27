//! TUI rendering and layout logic

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::tui::app::App;

pub fn render(frame: &mut Frame, app: &App) {
    let size = frame.area();
    
    // Create main layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3),  // Header
            Constraint::Min(0),     // Main content
            Constraint::Length(3),  // Footer
        ])
        .split(size);

    // Render header
    render_header(frame, chunks[0]);
    
    // Render main content
    render_main_content(frame, chunks[1], app);
    
    // Render footer
    render_footer(frame, chunks[2]);
}

fn render_header(frame: &mut Frame, area: Rect) {
    let title = Paragraph::new("Pali Todo Manager")
        .style(Style::default().fg(Color::Cyan))
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(title, area);
}

fn render_main_content(frame: &mut Frame, area: Rect, _app: &App) {
    let content = Paragraph::new("Welcome to Pali TUI!\n\nThis is a placeholder implementation.\nPress 'q' or 'Esc' to quit.")
        .block(Block::default().title("Main").borders(Borders::ALL))
        .style(Style::default().fg(Color::White));
    frame.render_widget(content, area);
}

fn render_footer(frame: &mut Frame, area: Rect) {
    let help_text = vec![
        Span::raw("Press "),
        Span::styled("q", Style::default().fg(Color::Yellow)),
        Span::raw(" to quit, "),
        Span::styled("?", Style::default().fg(Color::Yellow)),
        Span::raw(" for help"),
    ];
    
    let help = Paragraph::new(Line::from(help_text))
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default().fg(Color::Gray));
    frame.render_widget(help, area);
}