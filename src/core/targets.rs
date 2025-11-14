use ratatui::layout::Rect;
use std::collections::HashMap;

/// Action to perform when a navigation target is activated.
///
/// This enum defines the various actions that can be triggered when a user
/// selects or activates a navigation target. Custom actions can be defined
/// using the `Custom` variant with arbitrary strings.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TargetAction {
    /// Select or focus the target without activating it.
    /// This is useful for highlighting items before confirming a choice.
    Select,

    /// Activate or execute the target's primary action.
    /// For buttons this triggers a click, for list items it opens/expands them.
    Activate,

    /// Scroll to bring the target into view.
    /// This ensures the target is visible in the viewport.
    Scroll,

    /// Navigate to a specific location or route.
    /// The string parameter specifies the destination.
    Navigate(String),

    /// Custom action defined by the application.
    /// This allows plugins and applications to define domain-specific actions.
    Custom(String),
}

impl Default for TargetAction {
    fn default() -> Self {
        Self::Activate
    }
}

/// Visual state of a navigation target.
///
/// Represents the current visual/interaction state of a target,
/// which can be used to determine how the target should be rendered.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TargetState {
    /// Normal, inactive state - default appearance.
    Normal,

    /// Target is hovered or has keyboard focus.
    /// Usually rendered with subtle highlighting.
    Highlighted,

    /// Target is currently selected/active.
    /// Usually rendered with prominent highlighting.
    Selected,

    /// Target is disabled and cannot be interacted with.
    /// Usually rendered with muted colors.
    Disabled,
}

impl Default for TargetState {
    fn default() -> Self {
        Self::Normal
    }
}

/// Priority level for target selection.
///
/// When multiple targets overlap or compete for the same hint,
/// higher priority targets are preferred.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TargetPriority {
    /// Low priority - background elements, decorations.
    Low = 0,

    /// Normal priority - regular interactive elements.
    Normal = 1,

    /// High priority - important actions, primary navigation.
    High = 2,

    /// Critical priority - emergency actions, essential navigation.
    Critical = 3,
}

impl Default for TargetPriority {
    fn default() -> Self {
        Self::Normal
    }
}

/// Callback function type for target activation.
///
/// When a target is activated, this callback is invoked with the target ID.
/// Returns `true` if the action was handled, `false` otherwise.
pub type TargetCallback = Box<dyn Fn(u64) -> bool + Send + Sync>;

/// A navigable region in the UI, such as a list row, table cell, tab, or button.
///
/// Navigation targets represent interactive elements that users can focus on,
/// select, or activate using keyboard shortcuts. Each target has a unique ID,
/// screen position, optional metadata, and can be in different visual states.
///
/// # Examples
///
/// ```rust
/// use locust::core::targets::{NavTarget, TargetAction, TargetPriority};
/// use ratatui::layout::Rect;
///
/// // Create a simple button target
/// let button = NavTarget::new(1, Rect::new(10, 5, 20, 3))
///     .with_label("Submit")
///     .with_action(TargetAction::Activate)
///     .with_priority(TargetPriority::High);
///
/// // Create a navigation link
/// let link = NavTarget::new(2, Rect::new(10, 10, 30, 1))
///     .with_label("Go to Settings")
///     .with_action(TargetAction::Navigate("/settings".into()));
/// ```
#[derive(Debug, Clone)]
pub struct NavTarget {
    /// Unique identifier for this target.
    pub id: u64,

    /// Screen rectangle occupied by this target.
    pub rect: Rect,

    /// Optional human-readable label for the target.
    pub label: Option<String>,

    /// Action to perform when this target is activated.
    pub action: TargetAction,

    /// Current visual state of the target.
    pub state: TargetState,

    /// Priority level for hint generation and selection.
    pub priority: TargetPriority,

    /// Optional group identifier for related targets.
    /// Targets in the same group can be navigated as a unit.
    pub group: Option<String>,

    /// Optional metadata for application-specific data.
    pub metadata: HashMap<String, String>,
}

