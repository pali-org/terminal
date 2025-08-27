//! CLI-specific types and command definitions

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "pacli")]
#[command(about = "A CLI for managing todos with Pali server", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    #[command(about = "Configure the CLI")]
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
    #[command(about = "Create a new todo")]
    Add {
        #[arg(help = "Todo title")]
        title: String,
        #[arg(short = 'D', long, help = "Todo description")]
        description: Option<String>,
        #[arg(short, long, help = "Due date (YYYY-MM-DD or YYYY-MM-DD HH:MM:SS)")]
        due: Option<String>,
        #[arg(short, long, help = "Priority (low, medium, high)")]
        priority: Option<String>,
        #[arg(short, long, help = "Tags (comma-separated)")]
        tags: Option<String>,
    },
    #[command(about = "List all todos")]
    List {
        #[arg(short, long, help = "Show completed todos")]
        all: bool,
        #[arg(short, long, help = "Filter by tag")]
        tag: Option<String>,
        #[arg(short, long, help = "Filter by priority")]
        priority: Option<String>,
    },
    #[command(about = "Get a specific todo")]
    Get {
        #[arg(help = "Todo ID")]
        id: String,
    },
    #[command(about = "Update a todo")]
    Update {
        #[arg(help = "Todo ID")]
        id: String,
        #[arg(short, long, help = "New title")]
        title: Option<String>,
        #[arg(short = 'D', long, help = "New description")]
        description: Option<String>,
        #[arg(short, long, help = "New due date")]
        due: Option<String>,
        #[arg(short, long, help = "New priority")]
        priority: Option<String>,
        #[arg(short, long, help = "New tags (comma-separated)")]
        tags: Option<String>,
    },
    #[command(about = "Delete a todo")]
    Delete {
        #[arg(help = "Todo ID")]
        id: String,
    },
    #[command(about = "Toggle todo completion status")]
    Toggle {
        #[arg(help = "Todo ID")]
        id: String,
    },
    #[command(about = "Mark a todo as complete")]
    Complete {
        #[arg(help = "Todo ID")]
        id: String,
    },
    #[command(about = "Search todos")]
    Search {
        #[arg(help = "Search query")]
        query: String,
    },
    #[command(about = "Initialize server and configure CLI")]
    Init {
        #[arg(help = "Server URL (e.g., https://your-server.workers.dev)")]
        url: String,
    },
    #[command(about = "Admin operations")]
    Admin {
        #[command(subcommand)]
        action: AdminAction,
    },
}

#[derive(Subcommand)]
pub enum ConfigAction {
    #[command(about = "Set API endpoint")]
    Endpoint {
        #[arg(help = "API endpoint URL")]
        url: String,
    },
    #[command(about = "Set API key")]
    Key {
        #[arg(help = "API key")]
        key: String,
    },
    #[command(about = "Show current configuration")]
    Show,
}

#[derive(Subcommand)]
pub enum AdminAction {
    #[command(about = "Rotate admin API key")]
    RotateKey,
    #[command(about = "Generate a new API key")]
    GenerateKey {
        #[arg(short, long, help = "Key name")]
        name: Option<String>,
    },
    #[command(about = "List all API keys")]
    ListKeys,
    #[command(about = "Revoke an API key")]
    RevokeKey {
        #[arg(help = "Key ID")]
        id: String,
    },
    #[command(about = "Emergency server reset (deactivates ALL admin keys)")]
    Reinitialize,
}