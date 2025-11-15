//! Locust highlight plugin - Guided tours with spotlight highlights.
//!
//! This plugin provides guided tours with visual highlights (spotlights) that
//! dim the rest of the screen and focus attention on specific UI elements.
//!
//! # Features
//!
//! - Multi-step guided tours
//! - Spotlight/highlight with dim overlay
//! - Flexible message positioning
//! - Step navigation (next, previous, skip)
//! - Tour progress tracking
//! - Animated highlight effects
//! - Auto-advance support
//!
//! # Example
//!
//! ```rust,no_run
//! use locust::plugins::highlight::{HighlightPlugin, Tour, TourStep};
//! use locust::core::context::LocustContext;
//! use locust::core::plugin::LocustPlugin;
//! use ratatui::backend::TestBackend;
//! use ratatui::layout::Rect;
//!
//! let mut ctx = LocustContext::default();
//! let mut highlight = HighlightPlugin::new();
//!
//! // Create a tour
//! let tour = Tour::new("onboarding")
//!     .add_step(
//!         TourStep::new("Welcome", "Welcome to the application!")
//!             .with_area(Rect::new(10, 5, 50, 10))
//!     )
//!     .add_step(
//!         TourStep::new("Next Step", "Here's how to use feature X")
//!             .with_target(42) // Highlight NavTarget with ID 42
//!     );
//!
//! highlight.register_tour(tour);
//! LocustPlugin::<TestBackend>::init(&mut highlight, &mut ctx);
//! ```

pub mod config;
pub mod render;
pub mod tour;

// Re-export for easier access
pub use config::{HighlightAnimation, HighlightBorderStyle, HighlightConfig};
pub use tour::{MessagePosition, Tour, TourState, TourStep};

use crate::core::context::LocustContext;
use crate::core::input::PluginEventResult;
use crate::core::overlay::OverlayLayer;
use crate::core::plugin::LocustPlugin;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use ratatui::backend::Backend;
use ratatui::Frame;
use render::HighlightRenderer;
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Highlight plugin for guided tours and spotlights.
///
/// This plugin manages multiple tours and displays them with visual
/// highlights that focus user attention on specific UI elements.
///
/// # Configuration
///
/// The plugin can be customized using `HighlightConfig`:
///
/// ```rust
/// use locust::plugins::highlight::{HighlightPlugin, HighlightConfig};
///
/// let config = HighlightConfig::new()
///     .with_activation_key('h')
///     .with_dim_opacity(200)
///     .with_animation_speed(300);
///
/// let highlight = HighlightPlugin::with_config(config);
/// ```
pub struct HighlightPlugin {
    /// Plugin configuration
    config: HighlightConfig,

    /// All registered tours by ID
    tours: HashMap<String, Tour>,

    /// Currently active tour ID
    active_tour_id: Option<String>,

    /// Renderer for highlights and messages
    renderer: HighlightRenderer,

    /// Completed tour IDs (for progress tracking)
    completed_tours: Vec<String>,

    /// Last animation tick time
    last_tick: Option<Instant>,
}

impl Default for HighlightPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl HighlightPlugin {
    /// Creates a new highlight plugin with default configuration.
    pub fn new() -> Self {
        Self::with_config(HighlightConfig::default())
    }

    /// Creates a new highlight plugin with custom configuration.
    pub fn with_config(config: HighlightConfig) -> Self {
        Self {
            config,
            tours: HashMap::new(),
            active_tour_id: None,
            renderer: HighlightRenderer::new(),
            completed_tours: Vec::new(),
            last_tick: None,
        }
    }

    /// Returns the current configuration.
    pub fn config(&self) -> &HighlightConfig {
        &self.config
    }

    /// Registers a new tour.
    ///
    /// If a tour with the same ID exists, it will be replaced.
    pub fn register_tour(&mut self, tour: Tour) {
        self.tours.insert(tour.id.clone(), tour);
    }

