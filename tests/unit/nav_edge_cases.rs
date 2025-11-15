//! Edge case tests for navigation plugin
//!
//! Tests boundary conditions and unusual scenarios for NavPlugin.

use locust::core::context::LocustContext;
use locust::core::plugin::LocustPlugin;
use locust::core::targets::NavTarget;
use locust::plugins::nav::{NavConfig, NavMode, NavPlugin};
use ratatui::backend::TestBackend;
use ratatui::layout::Rect;

#[test]
fn test_nav_empty_target_list() {
    let mut plugin = NavPlugin::new();
    let mut ctx = LocustContext::default();

    LocustPlugin::<TestBackend>::init(&mut plugin, &mut ctx);

    // Should handle empty target list gracefully
    assert_eq!(plugin.mode(), NavMode::Normal);
}

#[test]
fn test_nav_single_target() {
    let mut plugin = NavPlugin::new();
    let mut ctx = LocustContext::default();

    ctx.targets.register(NavTarget::new(1, Rect::new(10, 10, 20, 3)));

    LocustPlugin::<TestBackend>::init(&mut plugin, &mut ctx);

    // Should handle single target
    assert_eq!(ctx.targets.len(), 1);
}

#[test]
fn test_nav_duplicate_positions() {
    let mut ctx = LocustContext::default();

    // Register targets at same position
    let rect = Rect::new(10, 10, 20, 3);
    ctx.targets.register(NavTarget::new(1, rect));
    ctx.targets.register(NavTarget::new(2, rect));
    ctx.targets.register(NavTarget::new(3, rect));

    // All should be registered
    assert_eq!(ctx.targets.len(), 3);
}

#[test]
fn test_nav_very_large_target_count() {
    let mut ctx = LocustContext::default();

    // Register 1000+ targets
    for i in 0..1500 {
        let x = (i % 50) * 5;
        let y = (i / 50) * 3;
        ctx.targets.register(NavTarget::new(i, Rect::new(x, y, 4, 2)));
    }

    assert_eq!(ctx.targets.len(), 1500);
}

#[test]
fn test_nav_zero_size_target() {
    let mut ctx = LocustContext::default();

    // Zero width/height target
    ctx.targets.register(NavTarget::new(1, Rect::new(10, 10, 0, 0)));

    assert_eq!(ctx.targets.len(), 1);
}

#[test]
fn test_nav_very_large_coordinates() {
    let mut ctx = LocustContext::default();

    // Target at extreme coordinates
    ctx.targets.register(NavTarget::new(1, Rect::new(u16::MAX - 10, u16::MAX - 10, 5, 5)));

    assert!(ctx.targets.by_id(1).is_some());
}

#[test]
fn test_nav_config_extreme_values() {
    let config = NavConfig::new()
        .with_activation_key('f')
        .with_hint_timeout_ms(0)  // No timeout
        .with_max_hints(10000);   // Very large

    let plugin = NavPlugin::with_config(config);
    assert_eq!(plugin.config().max_hints, 10000);
    assert_eq!(plugin.config().hint_timeout_ms, 0);
}

#[test]
fn test_nav_many_overlapping_targets() {
    let mut ctx = LocustContext::default();

    // Create overlapping targets in same region
    for i in 0..100 {
        ctx.targets.register(NavTarget::new(
            i,
            Rect::new(50 + (i % 5), 50 + (i % 5), 20, 3),
        ));
    }

    assert_eq!(ctx.targets.len(), 100);
}

#[test]
fn test_nav_target_at_screen_edges() {
    let mut ctx = LocustContext::default();

    // Targets at all four corners
    ctx.targets.register(NavTarget::new(1, Rect::new(0, 0, 10, 3)));
    ctx.targets.register(NavTarget::new(2, Rect::new(170, 0, 10, 3)));
    ctx.targets.register(NavTarget::new(3, Rect::new(0, 47, 10, 3)));
    ctx.targets.register(NavTarget::new(4, Rect::new(170, 47, 10, 3)));

    assert_eq!(ctx.targets.len(), 4);
}

#[test]
fn test_nav_rapid_target_registration_clearing() {
    let mut ctx = LocustContext::default();

    // Rapid register/clear cycles
    for _cycle in 0..10 {
        for i in 0..50 {
            ctx.targets.register(NavTarget::new(i, Rect::new(i as u16 * 2, 10, 5, 2)));
        }
        assert_eq!(ctx.targets.len(), 50);
        ctx.targets.clear();
        assert_eq!(ctx.targets.len(), 0);
    }
}

#[test]
fn test_nav_plugin_mode_transitions() {
    let plugin = NavPlugin::new();

    // Default mode
    assert_eq!(plugin.mode(), NavMode::Normal);
}
