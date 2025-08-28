use crate::cli::types::AdminAction;
use crate::{api::ApiClient, config::Config, ID_DISPLAY_LENGTH};
use anyhow::Result;
use chrono::TimeZone;
use colored::Colorize;

/// Handles admin actions (key rotation, generation, listing, revocation)
///
/// # Errors
///
/// Returns an error if:
/// - Network request fails
/// - API key is invalid or lacks admin privileges
/// - Server returns an error response
/// - Configuration cannot be saved (for key operations)
pub async fn handle(action: AdminAction) -> Result<()> {
    match action {
        AdminAction::RotateKey => rotate_key().await,
        AdminAction::GenerateKey { name } => generate_key(name).await,
        AdminAction::ListKeys => list_keys().await,
        AdminAction::RevokeKey { id } => revoke_key(id).await,
        AdminAction::Reinitialize => reinitialize().await,
    }
}

async fn rotate_key() -> Result<()> {
    let client = ApiClient::new()?;
    let new_key = client.rotate_admin_key().await?;

    let mut config = Config::load()?;
    config.set_api_key(&new_key);
    config.save()?;

    println!("{} Admin key rotated successfully", "✓".green());
    println!("{} New key has been saved to config", "✓".green());
    println!();
    println!("{} {}", "New API Key:".yellow().bold(), new_key.cyan());
    println!();
    println!(
        "{} Store this key securely - it won't be shown again!",
        "⚠".yellow()
    );
    println!(
        "{} API key is stored in plain text at: {}",
        "⚠".yellow(),
        Config::config_path()?.display().to_string().dimmed()
    );

    Ok(())
}

async fn generate_key(name: Option<String>) -> Result<()> {
    let client = ApiClient::new()?;
    let response = client.generate_api_key(name.as_deref()).await?;

    println!("{} Generated new API key", "✓".green());

    if let Some(n) = name {
        println!("  {} {}", "Name:".cyan(), n);
    }

    println!("  {} {}", "ID:".cyan(), response.id);
    println!();
    println!("{} {}", "API Key:".yellow().bold(), response.key.cyan());
    println!();
    println!(
        "{} Store this key securely - it won't be shown again!",
        "⚠".yellow()
    );
    println!(
        "{} API keys are stored in plain text in your config file",
        "⚠".yellow()
    );

    Ok(())
}

async fn list_keys() -> Result<()> {
    let client = ApiClient::new()?;
    let keys = client.list_api_keys().await?;

    if keys.is_empty() {
        println!("{}", "No API keys found".yellow());
        return Ok(());
    }

    println!("{}", format!("Found {} API key(s):", keys.len()).bold());
    println!();

    for key in keys {
        let created_dt = chrono::Utc
            .timestamp_opt(key.created_at, 0)
            .latest()
            .map(|dt| dt.with_timezone(&chrono::Local))
            .map_or_else(
                || "Invalid date".to_string(),
                |dt| dt.format("%Y-%m-%d %H:%M:%S").to_string(),
            );

        let status = if key.active {
            "active".green()
        } else {
            "inactive".red()
        };

        let key_type_str = match key.key_type {
            pali_types::KeyType::Admin => "admin",
            pali_types::KeyType::Client => "client",
        };

        print!(
            "  {} {} - {} ({})",
            format!("[{}]", &key.id[..ID_DISPLAY_LENGTH]).cyan(),
            key.client_name.bold(),
            key_type_str.dimmed(),
            status
        );

        if let Some(last_used) = key.last_used {
            let last_used_dt = chrono::Utc
                .timestamp_opt(last_used, 0)
                .latest()
                .map(|dt| dt.with_timezone(&chrono::Local))
                .map_or_else(
                    || "Invalid date".to_string(),
                    |dt| dt.format("%Y-%m-%d %H:%M:%S").to_string(),
                );
            print!(" [Last used: {}]", last_used_dt.dimmed());
        }

        println!();
        println!("    Created: {}", created_dt.dimmed());
    }

    Ok(())
}

async fn revoke_key(id: String) -> Result<()> {
    let client = ApiClient::new()?;
    client.revoke_api_key(&id).await?;

    println!("{} Revoked API key: {}", "✓".green(), id.cyan());

    Ok(())
}

/// Initializes the Pali server with a new endpoint URL and retrieves the first admin key
///
/// # Errors
///
/// Returns an error if:
/// - Configuration cannot be saved to disk
/// - Network request to server fails
/// - Server is already initialized
/// - Server returns an error response
/// - API key cannot be saved to configuration
pub async fn initialize_with_url(url: String) -> Result<()> {
    // Create config with the provided URL first
    let mut config = Config::load().unwrap_or_default();
    config.set_endpoint(&url);
    config.save()?;

    println!("{} Set API endpoint to: {}", "✓".green(), url.cyan());

    // Now create client with the new config and initialize
    let client = ApiClient::new()?;
    let admin_key = client.initialize().await?;

    // Save the admin key to config
    config.set_api_key(&admin_key);
    config.save()?;

    println!("{} Server initialized successfully", "✓".green());
    println!(
        "{} First admin key generated and saved to config",
        "✓".green()
    );
    println!();
    println!("{} {}", "Admin Key:".yellow().bold(), admin_key.cyan());
    println!();
    println!(
        "{} Setup complete! You can now use all CLI commands.",
        "🚀".green()
    );
    println!(
        "{} API key is stored in plain text at: {}",
        "⚠".yellow(),
        Config::config_path()?.display().to_string().dimmed()
    );

    Ok(())
}

async fn reinitialize() -> Result<()> {
    let client = ApiClient::new()?;
    let admin_key = client.reinitialize().await?;

    // Save the new admin key to config
    let mut config = Config::load()?;
    config.set_api_key(&admin_key);
    config.save()?;

    println!("{} Server reinitialized successfully", "✓".green());
    println!(
        "{} ALL previous admin keys have been deactivated",
        "⚠".yellow()
    );
    println!(
        "{} New admin key generated and saved to config",
        "✓".green()
    );
    println!();
    println!("{} {}", "New Admin Key:".yellow().bold(), admin_key.cyan());
    println!();
    println!(
        "{} This emergency reset invalidated ALL previous admin keys!",
        "🚨".red()
    );
    println!(
        "{} API key is stored in plain text at: {}",
        "⚠".yellow(),
        Config::config_path()?.display().to_string().dimmed()
    );

    Ok(())
}
