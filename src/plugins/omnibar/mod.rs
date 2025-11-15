//! Locust Omnibar plugin - Command palette for terminal UIs.
//!
//! This plugin provides a command palette (omnibar) for quick command execution.
//! Press '/' (configurable) to activate, type commands, and press Enter to execute.
//!
//! # Features
//!
//! - Fast command input with visual feedback
//! - Command history (last 10 commands by default)
//! - Cursor movement and editing
//! - Customizable styling and keybindings
//! - Centered popup overlay
//!
//! # Example
//!
//! ```rust,no_run
//! use locust::plugins::omnibar::OmnibarPlugin;
//! use locust::core::context::LocustContext;
//! use locust::core::plugin::LocustPlugin;
//! use ratatui::backend::TestBackend;
//!
//! let mut ctx = LocustContext::default();
//! let mut omnibar = OmnibarPlugin::new();
//! LocustPlugin::<TestBackend>::init(&mut omnibar, &mut ctx);
//! ```

pub mod config;
pub mod render;
pub mod state;

// Re-export for easier access
pub use config::{BorderType, OmnibarConfig};
pub use state::OmnibarMode;

use crate::core::context::LocustContext;
use crate::core::input::PluginEventResult;
use crate::core::plugin::LocustPlugin;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use ratatui::backend::Backend;
use ratatui::Frame;
use render::OmnibarRenderer;
use state::OmnibarState;

/// Command palette plugin for quick command execution.
///
/// This plugin manages the complete omnibar workflow:
/// 1. User presses activation key (default: '/')
/// 2. Omnibar popup appears with input field
/// 3. User types command and can navigate history
/// 4. Press Enter to execute, Esc to cancel
///
/// # Configuration
///
/// The plugin can be customized using `OmnibarConfig`:
///
/// ```rust
/// use locust::plugins::omnibar::{OmnibarPlugin, OmnibarConfig};
///
/// let config = OmnibarConfig::new()
///     .with_activation_key(':')
///     .with_max_width(80)
///     .with_placeholder("Enter command...");
///
/// let omnibar = OmnibarPlugin::with_config(config);
/// ```
///
/// # Command Execution
///
/// When a command is submitted (Enter key), the plugin currently logs it
/// for debugging. In future versions (WS-06), commands will be dispatched
/// to a command registry for actual execution.
pub struct OmnibarPlugin {
    /// Plugin configuration
    config: OmnibarConfig,

    /// Current state
    state: OmnibarState,

    /// Renderer
    renderer: OmnibarRenderer,
}

impl Default for OmnibarPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl OmnibarPlugin {
    /// Creates a new omnibar plugin with default configuration.
    pub fn new() -> Self {
        Self::with_config(OmnibarConfig::default())
    }

    /// Creates a new omnibar plugin with custom configuration.
    pub fn with_config(config: OmnibarConfig) -> Self {
        let max_history = config.max_history;
        Self {
            state: OmnibarState::new(max_history),
            config,
            renderer: OmnibarRenderer::new(),
        }
    }

    /// Returns the current configuration.
    pub fn config(&self) -> &OmnibarConfig {
        &self.config
    }

    /// Returns the current state.
    pub fn state(&self) -> &OmnibarState {
        &self.state
    }

    /// Returns mutable reference to state (for testing).
    #[cfg(test)]
    pub fn state_mut(&mut self) -> &mut OmnibarState {
        &mut self.state
    }

    /// Activates the omnibar and marks overlay.
    fn activate(&mut self, ctx: &mut LocustContext) {
        self.state.activate();
        ctx.overlay.mark_has_overlay();
    }

    /// Deactivates the omnibar.
    fn deactivate(&mut self) {
        self.state.deactivate();
    }

    /// Handles command submission.
    ///
    /// Currently logs the command. In WS-06, this will dispatch to
    /// the command registry for execution.
    fn handle_submit(&mut self) {
        if let Some(command) = self.state.submit() {
            // TODO (WS-06): Dispatch to command registry
            eprintln!("Locust Omnibar: Command submitted: '{}'", command);
        } else {
            // Empty input - just deactivate
            self.deactivate();
        }
    }
}

