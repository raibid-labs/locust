//! Unified configuration system for Locust.
//!
//! This module provides a flexible, hierarchical configuration system with:
//! - Per-plugin configuration support
//! - TOML and JSON file loading/saving
//! - Runtime configuration updates
//! - Hot reload detection
//! - Comprehensive validation
//!
//! # Example
//!
//! ```rust,no_run
//! use locust::core::config::LocustConfig;
//! use std::path::Path;
//!
//! // Load from file
//! let config = LocustConfig::from_file(Path::new("locust.toml"))?;
//!
//! // Or create programmatically
//! let mut config = LocustConfig::new();
//! config.global.fps_limit = Some(60);
//!
//! // Save to file
//! config.save()?;
//! # Ok::<(), locust::core::config::ConfigError>(())
//! ```

use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// Log level for Locust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LogLevel {
    /// Most verbose, shows all messages
    Trace,
    /// Debug information
    Debug,
    /// Informational messages
    Info,
    /// Warnings only
    Warn,
    /// Errors only
    Error,
}

impl Default for LogLevel {
    fn default() -> Self {
        Self::Info
    }
}

/// Global configuration settings for Locust.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GlobalConfig {
    /// Enable logging to stderr
    #[serde(default)]
    pub enable_logging: bool,

    /// Logging level
    #[serde(default)]
    pub log_level: LogLevel,

    /// Frame rate limit (None = unlimited)
    #[serde(default)]
    pub fps_limit: Option<u32>,

    /// Enable mouse support for hover interactions
    #[serde(default = "default_mouse_support")]
    pub mouse_support: bool,
}

fn default_mouse_support() -> bool {
    true
}

impl Default for GlobalConfig {
    fn default() -> Self {
        Self {
            enable_logging: false,
            log_level: LogLevel::Info,
            fps_limit: Some(60),
            mouse_support: true,
        }
    }
}

/// Navigation plugin configuration (simplified for serialization).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NavConfig {
    pub hint_key: char,
    pub charset: String,
    pub min_target_area: u32,
    pub max_hints: usize,
}

impl Default for NavConfig {
    fn default() -> Self {
        Self {
            hint_key: 'f',
            charset: "asdfghjkl".to_string(),
            min_target_area: 1,
            max_hints: 0,
        }
    }
}

/// Omnibar plugin configuration (simplified for serialization).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OmnibarConfig {
    pub activation_key: char,
    pub activation_modifiers: Vec<String>,
    pub max_results: usize,
    pub fuzzy_threshold: f64,
}

impl Default for OmnibarConfig {
    fn default() -> Self {
        Self {
            activation_key: 'p',
            activation_modifiers: vec!["Ctrl".to_string()],
            max_results: 10,
            fuzzy_threshold: 0.6,
        }
    }
}

/// Tooltip positioning preference.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TooltipPosition {
    Right,
    Left,
    Above,
    Below,
    Auto,
}

impl Default for TooltipPosition {
    fn default() -> Self {
        Self::Right
    }
}

/// Tooltip plugin configuration (simplified for serialization).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TooltipConfig {
    pub default_delay_ms: u64,
    pub default_position: TooltipPosition,
    pub auto_hide_ms: u64,
}

impl Default for TooltipConfig {
    fn default() -> Self {
        Self {
            default_delay_ms: 500,
            default_position: TooltipPosition::Right,
            auto_hide_ms: 3000,
        }
    }
}

/// Highlight animation type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HighlightAnimation {
    None,
    Pulse,
    Shimmer,
    Breathe,
}

impl Default for HighlightAnimation {
    fn default() -> Self {
        Self::Pulse
    }
}

/// Highlight plugin configuration (simplified for serialization).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HighlightConfig {
    pub default_animation: HighlightAnimation,
    pub dim_opacity: f32,
    pub border_thickness: u16,
}

impl Default for HighlightConfig {
    fn default() -> Self {
        Self {
            default_animation: HighlightAnimation::Pulse,
            dim_opacity: 0.7,
            border_thickness: 2,
        }
    }
}

