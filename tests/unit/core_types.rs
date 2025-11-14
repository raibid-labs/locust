//! Unit tests for core types.

use locust::prelude::*;
use ratatui::layout::Rect;

#[test]
fn test_nav_target_creation() {
    let rect = Rect::new(10, 20, 30, 40);
    let target = NavTarget::new(1, rect);

    assert_eq!(target.id, 1);
    assert_eq!(target.rect, rect);
    assert_eq!(target.label, None);
    assert_eq!(target.priority, TargetPriority::Normal);
    assert_eq!(target.state, TargetState::Normal);
}

#[test]
fn test_nav_target_builder() {
    let rect = Rect::new(0, 0, 100, 50);
    let target = NavTarget::new(42, rect)
        .with_label("Test Button")
        .with_priority(TargetPriority::High)
        .with_group("buttons");

    assert_eq!(target.id, 42);
    assert_eq!(target.label, Some("Test Button".to_string()));
    assert_eq!(target.priority, TargetPriority::High);
    assert_eq!(target.group, Some("buttons".to_string()));
}

#[test]
fn test_nav_target_contains() {
    let rect = Rect::new(10, 20, 30, 40);
    let target = NavTarget::new(1, rect);

    // Inside
    assert!(target.contains_point(10, 20));
    assert!(target.contains_point(15, 25));
    assert!(target.contains_point(39, 59));

    // Outside
    assert!(!target.contains_point(9, 20));
    assert!(!target.contains_point(10, 19));
    assert!(!target.contains_point(40, 20));
    assert!(!target.contains_point(10, 60));
}

#[test]
fn test_target_registry_basic() {
    let mut registry = TargetRegistry::new();

    assert!(registry.is_empty());
    assert_eq!(registry.len(), 0);

    let target = NavTarget::new(1, Rect::new(0, 0, 10, 10));
    registry.register(target);

    assert!(!registry.is_empty());
    assert_eq!(registry.len(), 1);

    registry.clear();
    assert!(registry.is_empty());
}

#[test]
fn test_target_registry_find_by_id() {
    let mut registry = TargetRegistry::new();

    registry.register(NavTarget::new(1, Rect::new(0, 0, 10, 10)));
    registry.register(NavTarget::new(2, Rect::new(10, 10, 20, 20)));
    registry.register(NavTarget::new(3, Rect::new(20, 20, 30, 30)));

    assert!(registry.by_id(1).is_some());
    assert!(registry.by_id(2).is_some());
    assert!(registry.by_id(3).is_some());
    assert!(registry.by_id(999).is_none());

    let target = registry.by_id(2).unwrap();
    assert_eq!(target.id, 2);
}

#[test]
fn test_target_registry_find_at_point() {
    let mut registry = TargetRegistry::new();

    registry.register(NavTarget::new(1, Rect::new(0, 0, 10, 10)));
    registry.register(NavTarget::new(2, Rect::new(5, 5, 10, 10)));
    registry.register(NavTarget::new(3, Rect::new(20, 20, 10, 10)));

    let at_origin = registry.at_point(0, 0);
    assert_eq!(at_origin.len(), 1);
    assert_eq!(at_origin[0].id, 1);

    let at_overlap = registry.at_point(7, 7);
    assert_eq!(at_overlap.len(), 2);

    let at_far = registry.at_point(25, 25);
    assert_eq!(at_far.len(), 1);
    assert_eq!(at_far[0].id, 3);

    let at_empty = registry.at_point(100, 100);
    assert_eq!(at_empty.len(), 0);
}

#[test]
fn test_target_registry_filter_by_group() {
    let mut registry = TargetRegistry::new();

    registry.register(NavTarget::new(1, Rect::new(0, 0, 10, 10)).with_group("buttons"));
    registry.register(NavTarget::new(2, Rect::new(10, 10, 20, 20)).with_group("links"));
    registry.register(NavTarget::new(3, Rect::new(20, 20, 30, 30)).with_group("buttons"));

    let buttons = registry.by_group("buttons");
    assert_eq!(buttons.len(), 2);
    assert!(buttons.iter().any(|t| t.id == 1));
    assert!(buttons.iter().any(|t| t.id == 3));

    let links = registry.by_group("links");
    assert_eq!(links.len(), 1);
    assert_eq!(links[0].id, 2);

    let inputs = registry.by_group("inputs");
    assert_eq!(inputs.len(), 0);
}

#[test]
fn test_target_registry_sort_by_priority() {
    let mut registry = TargetRegistry::new();

    registry
        .register(NavTarget::new(1, Rect::new(0, 0, 10, 10)).with_priority(TargetPriority::Low));
    registry.register(
        NavTarget::new(2, Rect::new(10, 10, 20, 20)).with_priority(TargetPriority::Critical),
    );
    registry.register(
        NavTarget::new(3, Rect::new(20, 20, 30, 30)).with_priority(TargetPriority::Normal),
    );

    let sorted = registry.sorted_by_priority();

    assert_eq!(sorted[0].id, 2); // Critical
    assert_eq!(sorted[1].id, 3); // Normal
    assert_eq!(sorted[2].id, 1); // Low
}

