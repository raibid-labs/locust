use locust::core::targets::{TargetPriority, TargetRegistry, TargetState};
use locust::ratatui_ext::adapters::{
    NavigableList, NavigableTable, NavigableTabs, NavigableTree, TableNavMode, TreeNode,
};
use ratatui::layout::{Constraint, Rect};
use ratatui::widgets::{List, ListItem, Row, Table, Tabs};

/// Test integration between multiple widget types
#[test]
fn test_multi_widget_registration() {
    let mut registry = TargetRegistry::new();

    // List widget
    let items = vec![ListItem::new("Item 1"), ListItem::new("Item 2")];
    let list = List::new(items.clone());
    let nav_list = NavigableList::new(list, items.len());
    nav_list.register_targets(Rect::new(0, 0, 20, 2), &mut registry);

    // Table widget
    let rows = vec![Row::new(vec!["Cell 1", "Cell 2"])];
    let table = Table::new(rows, vec![Constraint::Percentage(50), Constraint::Percentage(50)]);
    let nav_table = NavigableTable::new(table, 1, vec![10, 10]);
    nav_table.register_targets(Rect::new(0, 3, 20, 1), &mut registry, TableNavMode::Row);

    // Tabs widget
    let titles = vec!["Home".into(), "Settings".into()];
    let tabs = Tabs::new(titles.clone());
    let nav_tabs = NavigableTabs::new(tabs, titles, 0);
    nav_tabs.register_targets(Rect::new(0, 5, 20, 1), &mut registry);

    // Should have targets from all widgets
    assert_eq!(registry.len(), 5); // 2 list items + 1 table row + 2 tabs
}

/// Test spatial queries across different widget types
#[test]
fn test_spatial_queries_across_widgets() {
    let mut registry = TargetRegistry::new();

    // Create widgets at different positions
    let items = vec![ListItem::new("Item 1")];
    let list = List::new(items.clone());
    let nav_list = NavigableList::new(list, items.len());
    nav_list.register_targets(Rect::new(0, 0, 20, 1), &mut registry);

    let rows = vec![Row::new(vec!["Cell"])];
    let table = Table::new(rows, vec![Constraint::Percentage(100)]);
    let nav_table = NavigableTable::new(table, 1, vec![20]);
    nav_table.register_targets(Rect::new(0, 5, 20, 1), &mut registry, TableNavMode::Row);

    // Query specific areas
    let top_targets = registry.in_area(Rect::new(0, 0, 20, 3));
    assert_eq!(top_targets.len(), 1); // Only list item

    let bottom_targets = registry.in_area(Rect::new(0, 4, 20, 3));
    assert_eq!(bottom_targets.len(), 1); // Only table row

    let all_targets = registry.in_area(Rect::new(0, 0, 20, 10));
    assert_eq!(all_targets.len(), 2); // Both
}

/// Test priority-based selection with mixed widgets
#[test]
fn test_priority_selection_mixed_widgets() {
    let mut registry = TargetRegistry::new();
    let mut builder = locust::core::targets::TargetBuilder::new();

    // Low priority list
    let items = vec![ListItem::new("Low Priority")];
    let list = List::new(items.clone());
    let nav_list = NavigableList::new(list, items.len());
    nav_list.register_targets(Rect::new(0, 0, 20, 1), &mut registry);

    // High priority tabs
    let titles = vec!["Important".into()];
    let tabs = Tabs::new(titles.clone());
    let nav_tabs = NavigableTabs::new(tabs, titles, 0);
    nav_tabs.register_targets(Rect::new(0, 2, 20, 1), &mut registry);

    let sorted = registry.sorted_by_priority();
    assert_eq!(sorted.len(), 2);

    // Tabs should come first (higher priority)
    let first = sorted[0];
    assert_eq!(first.priority, TargetPriority::High);
    assert_eq!(first.group, Some("tabs".into()));
}

