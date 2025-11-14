use locust::core::targets::{NavTarget, TargetAction, TargetPriority, TargetState, TargetRegistry, TargetBuilder};
use ratatui::layout::Rect;

// ===== TargetAction Tests =====

#[test]
fn test_target_action_default() {
    assert_eq!(TargetAction::default(), TargetAction::Activate);
}

#[test]
fn test_target_action_equality() {
    assert_eq!(TargetAction::Select, TargetAction::Select);
    assert_eq!(TargetAction::Activate, TargetAction::Activate);
    assert_ne!(TargetAction::Select, TargetAction::Activate);

    let nav1 = TargetAction::Navigate("/home".to_string());
    let nav2 = TargetAction::Navigate("/home".to_string());
    let nav3 = TargetAction::Navigate("/settings".to_string());
    assert_eq!(nav1, nav2);
    assert_ne!(nav1, nav3);
}

// ===== TargetState Tests =====

#[test]
fn test_target_state_default() {
    assert_eq!(TargetState::default(), TargetState::Normal);
}

#[test]
fn test_target_state_transitions() {
    let mut target = NavTarget::new(1, Rect::new(0, 0, 10, 1));
    assert_eq!(target.state, TargetState::Normal);

    target = target.with_state(TargetState::Highlighted);
    assert_eq!(target.state, TargetState::Highlighted);

    target = target.with_state(TargetState::Selected);
    assert_eq!(target.state, TargetState::Selected);

    target = target.with_state(TargetState::Disabled);
    assert_eq!(target.state, TargetState::Disabled);
}

// ===== TargetPriority Tests =====

#[test]
fn test_target_priority_ordering() {
    assert!(TargetPriority::Critical > TargetPriority::High);
    assert!(TargetPriority::High > TargetPriority::Normal);
    assert!(TargetPriority::Normal > TargetPriority::Low);

    let mut priorities = vec![
        TargetPriority::Low,
        TargetPriority::Critical,
        TargetPriority::Normal,
        TargetPriority::High,
    ];
    priorities.sort();

    assert_eq!(priorities[0], TargetPriority::Low);
    assert_eq!(priorities[1], TargetPriority::Normal);
    assert_eq!(priorities[2], TargetPriority::High);
    assert_eq!(priorities[3], TargetPriority::Critical);
}

// ===== NavTarget Tests =====

#[test]
fn test_target_builder_pattern() {
    let target = NavTarget::new(42, Rect::new(10, 20, 30, 40))
        .with_label("Test Target")
        .with_action(TargetAction::Scroll)
        .with_priority(TargetPriority::High)
        .with_state(TargetState::Highlighted)
        .with_group("navigation")
        .with_metadata("key1", "value1")
        .with_metadata("key2", "value2");

    assert_eq!(target.id, 42);
    assert_eq!(target.rect, Rect::new(10, 20, 30, 40));
    assert_eq!(target.label, Some("Test Target".to_string()));
    assert_eq!(target.action, TargetAction::Scroll);
    assert_eq!(target.priority, TargetPriority::High);
    assert_eq!(target.state, TargetState::Highlighted);
    assert_eq!(target.group, Some("navigation".to_string()));
    assert_eq!(target.metadata.get("key1"), Some(&"value1".to_string()));
    assert_eq!(target.metadata.get("key2"), Some(&"value2".to_string()));
}

#[test]
fn test_target_contains_point_edge_cases() {
    let target = NavTarget::new(1, Rect::new(10, 10, 20, 20));

    // Inside
    assert!(target.contains_point(15, 15));
    assert!(target.contains_point(20, 20));

    // Top-left corner (inclusive)
    assert!(target.contains_point(10, 10));

    // Bottom-right corner (exclusive)
    assert!(!target.contains_point(30, 30));
    assert!(target.contains_point(29, 29));

    // Edges
    assert!(target.contains_point(10, 15));
    assert!(target.contains_point(15, 10));
    assert!(!target.contains_point(30, 15));
    assert!(!target.contains_point(15, 30));
}

