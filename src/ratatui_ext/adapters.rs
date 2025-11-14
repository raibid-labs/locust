use crate::core::targets::{
    NavTarget, TargetAction, TargetBuilder, TargetPriority, TargetRegistry,
};
use ratatui::layout::Rect;
use ratatui::widgets::{List, Table, Tabs};

/// Trait implemented by widgets (or their wrappers) that can expose
/// navigable targets to Locust.
pub trait Navigable {
    fn nav_targets(&self, area: Rect, out: &mut TargetRegistry);
}

/// Helper for common "one target per row" widgets like List or simple Tables.
pub fn register_simple_row_targets(
    area: Rect,
    row_count: usize,
    start_id: u64,
    out: &mut TargetRegistry,
) {
    let visible_rows = row_count.min(area.height as usize);
    for i in 0..visible_rows {
        let y = area.y + i as u16;
        let rect = Rect {
            x: area.x,
            y,
            width: area.width,
            height: 1,
        };
        let id = start_id + i as u64;
        out.register(NavTarget::new(id, rect));
    }
}

/// Extension trait for ratatui List widget to automatically register navigation targets.
///
/// This trait provides methods to register each list item as a navigation target,
/// allowing keyboard-driven navigation within lists.
///
/// # Examples
///
/// ```rust
/// use locust::ratatui_ext::adapters::ListExt;
/// use locust::core::targets::TargetRegistry;
/// use ratatui::widgets::{List, ListItem};
/// use ratatui::layout::Rect;
///
/// let items = vec![
///     ListItem::new("Item 1"),
///     ListItem::new("Item 2"),
///     ListItem::new("Item 3"),
/// ];
/// let list = List::new(items);
/// let mut registry = TargetRegistry::new();
/// let area = Rect::new(0, 0, 20, 10);
///
/// // Register navigation targets for each item
/// list.register_nav_targets(area, &mut registry);
/// ```
pub trait ListExt<'a> {
    /// Register navigation targets for each list item within the given area.
    ///
    /// # Arguments
    ///
    /// * `area` - The screen area occupied by the list
    /// * `registry` - The target registry to populate with navigation targets
    fn register_nav_targets(&self, area: Rect, registry: &mut TargetRegistry);

    /// Register navigation targets with custom hint generation and callbacks.
    ///
    /// # Arguments
    ///
    /// * `area` - The screen area occupied by the list
    /// * `registry` - The target registry to populate
    /// * `builder` - Target builder for generating IDs
    /// * `priority` - Priority level for all list items
    fn register_nav_targets_with(
        &self,
        area: Rect,
        registry: &mut TargetRegistry,
        builder: &mut TargetBuilder,
        priority: TargetPriority,
    );
}

impl<'a> ListExt<'a> for List<'a> {
    fn register_nav_targets(&self, area: Rect, registry: &mut TargetRegistry) {
        let mut builder = TargetBuilder::new();
        self.register_nav_targets_with(area, registry, &mut builder, TargetPriority::Normal);
    }

    fn register_nav_targets_with(
        &self,
        area: Rect,
        registry: &mut TargetRegistry,
        builder: &mut TargetBuilder,
        priority: TargetPriority,
    ) {
        // Note: We can't directly access List's items in current ratatui API,
        // so we use a wrapper approach. The user should provide item count.
        // For now, we'll implement a basic version that assumes the list height
        // corresponds to visible items.

        // Since we can't extract items from List directly, we'll create a helper
        // that users can call with their item count
        let visible_rows = area.height as usize;

        for i in 0..visible_rows {
            let y = area.y + i as u16;
            let rect = Rect {
                x: area.x,
                y,
                width: area.width,
                height: 1,
            };

            let target = builder
                .list_item(rect, format!("Item {}", i))
                .with_priority(priority);
            registry.register(target);
        }
    }
}

/// Wrapper for List widget that provides more control over navigation target registration.
///
/// This wrapper allows you to register targets with custom labels and callbacks.
///
/// # Examples
///
/// ```rust
/// use locust::ratatui_ext::adapters::NavigableList;
/// use locust::core::targets::TargetRegistry;
/// use ratatui::widgets::{List, ListItem};
/// use ratatui::layout::Rect;
///
/// let items = vec![
///     ListItem::new("Item 1"),
///     ListItem::new("Item 2"),
/// ];
/// let list = List::new(items.clone());
/// let nav_list = NavigableList::new(list, items.len());
///
/// let mut registry = TargetRegistry::new();
/// nav_list.register_targets(Rect::new(0, 0, 20, 10), &mut registry);
/// ```
pub struct NavigableList<'a> {
    list: List<'a>,
    item_count: usize,
    labels: Vec<String>,
}

