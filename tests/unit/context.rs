//! Unit tests for LocustContext
//!
//! Tests for the core context type that manages shared state across plugins.

use locust::core::context::LocustContext;
use locust::core::targets::{NavTarget, TargetBuilder};
use locust::plugins::tooltip::TooltipContent;
use ratatui::layout::Rect;

#[test]
fn test_context_creation() {
    let ctx = LocustContext::default();
    assert_eq!(ctx.targets.len(), 0);
    assert_eq!(ctx.tooltips.len(), 0);
}

#[test]
fn test_context_target_registration() {
    let mut ctx = LocustContext::default();
    let target = NavTarget::new(1, Rect::new(10, 10, 20, 5));

    ctx.targets.register(target);
    assert_eq!(ctx.targets.len(), 1);
    assert!(ctx.targets.by_id(1).is_some());
}

#[test]
fn test_context_target_retrieval() {
    let mut ctx = LocustContext::default();
    let rect = Rect::new(5, 5, 15, 3);
    ctx.targets.register(NavTarget::new(42, rect));

    let target = ctx.targets.by_id(42).unwrap();
    assert_eq!(target.id, 42);
    assert_eq!(target.rect, rect);
}

#[test]
fn test_context_multiple_targets() {
    let mut ctx = LocustContext::default();

    for i in 1..=10 {
        ctx.targets.register(NavTarget::new(i, Rect::new(i as u16 * 5, 10, 10, 2)));
    }

    assert_eq!(ctx.targets.len(), 10);
    assert!(ctx.targets.by_id(5).is_some());
    assert!(ctx.targets.by_id(11).is_none());
}

#[test]
fn test_context_tooltip_registration() {
    let mut ctx = LocustContext::default();
    let content = TooltipContent::new("Test tooltip");

    ctx.tooltips.register(1, content);
    assert_eq!(ctx.tooltips.len(), 1);
    assert!(ctx.tooltips.get(1).is_some());
}

#[test]
fn test_context_tooltip_content() {
    let mut ctx = LocustContext::default();
    let content = TooltipContent::new("Important information")
        .with_title("Info");

    ctx.tooltips.register(5, content);

    let retrieved = ctx.tooltips.get(5).unwrap();
    assert_eq!(retrieved.text(), "Important information");
    assert_eq!(retrieved.title(), Some("Info"));
}

#[test]
fn test_context_overlay_state() {
    let mut ctx = LocustContext::default();
    assert!(!ctx.overlay.has_overlay);

    ctx.overlay.mark_has_overlay();
    assert!(ctx.overlay.has_overlay);
}

#[test]
fn test_context_clear_targets() {
    let mut ctx = LocustContext::default();

    ctx.targets.register(NavTarget::new(1, Rect::new(0, 0, 10, 10)));
    ctx.targets.register(NavTarget::new(2, Rect::new(10, 10, 10, 10)));
    assert_eq!(ctx.targets.len(), 2);

    ctx.targets.clear();
    assert_eq!(ctx.targets.len(), 0);
}

#[test]
fn test_context_target_builder_integration() {
    let mut ctx = LocustContext::default();

    let target = TargetBuilder::new()
        .id(100)
        .rect(Rect::new(20, 20, 30, 4))
        .label("Test Target".to_string())
        .build();

    ctx.targets.register(target);

    let retrieved = ctx.targets.by_id(100).unwrap();
    assert_eq!(retrieved.label, Some("Test Target".to_string()));
}

#[test]
fn test_context_spatial_queries() {
    let mut ctx = LocustContext::default();

    ctx.targets.register(NavTarget::new(1, Rect::new(0, 0, 10, 10)));
    ctx.targets.register(NavTarget::new(2, Rect::new(20, 20, 10, 10)));
    ctx.targets.register(NavTarget::new(3, Rect::new(5, 5, 10, 10)));

    // Test nearest target
    let nearest = ctx.targets.nearest_to(8, 8);
    assert!(nearest.is_some());
}

#[test]
fn test_context_tooltip_removal() {
    let mut ctx = LocustContext::default();

    ctx.tooltips.register(1, TooltipContent::new("Test"));
    assert_eq!(ctx.tooltips.len(), 1);

    ctx.tooltips.remove(1);
    assert_eq!(ctx.tooltips.len(), 0);
}

#[test]
fn test_context_multiple_tooltip_styles() {
    use locust::plugins::tooltip::TooltipStyle;

    let mut ctx = LocustContext::default();

    ctx.tooltips.register(1, TooltipContent::new("Info").with_style(TooltipStyle::Info));
    ctx.tooltips.register(2, TooltipContent::new("Warning").with_style(TooltipStyle::Warning));
    ctx.tooltips.register(3, TooltipContent::new("Error").with_style(TooltipStyle::Error));

    assert_eq!(ctx.tooltips.len(), 3);
}

#[test]
fn test_context_empty_operations() {
    let mut ctx = LocustContext::default();

    // Should not panic on empty context
    assert!(ctx.targets.by_id(999).is_none());
    assert!(ctx.tooltips.get(999).is_none());
    ctx.targets.clear();
    ctx.tooltips.clear();
}
