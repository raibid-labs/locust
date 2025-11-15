//! Rendering utilities for the Omnibar plugin.
//!
//! Handles drawing the popup overlay, input field, cursor, and placeholder text.

use super::config::OmnibarConfig;
use super::state::OmnibarState;
use ratatui::layout::{Alignment, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;

/// Renderer for the Omnibar overlay.
pub struct OmnibarRenderer;

impl OmnibarRenderer {
    /// Creates a new omnibar renderer.
    pub fn new() -> Self {
        Self
    }

    /// Renders the omnibar overlay.
    ///
    /// # Arguments
    ///
    /// * `frame` - The ratatui frame to render into
    /// * `state` - Current omnibar state
    /// * `config` - Omnibar configuration
    pub fn render(&self, frame: &mut Frame, state: &OmnibarState, config: &OmnibarConfig) {
        let area = frame.area();

        // Calculate popup area (centered, using configured width/height)
        let popup_area = self.calculate_popup_area(area, config);

        // Clear the area behind the popup
        frame.render_widget(Clear, popup_area);

        // Create the border block
        let border = Block::default()
            .title(" Omnibar ")
            .title_alignment(Alignment::Center)
            .borders(Borders::ALL)
            .border_type(config.border_type.to_ratatui_border())
            .border_style(config.border_style)
            .title_style(config.title_style);

        // Get the inner area for content
        let inner_area = border.inner(popup_area);

        // Render the border
        frame.render_widget(border, popup_area);

        // Render the input field
        self.render_input(frame, inner_area, state, config);
    }

    /// Calculates the centered popup area based on configuration.
    fn calculate_popup_area(&self, area: Rect, config: &OmnibarConfig) -> Rect {
        let width = (area.width * config.max_width_percent / 100).min(area.width);
        let height = config.max_height.min(area.height);

        // Center horizontally
        let x = area.x + (area.width.saturating_sub(width)) / 2;

        // Position in upper third of screen
        let y = area.y + area.height / 4;

        Rect {
            x,
            y,
            width,
            height,
        }
    }

    /// Renders the input field with cursor.
    fn render_input(
        &self,
        frame: &mut Frame,
        area: Rect,
        state: &OmnibarState,
        config: &OmnibarConfig,
    ) {
        // Create the input line with cursor
        let input_line = if state.buffer().is_empty() {
            // Show placeholder
            Line::from(Span::styled(
                config.placeholder_text.clone(),
                config.placeholder_style,
            ))
        } else {
            // Show input with cursor
            self.create_input_line_with_cursor(state, config)
        };

        let paragraph = Paragraph::new(input_line);
        frame.render_widget(paragraph, area);
    }

    /// Creates an input line with visible cursor.
    fn create_input_line_with_cursor<'a>(
        &self,
        state: &'a OmnibarState,
        config: &'a OmnibarConfig,
    ) -> Line<'a> {
        let buffer = state.buffer();
        let cursor_pos = state.cursor();

        let mut spans = Vec::new();

        // Text before cursor
        if cursor_pos > 0 {
            spans.push(Span::styled(
                buffer[..cursor_pos].to_string(),
                config.input_style,
            ));
        }

        // Cursor
        if cursor_pos < buffer.len() {
            // Cursor on character
            let char_at_cursor = buffer[cursor_pos..]
                .chars()
                .next()
                .unwrap_or(' ')
                .to_string();
            spans.push(Span::styled(char_at_cursor, config.cursor_style));

            // Text after cursor
            let next_char_start = cursor_pos
                + buffer[cursor_pos..]
                    .chars()
                    .next()
                    .map(|c| c.len_utf8())
                    .unwrap_or(0);
            if next_char_start < buffer.len() {
                spans.push(Span::styled(
                    buffer[next_char_start..].to_string(),
                    config.input_style,
                ));
            }
        } else {
            // Cursor at end (show block cursor)
            spans.push(Span::styled(" ", config.cursor_style));
        }

        Line::from(spans)
    }
}

impl Default for OmnibarRenderer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_renderer_creation() {
        let renderer = OmnibarRenderer::new();
        let config = OmnibarConfig::default();
        let area = Rect::new(0, 0, 100, 50);

        let popup = renderer.calculate_popup_area(area, &config);

        // Should be centered and 60% width
        assert_eq!(popup.width, 60);
        assert_eq!(popup.height, 3);
        assert_eq!(popup.x, 20); // (100 - 60) / 2
    }

    #[test]
    fn test_small_screen() {
        let renderer = OmnibarRenderer::new();
        let config = OmnibarConfig::default();
        let area = Rect::new(0, 0, 40, 10);

        let popup = renderer.calculate_popup_area(area, &config);

        // Should not exceed screen size
        assert!(popup.width <= 40);
        assert!(popup.height <= 10);
    }

    #[test]
    fn test_custom_dimensions() {
        let renderer = OmnibarRenderer::new();
        let config = OmnibarConfig::new().with_max_width(80).with_max_height(5);
        let area = Rect::new(0, 0, 100, 50);

        let popup = renderer.calculate_popup_area(area, &config);

        assert_eq!(popup.width, 80);
        assert_eq!(popup.height, 5);
    }
}
