//! Integration tests for configuration management
//!
//! Tests config loading, theme application, and keybinding management.

use locust::core::config::{LocustConfig, Theme, Keybindings};
use locust::core::context::LocustContext;
use locust::plugins::nav::NavConfig;
use locust::plugins::omnibar::OmnibarConfig;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_config_default_creation() {
    let config = LocustConfig::default();

    assert!(config.theme.is_some());
    assert!(config.keybindings.is_some());
}

#[test]
fn test_config_serialization() {
    let config = LocustConfig::default();
    let json = serde_json::to_string_pretty(&config);

    assert!(json.is_ok());
}

#[test]
fn test_config_deserialization() {
    let json = r#"{
        "theme": null,
        "keybindings": null
    }"#;

    let config: Result<LocustConfig, _> = serde_json::from_str(json);
    assert!(config.is_ok());
}

#[test]
fn test_config_round_trip() {
    let original = LocustConfig::default();
    let json = serde_json::to_string(&original).unwrap();
    let restored: LocustConfig = serde_json::from_str(&json).unwrap();

    // Should be equivalent
    assert_eq!(original.theme.is_some(), restored.theme.is_some());
}

#[test]
fn test_theme_default() {
    let theme = Theme::default();

    // Should have default colors
    assert_eq!(theme.accent.r, 0);
    assert_eq!(theme.accent.g, 150);
    assert_eq!(theme.accent.b, 255);
}

#[test]
fn test_theme_custom_colors() {
    use ratatui::style::Color;

    let mut theme = Theme::default();
    theme.accent = Color::Rgb(255, 0, 0);
    theme.background = Color::Rgb(30, 30, 30);

    assert_eq!(theme.accent, Color::Rgb(255, 0, 0));
}

#[test]
fn test_theme_serialization() {
    let theme = Theme::default();
    let json = serde_json::to_string(&theme);

    assert!(json.is_ok());
}

#[test]
fn test_keybindings_default() {
    let bindings = Keybindings::default();

    // Should have default bindings
    assert_eq!(bindings.nav_activate, 'f');
    assert_eq!(bindings.omnibar_activate, ':');
}

#[test]
fn test_keybindings_custom() {
    let mut bindings = Keybindings::default();
    bindings.nav_activate = 'h';
    bindings.omnibar_activate = ';';

    assert_eq!(bindings.nav_activate, 'h');
    assert_eq!(bindings.omnibar_activate, ';');
}

#[test]
fn test_config_file_save_load() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.json");

    let config = LocustConfig::default();

    // Save
    let json = serde_json::to_string_pretty(&config).unwrap();
    fs::write(&config_path, json).unwrap();

    // Load
    let loaded_json = fs::read_to_string(&config_path).unwrap();
    let loaded: LocustConfig = serde_json::from_str(&loaded_json).unwrap();

    assert!(loaded.theme.is_some());
}

#[test]
fn test_plugin_config_integration() {
    let nav_config = NavConfig::default();
    let omnibar_config = OmnibarConfig::default();

    // Should have sensible defaults
    assert_eq!(nav_config.activation_key, 'f');
    assert_eq!(omnibar_config.activation_key, ':');
}

#[test]
fn test_theme_application_to_context() {
    let mut ctx = LocustContext::default();
    let theme = Theme::default();

    ctx.theme = theme;

    // Theme should be accessible
    assert_eq!(ctx.theme.accent.r, 0);
}

#[test]
fn test_multiple_config_updates() {
    let mut config = LocustConfig::default();

    // Update theme
    let mut theme = Theme::default();
    theme.accent = ratatui::style::Color::Red;
    config.theme = Some(theme);

    // Update keybindings
    let mut bindings = Keybindings::default();
    bindings.nav_activate = 'g';
    config.keybindings = Some(bindings);

    assert_eq!(config.keybindings.unwrap().nav_activate, 'g');
}

#[test]
fn test_config_validation() {
    // Config with minimal valid data
    let json = r#"{}"#;
    let config: Result<LocustConfig, _> = serde_json::from_str(json);

    // Should deserialize with defaults
    assert!(config.is_ok());
}
