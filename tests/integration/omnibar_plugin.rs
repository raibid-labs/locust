//! Integration tests for OmnibarPlugin.
//!
//! These tests verify:
//! - Plugin lifecycle (init, cleanup)
//! - Event handling and consumption
//! - Activation/deactivation via events
//! - Command submission workflow
//! - History navigation via events

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use locust::core::context::LocustContext;
use locust::core::input::PluginEventResult;
use locust::core::plugin::LocustPlugin;
use locust::plugins::omnibar::{OmnibarConfig, OmnibarMode, OmnibarPlugin};
use ratatui::backend::TestBackend;

type Backend = TestBackend;

// Helper functions to work around type inference issues
fn plugin_init(plugin: &mut OmnibarPlugin, ctx: &mut LocustContext) {
    <OmnibarPlugin as LocustPlugin<Backend>>::init(plugin, ctx);
}

fn plugin_on_event(
    plugin: &mut OmnibarPlugin,
    event: &Event,
    ctx: &mut LocustContext,
) -> PluginEventResult {
    <OmnibarPlugin as LocustPlugin<Backend>>::on_event(plugin, event, ctx)
}

fn plugin_cleanup(plugin: &mut OmnibarPlugin, ctx: &mut LocustContext) {
    <OmnibarPlugin as LocustPlugin<Backend>>::cleanup(plugin, ctx);
}

fn plugin_id(plugin: &OmnibarPlugin) -> &'static str {
    <OmnibarPlugin as LocustPlugin<Backend>>::id(plugin)
}

fn plugin_priority(plugin: &OmnibarPlugin) -> i32 {
    <OmnibarPlugin as LocustPlugin<Backend>>::priority(plugin)
}

#[test]
fn test_plugin_initialization() {
    let mut plugin = OmnibarPlugin::new();
    let mut ctx = LocustContext::default();

    plugin_init(&mut plugin, &mut ctx);

    assert_eq!(plugin.state().mode(), OmnibarMode::Inactive);
    assert!(!plugin.state().is_active());
}

#[test]
fn test_activation_with_default_key() {
    let mut plugin = OmnibarPlugin::new();
    let mut ctx = LocustContext::default();

    let event = Event::Key(KeyEvent::new(KeyCode::Char('/'), KeyModifiers::NONE));
    let result = plugin_on_event(&mut plugin, &event, &mut ctx);

    assert_eq!(result, PluginEventResult::ConsumedRequestRedraw);
    assert!(plugin.state().is_active());
    assert_eq!(plugin.state().mode(), OmnibarMode::Input);
}

#[test]
fn test_activation_with_custom_key() {
    let config = OmnibarConfig::new().with_activation_key(':');
    let mut plugin = OmnibarPlugin::with_config(config);
    let mut ctx = LocustContext::default();

    let event = Event::Key(KeyEvent::new(KeyCode::Char(':'), KeyModifiers::NONE));
    let result = plugin_on_event(&mut plugin, &event, &mut ctx);

    assert_eq!(result, PluginEventResult::ConsumedRequestRedraw);
    assert!(plugin.state().is_active());
}

#[test]
fn test_activation_ignores_modified_keys() {
    let mut plugin = OmnibarPlugin::new();
    let mut ctx = LocustContext::default();

    // Should ignore Ctrl+/
    let event = Event::Key(KeyEvent::new(KeyCode::Char('/'), KeyModifiers::CONTROL));
    let result = plugin_on_event(&mut plugin, &event, &mut ctx);

    assert_eq!(result, PluginEventResult::NotHandled);
    assert!(!plugin.state().is_active());
}

#[test]
fn test_deactivation_with_escape() {
    let mut plugin = OmnibarPlugin::new();
    let mut ctx = LocustContext::default();

    // Activate
    let event = Event::Key(KeyEvent::new(KeyCode::Char('/'), KeyModifiers::NONE));
    plugin_on_event(&mut plugin, &event, &mut ctx);
    assert!(plugin.state().is_active());

    // Deactivate with Escape
    let event = Event::Key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
    let result = plugin_on_event(&mut plugin, &event, &mut ctx);

    assert_eq!(result, PluginEventResult::ConsumedRequestRedraw);
    assert!(!plugin.state().is_active());
    assert_eq!(plugin.state().mode(), OmnibarMode::Inactive);
}

#[test]
fn test_character_input_when_active() {
    let mut plugin = OmnibarPlugin::new();
    let mut ctx = LocustContext::default();

    // Activate
    plugin_on_event(
        &mut plugin,
        &Event::Key(KeyEvent::new(KeyCode::Char('/'), KeyModifiers::NONE)),
        &mut ctx,
    );

    // Type 'test'
    plugin_on_event(
        &mut plugin,
        &Event::Key(KeyEvent::new(KeyCode::Char('t'), KeyModifiers::NONE)),
        &mut ctx,
    );
    plugin_on_event(
        &mut plugin,
        &Event::Key(KeyEvent::new(KeyCode::Char('e'), KeyModifiers::NONE)),
        &mut ctx,
    );
    plugin_on_event(
        &mut plugin,
        &Event::Key(KeyEvent::new(KeyCode::Char('s'), KeyModifiers::NONE)),
        &mut ctx,
    );
    plugin_on_event(
        &mut plugin,
        &Event::Key(KeyEvent::new(KeyCode::Char('t'), KeyModifiers::NONE)),
        &mut ctx,
    );

    assert_eq!(plugin.state().buffer(), "test");
}

