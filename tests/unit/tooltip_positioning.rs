//! Unit tests for tooltip positioning algorithm.

use locust::plugins::tooltip::positioning::{
    ArrowDirection, TooltipPosition, TooltipPositioner,
};
use ratatui::layout::Rect;

#[test]
fn test_position_right_with_ample_space() {
    let positioner = TooltipPositioner::new(1, 1, 1, true, true, true);
    let target = Rect::new(20, 10, 10, 3);
    let screen = Rect::new(0, 0, 100, 30);

    let result = positioner.calculate(target, 20, 5, screen);

    assert_eq!(result.position, TooltipPosition::Right);
    assert_eq!(result.arrow_direction, ArrowDirection::Left);
    assert!(!result.was_flipped);

    // Tooltip should be to the right of target
    assert!(result.rect.x > target.x + target.width);
}

#[test]
fn test_position_flip_right_to_left_at_edge() {
    let positioner = TooltipPositioner::new(1, 1, 1, true, true, true);
    let target = Rect::new(80, 10, 10, 3); // Near right edge
    let screen = Rect::new(0, 0, 100, 30);

    let result = positioner.calculate(target, 20, 5, screen);

    // Should flip to left since right doesn't fit
    assert_eq!(result.position, TooltipPosition::Left);
    assert_eq!(result.arrow_direction, ArrowDirection::Right);
    assert!(result.was_flipped);

    // Tooltip should be to the left of target
    assert!(result.rect.x + result.rect.width <= target.x);
}

#[test]
fn test_position_bottom_with_space() {
    let positioner = TooltipPositioner::new(1, 1, 1, true, false, true);
    let target = Rect::new(20, 5, 10, 3);
    let screen = Rect::new(0, 0, 100, 30);

    let result = positioner.calculate(target, 15, 4, screen);

    assert_eq!(result.position, TooltipPosition::Bottom);
    assert_eq!(result.arrow_direction, ArrowDirection::Up);
    assert!(!result.was_flipped);

    // Tooltip should be below target
    assert!(result.rect.y > target.y + target.height);
}

#[test]
fn test_position_flip_bottom_to_top_at_edge() {
    let positioner = TooltipPositioner::new(1, 1, 1, true, false, true);
    let target = Rect::new(20, 25, 10, 3); // Near bottom edge
    let screen = Rect::new(0, 0, 100, 30);

    let result = positioner.calculate(target, 15, 4, screen);

    // Should flip to top since bottom doesn't fit
    assert_eq!(result.position, TooltipPosition::Top);
    assert_eq!(result.arrow_direction, ArrowDirection::Down);
    assert!(result.was_flipped);

    // Tooltip should be above target
    assert!(result.rect.y + result.rect.height <= target.y);
}

#[test]
fn test_position_prefer_left() {
    let positioner = TooltipPositioner::new(1, 1, 1, true, false, false);
    let target = Rect::new(50, 15, 10, 3);
    let screen = Rect::new(0, 0, 100, 30);

    let result = positioner.calculate(target, 15, 4, screen);

    // With prefer_right=false, should prefer left
    assert_eq!(result.position, TooltipPosition::Left);
    assert_eq!(result.arrow_direction, ArrowDirection::Right);
}

#[test]
fn test_dimensions_with_border_and_padding() {
    let positioner = TooltipPositioner::new(0, 0, 2, true, true, true);
    let target = Rect::new(20, 20, 10, 3);
    let screen = Rect::new(0, 0, 100, 50);

    let result = positioner.calculate(target, 10, 3, screen);

    // Width: content(10) + padding(2*2=4) + border(2) = 16
    // Height: content(3) + padding(2*2=4) + border(2) = 9
    assert_eq!(result.rect.width, 16);
    assert_eq!(result.rect.height, 9);
}

