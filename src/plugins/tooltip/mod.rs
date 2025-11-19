//! Locust tooltip plugin - Contextual help for UI elements.
//!
//! This plugin provides hoverable/focusable tooltips that display helpful
//! information when users interact with navigation targets.
//!
//! # Features
//!
//! - Hover or keyboard activation
//! - Smart positioning with edge detection
//! - Multiple visual styles (Info, Warning, Error, Success)
//! - Rich content with titles and multi-line text
//! - Optional arrows pointing to targets
//! - Auto-hide timeout support
//!
//! # Example
//!
//! ```rust,no_run
//! use locust::plugins::tooltip::{TooltipPlugin, TooltipContent, TooltipStyle};
//! use locust::core::context::LocustContext;
//! use locust::core::plugin::LocustPlugin;
//! use ratatui::backend::TestBackend;
//!
//! let mut ctx = LocustContext::default();
//! let mut tooltip_plugin = TooltipPlugin::new();
//!
//! // Initialize plugin
//! LocustPlugin::<TestBackend>::init(&mut tooltip_plugin, &mut ctx);
//!
//! // Register tooltips for targets
//! ctx.tooltips.register(
//!     1,
//!     TooltipContent::new("Activate navigation hints")
//!         .with_title("Navigation Mode")
//! );
//!
//! ctx.tooltips.register(
//!     2,
//!     TooltipContent::new("This action cannot be undone")
//!         .with_title("Warning")
//!         .with_style(TooltipStyle::Warning)
//! );
//! ```

pub mod config;
pub mod content;
pub mod positioning;
pub mod registry;
pub mod render;

// Re-exports for easier access
pub use config::TooltipConfig;
pub use content::{TooltipContent, TooltipStyle};
pub use registry::TooltipRegistry;

use crate::core::context::LocustContext;
use crate::core::input::PluginEventResult;
use crate::core::plugin::LocustPlugin;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use positioning::TooltipPositioner;
use ratatui::backend::Backend;
use ratatui::Frame;
use render::TooltipRenderer;
use std::time::{Duration, Instant};

/// Current tooltip display state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TooltipMode {
    /// No tooltip is being displayed.
    Hidden,

    /// Tooltip is currently visible.
    Visible,

    /// Waiting for hover delay before showing tooltip.
    Pending,
}

/// Tooltip plugin for contextual help overlays.
///
/// This plugin manages tooltip display for navigation targets:
/// 1. User hovers over a target (or presses activation key)
/// 2. After configured delay, tooltip appears
/// 3. Tooltip auto-hides after timeout (if configured) or on user action
///
/// # Configuration
///
/// ```rust
/// use locust::plugins::tooltip::{TooltipPlugin, TooltipConfig};
///
/// let config = TooltipConfig::new()
///     .with_activation_key('?')
///     .with_hover_delay_ms(500)
///     .with_auto_hide_timeout_ms(5000)
///     .with_max_width(60);
///
/// let tooltip_plugin = TooltipPlugin::with_config(config);
/// ```
pub struct TooltipPlugin {
    /// Plugin configuration.
    config: TooltipConfig,

    /// Current display mode.
    mode: TooltipMode,

    /// ID of the target currently showing a tooltip (if any).
    current_target_id: Option<u64>,

    /// When the current tooltip was shown (for auto-hide).
    shown_at: Option<Instant>,

    /// When hover started (for activation delay).
    hover_started_at: Option<Instant>,

    /// Target ID being hovered (for pending tooltips).
    pending_target_id: Option<u64>,

    /// Positioner for calculating tooltip placement.
    positioner: TooltipPositioner,

    /// Renderer for drawing tooltips.
    renderer: TooltipRenderer,
}

impl Default for TooltipPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl TooltipPlugin {
    /// Creates a new tooltip plugin with default configuration.
    pub fn new() -> Self {
        Self::with_config(TooltipConfig::default())
    }

    /// Creates a new tooltip plugin with custom configuration.
    pub fn with_config(config: TooltipConfig) -> Self {
        let positioner = TooltipPositioner::new(
            config.offset_x,
            config.offset_y,
            config.padding,
            config.show_border,
            config.prefer_right,
            config.prefer_bottom,
        );

        let renderer = TooltipRenderer::new(config.show_border, config.show_arrow);

        Self {
            config,
            mode: TooltipMode::Hidden,
            current_target_id: None,
            shown_at: None,
            hover_started_at: None,
            pending_target_id: None,
            positioner,
            renderer,
        }
    }

