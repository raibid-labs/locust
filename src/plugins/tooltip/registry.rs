//! Tooltip registry for managing target-to-tooltip mappings.
//!
//! This module provides a registry that maps navigation target IDs
//! to their associated tooltip content.

use super::content::TooltipContent;
use std::collections::HashMap;

/// Registry that maps target IDs to tooltip content.
///
/// Applications register tooltips for specific target IDs, and the
/// TooltipPlugin retrieves and displays them when targets are hovered.
///
/// # Examples
///
/// ```rust
/// use locust::plugins::tooltip::{TooltipRegistry, TooltipContent, TooltipStyle};
///
/// let mut registry = TooltipRegistry::new();
///
/// // Register a simple tooltip
/// registry.register(1, TooltipContent::new("Click to submit"));
///
/// // Register a tooltip with title and style
/// registry.register(
///     2,
///     TooltipContent::new("This will delete the item permanently")
///         .with_title("Warning")
///         .with_style(TooltipStyle::Warning)
/// );
///
/// // Retrieve tooltip
/// let tooltip = registry.get(1);
/// assert!(tooltip.is_some());
///
/// // Remove tooltip
/// registry.remove(1);
/// ```
#[derive(Debug, Default)]
pub struct TooltipRegistry {
    /// Map from target ID to tooltip content.
    tooltips: HashMap<u64, TooltipContent>,
}

impl TooltipRegistry {
    /// Creates a new empty tooltip registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a tooltip for a target.
    ///
    /// If a tooltip already exists for this target ID, it will be replaced.
    pub fn register(&mut self, target_id: u64, content: TooltipContent) {
        self.tooltips.insert(target_id, content);
    }

    /// Retrieves the tooltip for a target.
    ///
    /// Returns None if no tooltip is registered for this target.
    pub fn get(&self, target_id: u64) -> Option<&TooltipContent> {
        self.tooltips.get(&target_id)
    }

    /// Removes the tooltip for a target.
    ///
    /// Returns true if a tooltip was removed, false if none existed.
    pub fn remove(&mut self, target_id: u64) -> bool {
        self.tooltips.remove(&target_id).is_some()
    }

    /// Clears all registered tooltips.
    pub fn clear(&mut self) {
        self.tooltips.clear();
    }

    /// Returns the number of registered tooltips.
    pub fn len(&self) -> usize {
        self.tooltips.len()
    }

    /// Returns true if no tooltips are registered.
    pub fn is_empty(&self) -> bool {
        self.tooltips.is_empty()
    }

    /// Returns all registered target IDs.
    pub fn target_ids(&self) -> Vec<u64> {
        self.tooltips.keys().copied().collect()
    }

    /// Checks if a tooltip exists for a target.
    pub fn contains(&self, target_id: u64) -> bool {
        self.tooltips.contains_key(&target_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugins::tooltip::content::TooltipStyle;

    #[test]
    fn test_registry_creation() {
        let registry = TooltipRegistry::new();
        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);
    }

    #[test]
    fn test_register_and_retrieve() {
        let mut registry = TooltipRegistry::new();
        let content = TooltipContent::new("Test tooltip");

        registry.register(1, content.clone());
        assert_eq!(registry.len(), 1);
        assert!(registry.contains(1));

        let retrieved = registry.get(1);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().body, "Test tooltip");
    }

    #[test]
    fn test_register_overwrites() {
        let mut registry = TooltipRegistry::new();

        registry.register(1, TooltipContent::new("First"));
        registry.register(1, TooltipContent::new("Second"));

        assert_eq!(registry.len(), 1);
        assert_eq!(registry.get(1).unwrap().body, "Second");
    }

    #[test]
    fn test_remove() {
        let mut registry = TooltipRegistry::new();
        registry.register(1, TooltipContent::new("Test"));

        assert!(registry.remove(1));
        assert!(!registry.contains(1));
        assert!(registry.is_empty());

        // Removing non-existent returns false
        assert!(!registry.remove(99));
    }

    #[test]
    fn test_clear() {
        let mut registry = TooltipRegistry::new();
        registry.register(1, TooltipContent::new("Test 1"));
        registry.register(2, TooltipContent::new("Test 2"));

        assert_eq!(registry.len(), 2);

        registry.clear();
        assert!(registry.is_empty());
        assert!(!registry.contains(1));
        assert!(!registry.contains(2));
    }

    #[test]
    fn test_target_ids() {
        let mut registry = TooltipRegistry::new();
        registry.register(1, TooltipContent::new("Test 1"));
        registry.register(5, TooltipContent::new("Test 2"));
        registry.register(10, TooltipContent::new("Test 3"));

        let mut ids = registry.target_ids();
        ids.sort();
        assert_eq!(ids, vec![1, 5, 10]);
    }

    #[test]
    fn test_multiple_tooltips_with_styles() {
        let mut registry = TooltipRegistry::new();

        registry.register(
            1,
            TooltipContent::new("Info tooltip").with_style(TooltipStyle::Info),
        );
        registry.register(
            2,
            TooltipContent::new("Warning tooltip")
                .with_title("Warning")
                .with_style(TooltipStyle::Warning),
        );
        registry.register(
            3,
            TooltipContent::new("Error tooltip").with_style(TooltipStyle::Error),
        );

        assert_eq!(registry.len(), 3);
        assert_eq!(registry.get(1).unwrap().style, TooltipStyle::Info);
        assert_eq!(registry.get(2).unwrap().style, TooltipStyle::Warning);
        assert_eq!(registry.get(2).unwrap().title, Some("Warning".to_string()));
        assert_eq!(registry.get(3).unwrap().style, TooltipStyle::Error);
    }
}
