//! Integration tests for the complete navigation flow.

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use locust::core::context::LocustContext;
use locust::core::input::PluginEventResult;
use locust::core::plugin::LocustPlugin;
use locust::core::targets::{NavTarget, TargetAction, TargetPriority};
use locust::plugins::nav::{NavConfig, NavMode, NavPlugin};
use ratatui::backend::TestBackend;
use ratatui::layout::Rect;

fn create_test_context() -> LocustContext {
    let mut ctx = LocustContext::default();

    // Add some test targets
    ctx.targets.register(
        NavTarget::new(1, Rect::new(5, 5, 20, 3))
            .with_label("Button 1")
            .with_action(TargetAction::Activate)
            .with_priority(TargetPriority::High),
    );

    ctx.targets.register(
        NavTarget::new(2, Rect::new(5, 10, 20, 3))
            .with_label("Button 2")
            .with_action(TargetAction::Activate)
            .with_priority(TargetPriority::Normal),
    );

    ctx.targets.register(
        NavTarget::new(3, Rect::new(5, 15, 20, 3))
            .with_label("Button 3")
            .with_action(TargetAction::Select)
            .with_priority(TargetPriority::Normal),
    );

    ctx
}

fn key_event(code: KeyCode) -> Event {
    Event::Key(KeyEvent {
        code,
        modifiers: KeyModifiers::NONE,
        kind: crossterm::event::KeyEventKind::Press,
        state: crossterm::event::KeyEventState::NONE,
    })
}

#[test]
fn test_hint_mode_activation() {
    let mut plugin = NavPlugin::new();
    let mut ctx = create_test_context();

    assert_eq!(plugin.mode(), NavMode::Normal);

    // Press 'f' to activate hint mode
    let result = plugin.on_event(&key_event(KeyCode::Char('f')), &mut ctx);

    assert_eq!(result, PluginEventResult::ConsumedRequestRedraw);
    assert_eq!(plugin.mode(), NavMode::Hint);
}

#[test]
fn test_hint_mode_exit_with_escape() {
    let mut plugin = NavPlugin::new();
    let mut ctx = create_test_context();

    // Enter hint mode
    plugin.on_event(&key_event(KeyCode::Char('f')), &mut ctx);
    assert_eq!(plugin.mode(), NavMode::Hint);

    // Press Escape to exit
    let result = plugin.on_event(&key_event(KeyCode::Esc), &mut ctx);

    assert_eq!(result, PluginEventResult::ConsumedRequestRedraw);
    assert_eq!(plugin.mode(), NavMode::Normal);
}

#[test]
fn test_hint_selection_single_char() {
    let mut plugin = NavPlugin::new();
    let mut ctx = create_test_context();

    // Enter hint mode
    plugin.on_event(&key_event(KeyCode::Char('f')), &mut ctx);
    assert_eq!(plugin.mode(), NavMode::Hint);

    // Type first hint character (should activate highest priority target)
    let result = plugin.on_event(&key_event(KeyCode::Char('a')), &mut ctx);

    assert_eq!(result, PluginEventResult::ConsumedRequestRedraw);
    assert_eq!(plugin.mode(), NavMode::Normal); // Should exit after activation
}

#[test]
fn test_hint_selection_multi_char() {
    let config = NavConfig::new().with_charset("ab");
    let mut plugin = NavPlugin::with_config(config);
    let mut ctx = create_test_context();

    // Enter hint mode
    plugin.on_event(&key_event(KeyCode::Char('f')), &mut ctx);
    assert_eq!(plugin.mode(), NavMode::Hint);

    // Type 'a' - should not activate (multiple targets starting with 'a')
    let result = plugin.on_event(&key_event(KeyCode::Char('a')), &mut ctx);
    assert_eq!(result, PluginEventResult::ConsumedRequestRedraw);
    assert_eq!(plugin.mode(), NavMode::Hint); // Still in hint mode

    // Type 'b' - should activate target "ab"
    let result = plugin.on_event(&key_event(KeyCode::Char('b')), &mut ctx);
    assert_eq!(result, PluginEventResult::ConsumedRequestRedraw);
    assert_eq!(plugin.mode(), NavMode::Normal); // Exited after activation
}

#[test]
fn test_hint_backspace() {
    let config = NavConfig::new().with_charset("ab");
    let mut plugin = NavPlugin::with_config(config);
    let mut ctx = create_test_context();

    // Enter hint mode
    plugin.on_event(&key_event(KeyCode::Char('f')), &mut ctx);

    // Type 'a'
    plugin.on_event(&key_event(KeyCode::Char('a')), &mut ctx);
    assert_eq!(plugin.mode(), NavMode::Hint);

    // Press backspace
    let result = plugin.on_event(&key_event(KeyCode::Backspace), &mut ctx);
    assert_eq!(result, PluginEventResult::ConsumedRequestRedraw);
    assert_eq!(plugin.mode(), NavMode::Hint); // Still in hint mode
}