#[test]
fn test_target_overlaps_rect_cases() {
    let target = NavTarget::new(1, Rect::new(10, 10, 20, 20));

    // Fully contained
    assert!(target.overlaps_rect(&Rect::new(15, 15, 5, 5)));

    // Partially overlapping
    assert!(target.overlaps_rect(&Rect::new(5, 5, 10, 10)));
    assert!(target.overlaps_rect(&Rect::new(25, 25, 10, 10)));
    assert!(target.overlaps_rect(&Rect::new(5, 15, 10, 5)));
    assert!(target.overlaps_rect(&Rect::new(25, 15, 10, 5)));

    // Fully containing
    assert!(target.overlaps_rect(&Rect::new(5, 5, 30, 30)));

    // No overlap
    assert!(!target.overlaps_rect(&Rect::new(50, 50, 10, 10)));
    assert!(!target.overlaps_rect(&Rect::new(0, 0, 5, 5)));

    // Adjacent but not overlapping
    assert!(!target.overlaps_rect(&Rect::new(30, 10, 10, 10)));
    assert!(!target.overlaps_rect(&Rect::new(10, 30, 10, 10)));
}

#[test]
fn test_target_center_calculation() {
    let target1 = NavTarget::new(1, Rect::new(0, 0, 10, 10));
    assert_eq!(target1.center(), (5, 5));

    let target2 = NavTarget::new(2, Rect::new(10, 20, 20, 40));
    assert_eq!(target2.center(), (20, 40));

    // Odd dimensions
    let target3 = NavTarget::new(3, Rect::new(0, 0, 11, 11));
    assert_eq!(target3.center(), (5, 5)); // Integer division
}

#[test]
fn test_target_area_calculation() {
    let target1 = NavTarget::new(1, Rect::new(0, 0, 10, 10));
    assert_eq!(target1.area(), 100);

    let target2 = NavTarget::new(2, Rect::new(0, 0, 1, 1));
    assert_eq!(target2.area(), 1);

    let target3 = NavTarget::new(3, Rect::new(0, 0, 100, 50));
    assert_eq!(target3.area(), 5000);
}

// ===== TargetRegistry Tests =====

#[test]
fn test_registry_replace_existing_target() {
    let mut registry = TargetRegistry::new();

    registry.register(NavTarget::new(1, Rect::new(0, 0, 10, 1)).with_label("First"));
    registry.register(NavTarget::new(1, Rect::new(0, 0, 10, 1)).with_label("Second"));

    assert_eq!(registry.len(), 1);
    let target = registry.by_id(1).unwrap();
    assert_eq!(target.label, Some("Second".to_string()));
}

#[test]
fn test_registry_by_id_mut() {
    let mut registry = TargetRegistry::new();
    registry.register(NavTarget::new(1, Rect::new(0, 0, 10, 1)).with_label("Original"));

    if let Some(target) = registry.by_id_mut(1) {
        target.label = Some("Modified".to_string());
        target.state = TargetState::Selected;
    }

    let target = registry.by_id(1).unwrap();
    assert_eq!(target.label, Some("Modified".to_string()));
    assert_eq!(target.state, TargetState::Selected);
}

#[test]
fn test_registry_at_point_overlapping() {
    let mut registry = TargetRegistry::new();

    // Create overlapping targets
    registry.register(NavTarget::new(1, Rect::new(0, 0, 20, 20)));
    registry.register(NavTarget::new(2, Rect::new(10, 10, 20, 20)));
    registry.register(NavTarget::new(3, Rect::new(15, 15, 5, 5)));

    let targets = registry.at_point(5, 5);
    assert_eq!(targets.len(), 1);
    assert_eq!(targets[0].id, 1);

    let targets = registry.at_point(17, 17);
    assert_eq!(targets.len(), 3); // All three overlap at this point
}

#[test]
fn test_registry_in_area() {
    let mut registry = TargetRegistry::new();

    registry.register(NavTarget::new(1, Rect::new(0, 0, 10, 10)));
    registry.register(NavTarget::new(2, Rect::new(20, 20, 10, 10)));
    registry.register(NavTarget::new(3, Rect::new(5, 5, 10, 10)));

    let area = Rect::new(0, 0, 15, 15);
    let targets = registry.in_area(area);
    assert_eq!(targets.len(), 2); // Targets 1 and 3 overlap with area
}