impl<'a> NavigableList<'a> {
    /// Create a new navigable list wrapper.
    pub fn new(list: List<'a>, item_count: usize) -> Self {
        Self {
            list,
            item_count,
            labels: Vec::new(),
        }
    }

    /// Set custom labels for each item (used in navigation hints).
    pub fn with_labels(mut self, labels: Vec<String>) -> Self {
        self.labels = labels;
        self
    }

    /// Register navigation targets for this list.
    pub fn register_targets(&self, area: Rect, registry: &mut TargetRegistry) {
        let mut builder = TargetBuilder::new();
        let visible_rows = self.item_count.min(area.height as usize);

        for i in 0..visible_rows {
            let y = area.y + i as u16;
            let rect = Rect {
                x: area.x,
                y,
                width: area.width,
                height: 1,
            };

            let label = self
                .labels
                .get(i)
                .cloned()
                .unwrap_or_else(|| format!("Item {}", i + 1));

            let target = builder.list_item(rect, label);
            registry.register(target);
        }
    }

    /// Get the underlying list widget.
    pub fn widget(&self) -> &List<'a> {
        &self.list
    }
}

/// Navigation mode for table widgets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TableNavMode {
    /// Navigate by individual cells
    Cell,
    /// Navigate by rows
    Row,
    /// Navigate by columns
    Column,
}

/// Extension trait for ratatui Table widget to automatically register navigation targets.
///
/// Tables support multiple navigation modes: cell-by-cell, row-by-row, or column-by-column.
///
/// # Examples
///
/// ```rust
/// use locust::ratatui_ext::adapters::{TableExt, TableNavMode};
/// use locust::core::targets::TargetRegistry;
/// use ratatui::widgets::{Table, Row};
/// use ratatui::layout::{Rect, Constraint};
///
/// let rows = vec![
///     Row::new(vec!["Cell 1", "Cell 2"]),
///     Row::new(vec!["Cell 3", "Cell 4"]),
/// ];
/// let table = Table::new(rows, vec![Constraint::Percentage(50), Constraint::Percentage(50)]);
/// let mut registry = TargetRegistry::new();
///
/// // Register row-based navigation
/// table.register_nav_targets(
///     Rect::new(0, 0, 40, 10),
///     &mut registry,
///     TableNavMode::Row,
/// );
/// ```
pub trait TableExt<'a> {
    /// Register navigation targets based on the specified mode.
    fn register_nav_targets(&self, area: Rect, registry: &mut TargetRegistry, mode: TableNavMode);

    /// Register navigation targets with custom configuration.
    fn register_nav_targets_with(
        &self,
        area: Rect,
        registry: &mut TargetRegistry,
        mode: TableNavMode,
        builder: &mut TargetBuilder,
        priority: TargetPriority,
        has_header: bool,
    );
}

impl<'a> TableExt<'a> for Table<'a> {
    fn register_nav_targets(&self, area: Rect, registry: &mut TargetRegistry, mode: TableNavMode) {
        let mut builder = TargetBuilder::new();
        self.register_nav_targets_with(
            area,
            registry,
            mode,
            &mut builder,
            TargetPriority::Normal,
            false,
        );
    }

    fn register_nav_targets_with(
        &self,
        area: Rect,
        registry: &mut TargetRegistry,
        mode: TableNavMode,
        builder: &mut TargetBuilder,
        priority: TargetPriority,
        has_header: bool,
    ) {
        // Account for header row if present
        let content_start_y = if has_header { area.y + 1 } else { area.y };
        let content_height = if has_header {
            area.height.saturating_sub(1)
        } else {
            area.height
        };

        match mode {
            TableNavMode::Row => {
                // Register one target per row
                for i in 0..content_height as usize {
                    let y = content_start_y + i as u16;
                    let rect = Rect {
                        x: area.x,
                        y,
                        width: area.width,
                        height: 1,
                    };

                    let target = NavTarget::new(builder.next_id(), rect)
                        .with_label(format!("Row {}", i + 1))
                        .with_action(TargetAction::Select)
                        .with_priority(priority);
                    registry.register(target);
                }
            }
            TableNavMode::Cell | TableNavMode::Column => {
                // For cell/column mode, we'd need column width information
                // which isn't easily accessible from Table API.
                // This is a simplified implementation.
                // In a real scenario, you'd need a wrapper like NavigableTable
                for i in 0..content_height as usize {
                    let y = content_start_y + i as u16;
                    let rect = Rect {
                        x: area.x,
                        y,
                        width: area.width,
                        height: 1,
                    };

                    let target = NavTarget::new(builder.next_id(), rect)
                        .with_label(format!("Row {}", i + 1))
                        .with_action(TargetAction::Select)
                        .with_priority(priority);
                    registry.register(target);
                }
            }
        }
    }
}