/// Plugin-specific configuration.
///
/// This enum supports all built-in plugin types and allows
/// custom JSON values for user-defined plugins.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum PluginConfig {
    Nav(NavConfig),
    Omnibar(OmnibarConfig),
    Tooltip(TooltipConfig),
    Highlight(HighlightConfig),
    Custom(serde_json::Value),
}

/// Main configuration structure for Locust.
///
/// Supports hierarchical configuration with global settings
/// and per-plugin customization.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LocustConfig {
    /// Global settings
    #[serde(default)]
    pub global: GlobalConfig,

    /// Per-plugin configurations
    #[serde(default)]
    pub plugins: HashMap<String, PluginConfig>,

    /// Configuration file path (not serialized)
    #[serde(skip)]
    pub config_path: Option<PathBuf>,
}

impl Default for LocustConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl LocustConfig {
    /// Creates a new configuration with default values.
    pub fn new() -> Self {
        Self {
            global: GlobalConfig::default(),
            plugins: HashMap::new(),
            config_path: None,
        }
    }

    /// Loads configuration from a file.
    ///
    /// Supports both TOML and JSON formats, auto-detected by extension.
    ///
    /// # Errors
    ///
    /// Returns `ConfigError` if:
    /// - File cannot be read
    /// - File format is invalid
    /// - Required fields are missing
    pub fn from_file(path: &Path) -> Result<Self, ConfigError> {
        let content = fs::read_to_string(path).map_err(|e| ConfigError::Io {
            path: path.to_path_buf(),
            source: e,
        })?;

        let mut config: Self = match path.extension().and_then(|s| s.to_str()) {
            Some("toml") => toml::from_str(&content).map_err(|e| ConfigError::ParseToml {
                path: path.to_path_buf(),
                source: e,
            })?,
            Some("json") => serde_json::from_str(&content).map_err(|e| ConfigError::ParseJson {
                path: path.to_path_buf(),
                source: e,
            })?,
            _ => {
                // Try TOML first, then JSON
                toml::from_str::<Self>(&content)
                    .or_else(|_| serde_json::from_str::<Self>(&content))
                    .map_err(|e| ConfigError::ParseJson {
                        path: path.to_path_buf(),
                        source: e,
                    })?
            }
        };

        config.config_path = Some(path.to_path_buf());
        Ok(config)
    }

    /// Saves the configuration to a file.
    ///
    /// Uses the path from `config_path` if set, otherwise returns an error.
    /// Format is determined by file extension.
    ///
    /// # Errors
    ///
    /// Returns `ConfigError` if:
    /// - No config path is set
    /// - File cannot be written
    /// - Serialization fails
    pub fn save(&self) -> Result<(), ConfigError> {
        let path = self.config_path.as_ref().ok_or(ConfigError::NoConfigPath)?;

        self.save_to(path)
    }

    /// Saves the configuration to a specific file path.
    ///
    /// # Errors
    ///
    /// Returns `ConfigError` if file cannot be written or serialization fails.
    pub fn save_to(&self, path: &Path) -> Result<(), ConfigError> {
        let content = match path.extension().and_then(|s| s.to_str()) {
            Some("toml") => {
                toml::to_string_pretty(self).map_err(|e| ConfigError::SerializeToml {
                    path: path.to_path_buf(),
                    source: e,
                })?
            }
            Some("json") => {
                serde_json::to_string_pretty(self).map_err(|e| ConfigError::SerializeJson {
                    path: path.to_path_buf(),
                    source: e,
                })?
            }
            _ => toml::to_string_pretty(self).map_err(|e| ConfigError::SerializeToml {
                path: path.to_path_buf(),
                source: e,
            })?,
        };

        fs::write(path, content).map_err(|e| ConfigError::Io {
            path: path.to_path_buf(),
            source: e,
        })?;

        Ok(())
    }

    /// Reloads configuration from the original file.
    ///
    /// # Errors
    ///
    /// Returns `ConfigError` if no path is set or file cannot be loaded.
    pub fn reload(&mut self) -> Result<(), ConfigError> {
        let path = self.config_path.clone().ok_or(ConfigError::NoConfigPath)?;
        *self = Self::from_file(&path)?;
        Ok(())
    }

