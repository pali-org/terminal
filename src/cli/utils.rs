//! CLI utility functions for improved user experience

use crate::api::ApiClient;
use anyhow::Result;

/// Resolves a partial ID to a full ID by matching against todos
///
/// This allows users to type just the prefix they see in the list output
/// instead of needing the full UUID.
///
/// **Implementation Strategy:**
/// 1. First tries server-side resolution (when `/todos/resolve/{prefix}` is available)
/// 2. Falls back to client-side resolution (current implementation)
///
/// # Arguments
///
/// * `partial_id` - The partial ID prefix provided by the user
/// * `client` - API client to fetch todos if needed
///
/// # Returns
///
/// * `Ok(String)` - The full UUID if exactly one match is found
/// * `Err` - If no matches found or multiple ambiguous matches
///
/// # Examples
///
/// ```ignore
/// // User sees: [d2fadfdb] My Todo
/// // User types: pacli delete d2fa
/// let full_id = resolve_partial_id("d2fa", &client).await?;
/// // Returns: "d2fadfdb-5541-4ace-9443-d01cd917a640"
/// ```
pub async fn resolve_partial_id(partial_id: &str, client: &ApiClient) -> Result<String> {
    // If it looks like a full UUID already, just return it
    if partial_id.len() >= 36 && partial_id.contains('-') {
        return Ok(partial_id.to_string());
    }

    // Try server-side resolution first (much faster!)
    if let Ok(full_id) = client.resolve_id_prefix(partial_id).await {
        return Ok(full_id);
    }

    // Fallback: Client-side resolution (if server doesn't support it)
    // Fetch all todos to find matches
    let todos = client.list_todos(None, None).await?;

    // Find all todos whose ID starts with the partial
    let matches: Vec<_> = todos
        .iter()
        .filter(|todo| todo.id.starts_with(partial_id))
        .collect();

    match matches.len() {
        0 => anyhow::bail!(
            "No todo found with ID starting with '{partial_id}'. Please check the ID and try again."
        ),
        1 => Ok(matches[0].id.clone()),
        n => {
            // Multiple matches - show them to help the user
            let mut error_msg = format!(
                "Ambiguous ID '{partial_id}' matches {n} todos. Please be more specific:\n"
            );

            for (i, todo) in matches.iter().take(5).enumerate() {
                let id_preview =
                    &todo.id[..partial_id.len() + 4.min(todo.id.len() - partial_id.len())];
                error_msg.push_str(&format!("  - {id_preview} -> {}\n", todo.title));
                if i == 4 && n > 5 {
                    error_msg.push_str(&format!("  ... and {remaining} more\n", remaining = n - 5));
                    break;
                }
            }

            anyhow::bail!(error_msg)
        }
    }
}

/// Resolves multiple partial IDs to full IDs
///
/// Useful for bulk operations where user provides multiple partial IDs.
pub async fn resolve_partial_ids(
    partial_ids: &[String],
    client: &ApiClient,
) -> Result<Vec<String>> {
    let mut resolved = Vec::new();

    for partial in partial_ids {
        resolved.push(resolve_partial_id(partial, client).await?);
    }

    Ok(resolved)
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_full_uuid_detection() {
        let full_uuid = "d2fadfdb-5541-4ace-9443-d01cd917a640";
        assert!(full_uuid.len() >= 36 && full_uuid.contains('-'));

        let partial = "d2fadfdb";
        assert!(!(partial.len() >= 36 && partial.contains('-')));
    }
}
