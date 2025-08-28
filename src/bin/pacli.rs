//! Pali CLI - Command-line interface for Pali todo management

#[cfg(not(feature = "cli"))]
compile_error!("The 'cli' feature must be enabled to build pacli");

use anyhow::Result;
use clap::Parser;
use pali_terminal::{
    cli::{
        commands,
        types::{Cli, Commands},
    },
    init_logging,
};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Handle version flag
    if cli.version {
        println!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    // Initialize logging based on verbosity level
    init_logging(cli.verbose)?;

    // Require a command if no version flag
    let Some(command) = cli.command else {
        anyhow::bail!("A command is required. Use --help for usage information.");
    };

    match command {
        Commands::Config { action } => {
            commands::config::handle(action).await?;
        }
        Commands::Add {
            title,
            description,
            due,
            priority,
            tags,
        } => {
            commands::todo::add(title, description, due, priority, tags).await?;
        }
        Commands::List { all, tag, priority } => {
            commands::todo::list(all, tag, priority).await?;
        }
        Commands::Get { id } => {
            commands::todo::get(id).await?;
        }
        Commands::Update {
            id,
            title,
            description,
            due,
            priority,
            tags,
        } => {
            commands::todo::update(id, title, description, due, priority, tags).await?;
        }
        Commands::Delete { id } => {
            commands::todo::delete(id).await?;
        }
        Commands::Toggle { id } => {
            commands::todo::toggle(id).await?;
        }
        Commands::Complete { id } => {
            commands::todo::complete(id).await?;
        }
        Commands::Search { query } => {
            commands::todo::search(query).await?;
        }
        Commands::Init { url } => {
            commands::admin::initialize_with_url(url).await?;
        }
        Commands::Admin { action } => {
            commands::admin::handle(action).await?;
        }
    }

    Ok(())
}
