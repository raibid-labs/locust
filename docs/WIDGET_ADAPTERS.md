# Widget Adapters Documentation

## Overview

Locust provides extension traits and wrapper types for ratatui widgets to automatically register navigation targets. This allows keyboard-driven navigation within your TUI applications.

## Features

- **ListExt** - Extension trait for `List` widgets
- **TableExt** - Extension trait for `Table` widgets with multiple navigation modes
- **TabsExt** - Extension trait for `Tabs` widgets
- **NavigableTree** - Tree widget wrapper with expand/collapse support

## Quick Start

```rust
use locust::core::targets::TargetRegistry;
use locust::ratatui_ext::adapters::{NavigableList, TableNavMode, NavigableTable};
use ratatui::widgets::{List, ListItem, Table, Row};
use ratatui::layout::{Rect, Constraint};

let mut registry = TargetRegistry::new();

// List navigation
let items = vec![ListItem::new("Item 1"), ListItem::new("Item 2")];
let list = List::new(items.clone());
let nav_list = NavigableList::new(list, items.len());
nav_list.register_targets(Rect::new(0, 0, 20, 5), &mut registry);

// Table navigation (row mode)
let rows = vec![Row::new(vec!["A", "B"]), Row::new(vec!["C", "D"])];
let table = Table::new(rows, vec![Constraint::Percentage(50), Constraint::Percentage(50)]);
let nav_table = NavigableTable::new(table, 2, vec![10, 10]);
nav_table.register_targets(Rect::new(0, 0, 20, 3), &mut registry, TableNavMode::Row);
```

## List Widgets

### Extension Trait

```rust
use locust::ratatui_ext::adapters::ListExt;

let list = List::new(items);
list.register_nav_targets(area, &mut registry);
```

### Wrapper for Custom Labels

```rust
use locust::ratatui_ext::adapters::NavigableList;

let nav_list = NavigableList::new(list, items.len())
    .with_labels(vec!["Home".into(), "Settings".into(), "Exit".into()]);
nav_list.register_targets(area, &mut registry);
```

## Table Widgets

### Navigation Modes

Tables support three navigation modes:

1. **Row Mode** - Navigate by entire rows
2. **Cell Mode** - Navigate individual cells
3. **Column Mode** - Navigate entire columns

### Example: Row Navigation

```rust
use locust::ratatui_ext::adapters::{NavigableTable, TableNavMode};

let nav_table = NavigableTable::new(table, row_count, column_widths);
nav_table.register_targets(area, &mut registry, TableNavMode::Row);
```

### Example: Cell Navigation

```rust
nav_table.register_targets(area, &mut registry, TableNavMode::Cell);
```

### Example: With Header Row

```rust
let nav_table = NavigableTable::new(table, row_count, column_widths)
    .with_header(); // Skip first row
nav_table.register_targets(area, &mut registry, TableNavMode::Row);
```

## Tabs Widgets

### Wrapper with Selection State

```rust
use locust::ratatui_ext::adapters::NavigableTabs;

let titles = vec!["Home".into(), "Settings".into(), "About".into()];
let tabs = Tabs::new(titles.clone());
let mut nav_tabs = NavigableTabs::new(tabs, titles, 0); // Start at index 0

nav_tabs.register_targets(area, &mut registry);

// Update selection
nav_tabs.select(1); // Switch to Settings tab
```

## Tree Widgets

### Tree Node Structure

```rust
use locust::ratatui_ext::adapters::{NavigableTree, TreeNode};

let nodes = vec![
    TreeNode {
        id: 1,
        label: "src/".into(),
        expanded: true,
        level: 0,
        has_children: true,
    },
    TreeNode {
        id: 2,
        label: "main.rs".into(),
        expanded: false,
        level: 1,
        has_children: false,
    },
];

let mut tree = NavigableTree::new(nodes);
tree.register_targets(area, &mut registry);

// Toggle expansion
tree.toggle_node(1); // Collapse/expand node with id 1
```

### Tree Features

- Automatic indentation based on node level
- Visual expand/collapse indicators (▼ expanded, ▶ collapsed)
- Metadata storage for level and children status
- Toggle functionality for interactive expansion

## Target Registry Integration

All adapters register targets with a `TargetRegistry`:

```rust
use locust::core::targets::TargetRegistry;

let mut registry = TargetRegistry::new();

// Register widgets
nav_list.register_targets(area, &mut registry);
nav_table.register_targets(area, &mut registry, TableNavMode::Row);
nav_tabs.register_targets(area, &mut registry);

// Query registered targets
println!("Total targets: {}", registry.len());

// Spatial queries
let targets_at_point = registry.at_point(10, 5);
let targets_in_area = registry.in_area(Rect::new(0, 0, 20, 10));

// Priority queries
let high_priority = registry.by_priority(TargetPriority::High);

// Group queries
let tab_group = registry.by_group("tabs");
```

## Advanced Usage

### Custom Priorities

```rust
use locust::core::targets::{TargetBuilder, TargetPriority};

let mut builder = TargetBuilder::new();
list.register_nav_targets_with(
    area,
    &mut registry,
    &mut builder,
    TargetPriority::High,
);
```

### Multiple Widget Types