    /// Updates plugin configuration.
    ///
    /// # Example
    ///
    /// ```rust
    /// use locust::core::config::{LocustConfig, NavConfig};
    ///
    /// let mut config = LocustConfig::new();
    /// let nav_config = NavConfig {
    ///     hint_key: 'g',
    ///     charset: "abc".to_string(),
    ///     min_target_area: 10,
    ///     max_hints: 50,
    /// };
    /// config.update_plugin("nav", nav_config)?;
    /// # Ok::<(), locust::core::config::ConfigError>(())
    /// ```
    pub fn update_plugin<T: Serialize>(
        &mut self,
        plugin_id: &str,
        config: T,
    ) -> Result<(), ConfigError> {
        let value = serde_json::to_value(config).map_err(|e| ConfigError::InvalidPluginConfig {
            plugin_id: plugin_id.to_string(),
            source: e,
        })?;

        self.plugins
            .insert(plugin_id.to_string(), PluginConfig::Custom(value));
        Ok(())
    }

    /// Retrieves plugin configuration.
    ///
    /// # Example
    ///
    /// ```rust
    /// use locust::core::config::{LocustConfig, NavConfig};
    ///
    /// let config = LocustConfig::new();
    /// let nav_config: Option<NavConfig> = config.get_plugin_config("nav");
    /// ```
    pub fn get_plugin_config<T: DeserializeOwned>(&self, plugin_id: &str) -> Option<T> {
        let plugin_config = self.plugins.get(plugin_id)?;

        match plugin_config {
            PluginConfig::Custom(value) => serde_json::from_value(value.clone()).ok(),
            PluginConfig::Nav(cfg) => serde_json::from_value(serde_json::to_value(cfg).ok()?).ok(),
            PluginConfig::Omnibar(cfg) => {
                serde_json::from_value(serde_json::to_value(cfg).ok()?).ok()
            }
            PluginConfig::Tooltip(cfg) => {
                serde_json::from_value(serde_json::to_value(cfg).ok()?).ok()
            }
            PluginConfig::Highlight(cfg) => {
                serde_json::from_value(serde_json::to_value(cfg).ok()?).ok()
            }
        }
    }

    /// Validates the configuration.
    ///
    /// Returns a list of validation errors, if any.
    pub fn validate(&self) -> Vec<ValidationError> {
        let mut errors = Vec::new();

        // Validate global config
        if let Some(fps) = self.global.fps_limit {
            if fps == 0 {
                errors.push(ValidationError {
                    field: "global.fps_limit".to_string(),
                    message: "FPS limit must be greater than 0".to_string(),
                    severity: Severity::Error,
                });
            }
            if fps > 240 {
                errors.push(ValidationError {
                    field: "global.fps_limit".to_string(),
                    message: "FPS limit above 240 may cause performance issues".to_string(),
                    severity: Severity::Warning,
                });
            }
        }

        // Validate plugin configs
        for (plugin_id, plugin_config) in &self.plugins {
            if let PluginConfig::Nav(cfg) = plugin_config {
                if cfg.charset.is_empty() {
                    errors.push(ValidationError {
                        field: format!("plugins.{}.charset", plugin_id),
                        message: "Charset cannot be empty".to_string(),
                        severity: Severity::Error,
                    });
                }
                if cfg.min_target_area == 0 {
                    errors.push(ValidationError {
                        field: format!("plugins.{}.min_target_area", plugin_id),
                        message: "Minimum target area should be at least 1".to_string(),
                        severity: Severity::Warning,
                    });
                }
            }
        }

        errors
    }
}

/// Configuration watcher for hot reload support.
pub struct ConfigWatcher {
    path: PathBuf,
    last_modified: Option<SystemTime>,
}

impl ConfigWatcher {
    /// Creates a new config watcher for the given path.
    pub fn new(path: PathBuf) -> Self {
        let last_modified = fs::metadata(&path).and_then(|m| m.modified()).ok();

        Self {
            path,
            last_modified,
        }
    }

