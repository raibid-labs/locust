use locust::core::targets::{NavTarget, TargetAction, TargetPriority, TargetState, TargetRegistry, TargetBuilder};
use ratatui::layout::Rect;

#[test]
fn test_complete_target_lifecycle() {
    let mut registry = TargetRegistry::new();

    // Frame 1: Register targets
    registry.register(
        NavTarget::new(1, Rect::new(0, 0, 10, 1))
            .with_label("Button 1")
            .with_action(TargetAction::Activate)
            .with_priority(TargetPriority::High)
    );
    registry.register(
        NavTarget::new(2, Rect::new(0, 2, 10, 1))
            .with_label("Button 2")
            .with_priority(TargetPriority::Normal)
    );

    assert_eq!(registry.len(), 2);

    // User interaction: Select a target
    if let Some(target) = registry.by_id_mut(1) {
        target.state = TargetState::Selected;
    }

    let selected = registry.by_state(TargetState::Selected);
    assert_eq!(selected.len(), 1);
    assert_eq!(selected[0].id, 1);

    // Frame 2: Clear and re-register (simulating next frame)
    registry.clear();
    assert_eq!(registry.len(), 0);

    registry.register(
        NavTarget::new(1, Rect::new(0, 0, 10, 1))
            .with_label("Button 1")
            .with_state(TargetState::Selected) // Restore state
    );
    registry.register(
        NavTarget::new(2, Rect::new(0, 2, 10, 1))
            .with_label("Button 2")
    );

    assert_eq!(registry.len(), 2);
}

#[test]
fn test_multi_frame_interaction_scenario() {
    let mut registry = TargetRegistry::new();
    let mut builder = TargetBuilder::new();

    // Frame 1: Display a list
    for i in 0..5 {
        let target = builder.list_item(
            Rect::new(0, i * 2, 20, 1),
            format!("Item {}", i + 1)
        );
        registry.register(target);
    }

    assert_eq!(registry.len(), 5);

    // User hovers over item 3
    if let Some(target) = registry.by_id_mut(3) {
        target.state = TargetState::Highlighted;
    }

    // Find highlighted targets
    let highlighted = registry.by_state(TargetState::Highlighted);
    assert_eq!(highlighted.len(), 1);

    // User clicks on highlighted item
    if let Some(target) = registry.by_id_mut(3) {
        target.state = TargetState::Selected;
    }

    let selected = registry.by_state(TargetState::Selected);
    assert_eq!(selected.len(), 1);
    assert_eq!(selected[0].label, Some("Item 3".to_string()));
}

#[test]
fn test_spatial_query_workflow() {
    let mut registry = TargetRegistry::new();

    // Create a grid of targets
    for row in 0..3 {
        for col in 0..3 {
            registry.register(
                NavTarget::new(
                    (row * 3 + col + 1) as u64,
                    Rect::new(col * 10, row * 10, 8, 8)
                )
                .with_label(format!("Cell {},{}", row, col))
            );
        }
    }

    assert_eq!(registry.len(), 9);

    // Find targets in top-left quadrant
    let area = Rect::new(0, 0, 15, 15);
    let targets = registry.in_area(area);
    assert_eq!(targets.len(), 4); // 2x2 grid in top-left

    // Find target closest to center
    let closest = registry.closest_to(15, 15).unwrap();
    assert_eq!(closest.label, Some("Cell 1,1".to_string()));

    // Find targets at specific point
    let targets = registry.at_point(5, 5);
    assert_eq!(targets.len(), 1);
    assert_eq!(targets[0].label, Some("Cell 0,0".to_string()));
}

#[test]
fn test_priority_based_hint_generation() {
    let mut registry = TargetRegistry::new();

    // Register targets with different priorities
    registry.register(
        NavTarget::new(1, Rect::new(0, 0, 10, 1))
            .with_label("Save")
            .with_priority(TargetPriority::Critical)
    );
    registry.register(
        NavTarget::new(2, Rect::new(0, 2, 10, 1))
            .with_label("Cancel")
            .with_priority(TargetPriority::High)
    );
    registry.register(
        NavTarget::new(3, Rect::new(0, 4, 10, 1))
            .with_label("Info")
            .with_priority(TargetPriority::Normal)
    );
    registry.register(
        NavTarget::new(4, Rect::new(0, 6, 10, 1))
            .with_label("Background")
            .with_priority(TargetPriority::Low)
    );

    // Get targets sorted by priority for hint generation
    let sorted = registry.sorted_by_priority();

    assert_eq!(sorted[0].label, Some("Save".to_string()));
    assert_eq!(sorted[1].label, Some("Cancel".to_string()));
    assert_eq!(sorted[2].label, Some("Info".to_string()));
    assert_eq!(sorted[3].label, Some("Background".to_string()));
}

#[test]
fn test_grouped_navigation() {
    let mut registry = TargetRegistry::new();
    let mut builder = TargetBuilder::new();

    // Create tab group
    for i in 0..3 {
        let tab = builder.tab(Rect::new(i * 10, 0, 8, 1), format!("Tab {}", i + 1));
        registry.register(tab);
    }

    // Create button group
    for i in 0..2 {
        let button = builder.button(Rect::new(i * 12, 5, 10, 2), format!("Btn {}", i + 1))
            .with_group("buttons");
        registry.register(button);
    }

    // Query by group
    let tabs = registry.by_group("tabs");
    assert_eq!(tabs.len(), 3);

    let buttons = registry.by_group("buttons");
    assert_eq!(buttons.len(), 2);

    // Navigate within group
    for tab in tabs {
        assert_eq!(tab.action, TargetAction::Activate);
        assert_eq!(tab.priority, TargetPriority::High);
    }
}