#[test]
fn test_dimensions_without_border() {
    let positioner = TooltipPositioner::new(0, 0, 1, false, true, true);
    let target = Rect::new(20, 20, 10, 3);
    let screen = Rect::new(0, 0, 100, 50);

    let result = positioner.calculate(target, 10, 3, screen);

    // Width: content(10) + padding(2) + border(0) = 12
    // Height: content(3) + padding(2) + border(0) = 5
    assert_eq!(result.rect.width, 12);
    assert_eq!(result.rect.height, 5);
}

#[test]
fn test_zero_offset_and_padding() {
    let positioner = TooltipPositioner::new(0, 0, 0, false, true, true);
    let target = Rect::new(20, 20, 10, 3);
    let screen = Rect::new(0, 0, 100, 50);

    let result = positioner.calculate(target, 5, 2, screen);

    // Should position correctly with zero offset/padding
    assert_eq!(result.rect.x, 30); // 20 + 10 + 0
    assert_eq!(result.rect.width, 5); // Just content
    assert_eq!(result.rect.height, 2); // Just content
}

#[test]
fn test_negative_offsets() {
    let positioner = TooltipPositioner::new(-2, -1, 0, false, true, true);
    let target = Rect::new(20, 20, 10, 3);
    let screen = Rect::new(0, 0, 100, 50);

    let result = positioner.calculate(target, 5, 2, screen);

    // Negative offset should move tooltip closer to target
    assert_eq!(result.rect.x, 28); // 20 + 10 - 2
}

#[test]
fn test_large_tooltip_fallback() {
    let positioner = TooltipPositioner::new(1, 1, 1, true, true, true);
    let target = Rect::new(45, 12, 10, 3);
    let screen = Rect::new(0, 0, 60, 20); // Small screen

    // Request a tooltip that's too large to fit anywhere properly
    let result = positioner.calculate(target, 50, 15, screen);

    // Should return a position even if it doesn't fit perfectly
    assert!(result.rect.width > 0);
    assert!(result.rect.height > 0);
}

#[test]
fn test_corner_positioning_top_left() {
    let positioner = TooltipPositioner::new(1, 1, 1, true, true, true);
    let target = Rect::new(2, 2, 5, 2); // Top-left corner
    let screen = Rect::new(0, 0, 100, 30);

    let result = positioner.calculate(target, 15, 4, screen);

    // Should prefer right/bottom due to space constraints
    assert!(
        result.position == TooltipPosition::Right
            || result.position == TooltipPosition::Bottom
    );
}

#[test]
fn test_corner_positioning_bottom_right() {
    let positioner = TooltipPositioner::new(1, 1, 1, true, true, true);
    let target = Rect::new(90, 25, 5, 2); // Bottom-right corner
    let screen = Rect::new(0, 0, 100, 30);

    let result = positioner.calculate(target, 15, 4, screen);

    // Should flip to left/top due to space constraints
    assert!(result.was_flipped);
    assert!(
        result.position == TooltipPosition::Left || result.position == TooltipPosition::Top
    );
}

#[test]
fn test_arrow_directions_match_positions() {
    let positioner = TooltipPositioner::new(1, 1, 0, false, true, true);
    let target = Rect::new(40, 15, 10, 3);
    let screen = Rect::new(0, 0, 100, 40);

    // Test all positions by constraining space
    let positions = vec![
        (Rect::new(10, 15, 10, 3), TooltipPosition::Right, ArrowDirection::Left),
        (Rect::new(80, 15, 10, 3), TooltipPosition::Left, ArrowDirection::Right),
        (Rect::new(40, 5, 10, 3), TooltipPosition::Bottom, ArrowDirection::Up),
        (Rect::new(40, 35, 10, 3), TooltipPosition::Top, ArrowDirection::Down),
    ];

    for (test_target, expected_pos, expected_arrow) in positions {
        let result = positioner.calculate(test_target, 10, 3, screen);
        if result.position == expected_pos {
            assert_eq!(
                result.arrow_direction, expected_arrow,
                "Arrow mismatch for position {:?}",
                expected_pos
            );
        }
    }
}
