//! Smart tooltip positioning algorithm.
//!
//! This module handles calculating optimal tooltip positions relative to targets,
//! with edge detection and automatic positioning adjustments.

use ratatui::layout::Rect;

/// Preferred position for tooltip relative to target.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TooltipPosition {
    /// Position to the right of the target.
    Right,

    /// Position to the left of the target.
    Left,

    /// Position below the target.
    Bottom,

    /// Position above the target.
    Top,
}

/// Direction for optional arrow indicator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArrowDirection {
    /// Arrow pointing left (tooltip on right).
    Left,

    /// Arrow pointing right (tooltip on left).
    Right,

    /// Arrow pointing up (tooltip below).
    Up,

    /// Arrow pointing down (tooltip above).
    Down,
}

impl ArrowDirection {
    /// Returns the Unicode character for this arrow direction.
    pub fn as_char(&self) -> char {
        match self {
            Self::Left => '◂',
            Self::Right => '▸',
            Self::Up => '▴',
            Self::Down => '▾',
        }
    }
}

/// Result of positioning calculation with position and arrow direction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PositionResult {
    /// The calculated tooltip rectangle.
    pub rect: Rect,

    /// The actual position used (may differ from preference if edge-constrained).
    pub position: TooltipPosition,

    /// Direction for arrow (if enabled).
    pub arrow_direction: ArrowDirection,

    /// Whether the position had to be flipped due to screen edges.
    pub was_flipped: bool,
}

/// Smart tooltip positioner with edge detection.
///
/// Calculates the optimal position for a tooltip relative to a target,
/// respecting screen boundaries and positioning preferences.
pub struct TooltipPositioner {
    /// Horizontal offset from target.
    offset_x: i16,

    /// Vertical offset from target.
    offset_y: i16,

    /// Padding inside tooltip.
    padding: u16,

    /// Whether to show border (adds 2 cells to dimensions).
    show_border: bool,

    /// Prefer right positioning.
    prefer_right: bool,

    /// Prefer bottom positioning.
    prefer_bottom: bool,
}

impl TooltipPositioner {
    /// Creates a new positioner with the given configuration.
    pub fn new(
        offset_x: i16,
        offset_y: i16,
        padding: u16,
        show_border: bool,
        prefer_right: bool,
        prefer_bottom: bool,
    ) -> Self {
        Self {
            offset_x,
            offset_y,
            padding,
            show_border,
            prefer_right,
            prefer_bottom,
        }
    }

    /// Calculates the optimal position for a tooltip.
    ///
    /// # Arguments
    ///
    /// * `target_rect` - The rectangle of the target element
    /// * `content_width` - Width of tooltip content (excluding border/padding)
    /// * `content_height` - Height of tooltip content (excluding border/padding)
    /// * `screen_rect` - The available screen area
    ///
    /// # Returns
    ///
    /// A `PositionResult` with the calculated position, rect, and arrow direction.
    pub fn calculate(
        &self,
        target_rect: Rect,
        content_width: u16,
        content_height: u16,
        screen_rect: Rect,
    ) -> PositionResult {
        // Calculate total tooltip dimensions (content + padding + border)
        let border_size = if self.show_border { 2 } else { 0 };
        let total_padding = self.padding * 2;
        let tooltip_width = content_width + total_padding + border_size;
        let tooltip_height = content_height + total_padding + border_size;

        // Try preferred positions first
        let positions = if self.prefer_right {
            vec![
                TooltipPosition::Right,
                TooltipPosition::Left,
                TooltipPosition::Bottom,
                TooltipPosition::Top,
            ]
        } else if self.prefer_bottom {
            vec![
                TooltipPosition::Bottom,
                TooltipPosition::Top,
                TooltipPosition::Right,
                TooltipPosition::Left,
            ]
        } else {
            vec![
                TooltipPosition::Left,
                TooltipPosition::Right,
                TooltipPosition::Top,
                TooltipPosition::Bottom,
            ]
        };

        // Try each position until we find one that fits
        for position in positions.iter() {
            if let Some(rect) = self.try_position(
                *position,
                target_rect,
                tooltip_width,
                tooltip_height,
                screen_rect,
            ) {
                let arrow_direction = match position {
                    TooltipPosition::Right => ArrowDirection::Left,
                    TooltipPosition::Left => ArrowDirection::Right,
                    TooltipPosition::Bottom => ArrowDirection::Up,
                    TooltipPosition::Top => ArrowDirection::Down,
                };

                let was_flipped = *position != positions[0];

                return PositionResult {
                    rect,
                    position: *position,
                    arrow_direction,
                    was_flipped,
                };
            }
        }

        // Fallback: force first preference even if it doesn't fit
        // (better to clip than to not show at all)
        let position = positions[0];
        let rect = self
            .calculate_rect(position, target_rect, tooltip_width, tooltip_height)
            .unwrap_or_else(|| Rect::new(0, 0, tooltip_width, tooltip_height));

        let arrow_direction = match position {
            TooltipPosition::Right => ArrowDirection::Left,
            TooltipPosition::Left => ArrowDirection::Right,
            TooltipPosition::Bottom => ArrowDirection::Up,
            TooltipPosition::Top => ArrowDirection::Down,
        };

        PositionResult {
            rect,
            position,
            arrow_direction,
            was_flipped: false,
        }
    }

