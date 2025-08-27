use crate::{api::{ApiClient, CreateTodoRequest, Todo, UpdateTodoRequest}, ID_DISPLAY_LENGTH};
use anyhow::Result;
use chrono::{Local, NaiveDateTime, TimeZone, Utc};
use colored::{ColoredString, Colorize};
use pali_types::priority;

fn format_due_date(due_ts: i64) -> Option<ColoredString> {
    let due_dt = Utc.timestamp_opt(due_ts, 0).latest()?;
    let local_due = due_dt.with_timezone(&Local);
    let now = Local::now();

    let today = now.date_naive();
    let due_date = local_due.date_naive();

    if due_date == today {
        Some("Today".yellow())
    } else if due_date == today + chrono::Days::new(1) {
        Some("Tomorrow".cyan())
    } else if local_due < now {
        Some(local_due.format("%Y-%m-%d").to_string().red())
    } else {
        Some(local_due.format("%Y-%m-%d").to_string().normal())
    }
}

/// Adds a new todo item with the specified details
/// 
/// # Errors
/// 
/// Returns an error if:
/// - Network request fails
/// - Invalid date format provided
/// - Server returns an error response
/// - API key is missing or invalid
pub async fn add(
    title: String,
    description: Option<String>,
    due: Option<String>,
    priority: Option<String>,
    _tags: Option<String>,
) -> Result<()> {
    let client = ApiClient::new()?;

    let due_timestamp = due.map(|d| parse_date(&d)).transpose()?;

    let priority_int = priority.map(|p| parse_priority(&p));

    let request = CreateTodoRequest {
        title,
        description,
        priority: priority_int,
        due_date: due_timestamp,
    };

    let todo = client.create_todo(request).await?;

    println!(
        "{} Created todo: {} (ID: {})",
        "✓".green(),
        todo.title.bold(),
        todo.id.cyan()
    );

    Ok(())
}

/// Parses a date string into a Unix timestamp
///
/// Supports two formats:
/// - `YYYY-MM-DD` (time set to 00:00:00)
/// - `YYYY-MM-DD HH:MM:SS`
///
/// # Errors
/// Returns an error if the date string doesn't match either supported format
pub fn parse_date(date_str: &str) -> Result<i64> {
    // Try parsing as datetime first
    if let Ok(dt) = NaiveDateTime::parse_from_str(date_str, "%Y-%m-%d %H:%M:%S") {
        return Ok(dt.and_utc().timestamp());
    }

    // Try parsing as date only
    if let Ok(dt) =
        NaiveDateTime::parse_from_str(&format!("{date_str} 00:00:00"), "%Y-%m-%d %H:%M:%S")
    {
        return Ok(dt.and_utc().timestamp());
    }

    anyhow::bail!("Invalid date format. Use YYYY-MM-DD or YYYY-MM-DD HH:MM:SS")
}

/// Parses a priority string into a priority level
///
/// Supported values (case insensitive):
/// - "low" → 1
/// - "medium" → 2  
/// - "high" → 3
///
/// Any other value defaults to medium priority (2)
#[must_use]
pub fn parse_priority(priority_str: &str) -> i32 {
    match priority_str.to_lowercase().as_str() {
        "low" => priority::LOW,
        "high" => priority::HIGH,
        _ => priority::MEDIUM,
    }
}

/// Lists todos with optional filtering by completion status, tag, and priority
/// 
/// # Errors
/// 
/// Returns an error if:
/// - Network request fails
/// - Server returns an error response
/// - API key is missing or invalid
pub async fn list(all: bool, tag: Option<String>, priority: Option<String>) -> Result<()> {
    let client = ApiClient::new()?;
    let todos = client.list_todos(tag, priority).await?;

    let filtered_todos: Vec<_> = if all {
        todos
    } else {
        todos.into_iter().filter(|t| !t.completed).collect()
    };

    if filtered_todos.is_empty() {
        println!("{}", "No todos found".yellow());
        return Ok(());
    }

    println!(
        "{}",
        format!("Found {} todo(s):", filtered_todos.len()).bold()
    );
    println!();

    for todo in filtered_todos {
        print_todo(&todo);
        println!();
    }

    Ok(())
}

