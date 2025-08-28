use crate::cli::types::ConfigAction;
use crate::config::Config;
use anyhow::Result;
use colored::Colorize;

/// Handles configuration actions (set endpoint, set key, show config)
///
/// # Errors
///
/// Returns an error if:
/// - Configuration cannot be loaded or saved
/// - File I/O operations fail
/// - Configuration format is invalid
#[allow(clippy::unused_async)] // Function is async to match CLI command pattern
pub async fn handle(action: ConfigAction) -> Result<()> {
    match action {
        ConfigAction::Endpoint { url } => set_endpoint(&url),
        ConfigAction::Key { key } => set_key(key),
        ConfigAction::Show => show_config(),
    }
}

fn set_endpoint(url: &str) -> Result<()> {
    let mut config = Config::load()?;
    config.set_endpoint(url);
    config.save()?;

    println!("{} API endpoint set to: {}", "✓".green(), url.cyan());
    Ok(())
}

fn set_key(key: String) -> Result<()> {
    let mut config = Config::load()?;
    config.set_api_key(key);
    config.save()?;

    println!("{} API key configured successfully", "✓".green());
    println!(
        "{} API key is stored in plain text at: {}",
        "⚠".yellow(),
        Config::config_path()?.display().to_string().dimmed()
    );
    Ok(())
}

fn show_config() -> Result<()> {
    let config = Config::load()?;

    println!("{}", "Current Configuration:".bold());
    println!("  {} {}", "Endpoint:".cyan(), config.api_endpoint);
    println!(
        "  {} {}",
        "API Key:".cyan(),
        if config.api_key.is_some() {
            "[configured]".green().to_string()
        } else {
            "[not set]".yellow().to_string()
        }
    );

    if let Ok(path) = Config::config_path() {
        println!("  {} {}", "Config file:".cyan(), path.display());
    }

    if config.api_key.is_some() {
        println!();
        println!(
            "{} API key is stored in plain text in the config file",
            "⚠".yellow()
        );
    }

    Ok(())
}