impl NavTarget {
    /// Creates a new navigation target with the given ID and rectangle.
    ///
    /// The target is created with default action (`Activate`), state (`Normal`),
    /// and priority (`Normal`).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use locust::core::targets::NavTarget;
    /// use ratatui::layout::Rect;
    ///
    /// let target = NavTarget::new(1, Rect::new(0, 0, 10, 1));
    /// ```
    pub fn new(id: u64, rect: Rect) -> Self {
        Self {
            id,
            rect,
            label: None,
            action: TargetAction::default(),
            state: TargetState::default(),
            priority: TargetPriority::default(),
            group: None,
            metadata: HashMap::new(),
        }
    }

    /// Sets the label for this target.
    ///
    /// The label is displayed in hints or tooltips.
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Sets the action for this target.
    pub fn with_action(mut self, action: TargetAction) -> Self {
        self.action = action;
        self
    }

    /// Sets the state for this target.
    pub fn with_state(mut self, state: TargetState) -> Self {
        self.state = state;
        self
    }

    /// Sets the priority for this target.
    pub fn with_priority(mut self, priority: TargetPriority) -> Self {
        self.priority = priority;
        self
    }

    /// Sets the group identifier for this target.
    pub fn with_group(mut self, group: impl Into<String>) -> Self {
        self.group = Some(group.into());
        self
    }

    /// Adds a metadata key-value pair to this target.
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Checks if a point (x, y) is inside this target's rectangle.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use locust::core::targets::NavTarget;
    /// use ratatui::layout::Rect;
    ///
    /// let target = NavTarget::new(1, Rect::new(5, 5, 10, 10));
    /// assert!(target.contains_point(10, 10));
    /// assert!(!target.contains_point(0, 0));
    /// ```
    pub fn contains_point(&self, x: u16, y: u16) -> bool {
        x >= self.rect.x
            && x < self.rect.x + self.rect.width
            && y >= self.rect.y
            && y < self.rect.y + self.rect.height
    }

    /// Checks if this target overlaps with another rectangle.
    pub fn overlaps_rect(&self, other: &Rect) -> bool {
        self.rect.x < other.x + other.width
            && self.rect.x + self.rect.width > other.x
            && self.rect.y < other.y + other.height
            && self.rect.y + self.rect.height > other.y
    }

    /// Returns the center point of this target.
    pub fn center(&self) -> (u16, u16) {
        (
            self.rect.x + self.rect.width / 2,
            self.rect.y + self.rect.height / 2,
        )
    }

    /// Returns the area (width * height) of this target.
    pub fn area(&self) -> u32 {
        self.rect.width as u32 * self.rect.height as u32
    }
}

/// Registry of navigation targets discovered during a frame.
///
/// The registry maintains a collection of targets and provides efficient
/// spatial queries for finding targets by position, area, or priority.
/// Targets are typically registered during the rendering phase and cleared
/// at the start of each frame.
///
/// # Thread Safety
///
/// The registry is not thread-safe by default. If you need to share it
/// across threads, wrap it in `Arc<Mutex<TargetRegistry>>`.
///
/// # Examples
///
/// ```rust
/// use locust::core::targets::{NavTarget, TargetRegistry, TargetPriority};
/// use ratatui::layout::Rect;
///
/// let mut registry = TargetRegistry::new();
///
/// // Register some targets
/// registry.register(
///     NavTarget::new(1, Rect::new(0, 0, 10, 1))
///         .with_label("Button 1")
///         .with_priority(TargetPriority::High)
/// );
///
/// registry.register(
///     NavTarget::new(2, Rect::new(0, 2, 10, 1))
///         .with_label("Button 2")
/// );
///
/// // Query targets
/// assert_eq!(registry.len(), 2);
/// let high_priority = registry.by_priority(TargetPriority::High);
/// assert_eq!(high_priority.len(), 1);
/// ```
#[derive(Debug, Default)]
pub struct TargetRegistry {
    targets: Vec<NavTarget>,
    targets_by_id: HashMap<u64, usize>,
}

impl TargetRegistry {
    /// Creates a new empty target registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Clears all registered targets.
    ///
    /// This should be called at the beginning of each frame to remove
    /// targets from the previous frame.
    pub fn clear(&mut self) {
        self.targets.clear();
        self.targets_by_id.clear();
    }