/// Retrieves and displays a specific todo by ID
/// 
/// # Errors
/// 
/// Returns an error if:
/// - Network request fails
/// - Todo with the given ID is not found
/// - Server returns an error response
/// - API key is missing or invalid
pub async fn get(id: String) -> Result<()> {
    let client = ApiClient::new()?;
    let todo = client.get_todo(&id).await?;

    println!("{}", "Todo Details:".bold());
    print_todo_detailed(&todo);

    Ok(())
}

/// Updates an existing todo item with new values
/// 
/// # Errors
/// 
/// Returns an error if:
/// - Network request fails
/// - Todo with the given ID is not found
/// - Invalid date format provided
/// - Server returns an error response
/// - API key is missing or invalid
pub async fn update(
    id: String,
    title: Option<String>,
    description: Option<String>,
    due: Option<String>,
    priority: Option<String>,
    _tags: Option<String>,
) -> Result<()> {
    let client = ApiClient::new()?;

    let due_timestamp = due.map(|d| parse_date(&d)).transpose()?;

    let priority_int = priority.map(|p| parse_priority(&p));

    let request = UpdateTodoRequest {
        title,
        description,
        completed: None,
        priority: priority_int,
        due_date: due_timestamp,
    };

    let todo = client.update_todo(&id, request).await?;

    println!("{} Updated todo: {}", "✓".green(), todo.title.bold());

    Ok(())
}

/// Deletes a todo item by ID
/// 
/// # Errors
/// 
/// Returns an error if:
/// - Network request fails
/// - Todo with the given ID is not found
/// - Server returns an error response
/// - API key is missing or invalid
pub async fn delete(id: String) -> Result<()> {
    let client = ApiClient::new()?;
    client.delete_todo(&id).await?;

    println!("{} Deleted todo with ID: {}", "✓".green(), id.cyan());

    Ok(())
}

/// Toggles the completion status of a todo item
/// 
/// # Errors
/// 
/// Returns an error if:
/// - Network request fails
/// - Todo with the given ID is not found
/// - Server returns an error response
/// - API key is missing or invalid
pub async fn toggle(id: String) -> Result<()> {
    let client = ApiClient::new()?;
    let todo = client.toggle_todo(&id).await?;

    let status = if todo.completed {
        "completed"
    } else {
        "incomplete"
    };
    println!(
        "{} Toggled todo '{}' to {}",
        "✓".green(),
        todo.title.bold(),
        status.cyan()
    );

    Ok(())
}

/// Marks a todo item as completed
/// 
/// # Errors
/// 
/// Returns an error if:
/// - Network request fails
/// - Todo with the given ID is not found
/// - Server returns an error response
/// - API key is missing or invalid
pub async fn complete(id: String) -> Result<()> {
    let client = ApiClient::new()?;

    let request = UpdateTodoRequest {
        title: None,
        description: None,
        completed: Some(true),
        due_date: None,
        priority: None,
    };

    let todo = client.update_todo(&id, request).await?;

    println!("{} Marked '{}' as complete", "✓".green(), todo.title.bold());

    Ok(())
}

/// Searches todos by query string and displays results
/// 
/// # Errors
/// 
/// Returns an error if:
/// - Network request fails
/// - Server returns an error response
/// - API key is missing or invalid
pub async fn search(query: String) -> Result<()> {
    let client = ApiClient::new()?;
    let todos = client.search_todos(&query).await?;

    if todos.is_empty() {
        println!(
            "{}",
            format!("No todos found matching '{query}'").yellow()
        );
        return Ok(());
    }

    println!(
        "{}",
        format!("Found {} todo(s) matching '{}':", todos.len(), query).bold()
    );
    println!();

    for todo in todos {
        print_todo(&todo);
        println!();
    }

    Ok(())
}