    /// Tries to position the tooltip in the given position.
    ///
    /// Returns Some(rect) if it fits, None if it would overflow screen bounds.
    fn try_position(
        &self,
        position: TooltipPosition,
        target_rect: Rect,
        tooltip_width: u16,
        tooltip_height: u16,
        screen_rect: Rect,
    ) -> Option<Rect> {
        let rect = self.calculate_rect(position, target_rect, tooltip_width, tooltip_height)?;

        // Check if rect fits within screen bounds
        if rect.x >= screen_rect.x
            && rect.y >= screen_rect.y
            && rect.x + rect.width <= screen_rect.x + screen_rect.width
            && rect.y + rect.height <= screen_rect.y + screen_rect.height
        {
            Some(rect)
        } else {
            None
        }
    }

    /// Calculates the tooltip rectangle for a given position.
    fn calculate_rect(
        &self,
        position: TooltipPosition,
        target_rect: Rect,
        tooltip_width: u16,
        tooltip_height: u16,
    ) -> Option<Rect> {
        let (x, y) = match position {
            TooltipPosition::Right => {
                let x = (target_rect.x + target_rect.width).checked_add_signed(self.offset_x)?;
                let y = target_rect.y;
                (x, y)
            }
            TooltipPosition::Left => {
                let x = target_rect
                    .x
                    .checked_sub(tooltip_width)?
                    .checked_add_signed(self.offset_x)?;
                let y = target_rect.y;
                (x, y)
            }
            TooltipPosition::Bottom => {
                let x = target_rect.x;
                let y = (target_rect.y + target_rect.height).checked_add_signed(self.offset_y)?;
                (x, y)
            }
            TooltipPosition::Top => {
                let x = target_rect.x;
                let y = target_rect
                    .y
                    .checked_sub(tooltip_height)?
                    .checked_add_signed(self.offset_y)?;
                (x, y)
            }
        };

        Some(Rect::new(x, y, tooltip_width, tooltip_height))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arrow_direction_chars() {
        assert_eq!(ArrowDirection::Left.as_char(), '◂');
        assert_eq!(ArrowDirection::Right.as_char(), '▸');
        assert_eq!(ArrowDirection::Up.as_char(), '▴');
        assert_eq!(ArrowDirection::Down.as_char(), '▾');
    }

    #[test]
    fn test_position_right_with_space() {
        let positioner = TooltipPositioner::new(1, 1, 1, true, true, true);
        let target = Rect::new(10, 10, 10, 3);
        let screen = Rect::new(0, 0, 80, 24);

        let result = positioner.calculate(target, 20, 5, screen);

        // Tooltip should be positioned to the right
        assert_eq!(result.position, TooltipPosition::Right);
        assert_eq!(result.arrow_direction, ArrowDirection::Left);
        assert!(!result.was_flipped);

        // X should be target.x + target.width + offset_x
        // Content: 20, padding: 2, border: 2 = 24 width
        assert_eq!(result.rect.x, 21); // 10 + 10 + 1
        assert_eq!(result.rect.width, 24); // 20 + 2 + 2
    }

    #[test]
    fn test_position_flip_to_left() {
        let positioner = TooltipPositioner::new(1, 1, 1, true, true, true);
        let target = Rect::new(70, 10, 5, 3); // Near right edge
        let screen = Rect::new(0, 0, 80, 24);

        let result = positioner.calculate(target, 20, 5, screen);

        // Should flip to left since right doesn't fit
        assert_eq!(result.position, TooltipPosition::Left);
        assert_eq!(result.arrow_direction, ArrowDirection::Right);
        assert!(result.was_flipped);
    }

    #[test]
    fn test_position_bottom() {
        let positioner = TooltipPositioner::new(1, 1, 1, true, false, true);
        let target = Rect::new(10, 5, 10, 3);
        let screen = Rect::new(0, 0, 80, 24);

        let result = positioner.calculate(target, 20, 5, screen);

        // Should prefer bottom
        assert_eq!(result.position, TooltipPosition::Bottom);
        assert_eq!(result.arrow_direction, ArrowDirection::Up);

        // Y should be target.y + target.height + offset_y
        assert_eq!(result.rect.y, 9); // 5 + 3 + 1
    }

    #[test]
    fn test_position_flip_to_top() {
        let positioner = TooltipPositioner::new(1, 1, 1, true, false, true);
        let target = Rect::new(10, 20, 10, 3); // Near bottom edge
        let screen = Rect::new(0, 0, 80, 24);

        let result = positioner.calculate(target, 20, 5, screen);

        // Should flip to top since bottom doesn't fit
        assert_eq!(result.position, TooltipPosition::Top);
        assert_eq!(result.arrow_direction, ArrowDirection::Down);
        assert!(result.was_flipped);
    }

    #[test]
    fn test_dimensions_with_border_and_padding() {
        let positioner = TooltipPositioner::new(0, 0, 2, true, true, true);
        let target = Rect::new(10, 10, 5, 3);
        let screen = Rect::new(0, 0, 80, 24);

        let result = positioner.calculate(target, 10, 3, screen);

        // Width: content(10) + padding(4) + border(2) = 16
        // Height: content(3) + padding(4) + border(2) = 9
        assert_eq!(result.rect.width, 16);
        assert_eq!(result.rect.height, 9);
    }

    #[test]
    fn test_dimensions_without_border() {
        let positioner = TooltipPositioner::new(0, 0, 1, false, true, true);
        let target = Rect::new(10, 10, 5, 3);
        let screen = Rect::new(0, 0, 80, 24);

        let result = positioner.calculate(target, 10, 3, screen);

        // Width: content(10) + padding(2) + border(0) = 12
        // Height: content(3) + padding(2) + border(0) = 5
        assert_eq!(result.rect.width, 12);
        assert_eq!(result.rect.height, 5);
    }

    #[test]
    fn test_edge_cases_zero_offset() {
        let positioner = TooltipPositioner::new(0, 0, 0, false, true, true);
        let target = Rect::new(10, 10, 5, 3);
        let screen = Rect::new(0, 0, 80, 24);

        let result = positioner.calculate(target, 5, 2, screen);

        // Should still position correctly with zero offset/padding
        assert_eq!(result.rect.x, 15); // 10 + 5 + 0
        assert_eq!(result.rect.width, 5); // Just content
    }
}