#[test]
fn test_registry_by_priority() {
    let mut registry = TargetRegistry::new();

    registry.register(
        NavTarget::new(1, Rect::new(0, 0, 10, 1))
            .with_priority(TargetPriority::High)
    );
    registry.register(
        NavTarget::new(2, Rect::new(0, 2, 10, 1))
            .with_priority(TargetPriority::Normal)
    );
    registry.register(
        NavTarget::new(3, Rect::new(0, 4, 10, 1))
            .with_priority(TargetPriority::High)
    );

    let high_priority = registry.by_priority(TargetPriority::High);
    assert_eq!(high_priority.len(), 2);
}

#[test]
fn test_registry_by_group() {
    let mut registry = TargetRegistry::new();

    registry.register(NavTarget::new(1, Rect::new(0, 0, 10, 1)).with_group("buttons"));
    registry.register(NavTarget::new(2, Rect::new(0, 2, 10, 1)).with_group("buttons"));
    registry.register(NavTarget::new(3, Rect::new(0, 4, 10, 1)).with_group("tabs"));

    let buttons = registry.by_group("buttons");
    assert_eq!(buttons.len(), 2);

    let tabs = registry.by_group("tabs");
    assert_eq!(tabs.len(), 1);

    let nonexistent = registry.by_group("links");
    assert_eq!(nonexistent.len(), 0);
}

#[test]
fn test_registry_by_state() {
    let mut registry = TargetRegistry::new();

    registry.register(
        NavTarget::new(1, Rect::new(0, 0, 10, 1))
            .with_state(TargetState::Normal)
    );
    registry.register(
        NavTarget::new(2, Rect::new(0, 2, 10, 1))
            .with_state(TargetState::Selected)
    );
    registry.register(
        NavTarget::new(3, Rect::new(0, 4, 10, 1))
            .with_state(TargetState::Highlighted)
    );

    let selected = registry.by_state(TargetState::Selected);
    assert_eq!(selected.len(), 1);
    assert_eq!(selected[0].id, 2);
}

#[test]
fn test_registry_sorted_by_priority() {
    let mut registry = TargetRegistry::new();

    registry.register(
        NavTarget::new(1, Rect::new(0, 0, 10, 1))
            .with_priority(TargetPriority::Normal)
    );
    registry.register(
        NavTarget::new(2, Rect::new(0, 2, 10, 1))
            .with_priority(TargetPriority::Critical)
    );
    registry.register(
        NavTarget::new(3, Rect::new(0, 4, 10, 1))
            .with_priority(TargetPriority::Low)
    );

    let sorted = registry.sorted_by_priority();
    assert_eq!(sorted[0].id, 2); // Critical
    assert_eq!(sorted[1].id, 1); // Normal
    assert_eq!(sorted[2].id, 3); // Low
}

#[test]
fn test_registry_sorted_by_area() {
    let mut registry = TargetRegistry::new();

    registry.register(NavTarget::new(1, Rect::new(0, 0, 10, 10))); // area = 100
    registry.register(NavTarget::new(2, Rect::new(0, 0, 20, 20))); // area = 400
    registry.register(NavTarget::new(3, Rect::new(0, 0, 5, 5)));   // area = 25

    let sorted = registry.sorted_by_area();
    assert_eq!(sorted[0].id, 2); // Largest
    assert_eq!(sorted[1].id, 1);
    assert_eq!(sorted[2].id, 3); // Smallest
}

#[test]
fn test_registry_closest_to() {
    let mut registry = TargetRegistry::new();

    registry.register(NavTarget::new(1, Rect::new(10, 10, 10, 10)));
    let closest = registry.closest_to(0, 0).unwrap();
    assert_eq!(closest.id, 1);

    registry.register(NavTarget::new(2, Rect::new(50, 50, 10, 10)));
    registry.register(NavTarget::new(3, Rect::new(100, 100, 10, 10)));

    let closest = registry.closest_to(55, 55).unwrap();
    assert_eq!(closest.id, 2);
}

