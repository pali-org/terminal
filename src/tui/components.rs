//! TUI reusable components

use crate::ID_DISPLAY_LENGTH;
use pali_types::Todo;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};

/// Stateful todo list component
pub struct TodoListWidget {
    pub todos: Vec<Todo>,
    pub state: ListState,
}

impl TodoListWidget {
    #[must_use]
    pub fn new(todos: Vec<Todo>) -> Self {
        let mut state = ListState::default();
        if !todos.is_empty() {
            state.select(Some(0));
        }
        Self { todos, state }
    }

    pub fn next(&mut self) {
        if !self.todos.is_empty() {
            let i = match self.state.selected() {
                Some(i) => {
                    if i >= self.todos.len() - 1 {
                        0
                    } else {
                        i + 1
                    }
                }
                None => 0,
            };
            self.state.select(Some(i));
        }
    }

    pub fn previous(&mut self) {
        if !self.todos.is_empty() {
            let i = match self.state.selected() {
                Some(i) => {
                    if i == 0 {
                        self.todos.len() - 1
                    } else {
                        i - 1
                    }
                }
                None => 0,
            };
            self.state.select(Some(i));
        }
    }

    #[must_use]
    pub fn selected_todo(&self) -> Option<&Todo> {
        if let Some(i) = self.state.selected() {
            self.todos.get(i)
        } else {
            None
        }
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        let items: Vec<ListItem> = self
            .todos
            .iter()
            .map(|todo| {
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

                let line = format!(
                    "{} [{}] {} {}",
                    status, id_short, todo.title, priority_indicator
                );
                ListItem::new(line).style(style)
            })
            .collect();

        let title = if self.todos.is_empty() {
            "No todos yet - press 'n' to add one!"
        } else {
            "Todos (↑↓ to select, Enter to toggle, d to delete, n to add)"
        };

        let list = List::new(items)
            .block(Block::default().title(title).borders(Borders::ALL))
            .highlight_style(Style::default().bg(Color::Blue));

        frame.render_stateful_widget(list, area, &mut self.state);
    }
}

/// Input form component for adding/editing todos
pub struct InputForm {
    pub title: String,
    pub description: String,
    pub priority: i32,
    pub due_date: String, // Format: YYYY-MM-DD or YYYY-MM-DD HH:MM:SS
    pub current_field: InputField,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputField {
    Title,
    Description,
    Priority,
    DueDate,
}

impl InputForm {
    #[must_use]
    pub fn new() -> Self {
        Self {
            title: String::new(),
            description: String::new(),
            priority: 2, // Default to medium priority
            due_date: String::new(),
            current_field: InputField::Title,
        }
    }

    pub fn next_field(&mut self) {
        self.current_field = match self.current_field {
            InputField::Title => InputField::Description,
            InputField::Description => InputField::Priority,
            InputField::Priority => InputField::DueDate,
            InputField::DueDate => InputField::Title,
        };
    }

    pub fn previous_field(&mut self) {
        self.current_field = match self.current_field {
            InputField::Title => InputField::DueDate,
            InputField::Description => InputField::Title,
            InputField::Priority => InputField::Description,
            InputField::DueDate => InputField::Priority,
        };
    }

    pub fn handle_char(&mut self, c: char) {
        match self.current_field {
            InputField::Title => self.title.push(c),
            InputField::Description => self.description.push(c),
            InputField::Priority => {
                if let Some(digit) = c.to_digit(10) {
                    if (1..=3).contains(&digit) {
                        // Safe cast: digit is guaranteed to be 1, 2, or 3
                        #[allow(clippy::cast_possible_wrap)]
                        {
                            self.priority = digit as i32;
                        }
                    }
                }
            }
            InputField::DueDate => {
                // Allow digits, dashes, colons, and spaces for date/time input
                if c.is_ascii_digit() || c == '-' || c == ':' || c == ' ' {
                    self.due_date.push(c);
                }
            }
        }
    }

    pub fn handle_backspace(&mut self) {
        match self.current_field {
            InputField::Title => {
                self.title.pop();
            }
            InputField::Description => {
                self.description.pop();
            }
            InputField::Priority => {} // Priority doesn't support backspace
            InputField::DueDate => {
                self.due_date.pop();
            }
        }
    }