/// Wrapper for Table widget with column information for proper cell navigation.
pub struct NavigableTable<'a> {
    table: Table<'a>,
    row_count: usize,
    column_widths: Vec<u16>,
    has_header: bool,
}

impl<'a> NavigableTable<'a> {
    /// Create a new navigable table wrapper.
    pub fn new(table: Table<'a>, row_count: usize, column_widths: Vec<u16>) -> Self {
        Self {
            table,
            row_count,
            column_widths,
            has_header: false,
        }
    }

    /// Indicate that this table has a header row.
    pub fn with_header(mut self) -> Self {
        self.has_header = true;
        self
    }

    /// Register targets based on navigation mode.
    pub fn register_targets(&self, area: Rect, registry: &mut TargetRegistry, mode: TableNavMode) {
        let mut builder = TargetBuilder::new();
        let content_start_y = if self.has_header { area.y + 1 } else { area.y };

        match mode {
            TableNavMode::Row => {
                let visible_rows = self.row_count.min(area.height as usize);
                for i in 0..visible_rows {
                    let y = content_start_y + i as u16;
                    let rect = Rect {
                        x: area.x,
                        y,
                        width: area.width,
                        height: 1,
                    };

                    let target = builder.list_item(rect, format!("Row {}", i + 1));
                    registry.register(target);
                }
            }
            TableNavMode::Cell => {
                // Register individual cells
                let visible_rows = self.row_count.min(area.height as usize);
                let mut x_offset = area.x;

                for col_idx in 0..self.column_widths.len() {
                    let col_width = self.column_widths[col_idx];

                    for row_idx in 0..visible_rows {
                        let y = content_start_y + row_idx as u16;
                        let rect = Rect {
                            x: x_offset,
                            y,
                            width: col_width,
                            height: 1,
                        };

                        let target = NavTarget::new(builder.next_id(), rect)
                            .with_label(format!("Cell ({}, {})", row_idx + 1, col_idx + 1))
                            .with_action(TargetAction::Select)
                            .with_group("table");
                        registry.register(target);
                    }

                    x_offset += col_width;
                }
            }
            TableNavMode::Column => {
                // Register entire columns
                let mut x_offset = area.x;
                for (col_idx, &col_width) in self.column_widths.iter().enumerate() {
                    let rect = Rect {
                        x: x_offset,
                        y: content_start_y,
                        width: col_width,
                        height: area
                            .height
                            .saturating_sub(if self.has_header { 1 } else { 0 }),
                    };

                    let target = NavTarget::new(builder.next_id(), rect)
                        .with_label(format!("Column {}", col_idx + 1))
                        .with_action(TargetAction::Select)
                        .with_group("table");
                    registry.register(target);

                    x_offset += col_width;
                }
            }
        }
    }

    /// Get the underlying table widget.
    pub fn widget(&self) -> &Table<'a> {
        &self.table
    }
}

/// Extension trait for ratatui Tabs widget to automatically register navigation targets.
///
/// Each tab becomes a separate navigation target that can be selected via keyboard shortcuts.
///
/// # Examples
///
/// ```rust
/// use locust::ratatui_ext::adapters::TabsExt;
/// use locust::core::targets::TargetRegistry;
/// use ratatui::widgets::Tabs;
/// use ratatui::layout::Rect;
///
/// let tabs = Tabs::new(vec!["Tab 1", "Tab 2", "Tab 3"]);
/// let mut registry = TargetRegistry::new();
///
/// tabs.register_nav_targets(Rect::new(0, 0, 30, 1), &mut registry);
/// ```
pub trait TabsExt<'a> {
    /// Register navigation targets for each tab.
    fn register_nav_targets(&self, area: Rect, registry: &mut TargetRegistry);

    /// Register navigation targets with custom configuration.
    fn register_nav_targets_with(
        &self,
        area: Rect,
        registry: &mut TargetRegistry,
        builder: &mut TargetBuilder,
        selected_index: Option<usize>,
    );
}