#[test]
fn test_character_input_when_inactive_not_handled() {
    let mut plugin = OmnibarPlugin::new();
    let mut ctx = LocustContext::default();

    let event = Event::Key(KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE));
    let result = plugin_on_event(&mut plugin, &event, &mut ctx);

    // Should not handle when inactive (unless it's the activation key)
    if plugin.config().activation_key != 'a' {
        assert_eq!(result, PluginEventResult::NotHandled);
    }
}

#[test]
fn test_backspace_handling() {
    let mut plugin = OmnibarPlugin::new();
    let mut ctx = LocustContext::default();

    // Activate and type
    plugin_on_event(
        &mut plugin,
        &Event::Key(KeyEvent::new(KeyCode::Char('/'), KeyModifiers::NONE)),
        &mut ctx,
    );
    plugin_on_event(
        &mut plugin,
        &Event::Key(KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE)),
        &mut ctx,
    );
    plugin_on_event(
        &mut plugin,
        &Event::Key(KeyEvent::new(KeyCode::Char('b'), KeyModifiers::NONE)),
        &mut ctx,
    );
    plugin_on_event(
        &mut plugin,
        &Event::Key(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::NONE)),
        &mut ctx,
    );

    assert_eq!(plugin.state().buffer(), "abc");

    // Backspace
    let result = plugin_on_event(
        &mut plugin,
        &Event::Key(KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE)),
        &mut ctx,
    );

    assert_eq!(result, PluginEventResult::ConsumedRequestRedraw);
    assert_eq!(plugin.state().buffer(), "ab");
}

#[test]
fn test_enter_submits_command() {
    let mut plugin = OmnibarPlugin::new();
    let mut ctx = LocustContext::default();

    // Activate and type
    plugin_on_event(
        &mut plugin,
        &Event::Key(KeyEvent::new(KeyCode::Char('/'), KeyModifiers::NONE)),
        &mut ctx,
    );
    plugin_on_event(
        &mut plugin,
        &Event::Key(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::NONE)),
        &mut ctx,
    );
    plugin_on_event(
        &mut plugin,
        &Event::Key(KeyEvent::new(KeyCode::Char('m'), KeyModifiers::NONE)),
        &mut ctx,
    );
    plugin_on_event(
        &mut plugin,
        &Event::Key(KeyEvent::new(KeyCode::Char('d'), KeyModifiers::NONE)),
        &mut ctx,
    );

    // Submit
    let result = plugin_on_event(
        &mut plugin,
        &Event::Key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)),
        &mut ctx,
    );

    assert_eq!(result, PluginEventResult::ConsumedRequestRedraw);
    assert!(!plugin.state().is_active()); // Deactivated after submit
    assert_eq!(plugin.state().history().len(), 1);
    assert_eq!(plugin.state().history()[0], "cmd");
}

#[test]
fn test_enter_on_empty_input() {
    let mut plugin = OmnibarPlugin::new();
    let mut ctx = LocustContext::default();

    // Activate (no input)
    plugin_on_event(
        &mut plugin,
        &Event::Key(KeyEvent::new(KeyCode::Char('/'), KeyModifiers::NONE)),
        &mut ctx,
    );

    // Submit empty
    let result = plugin_on_event(
        &mut plugin,
        &Event::Key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)),
        &mut ctx,
    );

    assert_eq!(result, PluginEventResult::ConsumedRequestRedraw);
    assert!(!plugin.state().is_active()); // Still deactivated
    assert_eq!(plugin.state().history().len(), 0); // No history entry
}

#[test]
fn test_cursor_movement_left_right() {
    let mut plugin = OmnibarPlugin::new();
    let mut ctx = LocustContext::default();

    // Activate and type
    plugin_on_event(
        &mut plugin,
        &Event::Key(KeyEvent::new(KeyCode::Char('/'), KeyModifiers::NONE)),
        &mut ctx,
    );
    plugin_on_event(
        &mut plugin,
        &Event::Key(KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE)),
        &mut ctx,
    );
    plugin_on_event(
        &mut plugin,
        &Event::Key(KeyEvent::new(KeyCode::Char('b'), KeyModifiers::NONE)),
        &mut ctx,
    );
    plugin_on_event(
        &mut plugin,
        &Event::Key(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::NONE)),
        &mut ctx,
    );

    assert_eq!(plugin.state().cursor(), 3);

    // Move left
    plugin_on_event(
        &mut plugin,
        &Event::Key(KeyEvent::new(KeyCode::Left, KeyModifiers::NONE)),
        &mut ctx,
    );
    assert_eq!(plugin.state().cursor(), 2);

    // Move right
    plugin_on_event(
        &mut plugin,
        &Event::Key(KeyEvent::new(KeyCode::Right, KeyModifiers::NONE)),
        &mut ctx,
    );
    assert_eq!(plugin.state().cursor(), 3);
}

