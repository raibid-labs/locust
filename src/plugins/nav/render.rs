//! Hint rendering for navigation overlays.
//!
//! This module handles the visual presentation of hints on top of navigation targets.

use super::config::NavConfig;
use super::hints::{Hint, HintMatcher};
use crate::core::targets::{NavTarget, TargetRegistry};
use ratatui::{
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::{Block, Paragraph},
    Frame,
};
use std::collections::HashMap;

/// Position where the hint overlay should be rendered relative to the target.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HintPosition {
    /// Top-left corner of the target
    TopLeft,
    /// Top-right corner of the target
    TopRight,
    /// Center of the target
    Center,
    /// Bottom-left corner of the target
    BottomLeft,
    /// Bottom-right corner of the target
    BottomRight,
}

impl Default for HintPosition {
    fn default() -> Self {
        Self::TopLeft
    }
}

/// Renders hint overlays on navigation targets.
///
/// This struct is responsible for drawing hint labels on top of targets,
/// with proper styling for matched/unmatched characters and dimming.
pub struct HintRenderer {
    /// Hint position relative to target
    position: HintPosition,

    /// Whether to render a background box around hints
    render_background: bool,

    /// Padding around hint text (horizontal, vertical)
    padding: (u16, u16),
}

impl Default for HintRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl HintRenderer {
    /// Creates a new hint renderer with default settings.
    pub fn new() -> Self {
        Self {
            position: HintPosition::TopLeft,
            render_background: true,
            padding: (1, 0),
        }
    }

    /// Sets the hint position.
    pub fn with_position(mut self, position: HintPosition) -> Self {
        self.position = position;
        self
    }

    /// Sets whether to render a background box.
    pub fn with_background(mut self, render: bool) -> Self {
        self.render_background = render;
        self
    }

    /// Sets the padding around hint text.
    pub fn with_padding(mut self, horizontal: u16, vertical: u16) -> Self {
        self.padding = (horizontal, vertical);
        self
    }

    /// Renders all hints on the given frame.
    ///
    /// This is the main entry point for rendering. It renders hints for
    /// all targets in the registry, with proper styling based on match state.
    pub fn render(
        &self,
        frame: &mut Frame,
        matcher: &HintMatcher,
        registry: &TargetRegistry,
        config: &NavConfig,
    ) {
        // Build a map from target ID to hint
        let hint_map: HashMap<u64, &Hint> =
            matcher.hints().iter().map(|h| (h.target_id, h)).collect();

        // Get matching and non-matching hints for styling
        let matching: HashMap<u64, bool> = matcher
            .matching_hints()
            .iter()
            .map(|h| (h.target_id, true))
            .collect();

        // Render each hint
        for target in registry.all() {
            if let Some(hint) = hint_map.get(&target.id) {
                let is_matching = matching.contains_key(&target.id);
                self.render_hint(frame, hint, target, config, is_matching);
            }
        }
    }

    /// Renders a single hint for a target.
    fn render_hint(
        &self,
        frame: &mut Frame,
        hint: &Hint,
        target: &NavTarget,
        config: &NavConfig,
        is_matching: bool,
    ) {
        // Calculate hint area based on position
        let hint_area = self.calculate_hint_area(target, hint, frame.area());

        // Check if hint area is visible
        if hint_area.width == 0 || hint_area.height == 0 {
            return;
        }

        // Create styled text
        let text = self.create_hint_text(hint, config, is_matching);

        // Render the hint
        if self.render_background {
            let block = Block::default();
            let paragraph = Paragraph::new(text).block(block);
            frame.render_widget(paragraph, hint_area);
        } else {
            let paragraph = Paragraph::new(text);
            frame.render_widget(paragraph, hint_area);
        }
    }

    /// Calculates the area where the hint should be rendered.
    fn calculate_hint_area(&self, target: &NavTarget, hint: &Hint, frame_area: Rect) -> Rect {
        let hint_width = (hint.text.len() as u16) + (self.padding.0 * 2);
        let hint_height = 1 + (self.padding.1 * 2);

        let (x, y) = match self.position {
            HintPosition::TopLeft => (target.rect.x, target.rect.y),
            HintPosition::TopRight => (
                target.rect.x + target.rect.width.saturating_sub(hint_width),
                target.rect.y,
            ),
            HintPosition::Center => (
                target.rect.x + (target.rect.width.saturating_sub(hint_width) / 2),
                target.rect.y + (target.rect.height.saturating_sub(hint_height) / 2),
            ),
            HintPosition::BottomLeft => (
                target.rect.x,
                target.rect.y + target.rect.height.saturating_sub(hint_height),
            ),
            HintPosition::BottomRight => (
                target.rect.x + target.rect.width.saturating_sub(hint_width),
                target.rect.y + target.rect.height.saturating_sub(hint_height),
            ),
        };

        // Clamp to frame boundaries
        let x = x.min(frame_area.width.saturating_sub(hint_width));
        let y = y.min(frame_area.height.saturating_sub(hint_height));
        let width = hint_width.min(frame_area.width.saturating_sub(x));
        let height = hint_height.min(frame_area.height.saturating_sub(y));

        Rect {
            x,
            y,
            width,
            height,
        }
    }

