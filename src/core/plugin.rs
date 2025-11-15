use crate::core::context::LocustContext;
use crate::core::input::PluginEventResult;
use crossterm::event::Event;
use ratatui::backend::Backend;
use ratatui::Frame;

/// Trait implemented by all Locust plugins.
///
/// Plugins can:
/// - Observe and optionally consume input events.
/// - Render overlays on top of the existing ratatui UI.
/// - Initialize resources during setup.
/// - Clean up resources when being unregistered.
///
/// # Plugin Lifecycle
///
/// 1. **Construction**: Plugin is created by user code
/// 2. **Initialization**: `init()` is called when registered with Locust
/// 3. **Runtime**: `on_event()` and `render_overlay()` called each frame
/// 4. **Cleanup**: `cleanup()` called when plugin is unregistered or Locust is dropped
///
/// # Plugin Priority
///
/// Plugins are ordered by priority (lower numbers = higher priority).
/// Higher priority plugins receive events first and can consume them,
/// preventing lower priority plugins from seeing those events.
///
/// # Example
///
/// ```ignore
/// use locust::prelude::*;
/// use ratatui::backend::CrosstermBackend;
/// use crossterm::event::{Event, KeyCode, KeyEvent};
///
/// struct MyPlugin {
///     active: bool,
/// }
///
/// impl<B: Backend> LocustPlugin<B> for MyPlugin {
///     fn id(&self) -> &'static str {
///         "my_plugin"
///     }
///
///     fn priority(&self) -> i32 {
///         100  // Default priority
///     }
///
///     fn init(&mut self, ctx: &mut LocustContext) {
///         println!("Plugin initialized!");
///         self.active = true;
///     }
///
///     fn on_event(&mut self, event: &Event, ctx: &mut LocustContext) -> PluginEventResult {
///         if let Event::Key(KeyEvent { code: KeyCode::Char('q'), .. }) = event {
///             return PluginEventResult::ConsumedRequestRedraw;
///         }
///         PluginEventResult::NotHandled
///     }
///
///     fn render_overlay(&self, frame: &mut Frame, ctx: &LocustContext) {
///         // Render your overlay here
///     }
///
///     fn cleanup(&mut self, ctx: &mut LocustContext) {
///         println!("Plugin cleaned up!");
///         self.active = false;
///     }
/// }
/// ```
pub trait LocustPlugin<B>
where
    B: Backend,
{
    /// Stable, unique identifier for the plugin.
    ///
    /// This should be a constant string that uniquely identifies your plugin.
    /// Recommended format: `vendor.plugin_name` (e.g., "locust.nav")
    fn id(&self) -> &'static str;

    /// Plugin priority for event handling order.
    ///
    /// Lower numbers = higher priority (processed first).
    /// Default is 100. Built-in plugins use priority 0-99.
    ///
    /// # Priority Guidelines
    /// - 0-49: Critical system plugins
    /// - 50-99: Built-in Locust plugins
    /// - 100-199: User plugins (default)
    /// - 200+: Low priority / fallback plugins
    fn priority(&self) -> i32 {
        100
    }

    /// Called once when the plugin is registered with Locust.
    ///
    /// Use this to:
    /// - Initialize plugin state
    /// - Register navigation targets
    /// - Set up resources
    /// - Perform one-time configuration
    ///
    /// # Arguments
    /// * `ctx` - Mutable reference to LocustContext for initialization
    fn init(&mut self, _ctx: &mut LocustContext) {}

    /// Allow the plugin to react to and optionally consume an input event.
    ///
    /// This is called for each input event in priority order. If a higher
    /// priority plugin consumes an event, lower priority plugins won't see it.
    ///
    /// # Arguments
    /// * `event` - The input event from crossterm
    /// * `ctx` - Mutable context for updating state
    ///
    /// # Returns
    /// * `PluginEventResult::NotHandled` - Pass event to next plugin
    /// * `PluginEventResult::Consumed` - Event consumed, no redraw needed
    /// * `PluginEventResult::ConsumedRequestRedraw` - Event consumed, trigger redraw
    fn on_event(&mut self, event: &Event, ctx: &mut LocustContext) -> PluginEventResult;

    /// Render this plugin's overlay on top of the current frame.
    ///
    /// Called after the application has rendered its base UI. Plugins
    /// should only render if they have active overlays to display.
    ///
    /// # Arguments
    /// * `frame` - The ratatui Frame to render into
    /// * `ctx` - Read-only context with target registry and overlay state
    fn render_overlay(&self, frame: &mut Frame, ctx: &LocustContext);

    /// Called when the plugin is being unregistered or Locust is shutting down.
    ///
    /// Use this to:
    /// - Release resources
    /// - Save state
    /// - Perform cleanup operations
    ///
    /// # Arguments
    /// * `ctx` - Mutable reference to LocustContext for cleanup
    fn cleanup(&mut self, _ctx: &mut LocustContext) {}

    /// Called when configuration is reloaded.
    ///
    /// Plugins can use this to update their internal state based on
    /// new configuration values loaded from file or updated at runtime.
    ///
    /// # Arguments
    /// * `ctx` - Read-only context to access updated configuration
    ///
    /// # Default Implementation
    /// Does nothing. Override to implement configuration hot-reload support.
    fn reload_config(&mut self, _ctx: &LocustContext) {}
}
