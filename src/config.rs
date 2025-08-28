use anyhow::Result;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub api_endpoint: String,
    pub api_key: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            api_endpoint: "http://localhost:8787".to_string(),
            api_key: None,
        }
    }
}

impl Config {
    /// Loads configuration from disk, returning default config if file doesn't exist
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Configuration file exists but cannot be read
    /// - Configuration file format is invalid JSON
    /// - File permissions prevent access
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path()?;

        if config_path.exists() {
            let content = fs::read_to_string(config_path)?;
            Ok(serde_json::from_str(&content)?)
        } else {
            Ok(Self::default())
        }
    }

    /// Saves the current configuration to disk
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Cannot create config directory
    /// - Cannot write to config file
    /// - JSON serialization fails
    /// - File permissions prevent writing
    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path()?;

        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = serde_json::to_string_pretty(self)?;
        fs::write(config_path, content)?;
        Ok(())
    }

    /// Returns the path to the configuration file
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - System doesn't support standard config directories
    /// - HOME environment variable is not set
    /// - Cannot determine user's config directory
    pub fn config_path() -> Result<PathBuf> {
        let proj_dirs = ProjectDirs::from("com", "pali", "pali")
            .ok_or_else(|| anyhow::anyhow!(
                "Could not determine config directory. This usually means your system doesn't support standard config directories or the HOME environment variable is not set."
            ))?;

        Ok(proj_dirs.config_dir().join("config.json"))
    }

    pub fn set_endpoint(&mut self, endpoint: impl Into<String>) {
        self.api_endpoint = endpoint.into();
    }

    pub fn set_api_key(&mut self, key: impl Into<String>) {
        self.api_key = Some(key.into());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.api_endpoint, "http://localhost:8787");
        assert_eq!(config.api_key, None);
    }

    #[test]
    fn test_config_serialization() {
        let config = Config {
            api_endpoint: "https://api.example.com".to_string(),
            api_key: Some("test-key".to_string()),
        };

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: Config = serde_json::from_str(&json).unwrap();

        assert_eq!(config.api_endpoint, deserialized.api_endpoint);
        assert_eq!(config.api_key, deserialized.api_key);
    }

    #[test]
    fn test_set_endpoint() {
        let mut config = Config::default();
        config.set_endpoint("https://new-api.com".to_string());
        assert_eq!(config.api_endpoint, "https://new-api.com");
    }

    #[test]
    fn test_set_api_key() {
        let mut config = Config::default();
        config.set_api_key("new-key".to_string());
        assert_eq!(config.api_key, Some("new-key".to_string()));
    }

    #[test]
    fn test_config_path_generation() {
        let path = Config::config_path();
        assert!(path.is_ok());
        let path = path.unwrap();
        assert!(path.to_string_lossy().contains("pali"));
        assert!(path.to_string_lossy().ends_with("config.json"));
    }
}
