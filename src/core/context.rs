use crate::core::config::{ConfigError, LocustConfig as Config};
use crate::core::input::LocustEventOutcome;
use crate::core::keybindings::{KeyBinding, KeyMap, KeyMapError};
use crate::core::overlay::OverlayState;
use crate::core::plugin::LocustPlugin;
use crate::core::targets::TargetRegistry;
use crate::core::theme::{Theme, ThemeError};
use crate::core::theme_manager::ThemeManager;
use crate::plugins::tooltip::TooltipRegistry;
use crossterm::event::Event;
use std::any::Any;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use ratatui::backend::Backend;
use ratatui::Frame;

/// Legacy global configuration for Locust.
///
/// Note: This is kept for backward compatibility. New code should use
/// `crate::core::config::LocustConfig` for the full configuration system.
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
/// - Tooltip registry for contextual help
/// - Overlay state management
/// - Frame lifecycle tracking
/// - Configuration management
/// - Theme and keybinding management
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

    /// Registry of tooltips mapped to target IDs.
    pub tooltips: TooltipRegistry,

    /// State tracking for overlay rendering and management.
    pub overlay: OverlayState,

    /// Frame counter for tracking render cycles.
    pub frame_count: u64,

    /// Configuration (optional, can be loaded from file)
    pub config: Option<Config>,

    /// Theme manager for runtime theme switching
    pub theme_manager: ThemeManager,

    /// Keybinding configuration
    #[allow(clippy::derivable_impls)]
    pub keymap: KeyMap,
}

impl LocustContext {
    /// Updates the configuration and returns the old one if it existed.
    pub fn update_config(&mut self, config: Config) -> Option<Config> {
        self.config.replace(config)
    }

    /// Gets a reference to the global configuration.
    pub fn get_global_config(&self) -> Option<&crate::core::config::GlobalConfig> {
        self.config.as_ref().map(|c| &c.global)
    }

    /// Gets plugin-specific configuration.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use locust::core::config::NavConfig;
    ///
    /// if let Some(nav_config) = ctx.get_plugin_config::<NavConfig>("nav") {
    ///     println!("Hint key: {}", nav_config.hint_key);
    /// }
    /// ```
    pub fn get_plugin_config<T: serde::de::DeserializeOwned>(&self, plugin_id: &str) -> Option<T> {
        self.config.as_ref()?.get_plugin_config(plugin_id)
    }

    /// Sets the current theme by name.
    ///
    /// # Errors
    ///
    /// Returns `ThemeError::NotFound` if the theme doesn't exist.
    pub fn set_theme(&mut self, name: &str) -> Result<(), ThemeError> {
        self.theme_manager.set_theme(name)
    }

    /// Gets a reference to the current theme.
    pub fn get_theme(&self) -> &Theme {
        self.theme_manager.get_current_theme()
    }

    /// Gets a reference to the keymap.
    pub fn get_keymap(&self) -> &KeyMap {
        &self.keymap
    }

    /// Binds a key to an action.
    ///
    /// # Errors
    ///
    /// Returns `KeyMapError` if the binding is invalid.
    pub fn bind_key(&mut self, action: &str, binding: KeyBinding) -> Result<(), KeyMapError> {
        self.keymap.bind(action, binding)
    }

    /// Unbinds an action.
    pub fn unbind_key(&mut self, action: &str) {
        self.keymap.unbind(action)
    }
}

/// Central entry point for embedding Locust into a ratatui app.
pub struct Locust<B>
where
    B: Backend + 'static,
{
    pub config: LocustConfig,
    pub ctx: LocustContext,
    plugins: Vec<Box<dyn LocustPlugin<B>>>,
}

impl<B> Locust<B>
where
    B: Backend + 'static,
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

    /// Updates the runtime configuration and notifies all plugins.
    ///
    /// This triggers the `reload_config` hook on all registered plugins,
    /// allowing them to update their internal state.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use locust::core::config::LocustConfig;
    /// use std::path::Path;
    ///
    /// let config = LocustConfig::from_file(Path::new("locust.toml"))?;
    /// locust.update_config(config)?;
    /// # Ok::<(), locust::core::config::ConfigError>(())
    /// ```
    pub fn update_config(&mut self, config: Config) -> Result<(), ConfigError> {
        // Validate before applying
        let errors = config.validate();
        let has_errors = errors
            .iter()
            .any(|e| matches!(e.severity, crate::core::config::Severity::Error));

        if has_errors {
            return Err(ConfigError::NoConfigPath); // TODO: Better error type
        }

        self.ctx.update_config(config);

        // Notify all plugins of config change
        for plugin in self.plugins.iter_mut() {
            plugin.reload_config(&self.ctx);
        }

        Ok(())
    }

    /// Get a reference to the current configuration.
    pub fn get_config(&self) -> Option<&Config> {
        self.ctx.config.as_ref()
    }

    /// Get an immutable reference to a registered plugin.
    pub fn get_plugin<P: LocustPlugin<B> + 'static>(&self) -> Option<&P> {
        self.plugins
            .iter()
            .find_map(|plugin| (plugin.as_ref() as &dyn Any).downcast_ref::<P>())
    }

    /// Get a mutable reference to a registered plugin.
    pub fn get_plugin_mut<P: LocustPlugin<B> + 'static>(&mut self) -> Option<&mut P> {
        self.plugins
            .iter_mut()
            .find_map(|plugin| (plugin.as_mut() as &mut dyn Any).downcast_mut::<P>())
    }
}

impl<B> Drop for Locust<B>
where
    B: Backend + 'static,
{
    fn drop(&mut self) {
        // Call cleanup on all plugins in reverse order
        for plugin in self.plugins.iter_mut().rev() {
            plugin.cleanup(&mut self.ctx);
        }
    }
}