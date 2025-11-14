use locust::core::targets::{NavTarget, TargetAction, TargetPriority, TargetRegistry, TargetState};
use locust::ratatui_ext::adapters::{
    ListExt, NavigableList, NavigableTable, NavigableTabs, NavigableTree,
    TableExt, TableNavMode, TabsExt, TreeNode,
};
use ratatui::layout::{Constraint, Rect};
use ratatui::widgets::{List, ListItem, Row, Table, Tabs};

#[test]
fn test_list_ext_basic() {
    let items = vec![
        ListItem::new("Item 1"),
        ListItem::new("Item 2"),
        ListItem::new("Item 3"),
    ];
    let list = List::new(items);
    let mut registry = TargetRegistry::new();
    let area = Rect::new(0, 0, 20, 3);

    list.register_nav_targets(area, &mut registry);

    assert_eq!(registry.len(), 3);
    assert!(registry.by_id(1).is_some());
    assert!(registry.by_id(2).is_some());
    assert!(registry.by_id(3).is_some());
}

#[test]
fn test_navigable_list_with_custom_labels() {
    let items = vec![
        ListItem::new("Item 1"),
        ListItem::new("Item 2"),
        ListItem::new("Item 3"),
    ];
    let list = List::new(items.clone());
    let labels = vec!["Home".into(), "Settings".into(), "Exit".into()];
    let nav_list = NavigableList::new(list, items.len()).with_labels(labels);

    let mut registry = TargetRegistry::new();
    nav_list.register_targets(Rect::new(0, 0, 20, 5), &mut registry);

    assert_eq!(registry.len(), 3);

    let home_target = registry.by_id(1).unwrap();
    assert_eq!(home_target.label, Some("Home".into()));

    let settings_target = registry.by_id(2).unwrap();
    assert_eq!(settings_target.label, Some("Settings".into()));
}

#[test]
fn test_navigable_list_respects_visible_area() {
    let items = vec![
        ListItem::new("Item 1"),
        ListItem::new("Item 2"),
        ListItem::new("Item 3"),
        ListItem::new("Item 4"),
        ListItem::new("Item 5"),
    ];
    let list = List::new(items.clone());
    let nav_list = NavigableList::new(list, items.len());

    let mut registry = TargetRegistry::new();
    // Only 3 rows visible
    nav_list.register_targets(Rect::new(0, 0, 20, 3), &mut registry);

    // Should only register 3 targets (visible area constraint)
    assert_eq!(registry.len(), 3);
}

#[test]
fn test_table_ext_row_mode() {
    let rows = vec![
        Row::new(vec!["Cell 1", "Cell 2"]),
        Row::new(vec!["Cell 3", "Cell 4"]),
        Row::new(vec!["Cell 5", "Cell 6"]),
    ];
    let table = Table::new(rows, vec![Constraint::Percentage(50), Constraint::Percentage(50)]);
    let mut registry = TargetRegistry::new();

    table.register_nav_targets(
        Rect::new(0, 0, 40, 3),
        &mut registry,
        TableNavMode::Row,
    );

    assert_eq!(registry.len(), 3);
    assert!(registry.by_id(1).is_some());
}

#[test]
fn test_navigable_table_cell_mode() {
    let rows = vec![
        Row::new(vec!["A1", "A2", "A3"]),
        Row::new(vec!["B1", "B2", "B3"]),
    ];
    let table = Table::new(
        rows,
        vec![
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Length(10),
        ],
    );
    let nav_table = NavigableTable::new(table, 2, vec![10, 10, 10]);

    let mut registry = TargetRegistry::new();
    nav_table.register_targets(Rect::new(0, 0, 30, 2), &mut registry, TableNavMode::Cell);

    // 2 rows × 3 columns = 6 cells
    assert_eq!(registry.len(), 6);

    // Verify first cell
    let first_cell = registry.by_id(1).unwrap();
    assert_eq!(first_cell.rect.x, 0);
    assert_eq!(first_cell.rect.y, 0);
    assert_eq!(first_cell.rect.width, 10);
}

#[test]
fn test_navigable_table_with_header() {
    let rows = vec![
        Row::new(vec!["Data 1", "Data 2"]),
        Row::new(vec!["Data 3", "Data 4"]),
    ];
    let table = Table::new(rows, vec![Constraint::Percentage(50), Constraint::Percentage(50)]);
    let nav_table = NavigableTable::new(table, 2, vec![20, 20]).with_header();

    let mut registry = TargetRegistry::new();
    nav_table.register_targets(Rect::new(0, 0, 40, 3), &mut registry, TableNavMode::Row);

    // Should skip header row
    assert_eq!(registry.len(), 2);

    // First data row should start at y=1 (after header)
    let first_row = registry.by_id(1).unwrap();
    assert_eq!(first_row.rect.y, 1);
}