    /// Starts a tour by ID.
    ///
    /// Returns `true` if the tour was started, `false` if not found.
    pub fn start_tour(&mut self, tour_id: &str, ctx: &mut LocustContext) -> bool {
        if let Some(tour) = self.tours.get_mut(tour_id) {
            // Check if already completed
            if self.completed_tours.contains(&tour_id.to_string()) {
                // Restart from beginning
                tour.current_step = 0;
            }

            tour.start();
            self.active_tour_id = Some(tour_id.to_string());
            ctx.overlay.mark_has_overlay();

            // Register overlay layer
            ctx.overlay
                .add_layer(OverlayLayer::new("locust.highlight", self.config.z_index));

            true
        } else {
            false
        }
    }

    /// Stops the current tour.
    pub fn stop_tour(&mut self, ctx: &mut LocustContext) {
        if let Some(tour_id) = &self.active_tour_id {
            if let Some(tour) = self.tours.get_mut(tour_id) {
                // Save progress if enabled
                if self.config.save_progress
                    && tour.is_last_step()
                    && !self.completed_tours.contains(tour_id)
                {
                    self.completed_tours.push(tour_id.clone());
                }

                tour.stop();
            }
        }

        self.active_tour_id = None;
        ctx.overlay.remove_layer("locust.highlight");
    }

    /// Gets the currently active tour.
    fn active_tour(&self) -> Option<&Tour> {
        self.active_tour_id
            .as_ref()
            .and_then(|id| self.tours.get(id))
    }

    /// Gets the currently active tour mutably.
    fn active_tour_mut(&mut self) -> Option<&mut Tour> {
        self.active_tour_id
            .as_ref()
            .and_then(|id| self.tours.get_mut(id))
    }

    /// Checks if a tour is completed.
    pub fn is_tour_completed(&self, tour_id: &str) -> bool {
        self.completed_tours.contains(&tour_id.to_string())
    }

    /// Gets all registered tour IDs.
    pub fn tour_ids(&self) -> Vec<String> {
        self.tours.keys().cloned().collect()
    }

    /// Advances to the next step in the active tour.
    fn next_tour_step(&mut self, ctx: &mut LocustContext) {
        if let Some(tour) = self.active_tour_mut() {
            if !tour.next_step() {
                // Tour ended
                self.stop_tour(ctx);
            }
        }
    }

    /// Goes back to the previous step in the active tour.
    fn previous_tour_step(&mut self) {
        if let Some(tour) = self.active_tour_mut() {
            tour.previous_step();
        }
    }

    /// Updates animation state.
    fn update_animation(&mut self) {
        let now = Instant::now();

        if let Some(last_tick) = self.last_tick {
            let elapsed = now.duration_since(last_tick);
            let tick_duration = Duration::from_millis(self.config.animation_speed_ms);

            if elapsed >= tick_duration {
                self.renderer.tick();
                self.last_tick = Some(now);
            }
        } else {
            self.last_tick = Some(now);
        }
    }
}