    pub fn clear(&mut self) {
        self.title.clear();
        self.description.clear();
        self.priority = 2;
        self.due_date.clear();
        self.current_field = InputField::Title;
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Title
                Constraint::Length(3), // Description
                Constraint::Length(3), // Priority
                Constraint::Length(3), // Due Date
                Constraint::Min(0),    // Instructions
            ])
            .split(area);

        // Title field
        let title_style = if self.current_field == InputField::Title {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::White)
        };
        let title_widget = Paragraph::new(self.title.as_str())
            .style(title_style)
            .block(Block::default().title("Title *").borders(Borders::ALL));
        frame.render_widget(title_widget, chunks[0]);

        // Description field
        let desc_style = if self.current_field == InputField::Description {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::White)
        };
        let desc_widget = Paragraph::new(self.description.as_str())
            .style(desc_style)
            .block(
                Block::default()
                    .title("Description (optional)")
                    .borders(Borders::ALL),
            );
        frame.render_widget(desc_widget, chunks[1]);

        // Priority field
        let priority_text = match self.priority {
            1 => "1 - Low",
            3 => "3 - High",
            _ => "2 - Medium", // Default for 2 or any invalid value
        };
        let priority_style = if self.current_field == InputField::Priority {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::White)
        };
        let priority_widget = Paragraph::new(priority_text).style(priority_style).block(
            Block::default()
                .title("Priority (1-3)")
                .borders(Borders::ALL),
        );
        frame.render_widget(priority_widget, chunks[2]);

        // Due date field
        let due_style = if self.current_field == InputField::DueDate {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::White)
        };
        let due_widget = Paragraph::new(self.due_date.as_str())
            .style(due_style)
            .block(
                Block::default()
                    .title("Due Date (YYYY-MM-DD or YYYY-MM-DD HH:MM:SS, optional)")
                    .borders(Borders::ALL),
            );
        frame.render_widget(due_widget, chunks[3]);

        // Instructions
        let instructions = vec![
            Line::from(vec![
                Span::styled("Tab", Style::default().fg(Color::Yellow)),
                Span::raw(" - Next field  "),
                Span::styled("Enter", Style::default().fg(Color::Yellow)),
                Span::raw(" - Save  "),
                Span::styled("Esc", Style::default().fg(Color::Yellow)),
                Span::raw(" - Cancel"),
            ]),
            Line::from("Title is required. Use 1-3 for priority."),
            Line::from("Due date examples: 2024-03-15 or 2024-03-15 14:30:00"),
        ];
        let instructions_widget = Paragraph::new(instructions)
            .block(Block::default().title("Instructions").borders(Borders::ALL))
            .style(Style::default().fg(Color::Gray));
        frame.render_widget(instructions_widget, chunks[4]);

        // Show cursor for current field
        match self.current_field {
            InputField::Title => {
                let cursor_x = chunks[0].x
                    + u16::try_from(self.title.len())
                        .unwrap_or(u16::MAX.saturating_sub(chunks[0].x + 2))
                    + 1;
                frame.set_cursor_position((cursor_x, chunks[0].y + 1));
            }
            InputField::Description => {
                let cursor_x = chunks[1].x
                    + u16::try_from(self.description.len())
                        .unwrap_or(u16::MAX.saturating_sub(chunks[1].x + 2))
                    + 1;
                frame.set_cursor_position((cursor_x, chunks[1].y + 1));
            }
            InputField::Priority => {
                let cursor_x = chunks[2].x
                    + u16::try_from(priority_text.len())
                        .unwrap_or(u16::MAX.saturating_sub(chunks[2].x + 2))
                    + 1;
                frame.set_cursor_position((cursor_x, chunks[2].y + 1));
            }
            InputField::DueDate => {
                let cursor_x = chunks[3].x
                    + u16::try_from(self.due_date.len())
                        .unwrap_or(u16::MAX.saturating_sub(chunks[3].x + 2))
                    + 1;
                frame.set_cursor_position((cursor_x, chunks[3].y + 1));
            }
        }
    }

    #[must_use]
    pub fn is_valid(&self) -> bool {
        !self.title.trim().is_empty()
    }

    /// Validates and parses the due date input
    ///
    /// Returns Ok(Some(timestamp)) for valid dates, Ok(None) for empty input,
    /// or Err for invalid format
    pub fn parse_due_date(&self) -> Result<Option<i64>, String> {
        if self.due_date.trim().is_empty() {
            return Ok(None);
        }

        use chrono::NaiveDateTime;

        let date_str = self.due_date.trim();

        // Try parsing as datetime first
        if let Ok(dt) = NaiveDateTime::parse_from_str(date_str, "%Y-%m-%d %H:%M:%S") {
            return Ok(Some(dt.and_utc().timestamp()));
        }

        // Try parsing as date only
        if let Ok(dt) =
            NaiveDateTime::parse_from_str(&format!("{date_str} 00:00:00"), "%Y-%m-%d %H:%M:%S")
        {
            return Ok(Some(dt.and_utc().timestamp()));
        }

        Err("Invalid date format. Use YYYY-MM-DD or YYYY-MM-DD HH:MM:SS".to_string())
    }

    pub fn to_create_request(&self) -> Result<pali_types::CreateTodoRequest, String> {
        let mut request = pali_types::CreateTodoRequest::new(self.title.trim());

        if !self.description.trim().is_empty() {
            request = request.with_description(self.description.trim());
        }

        // Parse and validate due date
        let due_timestamp = self.parse_due_date()?;
        if let Some(timestamp) = due_timestamp {
            request = request.with_due_date(timestamp);
        }

        Ok(request.with_priority(self.priority))
    }
}