#[test]
fn test_navigable_table_column_mode() {
    let rows = vec![
        Row::new(vec!["A1", "B1"]),
        Row::new(vec!["A2", "B2"]),
        Row::new(vec!["A3", "B3"]),
    ];
    let table = Table::new(rows, vec![Constraint::Percentage(50), Constraint::Percentage(50)]);
    let nav_table = NavigableTable::new(table, 3, vec![20, 20]);

    let mut registry = TargetRegistry::new();
    nav_table.register_targets(Rect::new(0, 0, 40, 3), &mut registry, TableNavMode::Column);

    // 2 columns
    assert_eq!(registry.len(), 2);

    let first_col = registry.by_id(1).unwrap();
    assert_eq!(first_col.label, Some("Column 1".into()));
    assert_eq!(first_col.rect.width, 20);
    assert_eq!(first_col.rect.height, 3);
}

#[test]
fn test_tabs_ext_basic() {
    let titles = vec!["Home", "Settings", "About"];
    let tabs = Tabs::new(titles);
    let mut registry = TargetRegistry::new();

    tabs.register_nav_targets(Rect::new(0, 0, 30, 1), &mut registry);

    assert_eq!(registry.len(), 3);
}

#[test]
fn test_navigable_tabs_with_selection() {
    let titles = vec!["Home".into(), "Settings".into(), "About".into()];
    let tabs = Tabs::new(titles.clone());
    let nav_tabs = NavigableTabs::new(tabs, titles, 1); // Settings selected

    let mut registry = TargetRegistry::new();
    nav_tabs.register_targets(Rect::new(0, 0, 30, 1), &mut registry);

    assert_eq!(registry.len(), 3);

    // Check that second tab is selected
    let settings_tab = registry.by_id(2).unwrap();
    assert_eq!(settings_tab.state, TargetState::Selected);
    assert_eq!(settings_tab.label, Some("Settings".into()));

    // First tab should not be selected
    let home_tab = registry.by_id(1).unwrap();
    assert_eq!(home_tab.state, TargetState::Normal);
}

#[test]
fn test_navigable_tabs_selection_change() {
    let titles = vec!["Tab 1".into(), "Tab 2".into()];
    let tabs = Tabs::new(titles.clone());
    let mut nav_tabs = NavigableTabs::new(tabs, titles, 0);

    assert_eq!(nav_tabs.selected(), 0);

    nav_tabs.select(1);
    assert_eq!(nav_tabs.selected(), 1);

    // Test bounds checking
    nav_tabs.select(10);
    assert_eq!(nav_tabs.selected(), 1); // Should not change
}

#[test]
fn test_navigable_tree_basic() {
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
            label: "Child 1".into(),
            expanded: false,
            level: 1,
            has_children: false,
        },
        TreeNode {
            id: 3,
            label: "Child 2".into(),
            expanded: false,
            level: 1,
            has_children: true,
        },
    ];

    let tree = NavigableTree::new(nodes);
    let mut registry = TargetRegistry::new();
    tree.register_targets(Rect::new(0, 0, 40, 10), &mut registry);

    assert_eq!(registry.len(), 3);

    // Check root node
    let root = registry.by_id(1).unwrap();
    assert_eq!(root.metadata.get("level"), Some(&"0".to_string()));
    assert_eq!(root.metadata.get("has_children"), Some(&"true".to_string()));
    assert_eq!(root.rect.x, 0); // No indent at level 0

    // Check child node (should be indented)
    let child = registry.by_id(2).unwrap();
    assert_eq!(child.metadata.get("level"), Some(&"1".to_string()));
    assert_eq!(child.rect.x, 2); // Indented by level * 2
}

#[test]
fn test_tree_expansion_labels() {
    let nodes = vec![
        TreeNode {
            id: 1,
            label: "Folder".into(),
            expanded: true,
            level: 0,
            has_children: true,
        },
        TreeNode {
            id: 2,
            label: "File".into(),
            expanded: false,
            level: 1,
            has_children: false,
        },
    ];

    let tree = NavigableTree::new(nodes);
    let mut registry = TargetRegistry::new();
    tree.register_targets(Rect::new(0, 0, 40, 10), &mut registry);

    let folder = registry.by_id(1).unwrap();
    assert!(folder.label.as_ref().unwrap().contains("▼")); // Expanded

    let file = registry.by_id(2).unwrap();
    assert!(file.label.as_ref().unwrap().contains("  ")); // No arrow for leaf
}