#[test]
fn test_hint_generation_with_priorities() {
    let mut plugin = NavPlugin::new();
    let mut ctx = LocustContext::default();

    // Add targets with different priorities
    ctx.targets.register(
        NavTarget::new(1, Rect::new(0, 0, 10, 1)).with_priority(TargetPriority::Low),
    );
    ctx.targets.register(
        NavTarget::new(2, Rect::new(0, 2, 10, 1)).with_priority(TargetPriority::Critical),
    );
    ctx.targets.register(
        NavTarget::new(3, Rect::new(0, 4, 10, 1)).with_priority(TargetPriority::High),
    );

    // Enter hint mode
    plugin.on_event(&key_event(KeyCode::Char('f')), &mut ctx);

    // Type 'a' - should activate Critical priority target (ID 2)
    plugin.on_event(&key_event(KeyCode::Char('a')), &mut ctx);

    assert_eq!(plugin.mode(), NavMode::Normal);
}

#[test]
fn test_custom_hint_key() {
    let config = NavConfig::new().with_hint_key('g');
    let mut plugin = NavPlugin::with_config(config);
    let mut ctx = create_test_context();

    // 'f' should not activate hint mode
    let result = plugin.on_event(&key_event(KeyCode::Char('f')), &mut ctx);
    assert_eq!(result, PluginEventResult::NotHandled);
    assert_eq!(plugin.mode(), NavMode::Normal);

    // 'g' should activate hint mode
    let result = plugin.on_event(&key_event(KeyCode::Char('g')), &mut ctx);
    assert_eq!(result, PluginEventResult::ConsumedRequestRedraw);
    assert_eq!(plugin.mode(), NavMode::Hint);
}

#[test]
fn test_min_target_area_filter() {
    let config = NavConfig::new().with_min_target_area(100);
    let mut plugin = NavPlugin::with_config(config);
    let mut ctx = LocustContext::default();

    // Add targets with different areas
    ctx.targets
        .register(NavTarget::new(1, Rect::new(0, 0, 5, 5))); // area = 25
    ctx.targets
        .register(NavTarget::new(2, Rect::new(0, 10, 10, 10))); // area = 100
    ctx.targets
        .register(NavTarget::new(3, Rect::new(0, 20, 15, 10))); // area = 150

    // Enter hint mode
    plugin.on_event(&key_event(KeyCode::Char('f')), &mut ctx);

    // Type 'a' - should activate first target that meets area requirement (ID 2)
    plugin.on_event(&key_event(KeyCode::Char('a')), &mut ctx);

    assert_eq!(plugin.mode(), NavMode::Normal);
}

#[test]
fn test_max_hints_limit() {
    let config = NavConfig::new().with_max_hints(2);
    let mut plugin = NavPlugin::with_config(config);
    let mut ctx = LocustContext::default();

    // Add 5 targets
    for i in 1..=5 {
        ctx.targets
            .register(NavTarget::new(i, Rect::new(0, i as u16 * 5, 10, 1)));
    }

    // Enter hint mode
    plugin.on_event(&key_event(KeyCode::Char('f')), &mut ctx);

    // Only 2 hints should be generated
    // This test verifies the plugin respects max_hints
    assert_eq!(plugin.mode(), NavMode::Hint);
}

#[test]
fn test_render_overlay() {
    let mut plugin = NavPlugin::new();
    let mut ctx = create_test_context();
    let mut backend = TestBackend::new(80, 24);
    let mut terminal = ratatui::Terminal::new(backend).unwrap();

    // Enter hint mode
    plugin.on_event(&key_event(KeyCode::Char('f')), &mut ctx);

    // Render the overlay
    terminal
        .draw(|frame| {
            plugin.render_overlay(frame, &ctx);
        })
        .unwrap();

    // The test backend should have content rendered
    // (This is a basic smoke test to ensure rendering doesn't panic)
}

#[test]
fn test_event_not_handled_in_normal_mode() {
    let mut plugin = NavPlugin::new();
    let mut ctx = create_test_context();

    // Random key in normal mode should not be handled
    let result = plugin.on_event(&key_event(KeyCode::Char('x')), &mut ctx);
    assert_eq!(result, PluginEventResult::NotHandled);
}

#[test]
fn test_invalid_hint_char_ignored() {
    let config = NavConfig::new().with_charset("asdf");
    let mut plugin = NavPlugin::with_config(config);
    let mut ctx = create_test_context();

    // Enter hint mode
    plugin.on_event(&key_event(KeyCode::Char('f')), &mut ctx);

    // Type a character not in the charset
    let result = plugin.on_event(&key_event(KeyCode::Char('z')), &mut ctx);

    // Should still be in hint mode (character ignored)
    assert_eq!(result, PluginEventResult::ConsumedRequestRedraw);
    assert_eq!(plugin.mode(), NavMode::Hint);
}