#[test]
fn test_cursor_home_end() {
    let mut plugin = OmnibarPlugin::new();
    let mut ctx = LocustContext::default();

    // Activate and type
    plugin_on_event(
        &mut plugin,
        &Event::Key(KeyEvent::new(KeyCode::Char('/'), KeyModifiers::NONE)),
        &mut ctx,
    );
    plugin_on_event(
        &mut plugin,
        &Event::Key(KeyEvent::new(KeyCode::Char('h'), KeyModifiers::NONE)),
        &mut ctx,
    );
    plugin_on_event(
        &mut plugin,
        &Event::Key(KeyEvent::new(KeyCode::Char('e'), KeyModifiers::NONE)),
        &mut ctx,
    );
    plugin_on_event(
        &mut plugin,
        &Event::Key(KeyEvent::new(KeyCode::Char('l'), KeyModifiers::NONE)),
        &mut ctx,
    );
    plugin_on_event(
        &mut plugin,
        &Event::Key(KeyEvent::new(KeyCode::Char('l'), KeyModifiers::NONE)),
        &mut ctx,
    );
    plugin_on_event(
        &mut plugin,
        &Event::Key(KeyEvent::new(KeyCode::Char('o'), KeyModifiers::NONE)),
        &mut ctx,
    );

    // Home
    plugin_on_event(
        &mut plugin,
        &Event::Key(KeyEvent::new(KeyCode::Home, KeyModifiers::NONE)),
        &mut ctx,
    );
    assert_eq!(plugin.state().cursor(), 0);

    // End
    plugin_on_event(
        &mut plugin,
        &Event::Key(KeyEvent::new(KeyCode::End, KeyModifiers::NONE)),
        &mut ctx,
    );
    assert_eq!(plugin.state().cursor(), 5);
}

#[test]
fn test_history_navigation_up_down() {
    let mut plugin = OmnibarPlugin::new();
    let mut ctx = LocustContext::default();

    // Add history
    plugin_on_event(
        &mut plugin,
        &Event::Key(KeyEvent::new(KeyCode::Char('/'), KeyModifiers::NONE)),
        &mut ctx,
    );
    plugin_on_event(
        &mut plugin,
        &Event::Key(KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE)),
        &mut ctx,
    );
    plugin_on_event(
        &mut plugin,
        &Event::Key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)),
        &mut ctx,
    );

    plugin_on_event(
        &mut plugin,
        &Event::Key(KeyEvent::new(KeyCode::Char('/'), KeyModifiers::NONE)),
        &mut ctx,
    );
    plugin_on_event(
        &mut plugin,
        &Event::Key(KeyEvent::new(KeyCode::Char('b'), KeyModifiers::NONE)),
        &mut ctx,
    );
    plugin_on_event(
        &mut plugin,
        &Event::Key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)),
        &mut ctx,
    );

    // Navigate history
    plugin_on_event(
        &mut plugin,
        &Event::Key(KeyEvent::new(KeyCode::Char('/'), KeyModifiers::NONE)),
        &mut ctx,
    );

    plugin_on_event(
        &mut plugin,
        &Event::Key(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE)),
        &mut ctx,
    );
    assert_eq!(plugin.state().buffer(), "b");

    plugin_on_event(
        &mut plugin,
        &Event::Key(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE)),
        &mut ctx,
    );
    assert_eq!(plugin.state().buffer(), "a");

    plugin_on_event(
        &mut plugin,
        &Event::Key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE)),
        &mut ctx,
    );
    assert_eq!(plugin.state().buffer(), "b");
}

#[test]
fn test_plugin_cleanup() {
    let mut plugin = OmnibarPlugin::new();
    let mut ctx = LocustContext::default();

    // Activate
    plugin_on_event(
        &mut plugin,
        &Event::Key(KeyEvent::new(KeyCode::Char('/'), KeyModifiers::NONE)),
        &mut ctx,
    );
    assert!(plugin.state().is_active());

    // Cleanup
    plugin_cleanup(&mut plugin, &mut ctx);
    assert!(!plugin.state().is_active());
}

#[test]
fn test_plugin_priority() {
    let plugin = OmnibarPlugin::new();
    // Omnibar should have higher priority than nav plugin
    assert_eq!(plugin_priority(&plugin), 40);
    assert!(plugin_priority(&plugin) < 50);
}

#[test]
fn test_plugin_id() {
    let plugin = OmnibarPlugin::new();
    assert_eq!(plugin_id(&plugin), "locust.omnibar");
}
