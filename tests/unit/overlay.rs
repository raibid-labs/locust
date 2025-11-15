//! Unit tests for overlay management
//!
//! Tests for overlay state and layer management.

use locust::core::overlay::{OverlayLayer, OverlayState};

#[test]
fn test_overlay_state_creation() {
    let state = OverlayState::default();
    assert!(!state.has_overlay);
    assert_eq!(state.layers.len(), 0);
}

#[test]
fn test_overlay_mark_has_overlay() {
    let mut state = OverlayState::default();
    assert!(!state.has_overlay);

    state.mark_has_overlay();
    assert!(state.has_overlay);
}

#[test]
fn test_overlay_layer_creation() {
    let layer = OverlayLayer::new("test.plugin", 50);
    assert_eq!(layer.id, "test.plugin");
    assert_eq!(layer.z_index, 50);
    assert!(layer.visible);
}

#[test]
fn test_overlay_add_layer() {
    let mut state = OverlayState::default();
    let layer = OverlayLayer::new("plugin.nav", 100);

    state.add_layer(layer);
    assert_eq!(state.layers.len(), 1);
}

#[test]
fn test_overlay_remove_layer() {
    let mut state = OverlayState::default();
    state.add_layer(OverlayLayer::new("plugin.a", 50));
    state.add_layer(OverlayLayer::new("plugin.b", 60));

    assert_eq!(state.layers.len(), 2);

    state.remove_layer("plugin.a");
    assert_eq!(state.layers.len(), 1);
    assert_eq!(state.layers[0].id, "plugin.b");
}

#[test]
fn test_overlay_z_ordering() {
    let mut state = OverlayState::default();

    // Add in reverse z-order
    state.add_layer(OverlayLayer::new("low", 10));
    state.add_layer(OverlayLayer::new("high", 100));
    state.add_layer(OverlayLayer::new("medium", 50));

    // Should be sorted by z_index
    let z_indices: Vec<i32> = state.layers.iter().map(|l| l.z_index).collect();
    assert_eq!(z_indices, vec![10, 50, 100]);
}

#[test]
fn test_overlay_layer_visibility() {
    let mut layer = OverlayLayer::new("test", 50);
    assert!(layer.visible);

    layer.visible = false;
    assert!(!layer.visible);
}

#[test]
fn test_overlay_duplicate_layer_id() {
    let mut state = OverlayState::default();

    state.add_layer(OverlayLayer::new("duplicate", 50));
    state.add_layer(OverlayLayer::new("duplicate", 60));

    // Second should replace first
    assert_eq!(state.layers.len(), 1);
    assert_eq!(state.layers[0].z_index, 60);
}

#[test]
fn test_overlay_clear_all_layers() {
    let mut state = OverlayState::default();

    state.add_layer(OverlayLayer::new("a", 10));
    state.add_layer(OverlayLayer::new("b", 20));
    state.add_layer(OverlayLayer::new("c", 30));

    state.layers.clear();
    assert_eq!(state.layers.len(), 0);
}

#[test]
fn test_overlay_multiple_operations() {
    let mut state = OverlayState::default();

    state.mark_has_overlay();
    state.add_layer(OverlayLayer::new("nav", 50));
    state.add_layer(OverlayLayer::new("omnibar", 40));

    assert!(state.has_overlay);
    assert_eq!(state.layers.len(), 2);

    state.remove_layer("nav");
    assert_eq!(state.layers.len(), 1);
}