```rust
// Frame refresh pattern
fn draw_frame(f: &mut Frame, registry: &mut TargetRegistry) {
    registry.clear(); // Clear previous frame's targets

    // Register all widgets for current frame
    nav_list.register_targets(list_area, registry);
    nav_table.register_targets(table_area, registry, TableNavMode::Row);
    nav_tabs.register_targets(tabs_area, registry);

    // Use registry for hint generation, spatial queries, etc.
}
```

### Spatial Queries

```rust
// Find closest target to a point
let closest = registry.closest_to(cursor_x, cursor_y);

// Find all targets in a region
let visible = registry.in_area(viewport);

// Sort by priority
let important = registry.sorted_by_priority();

// Sort by size
let prominent = registry.sorted_by_area();
```

## Example Application

See `examples/widget_navigation.rs` for a complete demo application showcasing:

- List navigation with custom labels
- Table navigation with multiple modes
- Tab selection with state tracking
- Tree navigation with expand/collapse
- Real-time target registry stats

Run the demo:

```bash
cargo run --example widget_navigation
```

## Testing

### Unit Tests

Located in `tests/unit/ratatui_adapters.rs`:

- Individual widget adapter tests
- Label customization tests
- Priority and action tests
- State management tests

### Integration Tests

Located in `tests/integration/widget_adapters.rs`:

- Multi-widget registration
- Spatial query tests
- Complex navigation scenarios
- Large dataset handling

Run tests:

```bash
cargo test ratatui_adapters
cargo test widget_adapters
```

## Best Practices

1. **Clear Registry Per Frame** - Always clear the registry at the start of each frame to remove stale targets

2. **Use Wrappers for Rich Features** - Use wrapper types (`NavigableList`, `NavigableTable`, etc.) for custom labels and advanced features

3. **Choose Appropriate Navigation Modes** - Select table navigation mode based on your use case:
   - Row mode for list-like tables
   - Cell mode for spreadsheet-like editing
   - Column mode for column-focused operations

4. **Leverage Priority System** - Assign higher priorities to important actions (tabs, critical buttons)

5. **Use Groups for Related Targets** - Group related targets for batch operations

6. **Cache Column Widths** - For tables, calculate and cache column widths for consistent navigation

## Architecture

Widget adapters integrate with Locust's core systems:

```
┌─────────────────┐
│ ratatui Widget  │
│  (List, Table)  │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Extension Trait │
│ or Wrapper      │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ TargetBuilder   │
│ (Factory)       │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ NavTarget       │
│ (Core Type)     │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ TargetRegistry  │
│ (Collection)    │
└─────────────────┘
```

## Performance Considerations

- **Visibility Clipping** - Only visible rows/items are registered as targets
- **Efficient Queries** - Registry uses HashMap for O(1) ID lookups
- **Minimal Overhead** - Target registration is lightweight (~100 bytes per target)
- **Per-Frame Registration** - Targets are recreated each frame (no stale state)

## Future Enhancements

Planned features for future releases:

- Virtual scrolling support for large lists
- Custom tree widget implementations
- Grid layout adapters
- Form input adapters
- Chart/graph navigation support
- Async target registration

## Troubleshooting

### Issue: Targets Not Appearing

**Solution**: Ensure you're calling `register_targets` within the draw function, after clearing the registry.

### Issue: Wrong Target Positions

**Solution**: Verify that the `Rect` passed to `register_targets` matches the actual widget rendering area.

### Issue: Overlapping Targets

**Solution**: Use priority levels to disambiguate overlapping targets. Higher priority targets will be preferred.

### Issue: Tab Selection Not Updating

**Solution**: Create a new `NavigableTabs` instance with the updated selection index each frame.

## Contributing

Contributions welcome! Areas for improvement:

- Additional widget adapters (Progress, Gauge, etc.)
- Performance optimizations for large datasets
- Enhanced spatial query algorithms
- Better tree widget support

## Related Documentation

This widget adapters reference connects with other Locust documentation:

### Core Documentation
- **[PLUGINS.md](PLUGINS.md#navplugin)** - Navigation plugin that uses adapters
- **[INTEGRATION_GUIDE.md](INTEGRATION_GUIDE.md#widget-adapters)** - Integrate widget adapters
- **[ARCHITECTURE.md](ARCHITECTURE.md)** - Adapter pattern architecture

### Development
- **[PLUGIN_DEVELOPMENT_GUIDE.md](PLUGIN_DEVELOPMENT_GUIDE.md)** - Create custom adapters
- **[API_PATTERNS.md](API_PATTERNS.md)** - Adapter design patterns
- **[CONTRIBUTING.md](../CONTRIBUTING.md)** - Contributing new adapters

### Examples
- **[EXAMPLES.md](EXAMPLES.md)** - Widget adapter usage examples
- **[CASE_STUDIES.md](CASE_STUDIES.md)** - Real-world adapter implementations

### Configuration
- **[CONFIGURATION.md](CONFIGURATION.md)** - Configure widget adapters

### Troubleshooting
- **[TROUBLESHOOTING.md](TROUBLESHOOTING.md)** - Adapter issues and solutions
- **[MIGRATION_CHECKLIST.md](MIGRATION_CHECKLIST.md)** - Migrating to widget adapters

### Project Documentation
- **[README.md](../README.md)** - Widget adapter overview
- **[ROADMAP.md](ROADMAP.md)** - Future adapter improvements

## License

Same as Locust project (see main LICENSE file).