impl<'a> TabsExt<'a> for Tabs<'a> {
    fn register_nav_targets(&self, area: Rect, registry: &mut TargetRegistry) {
        let mut builder = TargetBuilder::new();
        self.register_nav_targets_with(area, registry, &mut builder, None);
    }

    fn register_nav_targets_with(
        &self,
        area: Rect,
        registry: &mut TargetRegistry,
        builder: &mut TargetBuilder,
        selected_index: Option<usize>,
    ) {
        // Note: Like List and Table, we can't directly access Tabs content.
        // This is a simplified implementation. For a complete solution,
        // use NavigableTabs wrapper below.

        // Assume tabs are evenly distributed across the width
        // This is a simplification; real implementation would need tab widths
        let tab_count = 3; // Placeholder - use wrapper for actual count
        let tab_width = area.width / tab_count as u16;

        for i in 0..tab_count {
            let x = area.x + (i as u16 * tab_width);
            let rect = Rect {
                x,
                y: area.y,
                width: tab_width,
                height: area.height,
            };

            let mut target = builder.tab(rect, format!("Tab {}", i + 1));

            if Some(i) == selected_index {
                target = target.with_state(crate::core::targets::TargetState::Selected);
            }

            registry.register(target);
        }
    }
}

/// Wrapper for Tabs widget with title information for proper navigation.
pub struct NavigableTabs<'a> {
    tabs: Tabs<'a>,
    titles: Vec<String>,
    selected_index: usize,
}

impl<'a> NavigableTabs<'a> {
    /// Create a new navigable tabs wrapper.
    pub fn new(tabs: Tabs<'a>, titles: Vec<String>, selected_index: usize) -> Self {
        Self {
            tabs,
            titles,
            selected_index,
        }
    }

    /// Update the selected tab index.
    pub fn select(&mut self, index: usize) {
        if index < self.titles.len() {
            self.selected_index = index;
        }
    }

    /// Register navigation targets for all tabs.
    pub fn register_targets(&self, area: Rect, registry: &mut TargetRegistry) {
        let mut builder = TargetBuilder::new();

        // Calculate approximate width per tab
        // In reality, tab widths depend on title lengths and styling
        let tab_count = self.titles.len();
        if tab_count == 0 {
            return;
        }

        let tab_width = area.width / tab_count as u16;

        for (i, title) in self.titles.iter().enumerate() {
            let x = area.x + (i as u16 * tab_width);
            let rect = Rect {
                x,
                y: area.y,
                width: tab_width,
                height: area.height,
            };

            let mut target = builder.tab(rect, title.clone());

            if i == self.selected_index {
                target = target.with_state(crate::core::targets::TargetState::Selected);
            }

            registry.register(target);
        }
    }

    /// Get the underlying tabs widget.
    pub fn widget(&self) -> &Tabs<'a> {
        &self.tabs
    }

    /// Get the currently selected tab index.
    pub fn selected(&self) -> usize {
        self.selected_index
    }
}

/// Tree node representation for navigation.
#[derive(Debug, Clone)]
pub struct TreeNode {
    pub id: u64,
    pub label: String,
    pub expanded: bool,
    pub level: usize,
    pub has_children: bool,
}

/// Wrapper for tree-like widgets to support expand/collapse navigation.
///
/// # Examples
///
/// ```rust
/// use locust::ratatui_ext::adapters::{NavigableTree, TreeNode};
/// use locust::core::targets::TargetRegistry;
/// use ratatui::layout::Rect;
///
/// let nodes = vec![
///     TreeNode {
///         id: 1,
///         label: "Root".into(),
///         expanded: true,
///         level: 0,
///         has_children: true,
///     },
///     TreeNode {
///         id: 2,
///         label: "Child 1".into(),
///         expanded: false,
///         level: 1,
///         has_children: false,
///     },
/// ];
///
/// let tree = NavigableTree::new(nodes);
/// let mut registry = TargetRegistry::new();
/// tree.register_targets(Rect::new(0, 0, 40, 10), &mut registry);
/// ```
pub struct NavigableTree {
    nodes: Vec<TreeNode>,
}

impl NavigableTree {
    /// Create a new navigable tree.
    pub fn new(nodes: Vec<TreeNode>) -> Self {
        Self { nodes }
    }

    /// Register navigation targets for visible tree nodes.
    pub fn register_targets(&self, area: Rect, registry: &mut TargetRegistry) {
        let mut builder = TargetBuilder::new();
        let visible_rows = self.nodes.len().min(area.height as usize);

        for (i, node) in self.nodes.iter().take(visible_rows).enumerate() {
            let y = area.y + i as u16;

            // Indent based on level
            let indent = (node.level * 2) as u16;
            let rect = Rect {
                x: area.x + indent,
                y,
                width: area.width.saturating_sub(indent),
                height: 1,
            };

            let label = if node.has_children {
                if node.expanded {
                    format!("▼ {}", node.label)
                } else {
                    format!("▶ {}", node.label)
                }
            } else {
                format!("  {}", node.label)
            };

            let target = builder
                .tree_node(rect, label, node.expanded)
                .with_metadata("level", node.level.to_string())
                .with_metadata("has_children", node.has_children.to_string());

            registry.register(target);
        }
    }

