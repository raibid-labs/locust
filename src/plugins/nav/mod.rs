//! Locust navigation plugin.
//!
//! This plugin provides Vimium-style hint-based navigation for terminal UIs.
//! Press 'f' to activate hint mode, then type hint characters to navigate to targets.
//!
//! # Features
//!
//! - Fast hint generation using home row keys
//! - Progressive hint matching (type partial hints)
//! - Priority-based hint assignment
//! - Customizable styling and configuration
//! - Automatic target discovery from TargetRegistry
//!
//! # Example
//!
//! ```rust,no_run
//! use locust::plugins::nav::NavPlugin;
//! use locust::core::context::LocustContext;
//! use locust::core::plugin::LocustPlugin;
//! use ratatui::backend::TestBackend;
//!
//! let mut ctx = LocustContext::default();
//! let mut nav_plugin = NavPlugin::new();
//! LocustPlugin::<TestBackend>::init(&mut nav_plugin, &mut ctx);
//! ```

pub mod config;
pub mod hints;
pub mod render;

// Re-export for easier access
pub use config::NavConfig;

use crate::core::context::LocustContext;
use crate::core::input::PluginEventResult;
use crate::core::plugin::LocustPlugin;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use hints::{HintGenerator, HintMatcher};
use ratatui::backend::Backend;
use ratatui::Frame;
use render::HintRenderer;
use log::info;

/// Current navigation mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NavMode {
    /// Normal mode - no hints visible
    Normal,
    /// Hint mode - hints are visible and accepting input
    Hint,
    // Future: Find, Command, Visual, etc.
}

/// Vimium-style navigation plugin with hint-based target selection.
///
/// This plugin manages the complete navigation workflow:
/// 1. User presses activation key (default: 'f')
/// 2. Hints are generated for all visible targets
/// 3. User types hint characters to narrow matches
/// 4. When a unique match is found, the target is activated
///
/// # Configuration
///
/// The plugin can be customized using `NavConfig`:
///
/// ```rust
/// use locust::plugins::nav::{NavPlugin, NavConfig};
///
/// let config = NavConfig::new()
///     .with_hint_key('g')
///     .with_charset("asdfghjkl;")
///     .with_max_hints(50);
///
/// let nav_plugin = NavPlugin::with_config(config);
/// ```
pub struct NavPlugin {
    /// Current navigation mode
    pub mode: NavMode,

    /// Plugin configuration
    config: NavConfig,

    /// Hint generator
    generator: HintGenerator,

    /// Hint matcher for input handling
    matcher: HintMatcher,

    /// Hint renderer
    renderer: HintRenderer,
}

impl Default for NavPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl NavPlugin {
    /// Creates a new navigation plugin with default configuration.
    pub fn new() -> Self {
        Self::with_config(NavConfig::default())
    }

    /// Creates a new navigation plugin with custom configuration.
    pub fn with_config(config: NavConfig) -> Self {
        let generator = HintGenerator::new(config.hint_charset.clone());
        let matcher = HintMatcher::new();
        let renderer = HintRenderer::new();

        Self {
            mode: NavMode::Normal,
            config,
            generator,
            matcher,
            renderer,
        }
    }

    /// Returns the current configuration.
    pub fn config(&self) -> &NavConfig {
        &self.config
    }

    /// Returns the current navigation mode.
    pub fn mode(&self) -> NavMode {
        self.mode
    }

    /// Enters hint mode and generates hints for visible targets.
    fn enter_hint_mode(&mut self, ctx: &mut LocustContext) {
        self.mode = NavMode::Hint;

        // Get all visible targets from registry
        let registry = &ctx.targets;
        let mut targets: Vec<_> = registry
            .all()
            .iter()
            .filter(|t| {
                // Filter by minimum area if configured
                if self.config.min_target_area > 1 {
                    t.area() >= self.config.min_target_area
                } else {
                    true
                }
            })
            .cloned()
            .collect();

        // Apply max hints limit if configured
        if self.config.max_hints > 0 && targets.len() > self.config.max_hints {
            // Sort by priority first
            targets.sort_by(|a, b| b.priority.cmp(&a.priority));
            targets.truncate(self.config.max_hints);
        }

        // Generate hints
        let hints = self.generator.generate(&targets);
        self.matcher.set_hints(hints);

        ctx.overlay.mark_has_overlay();
    }

    /// Exits hint mode and clears all hints.
    fn exit_hint_mode(&mut self) {
        self.mode = NavMode::Normal;
        self.matcher.clear();
    }

    /// Handles a character input in hint mode.
    ///
    /// Returns `Some(target_id)` if a target was selected.
    fn handle_hint_char(&mut self, c: char) -> Option<u64> {
        // Check if this character is in the charset
        if !self.config.hint_charset.contains(c) {
            return None;
        }

        self.matcher.push_char(c)
    }

    /// Activates the target with the given ID.
    fn activate_target(&mut self, target_id: u64, ctx: &mut LocustContext) {
        if let Some(target) = ctx.targets.by_id(target_id) {
            // Log the activation for debugging
            // In a real application, this would trigger the target's action
            info!(
                "Locust: Activated target {} ({:?})",
                target_id,
                target.label.as_deref().unwrap_or("unlabeled")
            );

            // TODO: Execute target action
            // match &target.action {
            //     TargetAction::Activate => { /* ... */ }
            //     TargetAction::Navigate(route) => { /* ... */ }
            //     _ => {}
            // }
        }

        // Exit hint mode after activation
        self.exit_hint_mode();
    }
}