impl<B> LocustPlugin<B> for HighlightPlugin
where
    B: Backend,
{
    fn id(&self) -> &'static str {
        "locust.highlight"
    }

    fn priority(&self) -> i32 {
        30 // Higher priority than nav and omnibar
    }

    fn init(&mut self, _ctx: &mut LocustContext) {
        // Plugin is initialized
    }

    fn on_event(&mut self, event: &Event, ctx: &mut LocustContext) -> PluginEventResult {
        // Update animation
        self.update_animation();

        if let Event::Key(KeyEvent {
            code, modifiers, ..
        }) = event
        {
            // If no active tour, check for activation
            if self.active_tour_id.is_none() {
                if let KeyCode::Char(c) = code {
                    if *c == self.config.activation_key && *modifiers == KeyModifiers::NONE {
                        // Start first available tour (or could show tour selection)
                        if let Some(tour_id) = self.tours.keys().next().cloned() {
                            self.start_tour(&tour_id, ctx);
                            return PluginEventResult::ConsumedRequestRedraw;
                        }
                    }
                }
                return PluginEventResult::NotHandled;
            }

            // Handle tour navigation
            let tour_skippable = self.active_tour().map(|t| t.skippable).unwrap_or(true);

            match code {
                // Skip tour
                KeyCode::Esc if tour_skippable => {
                    self.stop_tour(ctx);
                    return PluginEventResult::ConsumedRequestRedraw;
                }

                // Next step
                KeyCode::Right | KeyCode::Char('n') if *modifiers == KeyModifiers::NONE => {
                    self.next_tour_step(ctx);
                    return PluginEventResult::ConsumedRequestRedraw;
                }

                // Previous step
                KeyCode::Left | KeyCode::Char('p') if *modifiers == KeyModifiers::NONE => {
                    self.previous_tour_step();
                    return PluginEventResult::ConsumedRequestRedraw;
                }

                // Enter to advance or finish
                KeyCode::Enter => {
                    self.next_tour_step(ctx);
                    return PluginEventResult::ConsumedRequestRedraw;
                }

                _ => {}
            }
        }

        PluginEventResult::NotHandled
    }

    fn render_overlay(&self, frame: &mut Frame, ctx: &LocustContext) {
        if let Some(tour) = self.active_tour() {
            if tour.is_active() {
                self.renderer.render(frame, tour, ctx, &self.config);
            }
        }
    }

    fn cleanup(&mut self, ctx: &mut LocustContext) {
        self.stop_tour(ctx);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_creation() {
        let plugin = HighlightPlugin::new();
        assert!(plugin.active_tour_id.is_none());
        assert_eq!(plugin.tours.len(), 0);
    }

    #[test]
    fn test_plugin_custom_config() {
        let config = HighlightConfig::new()
            .with_activation_key('h')
            .with_dim_opacity(200);
        let plugin = HighlightPlugin::with_config(config);
        assert_eq!(plugin.config().activation_key, 'h');
        assert_eq!(plugin.config().dim_opacity, 200);
    }

    #[test]
    fn test_register_tour() {
        let mut plugin = HighlightPlugin::new();
        let tour = Tour::new("test").add_step(TourStep::new("Step 1", "First step"));

        plugin.register_tour(tour);
        assert_eq!(plugin.tours.len(), 1);
        assert!(plugin.tours.contains_key("test"));
    }

    #[test]
    fn test_start_stop_tour() {
        let mut plugin = HighlightPlugin::new();
        let mut ctx = LocustContext::default();

        let tour = Tour::new("test").add_step(TourStep::new("Step 1", "First step"));
        plugin.register_tour(tour);

        // Start tour
        assert!(plugin.start_tour("test", &mut ctx));
        assert_eq!(plugin.active_tour_id, Some("test".to_string()));
        assert!(ctx.overlay.has_overlay);

        // Stop tour
        plugin.stop_tour(&mut ctx);
        assert!(plugin.active_tour_id.is_none());
    }

    #[test]
    fn test_tour_navigation() {
        let mut plugin = HighlightPlugin::new();
        let mut ctx = LocustContext::default();

        let tour = Tour::new("test")
            .add_step(TourStep::new("Step 1", "First"))
            .add_step(TourStep::new("Step 2", "Second"))
            .add_step(TourStep::new("Step 3", "Third"));

        plugin.register_tour(tour);
        plugin.start_tour("test", &mut ctx);

        // Navigate forward
        plugin.next_tour_step(&mut ctx);
        assert_eq!(plugin.active_tour().unwrap().current_step, 1);

        // Navigate backward
        plugin.previous_tour_step();
        assert_eq!(plugin.active_tour().unwrap().current_step, 0);
    }

    #[test]
    fn test_tour_completion() {
        let mut plugin = HighlightPlugin::new();
        let mut ctx = LocustContext::default();

        let tour = Tour::new("test").add_step(TourStep::new("Step 1", "First"));

        plugin.register_tour(tour);
        plugin.start_tour("test", &mut ctx);

        // Complete the tour
        plugin.next_tour_step(&mut ctx);

        // Should be marked as completed
        assert!(plugin.is_tour_completed("test"));
        assert!(plugin.active_tour_id.is_none());
    }

    #[test]
    fn test_plugin_priority() {
        let plugin = HighlightPlugin::new();
        use ratatui::backend::TestBackend;
        // Highlight should have higher priority than nav (50) and omnibar (40)
        assert!(<HighlightPlugin as LocustPlugin<TestBackend>>::priority(&plugin) < 40);
    }
}
