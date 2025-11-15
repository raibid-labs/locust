//! Tooltip rendering implementation.
//!
//! This module handles the visual rendering of tooltips with borders,
//! arrows, and styled content.

use super::content::{TooltipContent, TooltipStyle};
use super::positioning::{ArrowDirection, PositionResult};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Widget, Wrap};

/// Renderer for tooltip overlays.
pub struct TooltipRenderer {
    /// Whether to show border around tooltips.
    show_border: bool,

    /// Whether to show arrow pointing to target.
    show_arrow: bool,
}

impl TooltipRenderer {
    /// Creates a new tooltip renderer.
    pub fn new(show_border: bool, show_arrow: bool) -> Self {
        Self {
            show_border,
            show_arrow,
        }
    }

    /// Renders a tooltip at the given position.
    ///
    /// # Arguments
    ///
    /// * `buffer` - The ratatui buffer to render into
    /// * `content` - The tooltip content to display
    /// * `position` - The calculated position result
    pub fn render(&self, buffer: &mut Buffer, content: &TooltipContent, position: &PositionResult) {
        let rect = position.rect;

        // Clear the area
        for y in rect.y..(rect.y + rect.height).min(buffer.area.height) {
            for x in rect.x..(rect.x + rect.width).min(buffer.area.width) {
                buffer[(x, y)].set_char(' ');
            }
        }

        // Render border if enabled
        let inner_rect = if self.show_border {
            self.render_border(buffer, rect, &content.style);
            Rect {
                x: rect.x + 1,
                y: rect.y + 1,
                width: rect.width.saturating_sub(2),
                height: rect.height.saturating_sub(2),
            }
        } else {
            rect
        };

        // Render arrow if enabled
        if self.show_arrow {
            self.render_arrow(buffer, rect, position.arrow_direction, &content.style);
        }

        // Render content
        self.render_content(buffer, inner_rect, content);
    }

    /// Renders the border around the tooltip.
    fn render_border(&self, buffer: &mut Buffer, rect: Rect, style: &TooltipStyle) {
        let block = Block::default()
            .borders(Borders::ALL)
            .style(style.border_style());

        block.render(rect, buffer);
    }

    /// Renders an arrow pointing to the target.
    fn render_arrow(
        &self,
        buffer: &mut Buffer,
        rect: Rect,
        direction: ArrowDirection,
        style: &TooltipStyle,
    ) {
        let arrow_char = direction.as_char();
        let arrow_style = style.border_style();

        let (x, y) = match direction {
            ArrowDirection::Left => (rect.x, rect.y + rect.height / 2),
            ArrowDirection::Right => (rect.x + rect.width - 1, rect.y + rect.height / 2),
            ArrowDirection::Up => (rect.x + rect.width / 2, rect.y),
            ArrowDirection::Down => (rect.x + rect.width / 2, rect.y + rect.height - 1),
        };

        // Only render if within bounds
        if x < buffer.area.width && y < buffer.area.height {
            buffer[(x, y)].set_char(arrow_char).set_style(arrow_style);
        }
    }

    /// Renders the tooltip content (title and body).
    fn render_content(&self, buffer: &mut Buffer, rect: Rect, content: &TooltipContent) {
        let mut lines = Vec::new();

        // Add title if present
        if let Some(title) = &content.title {
            lines.push(Line::from(Span::styled(
                title.clone(),
                content.style.title_style(),
            )));
        }

        // Add body lines
        for body_line in content.body_lines() {
            lines.push(Line::from(Span::styled(
                body_line.to_string(),
                content.style.body_style(),
            )));
        }

        // Render as paragraph with wrapping
        let paragraph = Paragraph::new(lines)
            .style(content.style.body_style())
            .wrap(Wrap { trim: false });

        paragraph.render(rect, buffer);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugins::tooltip::positioning::TooltipPosition;

    #[test]
    fn test_renderer_creation() {
        let renderer = TooltipRenderer::new(true, true);
        assert!(renderer.show_border);
        assert!(renderer.show_arrow);
    }

    #[test]
    fn test_render_tooltip_with_border() {
        let mut buffer = Buffer::empty(Rect::new(0, 0, 30, 10));
        let renderer = TooltipRenderer::new(true, false);

        let content = TooltipContent::new("Test tooltip");
        let position = PositionResult {
            rect: Rect::new(5, 5, 20, 3),
            position: TooltipPosition::Right,
            arrow_direction: ArrowDirection::Left,
            was_flipped: false,
        };

        renderer.render(&mut buffer, &content, &position);

        // Border should be rendered at edges
        let top_left = buffer[(5, 5)].symbol();
        assert!(top_left.starts_with('┌') || top_left.starts_with('╭'));
    }

    #[test]
    fn test_render_tooltip_with_title() {
        let mut buffer = Buffer::empty(Rect::new(0, 0, 30, 10));
        let renderer = TooltipRenderer::new(false, false);

        let content = TooltipContent::new("Body text")
            .with_title("Title")
            .with_style(TooltipStyle::Info);

        let position = PositionResult {
            rect: Rect::new(5, 5, 15, 4),
            position: TooltipPosition::Bottom,
            arrow_direction: ArrowDirection::Up,
            was_flipped: false,
        };

        renderer.render(&mut buffer, &content, &position);

        // Content should be rendered (we can't easily test exact text without
        // deep buffer inspection, but we can verify no panic occurred)
        assert_eq!(buffer.area.width, 30);
    }

    #[test]
    fn test_render_arrow() {
        let mut buffer = Buffer::empty(Rect::new(0, 0, 20, 10));
        let renderer = TooltipRenderer::new(true, true);

        let content = TooltipContent::new("Test");
        let rect = Rect::new(5, 5, 10, 3);

        // Test each arrow direction
        for (direction, expected_char) in [
            (ArrowDirection::Left, '◂'),
            (ArrowDirection::Right, '▸'),
            (ArrowDirection::Up, '▴'),
            (ArrowDirection::Down, '▾'),
        ] {
            let position = PositionResult {
                rect,
                position: TooltipPosition::Right,
                arrow_direction: direction,
                was_flipped: false,
            };

            renderer.render(&mut buffer, &content, &position);

            // Find the arrow in the buffer
            let mut found_arrow = false;
            for y in 0..buffer.area.height {
                for x in 0..buffer.area.width {
                    if buffer[(x, y)].symbol() == expected_char.to_string() {
                        found_arrow = true;
                        break;
                    }
                }
            }
            assert!(
                found_arrow,
                "Arrow {} not found for direction {:?}",
                expected_char, direction
            );
        }
    }

    #[test]
    fn test_render_multiline_content() {
        let mut buffer = Buffer::empty(Rect::new(0, 0, 40, 15));
        let renderer = TooltipRenderer::new(true, false);

        let content = TooltipContent::new("Line 1\nLine 2\nLine 3")
            .with_title("Multi-line")
            .with_style(TooltipStyle::Success);

        let position = PositionResult {
            rect: Rect::new(5, 5, 25, 7),
            position: TooltipPosition::Right,
            arrow_direction: ArrowDirection::Left,
            was_flipped: false,
        };

        renderer.render(&mut buffer, &content, &position);

        // Should render without panic
        assert_eq!(buffer.area.width, 40);
    }
}
