//! Rendering utilities for the Omnibar plugin.
//!
//! Handles drawing the popup overlay, input field, cursor, placeholder text,
//! and command suggestions with fuzzy match highlighting.

use super::config::OmnibarConfig;
use super::registry::CommandSuggestion;
use super::state::OmnibarState;
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, Paragraph};
use ratatui::Frame;

/// Renderer for the Omnibar overlay.
pub struct OmnibarRenderer;

impl OmnibarRenderer {
    /// Creates a new omnibar renderer.
    pub fn new() -> Self {
        Self
    }

    /// Renders the omnibar overlay with optional suggestions.
    ///
    /// # Arguments
    ///
    /// * `frame` - The ratatui frame to render into
    /// * `state` - Current omnibar state
    /// * `config` - Omnibar configuration
    /// * `suggestions` - Optional command suggestions to display
    pub fn render(
        &self,
        frame: &mut Frame,
        state: &OmnibarState,
        config: &OmnibarConfig,
        suggestions: &[CommandSuggestion],
    ) {
        let area = frame.area();

        // Calculate popup area (centered, using configured width/height)
        let popup_height = if suggestions.is_empty() {
            config.max_height
        } else {
            config.max_height + (suggestions.len().min(5) as u16)
        };
        let popup_area = self.calculate_popup_area_with_height(area, config, popup_height);

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

        // Split area for input and suggestions
        if !suggestions.is_empty() {
            let chunks = Layout::vertical([
                Constraint::Length(1),                            // Input line
                Constraint::Min(suggestions.len().min(5) as u16), // Suggestions
            ])
            .split(inner_area);

            self.render_input(frame, chunks[0], state, config);
            self.render_suggestions(frame, chunks[1], suggestions);
        } else {
            // Just input field
            self.render_input(frame, inner_area, state, config);
        }
    }

    /// Calculates the centered popup area with custom height.
    fn calculate_popup_area_with_height(
        &self,
        area: Rect,
        config: &OmnibarConfig,
        height: u16,
    ) -> Rect {
        let width = (area.width * config.max_width_percent / 100).min(area.width);
        let height = height.min(area.height);

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

    /// Renders command suggestions with fuzzy match highlighting.
    fn render_suggestions(&self, frame: &mut Frame, area: Rect, suggestions: &[CommandSuggestion]) {
        let items: Vec<ListItem> = suggestions
            .iter()
            .take(5) // Show max 5 suggestions
            .map(|suggestion| {
                let name_line =
                    self.create_highlighted_line(&suggestion.name, &suggestion.match_positions);
                let desc_line = Line::from(Span::styled(
                    format!("  {}", suggestion.description),
                    Style::default().fg(Color::DarkGray),
                ));

                ListItem::new(vec![name_line, desc_line])
            })
            .collect();

        let list = List::new(items);
        frame.render_widget(list, area);
    }

    /// Creates a line with highlighted characters at specified positions.
    fn create_highlighted_line<'a>(&self, text: &'a str, positions: &[usize]) -> Line<'a> {
        if positions.is_empty() {
            return Line::from(Span::styled(
                text.to_string(),
                Style::default().fg(Color::White),
            ));
        }

        let mut spans = Vec::new();
        let mut last_pos = 0;

        // Create a set of positions for fast lookup
        let position_set: std::collections::HashSet<usize> = positions.iter().copied().collect();

        // Iterate through characters and build spans
        for (byte_idx, ch) in text.char_indices() {
            if position_set.contains(&byte_idx) {
                // Add any unhighlighted text before this character
                if byte_idx > last_pos {
                    spans.push(Span::styled(
                        text[last_pos..byte_idx].to_string(),
                        Style::default().fg(Color::White),
                    ));
                }

                // Add highlighted character
                spans.push(Span::styled(
                    ch.to_string(),
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ));

                last_pos = byte_idx + ch.len_utf8();
            }
        }

        // Add remaining unhighlighted text
        if last_pos < text.len() {
            spans.push(Span::styled(
                text[last_pos..].to_string(),
                Style::default().fg(Color::White),
            ));
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

        let popup = renderer.calculate_popup_area_with_height(area, &config, config.max_height);

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

        let popup = renderer.calculate_popup_area_with_height(area, &config, config.max_height);

        // Should not exceed screen size
        assert!(popup.width <= 40);
        assert!(popup.height <= 10);
    }

    #[test]
    fn test_custom_dimensions() {
        let renderer = OmnibarRenderer::new();
        let config = OmnibarConfig::new().with_max_width(80).with_max_height(5);
        let area = Rect::new(0, 0, 100, 50);

        let popup = renderer.calculate_popup_area_with_height(area, &config, config.max_height);

        assert_eq!(popup.width, 80);
        assert_eq!(popup.height, 5);
    }

    #[test]
    fn test_highlighted_line_creation() {
        let renderer = OmnibarRenderer::new();
        let text = "hello";
        let positions = vec![0, 4]; // 'h' and 'o'

        let line = renderer.create_highlighted_line(text, &positions);

        // Should create a line with highlighted characters
        assert!(!line.spans.is_empty());
    }

    #[test]
    fn test_highlighted_line_no_positions() {
        let renderer = OmnibarRenderer::new();
        let text = "hello";
        let positions = vec![];

        let line = renderer.create_highlighted_line(text, &positions);

        // Should create a simple line
        assert_eq!(line.spans.len(), 1);
    }
}