impl<B> LocustPlugin<B> for OmnibarPlugin
where
    B: Backend,
{
    fn id(&self) -> &'static str {
        "locust.omnibar"
    }

    fn priority(&self) -> i32 {
        40 // Higher priority than nav plugin (50), processes events first
    }

    fn init(&mut self, _ctx: &mut LocustContext) {
        // Plugin is initialized
    }

    fn on_event(&mut self, event: &Event, ctx: &mut LocustContext) -> PluginEventResult {
        if let Event::Key(KeyEvent {
            code, modifiers, ..
        }) = event
        {
            match (self.state.mode(), code, modifiers) {
                // Inactive: activate on configured key
                (OmnibarMode::Inactive, KeyCode::Char(c), m)
                    if *c == self.config.activation_key && *m == KeyModifiers::NONE =>
                {
                    self.activate(ctx);
                    return PluginEventResult::ConsumedRequestRedraw;
                }

                // Active: handle escape to cancel
                (OmnibarMode::Input, KeyCode::Esc, _) => {
                    self.deactivate();
                    return PluginEventResult::ConsumedRequestRedraw;
                }

                // Active: handle enter to submit
                (OmnibarMode::Input, KeyCode::Enter, _) => {
                    self.handle_submit();
                    return PluginEventResult::ConsumedRequestRedraw;
                }

                // Active: handle backspace
                (OmnibarMode::Input, KeyCode::Backspace, _) => {
                    self.state.delete_char();
                    return PluginEventResult::ConsumedRequestRedraw;
                }

                // Active: handle character input
                (OmnibarMode::Input, KeyCode::Char(c), m) if *m == KeyModifiers::NONE => {
                    self.state.insert_char(*c);
                    return PluginEventResult::ConsumedRequestRedraw;
                }

                // Active: cursor movement
                (OmnibarMode::Input, KeyCode::Left, _) => {
                    self.state.move_cursor_left();
                    return PluginEventResult::ConsumedRequestRedraw;
                }
                (OmnibarMode::Input, KeyCode::Right, _) => {
                    self.state.move_cursor_right();
                    return PluginEventResult::ConsumedRequestRedraw;
                }
                (OmnibarMode::Input, KeyCode::Home, _) => {
                    self.state.move_cursor_home();
                    return PluginEventResult::ConsumedRequestRedraw;
                }
                (OmnibarMode::Input, KeyCode::End, _) => {
                    self.state.move_cursor_end();
                    return PluginEventResult::ConsumedRequestRedraw;
                }

                // Active: history navigation
                (OmnibarMode::Input, KeyCode::Up, _) => {
                    self.state.history_prev();
                    return PluginEventResult::ConsumedRequestRedraw;
                }
                (OmnibarMode::Input, KeyCode::Down, _) => {
                    self.state.history_next();
                    return PluginEventResult::ConsumedRequestRedraw;
                }

                _ => {}
            }
        }

        PluginEventResult::NotHandled
    }

    fn render_overlay(&self, frame: &mut Frame, _ctx: &LocustContext) {
        if !self.state.is_active() {
            return;
        }

        self.renderer.render(frame, &self.state, &self.config);
    }

    fn cleanup(&mut self, _ctx: &mut LocustContext) {
        // Cleanup if needed
        self.deactivate();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_creation() {
        let plugin = OmnibarPlugin::new();
        assert_eq!(plugin.state().mode(), OmnibarMode::Inactive);
        assert!(!plugin.state().is_active());
    }

    #[test]
    fn test_plugin_custom_config() {
        let config = OmnibarConfig::new()
            .with_activation_key(':')
            .with_max_width(80);
        let plugin = OmnibarPlugin::with_config(config);
        assert_eq!(plugin.config().activation_key, ':');
        assert_eq!(plugin.config().max_width_percent, 80);
    }

    #[test]
    fn test_activation_deactivation() {
        let mut plugin = OmnibarPlugin::new();
        let mut ctx = LocustContext::default();

        assert!(!plugin.state().is_active());

        plugin.activate(&mut ctx);
        assert!(plugin.state().is_active());
        assert_eq!(plugin.state().mode(), OmnibarMode::Input);

        plugin.deactivate();
        assert!(!plugin.state().is_active());
        assert_eq!(plugin.state().mode(), OmnibarMode::Inactive);
    }

    #[test]
    fn test_input_handling() {
        let mut plugin = OmnibarPlugin::new();
        let mut ctx = LocustContext::default();

        plugin.activate(&mut ctx);
        plugin.state_mut().insert_char('t');
        plugin.state_mut().insert_char('e');
        plugin.state_mut().insert_char('s');
        plugin.state_mut().insert_char('t');

        assert_eq!(plugin.state().buffer(), "test");
        assert_eq!(plugin.state().cursor(), 4);
    }

    #[test]
    fn test_command_submission() {
        let mut plugin = OmnibarPlugin::new();
        let mut ctx = LocustContext::default();

        plugin.activate(&mut ctx);
        plugin.state_mut().insert_char('c');
        plugin.state_mut().insert_char('m');
        plugin.state_mut().insert_char('d');

        plugin.handle_submit();

        // Should be deactivated after submit
        assert!(!plugin.state().is_active());

        // History should contain the command
        assert_eq!(plugin.state().history().len(), 1);
        assert_eq!(plugin.state().history()[0], "cmd");
    }

    #[test]
    fn test_plugin_priority() {
        let plugin = OmnibarPlugin::new();
        use ratatui::backend::TestBackend;
        // Omnibar should have higher priority than nav (50)
        assert!(<OmnibarPlugin as LocustPlugin<TestBackend>>::priority(&plugin) < 50);
    }
}
