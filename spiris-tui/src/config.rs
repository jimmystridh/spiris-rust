//! Configuration management module.
//!
//! Handles loading and saving user preferences from a TOML configuration file.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Application configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Display settings
    #[serde(default)]
    pub display: DisplayConfig,

    /// Pagination settings
    #[serde(default)]
    pub pagination: PaginationConfig,

    /// Export settings
    #[serde(default)]
    pub export: ExportConfig,

    /// Theme settings
    #[serde(default)]
    pub theme: ThemeConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayConfig {
    /// Show line numbers in lists
    #[serde(default = "default_true")]
    pub show_line_numbers: bool,

    /// Show keyboard hints in status bar
    #[serde(default = "default_true")]
    pub show_keyboard_hints: bool,

    /// Auto-refresh interval in seconds (0 = disabled)
    #[serde(default)]
    pub auto_refresh_interval: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationConfig {
    /// Default page size
    #[serde(default = "default_page_size")]
    pub default_page_size: u32,

    /// Maximum items to load at once
    #[serde(default = "default_max_items")]
    pub max_items: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportConfig {
    /// Default export format (csv or json)
    #[serde(default = "default_export_format")]
    pub default_format: String,

    /// Export directory
    #[serde(default = "default_export_dir")]
    pub export_directory: String,

    /// Include timestamps in export filenames
    #[serde(default = "default_true")]
    pub include_timestamp: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeConfig {
    /// Primary color (for highlights, selections)
    #[serde(default = "default_primary_color")]
    pub primary_color: String,

    /// Error color
    #[serde(default = "default_error_color")]
    pub error_color: String,

    /// Success color
    #[serde(default = "default_success_color")]
    pub success_color: String,
}

// Default value functions
fn default_true() -> bool {
    true
}

fn default_page_size() -> u32 {
    50
}

fn default_max_items() -> u32 {
    1000
}

fn default_export_format() -> String {
    "csv".to_string()
}

fn default_export_dir() -> String {
    ".".to_string()
}

fn default_primary_color() -> String {
    "cyan".to_string()
}

fn default_error_color() -> String {
    "red".to_string()
}

fn default_success_color() -> String {
    "green".to_string()
}

impl Default for DisplayConfig {
    fn default() -> Self {
        Self {
            show_line_numbers: true,
            show_keyboard_hints: true,
            auto_refresh_interval: 0,
        }
    }
}

impl Default for PaginationConfig {
    fn default() -> Self {
        Self {
            default_page_size: 50,
            max_items: 1000,
        }
    }
}

impl Default for ExportConfig {
    fn default() -> Self {
        Self {
            default_format: "csv".to_string(),
            export_directory: ".".to_string(),
            include_timestamp: true,
        }
    }
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            primary_color: "cyan".to_string(),
            error_color: "red".to_string(),
            success_color: "green".to_string(),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            display: DisplayConfig::default(),
            pagination: PaginationConfig::default(),
            export: ExportConfig::default(),
            theme: ThemeConfig::default(),
        }
    }
}

impl Config {
    /// Load configuration from file, or create default if not exists
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path()?;

        if config_path.exists() {
            let contents = fs::read_to_string(&config_path)
                .context("Failed to read config file")?;
            let config: Config = toml::from_str(&contents)
                .context("Failed to parse config file")?;
            Ok(config)
        } else {
            // Create default config
            let config = Config::default();
            config.save()?;
            Ok(config)
        }
    }

    /// Save configuration to file
    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path()?;

        // Ensure parent directory exists
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)
                .context("Failed to create config directory")?;
        }

        let contents = toml::to_string_pretty(self)
            .context("Failed to serialize config")?;
        fs::write(&config_path, contents)
            .context("Failed to write config file")?;

        Ok(())
    }

    /// Get the path to the configuration file
    pub fn config_path() -> Result<PathBuf> {
        let config_dir = if let Ok(home) = std::env::var("HOME") {
            PathBuf::from(home).join(".config").join("spiris-tui")
        } else if let Ok(user_profile) = std::env::var("USERPROFILE") {
            PathBuf::from(user_profile).join(".config").join("spiris-tui")
        } else {
            PathBuf::from(".spiris-tui")
        };

        Ok(config_dir.join("config.toml"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.pagination.default_page_size, 50);
        assert_eq!(config.export.default_format, "csv");
        assert!(config.display.show_keyboard_hints);
    }

    #[test]
    fn test_config_serialization() {
        let config = Config::default();
        let toml_str = toml::to_string(&config).unwrap();
        assert!(toml_str.contains("default_page_size"));
        assert!(toml_str.contains("default_format"));
    }

    #[test]
    fn test_config_deserialization() {
        let toml_str = r#"
            [display]
            show_line_numbers = false
            show_keyboard_hints = true
            auto_refresh_interval = 30

            [pagination]
            default_page_size = 100
            max_items = 500

            [export]
            default_format = "json"
            export_directory = "/tmp"
            include_timestamp = false

            [theme]
            primary_color = "blue"
            error_color = "red"
            success_color = "green"
        "#;

        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.pagination.default_page_size, 100);
        assert_eq!(config.export.default_format, "json");
        assert!(!config.display.show_line_numbers);
        assert_eq!(config.display.auto_refresh_interval, 30);
    }
}