#[test]
fn test_registry_remove() {
    let mut registry = TargetRegistry::new();

    registry.register(NavTarget::new(1, Rect::new(0, 0, 10, 1)));
    registry.register(NavTarget::new(2, Rect::new(0, 2, 10, 1)));
    registry.register(NavTarget::new(3, Rect::new(0, 4, 10, 1)));

    assert_eq!(registry.len(), 3);

    // Remove existing target
    assert!(registry.remove(2));
    assert_eq!(registry.len(), 2);
    assert!(registry.by_id(2).is_none());
    assert!(registry.by_id(1).is_some());
    assert!(registry.by_id(3).is_some());

    // Try to remove non-existent target
    assert!(!registry.remove(99));
    assert_eq!(registry.len(), 2);
}

// ===== TargetBuilder Tests =====

#[test]
fn test_target_builder_id_generation() {
    let mut builder = TargetBuilder::new();

    let target1 = builder.button(Rect::new(0, 0, 10, 1), "Button 1");
    let target2 = builder.button(Rect::new(0, 2, 10, 1), "Button 2");
    let target3 = builder.list_item(Rect::new(0, 4, 10, 1), "Item 1");

    assert_eq!(target1.id, 1);
    assert_eq!(target2.id, 2);
    assert_eq!(target3.id, 3);
}

#[test]
fn test_target_builder_with_start_id() {
    let mut builder = TargetBuilder::with_start_id(100);

    let target1 = builder.button(Rect::new(0, 0, 10, 1), "Button 1");
    let target2 = builder.list_item(Rect::new(0, 2, 10, 1), "Item 1");

    assert_eq!(target1.id, 100);
    assert_eq!(target2.id, 101);
}

#[test]
fn test_target_builder_button() {
    let mut builder = TargetBuilder::new();
    let button = builder.button(Rect::new(0, 0, 10, 1), "Submit");

    assert_eq!(button.label, Some("Submit".to_string()));
    assert_eq!(button.action, TargetAction::Activate);
    assert_eq!(button.priority, TargetPriority::High);
}

#[test]
fn test_target_builder_list_item() {
    let mut builder = TargetBuilder::new();
    let item = builder.list_item(Rect::new(0, 0, 10, 1), "Item 1");

    assert_eq!(item.label, Some("Item 1".to_string()));
    assert_eq!(item.action, TargetAction::Select);
    assert_eq!(item.priority, TargetPriority::Normal);
}

#[test]
fn test_target_builder_tab() {
    let mut builder = TargetBuilder::new();
    let tab = builder.tab(Rect::new(0, 0, 8, 1), "Settings");

    assert_eq!(tab.label, Some("Settings".to_string()));
    assert_eq!(tab.action, TargetAction::Activate);
    assert_eq!(tab.priority, TargetPriority::High);
    assert_eq!(tab.group, Some("tabs".to_string()));
}

#[test]
fn test_target_builder_tree_node() {
    let mut builder = TargetBuilder::new();

    let collapsed = builder.tree_node(Rect::new(0, 0, 10, 1), "Folder", false);
    assert_eq!(collapsed.metadata.get("expanded"), Some(&"false".to_string()));

    let expanded = builder.tree_node(Rect::new(0, 2, 10, 1), "Folder", true);
    assert_eq!(expanded.metadata.get("expanded"), Some(&"true".to_string()));
}

#[test]
fn test_target_builder_link() {
    let mut builder = TargetBuilder::new();
    let link = builder.link(Rect::new(0, 0, 10, 1), "Go Home", "/home");

    assert_eq!(link.label, Some("Go Home".to_string()));
    assert_eq!(link.action, TargetAction::Navigate("/home".to_string()));
    assert_eq!(link.priority, TargetPriority::Normal);
}

#[test]
fn test_target_builder_custom() {
    let mut builder = TargetBuilder::new();
    let custom = builder.custom(
        Rect::new(0, 0, 10, 1),
        "Custom",
        TargetAction::Custom("refresh".to_string()),
        TargetPriority::Critical,
    );

    assert_eq!(custom.label, Some("Custom".to_string()));
    assert_eq!(custom.action, TargetAction::Custom("refresh".to_string()));
    assert_eq!(custom.priority, TargetPriority::Critical);
}