impl Default for InputForm {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_todo_list_widget() {
        let todos = vec![
            Todo {
                id: "test1".to_string(),
                title: "Test Todo 1".to_string(),
                description: None,
                completed: false,
                priority: 2,
                due_date: None,
                created_at: 1640995200,
                updated_at: 1640995200,
            },
            Todo {
                id: "test2".to_string(),
                title: "Test Todo 2".to_string(),
                description: None,
                completed: true,
                priority: 1,
                due_date: None,
                created_at: 1640995200,
                updated_at: 1640995200,
            },
        ];

        let mut widget = TodoListWidget::new(todos);
        assert_eq!(widget.state.selected(), Some(0));

        widget.next();
        assert_eq!(widget.state.selected(), Some(1));

        widget.next();
        assert_eq!(widget.state.selected(), Some(0)); // Should wrap around

        widget.previous();
        assert_eq!(widget.state.selected(), Some(1));
    }

    #[test]
    fn test_input_form() {
        let mut form = InputForm::new();
        assert_eq!(form.current_field, InputField::Title);
        assert_eq!(form.priority, 2);

        form.handle_char('H');
        form.handle_char('i');
        assert_eq!(form.title, "Hi");

        form.next_field();
        assert_eq!(form.current_field, InputField::Description);

        form.handle_char('T');
        form.handle_char('e');
        form.handle_char('s');
        form.handle_char('t');
        assert_eq!(form.description, "Test");

        form.next_field();
        assert_eq!(form.current_field, InputField::Priority);

        form.handle_char('3');
        assert_eq!(form.priority, 3);

        form.handle_char('5'); // Invalid, should be ignored
        assert_eq!(form.priority, 3);

        assert!(form.is_valid());

        let request = form.to_create_request().unwrap();
        assert_eq!(request.title, "Hi");
        assert_eq!(request.description, Some("Test".to_string()));
        assert_eq!(request.priority, Some(3));
    }

    #[test]
    fn test_input_form_validation() {
        let mut form = InputForm::new();
        assert!(!form.is_valid()); // Empty title should be invalid

        form.handle_char(' ');
        assert!(!form.is_valid()); // Whitespace only should be invalid

        form.handle_char('T');
        assert!(form.is_valid()); // Non-empty title should be valid
    }

    #[test]
    fn test_empty_todo_list_widget() {
        let mut widget = TodoListWidget::new(vec![]);
        assert_eq!(widget.state.selected(), None);

        widget.next(); // Should not crash
        assert_eq!(widget.state.selected(), None);

        widget.previous(); // Should not crash
        assert_eq!(widget.state.selected(), None);

        assert!(widget.selected_todo().is_none());
    }
}
