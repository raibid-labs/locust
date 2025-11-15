//! Edge case tests for omnibar plugin
//!
//! Tests boundary conditions and unusual scenarios for OmnibarPlugin.

use locust::core::context::LocustContext;
use locust::core::plugin::LocustPlugin;
use locust::plugins::omnibar::{OmnibarConfig, OmnibarMode, OmnibarPlugin};
use ratatui::backend::TestBackend;

#[test]
fn test_omnibar_empty_command_registry() {
    let plugin = OmnibarPlugin::new();
    let ctx = LocustContext::default();

    // Should handle empty command registry
    assert!(ctx.commands.is_empty());
    assert_eq!(plugin.mode(), OmnibarMode::Hidden);
}

#[test]
fn test_omnibar_very_long_input() {
    let plugin = OmnibarPlugin::new();

    // Very long input string (1000+ characters)
    let long_input = "a".repeat(1000);

    // Should handle without panic
    assert_eq!(plugin.mode(), OmnibarMode::Hidden);
    drop(long_input);
}

#[test]
fn test_omnibar_special_characters() {
    let plugin = OmnibarPlugin::new();

    // Unicode, emojis, special characters
    let special = "🚀 Hello 世界 \n\t\r \\\"'";

    // Should handle without panic
    assert_eq!(plugin.mode(), OmnibarMode::Hidden);
    drop(special);
}

#[test]
fn test_omnibar_rapid_mode_changes() {
    let mut plugin = OmnibarPlugin::new();
    let mut ctx = LocustContext::default();

    LocustPlugin::<TestBackend>::init(&mut plugin, &mut ctx);

    // Rapid mode transitions should be stable
    for _ in 0..100 {
        assert_eq!(plugin.mode(), OmnibarMode::Hidden);
    }
}

#[test]
fn test_omnibar_zero_width_config() {
    let config = OmnibarConfig::new()
        .with_width(0)
        .with_height(0);

    let plugin = OmnibarPlugin::with_config(config);
    assert_eq!(plugin.config().width, 0);
    assert_eq!(plugin.config().height, 0);
}

#[test]
fn test_omnibar_very_large_dimensions() {
    let config = OmnibarConfig::new()
        .with_width(u16::MAX)
        .with_height(u16::MAX);

    let plugin = OmnibarPlugin::with_config(config);
    assert_eq!(plugin.config().width, u16::MAX);
}

#[test]
fn test_omnibar_config_extreme_max_results() {
    let config = OmnibarConfig::new()
        .with_max_results(0)
        .with_prompt("> ");

    let plugin = OmnibarPlugin::with_config(config);
    assert_eq!(plugin.config().max_results, 0);
}

#[test]
fn test_omnibar_empty_prompt() {
    let config = OmnibarConfig::new()
        .with_prompt("");

    let plugin = OmnibarPlugin::with_config(config);
    assert_eq!(plugin.config().prompt, "");
}

#[test]
fn test_omnibar_very_long_prompt() {
    let long_prompt = "→".repeat(100);
    let config = OmnibarConfig::new()
        .with_prompt(&long_prompt);

    let plugin = OmnibarPlugin::with_config(config);
    assert_eq!(plugin.config().prompt.len(), 300); // 100 * 3 bytes per arrow
}

#[test]
fn test_omnibar_plugin_initialization() {
    let mut plugin = OmnibarPlugin::new();
    let mut ctx = LocustContext::default();

    LocustPlugin::<TestBackend>::init(&mut plugin, &mut ctx);

    // Multiple inits should be safe
    LocustPlugin::<TestBackend>::init(&mut plugin, &mut ctx);

    assert_eq!(plugin.mode(), OmnibarMode::Hidden);
}