/// Test complex table navigation scenarios
#[test]
fn test_complex_table_navigation() {
    let rows = vec![
        Row::new(vec!["A1", "A2", "A3"]),
        Row::new(vec!["B1", "B2", "B3"]),
        Row::new(vec!["C1", "C2", "C3"]),
    ];
    let table = Table::new(
        rows,
        vec![
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Length(10),
        ],
    );

    let nav_table = NavigableTable::new(table, 3, vec![10, 10, 10]);

    // Test cell mode navigation
    let mut cell_registry = TargetRegistry::new();
    nav_table.register_targets(
        Rect::new(0, 0, 30, 3),
        &mut cell_registry,
        TableNavMode::Cell,
    );

    // 3 rows × 3 columns = 9 cells
    assert_eq!(cell_registry.len(), 9);

    // Find targets in middle column
    let middle_col = cell_registry.in_area(Rect::new(10, 0, 10, 3));
    assert_eq!(middle_col.len(), 3); // 3 cells in middle column

    // Test row mode navigation
    let mut row_registry = TargetRegistry::new();
    nav_table.register_targets(
        Rect::new(0, 0, 30, 3),
        &mut row_registry,
        TableNavMode::Row,
    );

    assert_eq!(row_registry.len(), 3); // 3 rows

    // Test column mode navigation
    let mut col_registry = TargetRegistry::new();
    nav_table.register_targets(
        Rect::new(0, 0, 30, 3),
        &mut col_registry,
        TableNavMode::Column,
    );

    assert_eq!(col_registry.len(), 3); // 3 columns
}

/// Test tree with multiple levels and expansion
#[test]
fn test_complex_tree_structure() {
    let nodes = vec![
        TreeNode {
            id: 1,
            label: "Root".into(),
            expanded: true,
            level: 0,
            has_children: true,
        },
        TreeNode {
            id: 2,
            label: "Parent 1".into(),
            expanded: true,
            level: 1,
            has_children: true,
        },
        TreeNode {
            id: 3,
            label: "Child 1.1".into(),
            expanded: false,
            level: 2,
            has_children: false,
        },
        TreeNode {
            id: 4,
            label: "Child 1.2".into(),
            expanded: false,
            level: 2,
            has_children: false,
        },
        TreeNode {
            id: 5,
            label: "Parent 2".into(),
            expanded: false,
            level: 1,
            has_children: true,
        },
    ];

    let mut tree = NavigableTree::new(nodes);
    let mut registry = TargetRegistry::new();
    tree.register_targets(Rect::new(0, 0, 40, 10), &mut registry);

    assert_eq!(registry.len(), 5);

    // Check indentation for different levels
    let root = registry.by_id(1).unwrap();
    let parent1 = registry.by_id(2).unwrap();
    let child11 = registry.by_id(3).unwrap();

    assert_eq!(root.rect.x, 0); // Level 0
    assert_eq!(parent1.rect.x, 2); // Level 1, indent 2
    assert_eq!(child11.rect.x, 4); // Level 2, indent 4

    // Test toggling
    tree.toggle_node(2);
    assert!(!tree.nodes()[1].expanded); // Parent 1 collapsed

    tree.toggle_node(5);
    assert!(tree.nodes()[4].expanded); // Parent 2 expanded
}

/// Test state management across frame updates
#[test]
fn test_state_management_across_frames() {
    let mut registry = TargetRegistry::new();

    // Frame 1: Register tabs
    let titles = vec!["Tab 1".into(), "Tab 2".into(), "Tab 3".into()];
    let tabs = Tabs::new(titles.clone());
    let nav_tabs = NavigableTabs::new(tabs, titles.clone(), 0);
    nav_tabs.register_targets(Rect::new(0, 0, 30, 1), &mut registry);

    assert_eq!(registry.len(), 3);
    let tab1 = registry.by_id(1).unwrap();
    assert_eq!(tab1.state, TargetState::Selected);

    // Frame 2: Clear and re-register with different selection
    registry.clear();
    let tabs2 = Tabs::new(titles.clone());
    let nav_tabs2 = NavigableTabs::new(tabs2, titles, 1);
    nav_tabs2.register_targets(Rect::new(0, 0, 30, 1), &mut registry);

    assert_eq!(registry.len(), 3);
    let tab2 = registry.by_id(2).unwrap();
    assert_eq!(tab2.state, TargetState::Selected);
}