    /// Creates styled text for a hint.
    ///
    /// Matched characters use `hint_matched_style`, unmatched characters
    /// use `hint_text_style`, and non-matching hints use `hint_dimmed_style`.
    fn create_hint_text(&self, hint: &Hint, config: &NavConfig, is_matching: bool) -> Line<'_> {
        let mut spans = Vec::new();

        // Add left padding
        if self.padding.0 > 0 {
            spans.push(Span::raw(" ".repeat(self.padding.0 as usize)));
        }

        if is_matching {
            // Render matched portion
            let matched = hint.matched();
            if !matched.is_empty() {
                spans.push(Span::styled(matched.to_string(), config.hint_matched_style));
            }

            // Render unmatched portion
            let unmatched = hint.unmatched();
            if !unmatched.is_empty() {
                spans.push(Span::styled(unmatched.to_string(), config.hint_text_style));
            }
        } else {
            // Non-matching hint - render dimmed
            spans.push(Span::styled(hint.text.clone(), config.hint_dimmed_style));
        }

        // Add right padding
        if self.padding.0 > 0 {
            spans.push(Span::raw(" ".repeat(self.padding.0 as usize)));
        }

        Line::from(spans)
    }
}

/// Renders the hint mode status banner.
///
/// Shows the current input and hint count at the top of the screen.
pub fn render_hint_banner(frame: &mut Frame, matcher: &HintMatcher, style: Style) {
    let area = {
        let size = frame.area();
        Rect {
            x: size.x,
            y: size.y,
            width: size.width,
            height: 1,
        }
    };

    let matching_count = matcher.matching_hints().len();
    let total_count = matcher.hints().len();
    let input = matcher.input();

    let text = if input.is_empty() {
        format!(" Hint mode: {} targets (press Esc to exit) ", total_count)
    } else {
        format!(
            " Hint mode: {} [{}/{}] ",
            input, matching_count, total_count
        )
    };

    let line = Line::from(vec![Span::styled(text, style)]);
    let paragraph = Paragraph::new(line);
    frame.render_widget(paragraph, area);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::targets::NavTarget;

    #[test]
    fn test_hint_position_calculation() {
        let renderer = HintRenderer::new();
        let target = NavTarget::new(1, Rect::new(10, 10, 20, 5));
        let hint = Hint::new("as".to_string(), 1);
        let frame_area = Rect::new(0, 0, 100, 50);

        let area = renderer.calculate_hint_area(&target, &hint, frame_area);

        // Top-left position with padding (1, 0)
        // Hint width = 2 chars + 2 padding = 4
        // Hint height = 1 + 0 padding = 1
        assert_eq!(area.x, 10);
        assert_eq!(area.y, 10);
        assert_eq!(area.width, 4);
        assert_eq!(area.height, 1);
    }

    #[test]
    fn test_hint_position_center() {
        let renderer = HintRenderer::new().with_position(HintPosition::Center);
        let target = NavTarget::new(1, Rect::new(10, 10, 20, 10));
        let hint = Hint::new("ab".to_string(), 1);
        let frame_area = Rect::new(0, 0, 100, 50);

        let area = renderer.calculate_hint_area(&target, &hint, frame_area);

        // Center position: target is 20 wide, hint is 4 wide (2 + 2 padding)
        // x = 10 + (20 - 4) / 2 = 10 + 8 = 18
        // y = 10 + (10 - 1) / 2 = 10 + 4 = 14
        assert_eq!(area.x, 18);
        assert_eq!(area.y, 14);
    }

    #[test]
    fn test_hint_text_creation_no_match() {
        let renderer = HintRenderer::new();
        let config = NavConfig::default();
        let hint = Hint::new("asd".to_string(), 1);

        let line = renderer.create_hint_text(&hint, &config, true);
        let text: String = line.spans.iter().map(|s| s.content.as_ref()).collect();

        // With padding (1, 0): " asd "
        assert_eq!(text.trim(), "asd");
    }

    #[test]
    fn test_hint_text_creation_partial_match() {
        let renderer = HintRenderer::new();
        let config = NavConfig::default();
        let mut hint = Hint::new("asd".to_string(), 1);
        hint.update_match("as");

        let line = renderer.create_hint_text(&hint, &config, true);
        let spans = line.spans;

        // Should have: padding, matched "as", unmatched "d", padding
        assert!(spans.len() >= 3);
    }
}
