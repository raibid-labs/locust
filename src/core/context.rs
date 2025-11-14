use crate::core::input::LocustEventOutcome;
use crate::core::overlay::OverlayState;
use crate::core::plugin::LocustPlugin;
use crate::core::targets::TargetRegistry;
use crossterm::event::Event;
use ratatui::backend::Backend;
use ratatui::Frame;

/// Global configuration for Locust.
#[derive(Debug, Clone)]
pub struct LocustConfig {
    /// Whether the built-in navigation plugin should be registered by default.
    pub enable_nav_plugin: bool,
}

impl Default for LocustConfig {
    fn default() -> Self {
        Self {
            enable_nav_plugin: true,
        }
    }
}

/// Shared state passed to plugins: targets, overlay bookkeeping, etc.
///
/// This context is shared between all plugins and provides:
/// - Target registry for navigation
/// - Overlay state management
/// - Frame lifecycle tracking
/// - Plugin communication channels (future)
///
/// # Thread Safety
///
/// LocustContext is designed for single-threaded use within the ratatui
/// event loop. For multi-threaded scenarios, wrap in Arc<Mutex<>> or similar.
#[derive(Debug, Default)]
pub struct LocustContext {
    /// Registry of all navigation targets discovered in the current frame.
    pub targets: TargetRegistry,

    /// State tracking for overlay rendering and management.
    pub overlay: OverlayState,

    /// Frame counter for tracking render cycles.
    pub frame_count: u64,
}

/// Central entry point for embedding Locust into a ratatui app.
pub struct Locust<B>
where
    B: Backend,
{
    pub config: LocustConfig,
    pub ctx: LocustContext,
    plugins: Vec<Box<dyn LocustPlugin<B>>>,
}

impl<B> Locust<B>
where
    B: Backend,
{
    /// Create a new Locust instance with optional configuration.
    ///
    /// The caller is responsible for registering plugins. A convenience
    /// constructor that installs the built-in navigation plugin lives in
    /// the `plugins` module.
    pub fn new(config: LocustConfig) -> Self {
        Self {
            config,
            ctx: LocustContext::default(),
            plugins: Vec::new(),
        }
    }

    /// Register a plugin. Its `init` hook will be called immediately.
    ///
    /// Plugins are automatically sorted by priority after registration.
    /// Lower priority numbers are processed first.
    pub fn register_plugin<P>(&mut self, mut plugin: P)
    where
        P: LocustPlugin<B> + 'static,
    {
        plugin.init(&mut self.ctx);
        self.plugins.push(Box::new(plugin));
        // Sort by priority: lower numbers first
        self.plugins.sort_by_key(|p| p.priority());
    }

    /// Offer an input event to all plugins in registration order.
    ///
    /// Returns whether the event was consumed and whether the caller
    /// should trigger a redraw.
    pub fn on_event(&mut self, event: &Event) -> LocustEventOutcome {
        let mut results = Vec::with_capacity(self.plugins.len());
        for plugin in self.plugins.iter_mut() {
            let res = plugin.on_event(event, &mut self.ctx);
            results.push(res);
            if res.is_consumed() {
                // Stop on first consumer; plugins are ordered.
                break;
            }
        }

        use crate::core::input::LocustEventOutcome as Outcome;
        if results.is_empty() {
            Outcome::NOT_HANDLED
        } else {
            let consumed = results.iter().any(|r| r.is_consumed());
            let request_redraw = results.iter().any(|r| r.requests_redraw());
            Outcome {
                consumed,
                request_redraw,
            }
        }
    }

    /// Clear any per-frame state before drawing.
    ///
    /// This should be called at the start of each render loop iteration.
    pub fn begin_frame(&mut self) {
        self.ctx.targets.clear();
        self.ctx.overlay.begin_frame();
        self.ctx.frame_count = self.ctx.frame_count.wrapping_add(1);
    }

    /// Ask all plugins to render their overlays on top of the frame.
    ///
    /// This should be called *after* the application has rendered its
    /// widgets for the current frame. Plugins render in priority order.
    pub fn render_overlay(&self, frame: &mut Frame) {
        for plugin in self.plugins.iter() {
            plugin.render_overlay(frame, &self.ctx);
        }
    }

    /// Get the number of registered plugins.
    pub fn plugin_count(&self) -> usize {
        self.plugins.len()
    }

    /// Check if a plugin with the given ID is registered.
    pub fn has_plugin(&self, id: &str) -> bool {
        self.plugins.iter().any(|p| p.id() == id)
    }
}

impl<B> Drop for Locust<B>
where
    B: Backend,
{
    fn drop(&mut self) {
        // Call cleanup on all plugins in reverse order
        for plugin in self.plugins.iter_mut().rev() {
            plugin.cleanup(&mut self.ctx);
        }
    }
}