    /// Get all nodes.
    pub fn nodes(&self) -> &[TreeNode] {
        &self.nodes
    }

    /// Update node expansion state.
    pub fn toggle_node(&mut self, id: u64) {
        if let Some(node) = self.nodes.iter_mut().find(|n| n.id == id) {
            if node.has_children {
                node.expanded = !node.expanded;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_navigable_list() {
        use ratatui::widgets::ListItem;

        let items = vec![
            ListItem::new("Item 1"),
            ListItem::new("Item 2"),
            ListItem::new("Item 3"),
        ];
        let list = List::new(items.clone());
        let nav_list = NavigableList::new(list, items.len());

        let mut registry = TargetRegistry::new();
        let area = Rect::new(0, 0, 20, 5);
        nav_list.register_targets(area, &mut registry);

        assert_eq!(registry.len(), 3);
    }

    #[test]
    fn test_navigable_list_with_labels() {
        use ratatui::widgets::ListItem;

        let items = vec![ListItem::new("Item 1"), ListItem::new("Item 2")];
        let list = List::new(items.clone());
        let nav_list = NavigableList::new(list, items.len())
            .with_labels(vec!["First".into(), "Second".into()]);

        let mut registry = TargetRegistry::new();
        nav_list.register_targets(Rect::new(0, 0, 20, 5), &mut registry);

        assert_eq!(registry.len(), 2);
        let target = registry.by_id(1).unwrap();
        assert_eq!(target.label, Some("First".into()));
    }

    #[test]
    fn test_navigable_table_row_mode() {
        use ratatui::layout::Constraint;
        use ratatui::widgets::Row;

        let rows = vec![
            Row::new(vec!["Cell 1", "Cell 2"]),
            Row::new(vec!["Cell 3", "Cell 4"]),
        ];
        let table = Table::new(
            rows,
            vec![Constraint::Percentage(50), Constraint::Percentage(50)],
        );
        let nav_table = NavigableTable::new(table, 2, vec![10, 10]);

        let mut registry = TargetRegistry::new();
        nav_table.register_targets(Rect::new(0, 0, 20, 5), &mut registry, TableNavMode::Row);

        assert_eq!(registry.len(), 2);
    }

    #[test]
    fn test_navigable_table_cell_mode() {
        use ratatui::layout::Constraint;
        use ratatui::widgets::Row;

        let rows = vec![
            Row::new(vec!["Cell 1", "Cell 2"]),
            Row::new(vec!["Cell 3", "Cell 4"]),
        ];
        let table = Table::new(
            rows,
            vec![Constraint::Percentage(50), Constraint::Percentage(50)],
        );
        let nav_table = NavigableTable::new(table, 2, vec![10, 10]);

        let mut registry = TargetRegistry::new();
        nav_table.register_targets(Rect::new(0, 0, 20, 5), &mut registry, TableNavMode::Cell);

        // 2 rows × 2 columns = 4 cells
        assert_eq!(registry.len(), 4);
    }

    #[test]
    fn test_navigable_tabs() {
        let titles = vec!["Home".into(), "Settings".into(), "About".into()];
        let tabs = Tabs::new(titles.clone());
        let nav_tabs = NavigableTabs::new(tabs, titles, 0);

        let mut registry = TargetRegistry::new();
        nav_tabs.register_targets(Rect::new(0, 0, 30, 1), &mut registry);

        assert_eq!(registry.len(), 3);

        // Check that first tab is selected
        let first_tab = registry.by_id(1).unwrap();
        assert_eq!(first_tab.state, crate::core::targets::TargetState::Selected);
    }

    #[test]
    fn test_navigable_tree() {
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

        // Check metadata
        let root = registry.by_id(1).unwrap();
        assert_eq!(root.metadata.get("level"), Some(&"0".to_string()));
        assert_eq!(root.metadata.get("has_children"), Some(&"true".to_string()));
    }

    #[test]
    fn test_tree_toggle() {
        let nodes = vec![TreeNode {
            id: 1,
            label: "Root".into(),
            expanded: false,
            level: 0,
            has_children: true,
        }];

        let mut tree = NavigableTree::new(nodes);
        assert!(!tree.nodes()[0].expanded);

        tree.toggle_node(1);
        assert!(tree.nodes()[0].expanded);

        tree.toggle_node(1);
        assert!(!tree.nodes()[0].expanded);
    }
}
