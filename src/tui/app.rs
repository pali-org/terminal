//! TUI application state and logic

use anyhow::Result;
use crate::{ApiClient, Config};

pub struct App {
    pub should_quit: bool,
    pub api_client: ApiClient,
    pub config: Config,
}

impl App {
    /// Creates a new TUI application instance with loaded configuration
    /// 
    /// # Errors
    /// 
    /// Returns an error if:
    /// - Configuration cannot be loaded from disk
    /// - Configuration file format is invalid
    /// - API client initialization fails
    pub fn new() -> Result<Self> {
        let config = Config::load()?;
        let api_client = ApiClient::new()?;
        
        Ok(Self {
            should_quit: false,
            api_client,
            config,
        })
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    /// Handles keyboard input events
    /// 
    /// # Errors
    /// 
    /// Returns an error if key handling fails (currently never fails)
    pub fn handle_key(&mut self, key: crossterm::event::KeyCode) -> Result<()> {
        match key {
            crossterm::event::KeyCode::Char('q') | crossterm::event::KeyCode::Esc => self.quit(),
            _ => {}
        }
        Ok(())
    }
}

// Note: Default implementation removed - use App::new() instead
// as config loading can fail and should be handled explicitly