    /// Registers a new navigation target.
    ///
    /// If a target with the same ID already exists, it will be replaced.
    pub fn register(&mut self, target: NavTarget) {
        if let Some(&idx) = self.targets_by_id.get(&target.id) {
            self.targets[idx] = target;
        } else {
            let idx = self.targets.len();
            self.targets_by_id.insert(target.id, idx);
            self.targets.push(target);
        }
    }

    /// Returns all registered targets as a slice.
    pub fn all(&self) -> &[NavTarget] {
        &self.targets
    }

    /// Returns the number of registered targets.
    pub fn len(&self) -> usize {
        self.targets.len()
    }

    /// Returns true if no targets are registered.
    pub fn is_empty(&self) -> bool {
        self.targets.is_empty()
    }

    /// Finds a target by its ID.
    pub fn by_id(&self, id: u64) -> Option<&NavTarget> {
        self.targets_by_id.get(&id).map(|&idx| &self.targets[idx])
    }

    /// Finds a mutable reference to a target by its ID.
    pub fn by_id_mut(&mut self, id: u64) -> Option<&mut NavTarget> {
        self.targets_by_id
            .get(&id)
            .map(|&idx| &mut self.targets[idx])
    }

    /// Returns all targets at the given point (x, y).
    ///
    /// If multiple targets overlap at this point, all are returned.
    pub fn at_point(&self, x: u16, y: u16) -> Vec<&NavTarget> {
        self.targets
            .iter()
            .filter(|t| t.contains_point(x, y))
            .collect()
    }

    /// Returns all targets that overlap with the given rectangle.
    pub fn in_area(&self, area: Rect) -> Vec<&NavTarget> {
        self.targets
            .iter()
            .filter(|t| t.overlaps_rect(&area))
            .collect()
    }

    /// Returns all targets with the specified priority level.
    pub fn by_priority(&self, priority: TargetPriority) -> Vec<&NavTarget> {
        self.targets
            .iter()
            .filter(|t| t.priority == priority)
            .collect()
    }

    /// Returns all targets in the specified group.
    pub fn by_group(&self, group: &str) -> Vec<&NavTarget> {
        self.targets
            .iter()
            .filter(|t| t.group.as_deref() == Some(group))
            .collect()
    }

    /// Returns all targets with the specified state.
    pub fn by_state(&self, state: TargetState) -> Vec<&NavTarget> {
        self.targets.iter().filter(|t| t.state == state).collect()
    }

    /// Returns targets sorted by priority (highest first).
    pub fn sorted_by_priority(&self) -> Vec<&NavTarget> {
        let mut targets: Vec<&NavTarget> = self.targets.iter().collect();
        targets.sort_by(|a, b| b.priority.cmp(&a.priority));
        targets
    }

    /// Returns targets sorted by area (largest first).
    ///
    /// This is useful for selecting the most prominent targets first.
    pub fn sorted_by_area(&self) -> Vec<&NavTarget> {
        let mut targets: Vec<&NavTarget> = self.targets.iter().collect();
        targets.sort_by_key(|b| std::cmp::Reverse(b.area()));
        targets
    }

    /// Returns the target closest to the given point.
    pub fn closest_to(&self, x: u16, y: u16) -> Option<&NavTarget> {
        self.targets.iter().min_by_key(|t| {
            let (cx, cy) = t.center();
            let dx = (cx as i32 - x as i32).abs();
            let dy = (cy as i32 - y as i32).abs();
            dx * dx + dy * dy // squared distance (no need for sqrt)
        })
    }

    /// Removes a target by its ID.
    ///
    /// Returns `true` if the target was removed, `false` if it didn't exist.
    pub fn remove(&mut self, id: u64) -> bool {
        if let Some(&idx) = self.targets_by_id.get(&id) {
            self.targets.remove(idx);
            self.targets_by_id.remove(&id);
            // Rebuild index since indices shifted
            self.rebuild_index();
            true
        } else {
            false
        }
    }

    /// Rebuilds the internal index after removal operations.
    fn rebuild_index(&mut self) {
        self.targets_by_id.clear();
        for (idx, target) in self.targets.iter().enumerate() {
            self.targets_by_id.insert(target.id, idx);
        }
    }
}