/// Test finding closest target across widget types
#[test]
fn test_closest_target_search() {
    let mut registry = TargetRegistry::new();

    // List at top
    let items = vec![ListItem::new("Top Item")];
    let list = List::new(items.clone());
    let nav_list = NavigableList::new(list, items.len());
    nav_list.register_targets(Rect::new(0, 0, 20, 1), &mut registry);

    // Table in middle
    let rows = vec![Row::new(vec!["Middle"])];
    let table = Table::new(rows, vec![Constraint::Percentage(100)]);
    let nav_table = NavigableTable::new(table, 1, vec![20]);
    nav_table.register_targets(Rect::new(0, 10, 20, 1), &mut registry, TableNavMode::Row);

    // Tabs at bottom
    let titles = vec!["Bottom".into()];
    let tabs = Tabs::new(titles.clone());
    let nav_tabs = NavigableTabs::new(tabs, titles, 0);
    nav_tabs.register_targets(Rect::new(0, 20, 20, 1), &mut registry);

    // Find closest to top
    let closest_to_top = registry.closest_to(10, 0);
    assert!(closest_to_top.is_some());
    let target = closest_to_top.unwrap();
    assert_eq!(target.rect.y, 0); // Should be the list

    // Find closest to middle
    let closest_to_middle = registry.closest_to(10, 10);
    assert!(closest_to_middle.is_some());
    let target = closest_to_middle.unwrap();
    assert_eq!(target.rect.y, 10); // Should be the table

    // Find closest to bottom
    let closest_to_bottom = registry.closest_to(10, 20);
    assert!(closest_to_bottom.is_some());
    let target = closest_to_bottom.unwrap();
    assert_eq!(target.rect.y, 20); // Should be the tabs
}

/// Test group-based queries
#[test]
fn test_group_based_navigation() {
    let mut registry = TargetRegistry::new();

    // Register multiple tabs (same group)
    let titles = vec!["Tab 1".into(), "Tab 2".into()];
    let tabs = Tabs::new(titles.clone());
    let nav_tabs = NavigableTabs::new(tabs, titles, 0);
    nav_tabs.register_targets(Rect::new(0, 0, 20, 1), &mut registry);

    // Register table cells (same group)
    let rows = vec![Row::new(vec!["A", "B"])];
    let table = Table::new(rows, vec![Constraint::Percentage(50), Constraint::Percentage(50)]);
    let nav_table = NavigableTable::new(table, 1, vec![10, 10]);
    nav_table.register_targets(Rect::new(0, 2, 20, 1), &mut registry, TableNavMode::Cell);

    // Query by group
    let tab_group = registry.by_group("tabs");
    assert_eq!(tab_group.len(), 2);

    let table_group = registry.by_group("table");
    assert_eq!(table_group.len(), 2);
}

/// Test large dataset handling
#[test]
fn test_large_dataset_performance() {
    let mut registry = TargetRegistry::new();

    // Create a large list
    let items: Vec<ListItem> = (0..100)
        .map(|i| ListItem::new(format!("Item {}", i)))
        .collect();
    let list = List::new(items.clone());
    let nav_list = NavigableList::new(list, items.len());

    // Only 20 rows visible
    nav_list.register_targets(Rect::new(0, 0, 40, 20), &mut registry);

    // Should only register visible items
    assert_eq!(registry.len(), 20);

    // Verify all targets are within visible area
    for target in registry.all() {
        assert!(target.rect.y < 20);
    }
}

/// Test table with header row
#[test]
fn test_table_with_header_integration() {
    let rows = vec![
        Row::new(vec!["Name", "Age"]), // Header
        Row::new(vec!["Alice", "30"]),
        Row::new(vec!["Bob", "25"]),
    ];
    let table = Table::new(rows, vec![Constraint::Percentage(50), Constraint::Percentage(50)]);
    let nav_table = NavigableTable::new(table, 2, vec![20, 20]).with_header();

    let mut registry = TargetRegistry::new();
    nav_table.register_targets(Rect::new(0, 0, 40, 3), &mut registry, TableNavMode::Row);

    assert_eq!(registry.len(), 2); // Only data rows

    // First data row should start after header
    let first_row = registry.by_id(1).unwrap();
    assert_eq!(first_row.rect.y, 1);

    let second_row = registry.by_id(2).unwrap();
    assert_eq!(second_row.rect.y, 2);
}

/// Test area-based filtering
#[test]
fn test_area_filtering() {
    let mut registry = TargetRegistry::new();

    // Create grid of targets
    for row in 0..5 {
        for col in 0..5 {
            let items = vec![ListItem::new(format!("Item {}x{}", row, col))];
            let list = List::new(items.clone());
            let nav_list = NavigableList::new(list, items.len());
            nav_list.register_targets(
                Rect::new(col as u16 * 10, row as u16 * 2, 10, 1),
                &mut registry,
            );
        }
    }

    assert_eq!(registry.len(), 25); // 5x5 grid

    // Filter by quadrant
    let top_left = registry.in_area(Rect::new(0, 0, 25, 5));
    assert_eq!(top_left.len(), 6); // Upper left quadrant

    let bottom_right = registry.in_area(Rect::new(25, 5, 25, 5));
    assert_eq!(bottom_right.len(), 6); // Lower right quadrant
}