#[test]
fn test_overlay_state_basic() {
    let mut state = OverlayState::new();

    assert!(!state.has_overlay);
    assert_eq!(state.total_overlay_frames, 0);

    state.mark_has_overlay();
    assert!(state.has_overlay);
    assert_eq!(state.total_overlay_frames, 1);

    // Multiple marks in same frame don't increment
    state.mark_has_overlay();
    assert_eq!(state.total_overlay_frames, 1);

    // New frame resets
    state.begin_frame();
    assert!(!state.has_overlay);

    state.mark_has_overlay();
    assert_eq!(state.total_overlay_frames, 2);
}

#[test]
fn test_overlay_layer_management() {
    let mut state = OverlayState::new();

    let layer1 = OverlayLayer::new("plugin1", 100);
    let layer2 = OverlayLayer::new("plugin2", 50);
    let layer3 = OverlayLayer::new("plugin3", 200);

    state.add_layer(layer1);
    state.add_layer(layer2);
    state.add_layer(layer3);

    assert_eq!(state.layers().len(), 3);

    // Check z-index sorting
    let layers = state.layers();
    assert_eq!(layers[0].plugin_id, "plugin2"); // z=50
    assert_eq!(layers[1].plugin_id, "plugin1"); // z=100
    assert_eq!(layers[2].plugin_id, "plugin3"); // z=200

    // Check layer existence
    assert!(state.has_layer("plugin1"));
    assert!(state.has_layer("plugin2"));
    assert!(state.has_layer("plugin3"));
    assert!(!state.has_layer("plugin4"));

    // Remove a layer
    state.remove_layer("plugin2");
    assert_eq!(state.layers().len(), 2);
    assert!(!state.has_layer("plugin2"));
}

#[test]
fn test_overlay_layer_visibility() {
    let mut state = OverlayState::new();

    state.add_layer(OverlayLayer::new("plugin1", 100));

    assert!(state.has_layer("plugin1"));

    state.set_layer_visibility("plugin1", false);
    assert!(!state.has_layer("plugin1")); // Not visible

    state.set_layer_visibility("plugin1", true);
    assert!(state.has_layer("plugin1")); // Visible again
}

#[test]
fn test_overlay_clear_layers() {
    let mut state = OverlayState::new();

    state.add_layer(OverlayLayer::new("plugin1", 100));
    state.add_layer(OverlayLayer::new("plugin2", 200));

    assert_eq!(state.layers().len(), 2);

    state.clear_layers();
    assert_eq!(state.layers().len(), 0);
}

#[test]
fn test_locust_context_frame_counter() {
    let mut ctx = LocustContext::default();

    assert_eq!(ctx.frame_count, 0);

    // Simulate frame lifecycle
    ctx.frame_count = ctx.frame_count.wrapping_add(1);
    assert_eq!(ctx.frame_count, 1);

    ctx.frame_count = ctx.frame_count.wrapping_add(1);
    assert_eq!(ctx.frame_count, 2);
}

#[test]
fn test_plugin_event_result() {
    let not_handled = PluginEventResult::NotHandled;
    assert!(!not_handled.is_consumed());
    assert!(!not_handled.requests_redraw());

    let consumed = PluginEventResult::Consumed;
    assert!(consumed.is_consumed());
    assert!(!consumed.requests_redraw());

    let consumed_redraw = PluginEventResult::ConsumedRequestRedraw;
    assert!(consumed_redraw.is_consumed());
    assert!(consumed_redraw.requests_redraw());
}

#[test]
fn test_locust_event_outcome() {
    let not_handled = LocustEventOutcome::NOT_HANDLED;
    assert!(!not_handled.consumed);
    assert!(!not_handled.request_redraw);

    let consumed = LocustEventOutcome::CONSUMED;
    assert!(consumed.consumed);
    assert!(!consumed.request_redraw);

    let consumed_redraw = LocustEventOutcome::CONSUMED_REDRAW;
    assert!(consumed_redraw.consumed);
    assert!(consumed_redraw.request_redraw);
}

#[test]
fn test_target_builder() {
    let mut builder = TargetBuilder::new();

    let button = builder.button(Rect::new(0, 0, 10, 3), "Click Me");
    assert_eq!(button.action, TargetAction::Activate);
    assert_eq!(button.priority, TargetPriority::High);

    let item = builder.list_item(Rect::new(0, 5, 20, 1), "Item 1");
    assert_eq!(item.action, TargetAction::Select);
    assert_eq!(item.priority, TargetPriority::Normal);
}