    /// Returns the current configuration.
    pub fn config(&self) -> &TooltipConfig {
        &self.config
    }

    /// Returns the current mode.
    pub fn mode(&self) -> TooltipMode {
        self.mode
    }

    /// Shows a tooltip for the given target immediately.
    fn show_tooltip(&mut self, target_id: u64, ctx: &mut LocustContext) {
        self.mode = TooltipMode::Visible;
        self.current_target_id = Some(target_id);
        self.shown_at = Some(Instant::now());
        self.hover_started_at = None;
        self.pending_target_id = None;
        ctx.overlay.mark_has_overlay();
    }

    /// Hides the current tooltip.
    fn hide_tooltip(&mut self) {
        self.mode = TooltipMode::Hidden;
        self.current_target_id = None;
        self.shown_at = None;
        self.hover_started_at = None;
        self.pending_target_id = None;
    }

    /// Starts the hover delay for a target.
    #[allow(dead_code)]
    fn start_hover(&mut self, target_id: u64) {
        if self.config.hover_delay_ms == 0 {
            // No delay, show immediately (but we still need context, so mark pending)
            self.mode = TooltipMode::Pending;
            self.pending_target_id = Some(target_id);
            self.hover_started_at = Some(Instant::now());
        } else {
            self.mode = TooltipMode::Pending;
            self.pending_target_id = Some(target_id);
            self.hover_started_at = Some(Instant::now());
        }
    }

    /// Checks if auto-hide timeout has expired.
    fn check_auto_hide(&mut self) -> bool {
        if self.config.auto_hide_timeout_ms == 0 {
            return false;
        }

        if let Some(shown_at) = self.shown_at {
            let elapsed = shown_at.elapsed();
            let timeout = Duration::from_millis(self.config.auto_hide_timeout_ms);

            if elapsed >= timeout {
                self.hide_tooltip();
                return true;
            }
        }

        false
    }

    /// Checks if hover delay has elapsed and shows tooltip if ready.
    fn check_hover_delay(&mut self, ctx: &mut LocustContext) -> bool {
        if self.mode != TooltipMode::Pending {
            return false;
        }

        if let (Some(started_at), Some(target_id)) = (self.hover_started_at, self.pending_target_id)
        {
            let elapsed = started_at.elapsed();
            let delay = Duration::from_millis(self.config.hover_delay_ms);

            if elapsed >= delay {
                // Check if target still has a tooltip registered
                if ctx.tooltips.contains(target_id) {
                    self.show_tooltip(target_id, ctx);
                    return true;
                } else {
                    self.hide_tooltip();
                }
            }
        }

        false
    }
}