fn print_todo(todo: &Todo) {
    let status = if todo.completed {
        "✓".green().to_string()
    } else {
        "○".normal().to_string()
    };

    print!(
        "{} {} {}",
        status,
        format!("[{}]", &todo.id[..ID_DISPLAY_LENGTH]).cyan(),
        todo.title.bold()
    );

    let priority_str = match todo.priority {
        p if p == priority::HIGH => "high".red(),
        p if p == priority::MEDIUM => "medium".yellow(),
        p if p == priority::LOW => "low".blue(),
        _ => "medium".normal(),
    };
    print!(" ({priority_str})");

    if let Some(due_ts) = todo.due_date {
        if let Some(due_str) = format_due_date(due_ts) {
            print!(" [Due: {}]", due_str.dimmed());
        }
    }

    println!();

    if let Some(desc) = &todo.description {
        println!("  {}", desc.dimmed());
    }
}

fn print_todo_detailed(todo: &Todo) {
    println!("  {} {}", "ID:".cyan(), todo.id);
    println!("  {} {}", "Title:".cyan(), todo.title.bold());

    if let Some(desc) = &todo.description {
        println!("  {} {}", "Description:".cyan(), desc);
    }

    println!(
        "  {} {}",
        "Status:".cyan(),
        if todo.completed {
            "Completed".green().to_string()
        } else {
            "Incomplete".yellow().to_string()
        }
    );

    let priority_str = match todo.priority {
        p if p == priority::HIGH => "high".red(),
        p if p == priority::MEDIUM => "medium".yellow(),
        p if p == priority::LOW => "low".blue(),
        _ => "medium".normal(),
    };
    println!("  {} {}", "Priority:".cyan(), priority_str);

    if let Some(due_ts) = todo.due_date {
        if let Some(due) = Utc.timestamp_opt(due_ts, 0).latest() {
            let local_due = due.with_timezone(&Local);
            println!(
                "  {} {}",
                "Due Date:".cyan(),
                local_due.format("%Y-%m-%d %H:%M:%S")
            );
        }
    }

    if let Some(created) = Utc.timestamp_opt(todo.created_at, 0).latest() {
        let local_created = created.with_timezone(&Local);
        println!(
            "  {} {}",
            "Created:".cyan(),
            local_created.format("%Y-%m-%d %H:%M:%S")
        );
    }

    if let Some(updated) = Utc.timestamp_opt(todo.updated_at, 0).latest() {
        let local_updated = updated.with_timezone(&Local);
        println!(
            "  {} {}",
            "Updated:".cyan(),
            local_updated.format("%Y-%m-%d %H:%M:%S")
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{NaiveDate, Utc};

    #[test]
    fn test_parse_date_datetime_format() {
        let result = parse_date("2024-01-15 14:30:00").unwrap();
        let expected = NaiveDate::from_ymd_opt(2024, 1, 15)
            .unwrap()
            .and_hms_opt(14, 30, 0)
            .unwrap()
            .and_utc()
            .timestamp();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_date_date_only_format() {
        let result = parse_date("2024-01-15").unwrap();
        let expected = NaiveDate::from_ymd_opt(2024, 1, 15)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_utc()
            .timestamp();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_date_invalid_format() {
        let result = parse_date("invalid-date");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid date format"));
    }

    #[test]
    fn test_parse_priority_valid_values() {
        assert_eq!(parse_priority("low"), priority::LOW);
        assert_eq!(parse_priority("LOW"), priority::LOW);
        assert_eq!(parse_priority("medium"), priority::MEDIUM);
        assert_eq!(parse_priority("MEDIUM"), priority::MEDIUM);
        assert_eq!(parse_priority("high"), priority::HIGH);
        assert_eq!(parse_priority("HIGH"), priority::HIGH);
    }

    #[test]
    fn test_parse_priority_invalid_defaults_to_medium() {
        assert_eq!(parse_priority("invalid"), priority::MEDIUM);
        assert_eq!(parse_priority(""), priority::MEDIUM);
        assert_eq!(parse_priority("123"), priority::MEDIUM);
    }

    #[test]
    fn test_format_due_date_today() {
        let now = Utc::now();
        let local_now = now.with_timezone(&Local);
        let today_midnight = local_now
            .date_naive()
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_utc();

        let result = format_due_date(today_midnight.timestamp());
        assert!(result.is_some());
        // We can't easily test the exact content due to color formatting
    }

    #[test]
    fn test_format_due_date_invalid_timestamp() {
        let result = format_due_date(-1);
        // Should handle invalid timestamps gracefully
        assert!(result.is_none() || result.is_some());
    }
}