/// Builder for creating common navigation target patterns.
///
/// Provides a convenient API for creating standard target types like buttons,
/// list items, tabs, and tree nodes with sensible defaults.
pub struct TargetBuilder {
    next_id: u64,
}

impl TargetBuilder {
    /// Creates a new target builder.
    pub fn new() -> Self {
        Self { next_id: 1 }
    }

    /// Creates a new target builder starting from a specific ID.
    pub fn with_start_id(next_id: u64) -> Self {
        Self { next_id }
    }

    /// Gets the next available ID and increments the counter.
    pub fn next_id(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    /// Creates a button target with high priority.
    pub fn button(&mut self, rect: Rect, label: impl Into<String>) -> NavTarget {
        NavTarget::new(self.next_id(), rect)
            .with_label(label)
            .with_action(TargetAction::Activate)
            .with_priority(TargetPriority::High)
    }

    /// Creates a list item target.
    pub fn list_item(&mut self, rect: Rect, label: impl Into<String>) -> NavTarget {
        NavTarget::new(self.next_id(), rect)
            .with_label(label)
            .with_action(TargetAction::Select)
            .with_priority(TargetPriority::Normal)
    }

    /// Creates a tab target with high priority.
    pub fn tab(&mut self, rect: Rect, label: impl Into<String>) -> NavTarget {
        NavTarget::new(self.next_id(), rect)
            .with_label(label)
            .with_action(TargetAction::Activate)
            .with_priority(TargetPriority::High)
            .with_group("tabs")
    }

    /// Creates a tree node target that can be expanded/collapsed.
    pub fn tree_node(&mut self, rect: Rect, label: impl Into<String>, expanded: bool) -> NavTarget {
        let mut target = NavTarget::new(self.next_id(), rect)
            .with_label(label)
            .with_action(TargetAction::Activate)
            .with_priority(TargetPriority::Normal);

        target
            .metadata
            .insert("expanded".into(), expanded.to_string());
        target
    }

    /// Creates a link target that navigates to a route.
    pub fn link(
        &mut self,
        rect: Rect,
        label: impl Into<String>,
        route: impl Into<String>,
    ) -> NavTarget {
        NavTarget::new(self.next_id(), rect)
            .with_label(label)
            .with_action(TargetAction::Navigate(route.into()))
            .with_priority(TargetPriority::Normal)
    }

    /// Creates a custom target with specified action and priority.
    pub fn custom(
        &mut self,
        rect: Rect,
        label: impl Into<String>,
        action: TargetAction,
        priority: TargetPriority,
    ) -> NavTarget {
        NavTarget::new(self.next_id(), rect)
            .with_label(label)
            .with_action(action)
            .with_priority(priority)
    }
}

impl Default for TargetBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_target_creation() {
        let target = NavTarget::new(1, Rect::new(10, 5, 20, 3))
            .with_label("Test Button")
            .with_action(TargetAction::Activate)
            .with_priority(TargetPriority::High);

        assert_eq!(target.id, 1);
        assert_eq!(target.label, Some("Test Button".to_string()));
        assert_eq!(target.action, TargetAction::Activate);
        assert_eq!(target.priority, TargetPriority::High);
        assert_eq!(target.state, TargetState::Normal);
    }

    #[test]
    fn test_target_contains_point() {
        let target = NavTarget::new(1, Rect::new(10, 10, 20, 20));

        assert!(target.contains_point(15, 15));
        assert!(target.contains_point(10, 10));
        assert!(target.contains_point(29, 29));
        assert!(!target.contains_point(30, 30));
        assert!(!target.contains_point(5, 5));
    }

    #[test]
    fn test_registry_basics() {
        let mut registry = TargetRegistry::new();

        registry.register(NavTarget::new(1, Rect::new(0, 0, 10, 1)));
        registry.register(NavTarget::new(2, Rect::new(0, 2, 10, 1)));

        assert_eq!(registry.len(), 2);
        assert!(!registry.is_empty());

        registry.clear();
        assert!(registry.is_empty());
    }
}