impl<B> LocustPlugin<B> for TooltipPlugin
where
    B: Backend + 'static,
{
    fn id(&self) -> &'static str {
        "locust.tooltip"
    }

    fn priority(&self) -> i32 {
        45 // Between omnibar (40) and nav (50)
    }

    fn init(&mut self, _ctx: &mut LocustContext) {
        // Plugin is initialized
    }

    fn on_event(&mut self, event: &Event, ctx: &mut LocustContext) -> PluginEventResult {
        // Check for auto-hide timeout
        if self.check_auto_hide() {
            return PluginEventResult::ConsumedRequestRedraw;
        }

        // Check for hover delay expiration
        if self.check_hover_delay(ctx) {
            return PluginEventResult::ConsumedRequestRedraw;
        }

        if let Event::Key(KeyEvent {
            code, modifiers, ..
        }) = event
        {
            match (code, modifiers) {
                // Activation key pressed (if configured)
                (KeyCode::Char(c), m)
                    if self.config.activation_key == Some(*c) && *m == KeyModifiers::NONE =>
                {
                    // TODO: Determine which target has focus
                    // For now, just toggle if already visible
                    if self.mode == TooltipMode::Visible {
                        self.hide_tooltip();
                    } else {
                        // In real implementation, would get focused target from context
                        // For demonstration, we'll just note this in comments
                        // self.show_tooltip(focused_target_id, ctx);
                    }
                    return PluginEventResult::ConsumedRequestRedraw;
                }

                // Escape or any movement hides tooltip
                (KeyCode::Esc, _)
                | (KeyCode::Up, _)
                | (KeyCode::Down, _)
                | (KeyCode::Left, _)
                | (KeyCode::Right, _)
                    if self.mode == TooltipMode::Visible =>
                {
                    self.hide_tooltip();
                    return PluginEventResult::Consumed;
                }

                _ => {}
            }
        }

        PluginEventResult::NotHandled
    }

    fn render_overlay(&self, frame: &mut Frame, ctx: &LocustContext) {
        if self.mode != TooltipMode::Visible {
            return;
        }

        let Some(target_id) = self.current_target_id else {
            return;
        };

        // Get target and tooltip content
        let Some(target) = ctx.targets.by_id(target_id) else {
            return;
        };

        let Some(content) = ctx.tooltips.get(target_id) else {
            return;
        };

        // Calculate dimensions
        let content_width = content.max_line_width().min(self.config.max_width as usize) as u16;
        let content_height = content.line_count().min(self.config.max_height as usize) as u16;

        // Calculate position
        let screen_rect = frame.area();
        let position =
            self.positioner
                .calculate(target.rect, content_width, content_height, screen_rect);

        // Render tooltip
        self.renderer.render(frame.buffer_mut(), content, &position);
    }

    fn cleanup(&mut self, _ctx: &mut LocustContext) {
        self.hide_tooltip();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::targets::NavTarget;
    use ratatui::layout::Rect;

    #[test]
    fn test_plugin_creation() {
        let plugin = TooltipPlugin::new();
        assert_eq!(plugin.mode(), TooltipMode::Hidden);
        assert!(plugin.current_target_id.is_none());
    }

    #[test]
    fn test_plugin_custom_config() {
        let config = TooltipConfig::new()
            .with_activation_key('?')
            .with_max_width(80);
        let plugin = TooltipPlugin::with_config(config);
        assert_eq!(plugin.config().activation_key, Some('?'));
        assert_eq!(plugin.config().max_width, 80);
    }

    #[test]
    fn test_show_hide_tooltip() {
        let mut plugin = TooltipPlugin::new();
        let mut ctx = LocustContext::default();

        // Register target and tooltip
        ctx.targets
            .register(NavTarget::new(1, Rect::new(10, 10, 20, 3)));
        ctx.tooltips
            .register(1, TooltipContent::new("Test tooltip"));

        assert_eq!(plugin.mode(), TooltipMode::Hidden);

        plugin.show_tooltip(1, &mut ctx);
        assert_eq!(plugin.mode(), TooltipMode::Visible);
        assert_eq!(plugin.current_target_id, Some(1));

        plugin.hide_tooltip();
        assert_eq!(plugin.mode(), TooltipMode::Hidden);
        assert!(plugin.current_target_id.is_none());
    }

    #[test]
    fn test_start_hover() {
        let mut plugin = TooltipPlugin::new();

        plugin.start_hover(5);
        assert_eq!(plugin.mode(), TooltipMode::Pending);
        assert_eq!(plugin.pending_target_id, Some(5));
        assert!(plugin.hover_started_at.is_some());
    }

    #[test]
    fn test_auto_hide_disabled_by_default() {
        let mut plugin = TooltipPlugin::new();
        let mut ctx = LocustContext::default();

        ctx.targets
            .register(NavTarget::new(1, Rect::new(10, 10, 20, 3)));
        ctx.tooltips.register(1, TooltipContent::new("Test"));

        plugin.show_tooltip(1, &mut ctx);

        // With default config (auto_hide_timeout_ms = 0), should not auto-hide
        assert!(!plugin.check_auto_hide());
        assert_eq!(plugin.mode(), TooltipMode::Visible);
    }

    #[test]
    fn test_plugin_priority() {
        use ratatui::backend::TestBackend;
        let plugin = TooltipPlugin::new();
        // Should be between omnibar (40) and nav (50)
        assert_eq!(
            <TooltipPlugin as LocustPlugin<TestBackend>>::priority(&plugin),
            45
        );
    }
}