#[test]
fn test_tree_toggle_expansion() {
    let nodes = vec![
        TreeNode {
            id: 1,
            label: "Root".into(),
            expanded: false,
            level: 0,
            has_children: true,
        },
        TreeNode {
            id: 2,
            label: "Leaf".into(),
            expanded: false,
            level: 0,
            has_children: false,
        },
    ];

    let mut tree = NavigableTree::new(nodes);

    // Toggle root (has children)
    assert!(!tree.nodes()[0].expanded);
    tree.toggle_node(1);
    assert!(tree.nodes()[0].expanded);
    tree.toggle_node(1);
    assert!(!tree.nodes()[0].expanded);

    // Toggle leaf (no children - should not change)
    tree.toggle_node(2);
    assert!(!tree.nodes()[1].expanded);
}

#[test]
fn test_tree_respects_visible_area() {
    let nodes: Vec<TreeNode> = (0..10)
        .map(|i| TreeNode {
            id: i + 1,
            label: format!("Node {}", i),
            expanded: false,
            level: 0,
            has_children: false,
        })
        .collect();

    let tree = NavigableTree::new(nodes);
    let mut registry = TargetRegistry::new();

    // Only 5 rows visible
    tree.register_targets(Rect::new(0, 0, 40, 5), &mut registry);

    assert_eq!(registry.len(), 5);
}

#[test]
fn test_target_priorities() {
    let items = vec![ListItem::new("Item")];
    let list = List::new(items);
    let mut registry = TargetRegistry::new();
    let mut builder = locust::core::targets::TargetBuilder::new();

    list.register_nav_targets_with(
        Rect::new(0, 0, 20, 1),
        &mut registry,
        &mut builder,
        TargetPriority::High,
    );

    let target = registry.by_id(1).unwrap();
    assert_eq!(target.priority, TargetPriority::High);
}

#[test]
fn test_target_actions() {
    let rows = vec![Row::new(vec!["Cell"])];
    let table = Table::new(rows, vec![Constraint::Percentage(100)]);
    let nav_table = NavigableTable::new(table, 1, vec![20]);

    let mut registry = TargetRegistry::new();
    nav_table.register_targets(Rect::new(0, 0, 20, 1), &mut registry, TableNavMode::Row);

    let target = registry.by_id(1).unwrap();
    assert_eq!(target.action, TargetAction::Select);
}

#[test]
fn test_target_groups() {
    let titles = vec!["Tab 1".into(), "Tab 2".into()];
    let tabs = Tabs::new(titles.clone());
    let nav_tabs = NavigableTabs::new(tabs, titles, 0);

    let mut registry = TargetRegistry::new();
    nav_tabs.register_targets(Rect::new(0, 0, 20, 1), &mut registry);

    let tab1 = registry.by_id(1).unwrap();
    assert_eq!(tab1.group, Some("tabs".into()));

    let tab2 = registry.by_id(2).unwrap();
    assert_eq!(tab2.group, Some("tabs".into()));

    // Verify group query works
    let tabs_group = registry.by_group("tabs");
    assert_eq!(tabs_group.len(), 2);
}

#[test]
fn test_rect_calculations() {
    let nodes = vec![
        TreeNode {
            id: 1,
            label: "Level 0".into(),
            expanded: false,
            level: 0,
            has_children: false,
        },
        TreeNode {
            id: 2,
            label: "Level 1".into(),
            expanded: false,
            level: 1,
            has_children: false,
        },
        TreeNode {
            id: 3,
            label: "Level 2".into(),
            expanded: false,
            level: 2,
            has_children: false,
        },
    ];

    let tree = NavigableTree::new(nodes);
    let mut registry = TargetRegistry::new();
    tree.register_targets(Rect::new(10, 5, 40, 10), &mut registry);

    let level0 = registry.by_id(1).unwrap();
    assert_eq!(level0.rect.x, 10);
    assert_eq!(level0.rect.y, 5);
    assert_eq!(level0.rect.width, 40);

    let level1 = registry.by_id(2).unwrap();
    assert_eq!(level1.rect.x, 12); // Indent by 2
    assert_eq!(level1.rect.y, 6);
    assert_eq!(level1.rect.width, 38); // Width reduced by indent

    let level2 = registry.by_id(3).unwrap();
    assert_eq!(level2.rect.x, 14); // Indent by 4
    assert_eq!(level2.rect.y, 7);
    assert_eq!(level2.rect.width, 36); // Width reduced by indent
}