#[test]
fn test_target_builder_integration() {
    let mut registry = TargetRegistry::new();
    let mut builder = TargetBuilder::new();

    // Build a complex UI
    let header_button = builder.button(Rect::new(0, 0, 10, 2), "Menu");
    registry.register(header_button);

    let tab1 = builder.tab(Rect::new(0, 3, 8, 1), "Home");
    let tab2 = builder.tab(Rect::new(9, 3, 8, 1), "Settings");
    registry.register(tab1);
    registry.register(tab2);

    let list_item1 = builder.list_item(Rect::new(0, 5, 20, 1), "Item 1");
    let list_item2 = builder.list_item(Rect::new(0, 7, 20, 1), "Item 2");
    registry.register(list_item1);
    registry.register(list_item2);

    let link = builder.link(Rect::new(0, 10, 15, 1), "View More", "/more");
    registry.register(link);

    assert_eq!(registry.len(), 6);

    // Verify different target types
    let high_priority = registry.by_priority(TargetPriority::High);
    assert_eq!(high_priority.len(), 3); // button + 2 tabs

    let tabs = registry.by_group("tabs");
    assert_eq!(tabs.len(), 2);
}

#[test]
fn test_dynamic_target_updates() {
    let mut registry = TargetRegistry::new();

    // Initial state
    registry.register(
        NavTarget::new(1, Rect::new(0, 0, 10, 1))
            .with_label("Loading...")
            .with_state(TargetState::Disabled)
    );

    // Simulate async load completion
    if let Some(target) = registry.by_id_mut(1) {
        target.label = Some("Click Here".to_string());
        target.state = TargetState::Normal;
    }

    let target = registry.by_id(1).unwrap();
    assert_eq!(target.label, Some("Click Here".to_string()));
    assert_eq!(target.state, TargetState::Normal);
}

#[test]
fn test_overlapping_targets_selection() {
    let mut registry = TargetRegistry::new();

    // Create overlapping targets with different priorities
    registry.register(
        NavTarget::new(1, Rect::new(0, 0, 20, 20))
            .with_label("Background")
            .with_priority(TargetPriority::Low)
    );
    registry.register(
        NavTarget::new(2, Rect::new(5, 5, 10, 10))
            .with_label("Overlay")
            .with_priority(TargetPriority::High)
    );
    registry.register(
        NavTarget::new(3, Rect::new(7, 7, 6, 6))
            .with_label("Button")
            .with_priority(TargetPriority::Critical)
    );

    // Get all targets at center point
    let targets = registry.at_point(10, 10);
    assert_eq!(targets.len(), 3);

    // Sort by priority to select the right one
    let mut sorted_targets: Vec<_> = targets.into_iter().collect();
    sorted_targets.sort_by(|a, b| b.priority.cmp(&a.priority));

    assert_eq!(sorted_targets[0].label, Some("Button".to_string()));
}

#[test]
fn test_metadata_usage() {
    let mut registry = TargetRegistry::new();
    let mut builder = TargetBuilder::new();

    // Create expandable tree nodes
    let node1 = builder.tree_node(Rect::new(0, 0, 20, 1), "Folder 1", false);
    let node2 = builder.tree_node(Rect::new(0, 2, 20, 1), "Folder 2", true);

    registry.register(node1);
    registry.register(node2);

    // Check metadata
    let target1 = registry.by_id(1).unwrap();
    assert_eq!(target1.metadata.get("expanded"), Some(&"false".to_string()));

    let target2 = registry.by_id(2).unwrap();
    assert_eq!(target2.metadata.get("expanded"), Some(&"true".to_string()));

    // Toggle expansion
    if let Some(target) = registry.by_id_mut(1) {
        target.metadata.insert("expanded".to_string(), "true".to_string());
    }

    let target1 = registry.by_id(1).unwrap();
    assert_eq!(target1.metadata.get("expanded"), Some(&"true".to_string()));
}

#[test]
fn test_large_target_set_performance() {
    let mut registry = TargetRegistry::new();

    // Register many targets
    for i in 0..1000 {
        registry.register(
            NavTarget::new(i, Rect::new((i % 50) * 10, (i / 50) * 2, 8, 1))
                .with_label(format!("Target {}", i))
                .with_priority(match i % 4 {
                    0 => TargetPriority::Critical,
                    1 => TargetPriority::High,
                    2 => TargetPriority::Normal,
                    _ => TargetPriority::Low,
                })
        );
    }

    assert_eq!(registry.len(), 1000);

    // Spatial query should still be fast
    let targets = registry.at_point(45, 10);
    assert!(!targets.is_empty());

    // Priority filtering
    let high_priority = registry.by_priority(TargetPriority::Critical);
    assert_eq!(high_priority.len(), 250);

    // Closest target query
    let closest = registry.closest_to(100, 20);
    assert!(closest.is_some());
}