impl<B> LocustPlugin<B> for NavPlugin
where
    B: Backend + 'static,
{
    fn id(&self) -> &'static str {
        "locust.nav"
    }

    fn priority(&self) -> i32 {
        50 // Built-in plugin, higher priority than user plugins
    }

    fn init(&mut self, _ctx: &mut LocustContext) {
        // Plugin is initialized
    }

    fn on_event(&mut self, event: &Event, ctx: &mut LocustContext) -> PluginEventResult {
        if let Event::Key(KeyEvent {
            code, modifiers, ..
        }) = event
        {
            match (self.mode, code, modifiers) {
                // Normal mode: activate hint mode on configured key
                (NavMode::Normal, KeyCode::Char(c), m)
                    if *c == self.config.hint_key && *m == KeyModifiers::NONE =>
                {
                    self.enter_hint_mode(ctx);
                    return PluginEventResult::ConsumedRequestRedraw;
                }

                // Hint mode: handle escape to exit
                (NavMode::Hint, KeyCode::Esc, _) => {
                    self.exit_hint_mode();
                    return PluginEventResult::ConsumedRequestRedraw;
                }

                // Hint mode: handle backspace to remove last character
                (NavMode::Hint, KeyCode::Backspace, _) => {
                    self.matcher.pop_char();
                    return PluginEventResult::ConsumedRequestRedraw;
                }

                // Hint mode: handle character input
                (NavMode::Hint, KeyCode::Char(c), m) if *m == KeyModifiers::NONE => {
                    if let Some(target_id) = self.handle_hint_char(*c) {
                        self.activate_target(target_id, ctx);
                    }
                    return PluginEventResult::ConsumedRequestRedraw;
                }

                _ => {}
            }
        }

        PluginEventResult::NotHandled
    }

    fn render_overlay(&self, frame: &mut Frame, ctx: &LocustContext) {
        if self.mode != NavMode::Hint {
            return;
        }

        // Render hint banner at top
        render::render_hint_banner(frame, &self.matcher, self.config.banner_style);

        // Render hints on targets
        self.renderer
            .render(frame, &self.matcher, &ctx.targets, &self.config);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::targets::{NavTarget, TargetPriority};
    use ratatui::layout::Rect;

    #[test]
    fn test_plugin_creation() {
        let plugin = NavPlugin::new();
        assert_eq!(plugin.mode, NavMode::Normal);
    }

    #[test]
    fn test_plugin_custom_config() {
        let config = NavConfig::new().with_hint_key('g').with_charset("abc");
        let plugin = NavPlugin::with_config(config);
        assert_eq!(plugin.config().hint_key, 'g');
        assert_eq!(plugin.config().hint_charset, "abc");
    }

    #[test]
    fn test_enter_exit_hint_mode() {
        let mut plugin = NavPlugin::new();
        let mut ctx = LocustContext::default();

        // Add some targets
        ctx.targets
            .register(NavTarget::new(1, Rect::new(0, 0, 10, 1)));
        ctx.targets
            .register(NavTarget::new(2, Rect::new(0, 2, 10, 1)));

        assert_eq!(plugin.mode, NavMode::Normal);

        plugin.enter_hint_mode(&mut ctx);
        assert_eq!(plugin.mode, NavMode::Hint);
        assert_eq!(plugin.matcher.hints().len(), 2);

        plugin.exit_hint_mode();
        assert_eq!(plugin.mode, NavMode::Normal);
        assert_eq!(plugin.matcher.hints().len(), 0);
    }

    #[test]
    fn test_hint_filtering_by_area() {
        let config = NavConfig::new().with_min_target_area(50);
        let mut plugin = NavPlugin::with_config(config);
        let mut ctx = LocustContext::default();

        // Add targets with different areas
        ctx.targets
            .register(NavTarget::new(1, Rect::new(0, 0, 10, 1))); // area = 10
        ctx.targets
            .register(NavTarget::new(2, Rect::new(0, 2, 10, 10))); // area = 100

        plugin.enter_hint_mode(&mut ctx);

        // Only the larger target should get a hint
        assert_eq!(plugin.matcher.hints().len(), 1);
        assert_eq!(plugin.matcher.hints()[0].target_id, 2);
    }

    #[test]
    fn test_max_hints_limit() {
        let config = NavConfig::new().with_max_hints(2);
        let mut plugin = NavPlugin::with_config(config);
        let mut ctx = LocustContext::default();

        // Add 3 targets with different priorities
        ctx.targets
            .register(NavTarget::new(1, Rect::new(0, 0, 10, 1)).with_priority(TargetPriority::Low));
        ctx.targets.register(
            NavTarget::new(2, Rect::new(0, 2, 10, 1)).with_priority(TargetPriority::High),
        );
        ctx.targets.register(
            NavTarget::new(3, Rect::new(0, 4, 10, 1)).with_priority(TargetPriority::Normal),
        );

        plugin.enter_hint_mode(&mut ctx);

        // Should only generate 2 hints for highest priority targets
        assert_eq!(plugin.matcher.hints().len(), 2);

        let hint_targets: Vec<_> = plugin.matcher.hints().iter().map(|h| h.target_id).collect();
        assert!(hint_targets.contains(&2)); // High priority
        assert!(hint_targets.contains(&3)); // Normal priority
        assert!(!hint_targets.contains(&1)); // Low priority filtered out
    }
}
