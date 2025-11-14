//! Example plugin demonstrating best practices.
//!
//! This example shows how to create a production-quality Locust plugin
//! that implements all lifecycle hooks properly.

use locust::prelude::*;

/// A plugin that displays a status bar overlay.
///
/// Press 's' to toggle the status bar visibility.
pub struct StatusBarPlugin {
    visible: bool,
    message: String,
}

impl StatusBarPlugin {
    /// Plugin ID constant
    pub const ID: &'static str = "example.status_bar";

    pub fn new() -> Self {
        Self {
            visible: true,
            message: String::from("Ready"),
        }
    }

    pub fn set_message(&mut self, msg: impl Into<String>) {
        self.message = msg.into();
    }
}

impl<B: Backend> LocustPlugin<B> for StatusBarPlugin {
    fn id(&self) -> &'static str {
        Self::ID
    }

    fn priority(&self) -> i32 {
        150 // User plugin, lower priority than built-in
    }

    fn init(&mut self, ctx: &mut LocustContext) {
        // Register the status bar as an overlay layer
        ctx.overlay.add_layer(OverlayLayer::new(Self::ID, 300));
        self.message = format!("Initialized at frame {}", ctx.frame_count);
    }

    fn on_event(&mut self, event: &Event, ctx: &mut LocustContext) -> PluginEventResult {
        // Toggle visibility on 's' key
        if let Event::Key(KeyEvent {
            code: KeyCode::Char('s'),
            ..
        }) = event
        {
            self.visible = !self.visible;
            ctx.overlay.set_layer_visibility(Self::ID, self.visible);

            if self.visible {
                ctx.overlay.mark_has_overlay();
            }

            return PluginEventResult::ConsumedRequestRedraw;
        }

        PluginEventResult::NotHandled
    }

    fn render_overlay(&self, frame: &mut Frame, _ctx: &LocustContext) {
        if !self.visible {
            return;
        }

        use ratatui::layout::Rect;
        use ratatui::style::{Color, Style};
        use ratatui::widgets::{Block, Borders, Paragraph};

        let size = frame.area();
        let area = Rect {
            x: 0,
            y: size.height.saturating_sub(1),
            width: size.width,
            height: 1,
        };

        let block = Block::default()
            .borders(Borders::TOP)
            .style(Style::default().bg(Color::DarkGray).fg(Color::White));

        let text = format!(" Status: {} | Press 's' to toggle ", self.message);
        let paragraph = Paragraph::new(text).block(block);

        frame.render_widget(paragraph, area);
    }

    fn cleanup(&mut self, ctx: &mut LocustContext) {
        // Clean up our overlay layer
        ctx.overlay.remove_layer(Self::ID);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::backend::TestBackend;

    #[test]
    fn test_status_bar_creation() {
        let plugin = StatusBarPlugin::new();
        assert_eq!(StatusBarPlugin::ID, "example.status_bar");
        assert_eq!(LocustPlugin::<TestBackend>::priority(&plugin), 150);
        assert!(plugin.visible);
    }

    #[test]
    fn test_status_bar_lifecycle() {
        let mut locust = Locust::<TestBackend>::new(LocustConfig::default());
        let mut plugin = StatusBarPlugin::new();

        assert!(locust.ctx.overlay.layers().is_empty());

        LocustPlugin::<TestBackend>::init(&mut plugin, &mut locust.ctx);

        // Check that overlay layer was registered
        assert_eq!(locust.ctx.overlay.layers().len(), 1);
        assert!(locust.ctx.overlay.has_layer(StatusBarPlugin::ID));

        LocustPlugin::<TestBackend>::cleanup(&mut plugin, &mut locust.ctx);

        // Check that overlay layer was removed
        assert_eq!(locust.ctx.overlay.layers().len(), 0);
    }

    #[test]
    fn test_status_bar_toggle() {
        let mut locust = Locust::<TestBackend>::new(LocustConfig::default());
        let plugin = StatusBarPlugin::new();

        locust.register_plugin(plugin);

        use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};

        // Press 's' to toggle
        let event = Event::Key(KeyEvent::new(KeyCode::Char('s'), KeyModifiers::NONE));
        let outcome = locust.on_event(&event);

        assert!(outcome.consumed);
        assert!(outcome.request_redraw);
    }
}
