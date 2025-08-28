//! # Pali Terminal
//!
//! Terminal interfaces (CLI and TUI) for the Pali todo management system.
//!
//! This crate provides both command-line and terminal user interface tools:
//! - `pacli` - Command-line interface for automation and scripting
//! - `patui` - Terminal user interface for interactive usage
//!
//! ## Features
//!
//! - `cli` - Enables command-line interface functionality
//! - `tui` - Enables terminal user interface functionality
//!
//! Both features are enabled by default.

// Core modules - always available
pub mod api;
pub mod config;

// Shared constants
pub const ID_DISPLAY_LENGTH: usize = 8;

// Logging utilities (CLI only for now)
#[cfg(feature = "cli")]
pub mod logging;

// CLI-specific modules
#[cfg(feature = "cli")]
pub mod cli {
    pub mod commands {
        pub mod admin;
        pub mod config;
        pub mod todo;
    }
    pub mod types;
    pub mod utils;
}

// TUI-specific modules
#[cfg(feature = "tui")]
pub mod tui {
    pub mod app;
    pub mod components;
    pub mod ui;
}

// Re-exports for convenience
pub use api::ApiClient;
pub use config::Config;

#[cfg(feature = "cli")]
pub use logging::init_logging;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_id_truncation() {
        let test_id = "abcdefghijklmnop"; // 16 characters
        let truncated = &test_id[..ID_DISPLAY_LENGTH];
        assert_eq!(truncated, "abcdefgh");
        assert_eq!(truncated.len(), ID_DISPLAY_LENGTH);
    }
}