    /// Checks if the configuration file has been modified.
    ///
    /// Returns `true` if the file has changed since the last check.
    pub fn check_for_changes(&mut self) -> bool {
        let current_modified = match fs::metadata(&self.path).and_then(|m| m.modified()) {
            Ok(time) => time,
            Err(_) => return false,
        };

        match self.last_modified {
            Some(last) if current_modified > last => {
                self.last_modified = Some(current_modified);
                true
            }
            None => {
                self.last_modified = Some(current_modified);
                false
            }
            _ => false,
        }
    }
}

/// Validation error severity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    /// Critical error that prevents usage
    Error,
    /// Non-critical issue or recommendation
    Warning,
}

/// Validation error with location and severity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationError {
    /// Field path (e.g., "global.fps_limit")
    pub field: String,
    /// Human-readable error message
    pub message: String,
    /// Error severity
    pub severity: Severity,
}

/// Configuration-related errors.
#[derive(Debug)]
pub enum ConfigError {
    /// I/O error while reading/writing config file
    Io { path: PathBuf, source: io::Error },
    /// TOML parsing error
    ParseToml {
        path: PathBuf,
        source: toml::de::Error,
    },
    /// JSON parsing error
    ParseJson {
        path: PathBuf,
        source: serde_json::Error,
    },
    /// TOML serialization error
    SerializeToml {
        path: PathBuf,
        source: toml::ser::Error,
    },
    /// JSON serialization error
    SerializeJson {
        path: PathBuf,
        source: serde_json::Error,
    },
    /// No config path set
    NoConfigPath,
    /// Invalid plugin configuration
    InvalidPluginConfig {
        plugin_id: String,
        source: serde_json::Error,
    },
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::Io { path, source } => {
                write!(f, "I/O error reading {}: {}", path.display(), source)
            }
            ConfigError::ParseToml { path, source } => {
                write!(f, "TOML parse error in {}: {}", path.display(), source)
            }
            ConfigError::ParseJson { path, source } => {
                write!(f, "JSON parse error in {}: {}", path.display(), source)
            }
            ConfigError::SerializeToml { path, source } => {
                write!(
                    f,
                    "TOML serialization error for {}: {}",
                    path.display(),
                    source
                )
            }
            ConfigError::SerializeJson { path, source } => {
                write!(
                    f,
                    "JSON serialization error for {}: {}",
                    path.display(),
                    source
                )
            }
            ConfigError::NoConfigPath => write!(f, "No configuration path set"),
            ConfigError::InvalidPluginConfig { plugin_id, source } => {
                write!(
                    f,
                    "Invalid plugin configuration for '{}': {}",
                    plugin_id, source
                )
            }
        }
    }
}

impl std::error::Error for ConfigError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ConfigError::Io { source, .. } => Some(source),
            ConfigError::ParseToml { source, .. } => Some(source),
            ConfigError::ParseJson { source, .. } => Some(source),
            ConfigError::SerializeToml { source, .. } => Some(source),
            ConfigError::SerializeJson { source, .. } => Some(source),
            ConfigError::InvalidPluginConfig { source, .. } => Some(source),
            ConfigError::NoConfigPath => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = LocustConfig::new();
        assert_eq!(config.global.fps_limit, Some(60));
        assert!(config.global.mouse_support);
        assert!(!config.global.enable_logging);
        assert_eq!(config.global.log_level, LogLevel::Info);
        assert!(config.plugins.is_empty());
    }

    #[test]
    fn test_plugin_config_update() {
        let mut config = LocustConfig::new();
        let nav_config = NavConfig {
            hint_key: 'g',
            charset: "abc".to_string(),
            min_target_area: 10,
            max_hints: 50,
        };

        config.update_plugin("nav", nav_config.clone()).unwrap();
        let retrieved: NavConfig = config.get_plugin_config("nav").unwrap();
        assert_eq!(retrieved, nav_config);
    }

    #[test]
    fn test_validation_errors() {
        let mut config = LocustConfig::new();
        config.global.fps_limit = Some(0);

        let errors = config.validate();
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].severity, Severity::Error);
        assert!(errors[0].message.contains("greater than 0"));
    }

    #[test]
    fn test_validation_warnings() {
        let mut config = LocustConfig::new();
        config.global.fps_limit = Some(300);

        let errors = config.validate();
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].severity, Severity::Warning);
    }
}
