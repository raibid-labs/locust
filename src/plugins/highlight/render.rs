//! Rendering utilities for the highlight plugin.

use super::config::{HighlightAnimation, HighlightBorderStyle, HighlightConfig};
use super::tour::{MessagePosition, Tour, TourStep};
use crate::core::context::LocustContext;
use ratatui::{
    layout::{Alignment, Rect},
    style::Style,
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

/// Renderer for highlight overlays and tour messages.
pub struct HighlightRenderer {
    /// Animation frame counter
    animation_frame: u32,
}

impl HighlightRenderer {
    /// Creates a new highlight renderer.
    pub fn new() -> Self {
        Self { animation_frame: 0 }
    }

    /// Advances the animation frame counter.
    pub fn tick(&mut self) {
        self.animation_frame = self.animation_frame.wrapping_add(1);
    }

    /// Renders the complete highlight overlay with tour step.
    pub fn render(
        &self,
        frame: &mut Frame,
        tour: &Tour,
        ctx: &LocustContext,
        config: &HighlightConfig,
    ) {
        let Some(step) = tour.current_step() else {
            return;
        };

        // Get the highlight rectangle
        let highlight_rect = step.highlight_rect(Some(&ctx.targets));

        // Render dim overlay with cutout
        self.render_dim_overlay(frame, highlight_rect, config);

        // Render highlight border if area exists
        if let Some(rect) = highlight_rect {
            self.render_highlight_border(frame, rect, config);
        }

        // Render message box
        self.render_message_box(frame, step, highlight_rect, tour, config);
    }

    /// Renders the dim overlay over the entire screen with optional cutout.
    fn render_dim_overlay(
        &self,
        frame: &mut Frame,
        highlight_rect: Option<Rect>,
        config: &HighlightConfig,
    ) {
        let area = frame.area();
        let style = Style::default().bg(config.dim_color);

        // If we have a highlight rect, we need to render the dim overlay
        // in sections around it (top, bottom, left, right)
        if let Some(highlight) = highlight_rect {
            let padded = self.apply_padding(highlight, config.highlight_padding, area);

            // Top section
            if padded.y > 0 {
                let top = Rect::new(area.x, area.y, area.width, padded.y);
                self.fill_area(frame, top, style);
            }

            // Bottom section
            let bottom_y = padded.y + padded.height;
            if bottom_y < area.height {
                let bottom = Rect::new(
                    area.x,
                    bottom_y,
                    area.width,
                    area.height.saturating_sub(bottom_y),
                );
                self.fill_area(frame, bottom, style);
            }

            // Left section (middle band)
            if padded.x > 0 {
                let left = Rect::new(area.x, padded.y, padded.x, padded.height);
                self.fill_area(frame, left, style);
            }

            // Right section (middle band)
            let right_x = padded.x + padded.width;
            if right_x < area.width {
                let right = Rect::new(
                    right_x,
                    padded.y,
                    area.width.saturating_sub(right_x),
                    padded.height,
                );
                self.fill_area(frame, right, style);
            }
        } else {
            // No highlight area, dim the entire screen
            self.fill_area(frame, area, style);
        }
    }

    /// Fills an area with a solid style.
    fn fill_area(&self, frame: &mut Frame, area: Rect, style: Style) {
        let block = Block::default().style(style);
        frame.render_widget(block, area);
    }

    /// Applies padding to a rectangle, ensuring it stays within bounds.
    fn apply_padding(&self, rect: Rect, padding: u16, bounds: Rect) -> Rect {
        Rect::new(
            rect.x.saturating_sub(padding).max(bounds.x),
            rect.y.saturating_sub(padding).max(bounds.y),
            (rect.width + padding * 2).min(bounds.width),
            (rect.height + padding * 2).min(bounds.height),
        )
    }

    /// Renders a border around the highlighted area.
    fn render_highlight_border(&self, frame: &mut Frame, rect: Rect, config: &HighlightConfig) {
        let borders = match config.border_style {
            HighlightBorderStyle::None => return,
            HighlightBorderStyle::Single => Borders::ALL,
            HighlightBorderStyle::Double => Borders::ALL,
            HighlightBorderStyle::Thick => Borders::ALL,
            HighlightBorderStyle::Rounded => Borders::ALL,
        };

        let block = Block::default()
            .borders(borders)
            .border_style(Style::default().fg(config.border_color));

        // Apply animation effect if enabled
        let block = match config.animation {
            HighlightAnimation::None => block,
            HighlightAnimation::Pulse => {
                // Pulse effect: alternate border thickness/intensity
                if (self.animation_frame / 10).is_multiple_of(2) {
                    block
                } else {
                    block.border_style(Style::default().fg(config.border_color))
                }
            }
            HighlightAnimation::Shimmer => block,
            HighlightAnimation::Breathe => block,
        };

        frame.render_widget(block, rect);
    }

    /// Renders the message box with title, content, and navigation.
    fn render_message_box(
        &self,
        frame: &mut Frame,
        step: &TourStep,
        highlight_rect: Option<Rect>,
        tour: &Tour,
        config: &HighlightConfig,
    ) {
        let area = frame.area();

        // Calculate message box area based on position
        let msg_area = self.calculate_message_area(area, highlight_rect, step.position, config);

        // Create message content
        let mut lines = Vec::new();

        // Title
        let title_line = Line::from(vec![Span::styled(
            step.title.clone(),
            config.message_title_style,
        )]);
        lines.push(title_line);
        lines.push(Line::from(""));

        // Message body
        for line in step.message.lines() {
            lines.push(Line::from(Span::styled(
                line.to_string(),
                config.message_body_style,
            )));
        }

        lines.push(Line::from(""));

        // Step indicator
        let (current, total) = tour.progress();
        let indicator = format!("Step {} of {}", current, total);
        lines.push(Line::from(Span::styled(
            indicator,
            config.step_indicator_style,
        )));

        // Navigation hints
        if config.show_navigation_hints {
            lines.push(Line::from(""));
            let mut hints = Vec::new();

            if !tour.is_first_step() {
                hints.push(Span::styled("[←] Previous", config.navigation_hints_style));
                hints.push(Span::raw("  "));
            }

            if !tour.is_last_step() {
                hints.push(Span::styled("[→] Next", config.navigation_hints_style));
                hints.push(Span::raw("  "));
            } else {
                hints.push(Span::styled(
                    "[Enter] Finish",
                    config.navigation_hints_style,
                ));
                hints.push(Span::raw("  "));
            }

            if tour.skippable {
                hints.push(Span::styled("[Esc] Skip", config.navigation_hints_style));
            }

            lines.push(Line::from(hints));
        }

        // Create paragraph widget
        let text = Text::from(lines);
        let paragraph = Paragraph::new(text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(config.message_border_color))
                    .style(Style::default().bg(config.message_bg_color)),
            )
            .wrap(Wrap { trim: false })
            .alignment(Alignment::Left);

        frame.render_widget(paragraph, msg_area);
    }

    /// Calculates the message box area based on position and constraints.
    fn calculate_message_area(
        &self,
        screen: Rect,
        highlight_rect: Option<Rect>,
        position: MessagePosition,
        config: &HighlightConfig,
    ) -> Rect {
        // Calculate max width
        let max_width = (screen.width * config.message_max_width_percent / 100).max(40);
        let box_width = max_width.min(screen.width.saturating_sub(4));

        // Estimate height (will be constrained by available space)
        let box_height = 12; // Reasonable default for message box

        match position {
            MessagePosition::Center => {
                // Center of screen
                let x = screen.x + (screen.width.saturating_sub(box_width)) / 2;
                let y = screen.y + (screen.height.saturating_sub(box_height)) / 2;
                Rect::new(x, y, box_width, box_height)
            }
            MessagePosition::Top => {
                let x = screen.x + (screen.width.saturating_sub(box_width)) / 2;
                let y = screen.y + 2;
                Rect::new(x, y, box_width, box_height)
            }
            MessagePosition::Bottom => {
                if let Some(highlight) = highlight_rect {
                    // Below the highlight
                    let x = screen.x + (screen.width.saturating_sub(box_width)) / 2;
                    let y = (highlight.y + highlight.height + 2).min(
                        screen
                            .height
                            .saturating_sub(box_height)
                            .saturating_sub(screen.y),
                    );
                    Rect::new(x, y, box_width, box_height)
                } else {
                    // Bottom of screen
                    let x = screen.x + (screen.width.saturating_sub(box_width)) / 2;
                    let y = screen.height.saturating_sub(box_height).saturating_sub(2);
                    Rect::new(x, y, box_width, box_height)
                }
            }
            MessagePosition::Left => {
                let x = screen.x + 2;
                let y = screen.y + (screen.height.saturating_sub(box_height)) / 2;
                Rect::new(x, y, box_width.min(screen.width / 3), box_height)
            }
            MessagePosition::Right => {
                let w = box_width.min(screen.width / 3);
                let x = screen.width.saturating_sub(w).saturating_sub(2);
                let y = screen.y + (screen.height.saturating_sub(box_height)) / 2;
                Rect::new(x, y, w, box_height)
            }
        }
    }
}

impl Default for HighlightRenderer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_renderer_creation() {
        let renderer = HighlightRenderer::new();
        assert_eq!(renderer.animation_frame, 0);
    }

    #[test]
    fn test_renderer_tick() {
        let mut renderer = HighlightRenderer::new();
        renderer.tick();
        assert_eq!(renderer.animation_frame, 1);
        renderer.tick();
        assert_eq!(renderer.animation_frame, 2);
    }

    #[test]
    fn test_apply_padding() {
        let renderer = HighlightRenderer::new();
        let rect = Rect::new(10, 10, 20, 10);
        let bounds = Rect::new(0, 0, 100, 100);

        let padded = renderer.apply_padding(rect, 2, bounds);
        assert_eq!(padded.x, 8);
        assert_eq!(padded.y, 8);
        assert_eq!(padded.width, 24);
        assert_eq!(padded.height, 14);
    }

    #[test]
    fn test_apply_padding_at_boundary() {
        let renderer = HighlightRenderer::new();
        let rect = Rect::new(0, 0, 20, 10);
        let bounds = Rect::new(0, 0, 100, 100);

        let padded = renderer.apply_padding(rect, 5, bounds);
        assert_eq!(padded.x, 0); // Can't go below 0
        assert_eq!(padded.y, 0);
        assert_eq!(padded.width, 30);
        assert_eq!(padded.height, 20);
    }

    #[test]
    fn test_calculate_message_area_center() {
        let renderer = HighlightRenderer::new();
        let config = HighlightConfig::default();
        let screen = Rect::new(0, 0, 100, 50);

        let msg_area =
            renderer.calculate_message_area(screen, None, MessagePosition::Center, &config);

        // Should be centered
        assert!(msg_area.x > 0);
        assert!(msg_area.y > 0);
        assert!(msg_area.width <= screen.width);
        assert!(msg_area.height <= screen.height);
    }

    #[test]
    fn test_calculate_message_area_bottom() {
        let renderer = HighlightRenderer::new();
        let config = HighlightConfig::default();
        let screen = Rect::new(0, 0, 100, 50);
        let highlight = Rect::new(40, 10, 20, 5);

        let msg_area = renderer.calculate_message_area(
            screen,
            Some(highlight),
            MessagePosition::Bottom,
            &config,
        );

        // Should be below highlight
        assert!(msg_area.y >= highlight.y + highlight.height);
    }
}
