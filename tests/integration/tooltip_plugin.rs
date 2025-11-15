//! Integration tests for TooltipPlugin.

use locust::core::context::LocustContext;
use locust::core::plugin::LocustPlugin;
use locust::core::targets::NavTarget;
use locust::plugins::tooltip::{
    TooltipConfig, TooltipContent, TooltipMode, TooltipPlugin, TooltipStyle,
};
use ratatui::backend::TestBackend;
use ratatui::layout::Rect;

#[test]
fn test_tooltip_plugin_lifecycle() {
    let mut ctx = LocustContext::default();
    let mut plugin = TooltipPlugin::new();

    // Initialize
    <TooltipPlugin as LocustPlugin<TestBackend>>::init(&mut plugin, &mut ctx);
    assert_eq!(plugin.mode(), TooltipMode::Hidden);

    // Cleanup
    <TooltipPlugin as LocustPlugin<TestBackend>>::cleanup(&mut plugin, &mut ctx);
    assert_eq!(plugin.mode(), TooltipMode::Hidden);
}

#[test]
fn test_tooltip_registration_and_retrieval() {
    let mut ctx = LocustContext::default();

    // Register target
    ctx.targets
        .register(NavTarget::new(1, Rect::new(10, 10, 20, 3)));

    // Register tooltip
    ctx.tooltips.register(
        1,
        TooltipContent::new("Click to activate navigation hints")
            .with_title("Navigation Mode"),
    );

    // Retrieve tooltip
    let tooltip = ctx.tooltips.get(1);
    assert!(tooltip.is_some());
    assert_eq!(
        tooltip.unwrap().body,
        "Click to activate navigation hints"
    );
    assert_eq!(tooltip.unwrap().title, Some("Navigation Mode".to_string()));
}

#[test]
fn test_tooltip_with_different_styles() {
    let mut ctx = LocustContext::default();

    // Register targets
    ctx.targets
        .register(NavTarget::new(1, Rect::new(10, 10, 20, 3)));
    ctx.targets
        .register(NavTarget::new(2, Rect::new(10, 15, 20, 3)));
    ctx.targets
        .register(NavTarget::new(3, Rect::new(10, 20, 20, 3)));
    ctx.targets
        .register(NavTarget::new(4, Rect::new(10, 25, 20, 3)));

    // Register tooltips with different styles
    ctx.tooltips
        .register(1, TooltipContent::new("Info").with_style(TooltipStyle::Info));
    ctx.tooltips.register(
        2,
        TooltipContent::new("Warning").with_style(TooltipStyle::Warning),
    );
    ctx.tooltips.register(
        3,
        TooltipContent::new("Error").with_style(TooltipStyle::Error),
    );
    ctx.tooltips.register(
        4,
        TooltipContent::new("Success").with_style(TooltipStyle::Success),
    );

    // Verify styles
    assert_eq!(ctx.tooltips.get(1).unwrap().style, TooltipStyle::Info);
    assert_eq!(ctx.tooltips.get(2).unwrap().style, TooltipStyle::Warning);
    assert_eq!(ctx.tooltips.get(3).unwrap().style, TooltipStyle::Error);
    assert_eq!(ctx.tooltips.get(4).unwrap().style, TooltipStyle::Success);
}

#[test]
fn test_tooltip_plugin_with_custom_config() {
    let config = TooltipConfig::new()
        .with_activation_key('?')
        .with_hover_delay_ms(100)
        .with_max_width(60)
        .with_border(false);

    let plugin = TooltipPlugin::with_config(config);

    assert_eq!(plugin.config().activation_key, Some('?'));
    assert_eq!(plugin.config().hover_delay_ms, 100);
    assert_eq!(plugin.config().max_width, 60);
    assert!(!plugin.config().show_border);
}

#[test]
fn test_tooltip_multiline_content() {
    let mut ctx = LocustContext::default();

    ctx.targets
        .register(NavTarget::new(1, Rect::new(10, 10, 20, 3)));

    let multiline = "Line 1: Introduction\nLine 2: Details\nLine 3: More info";
    ctx.tooltips.register(
        1,
        TooltipContent::new(multiline)
            .with_title("Help")
            .with_style(TooltipStyle::Info),
    );

    let tooltip = ctx.tooltips.get(1).unwrap();
    assert_eq!(tooltip.line_count(), 4); // 1 title + 3 body lines
    assert_eq!(tooltip.body_lines().len(), 3);
}

#[test]
fn test_tooltip_registry_operations() {
    let mut ctx = LocustContext::default();

    // Register multiple tooltips
    for i in 1..=5 {
        ctx.tooltips
            .register(i, TooltipContent::new(format!("Tooltip {}", i)));
    }

    assert_eq!(ctx.tooltips.len(), 5);
    assert!(ctx.tooltips.contains(3));
    assert!(!ctx.tooltips.contains(10));

    // Remove one
    assert!(ctx.tooltips.remove(3));
    assert_eq!(ctx.tooltips.len(), 4);
    assert!(!ctx.tooltips.contains(3));

    // Clear all
    ctx.tooltips.clear();
    assert_eq!(ctx.tooltips.len(), 0);
    assert!(ctx.tooltips.is_empty());
}

#[test]
fn test_tooltip_max_dimensions_respected() {
    let config = TooltipConfig::new().with_max_width(20).with_max_height(5);

    let plugin = TooltipPlugin::with_config(config);

    assert_eq!(plugin.config().max_width, 20);
    assert_eq!(plugin.config().max_height, 5);

    // Long content should be constrained
    let long_text = "This is a very long tooltip that exceeds the maximum width";
    let content = TooltipContent::new(long_text);

    assert!(content.max_line_width() > 20);
    // Plugin should clamp to max_width during rendering
}

#[test]
fn test_tooltip_hover_only_mode() {
    let config = TooltipConfig::new().hover_only();
    let plugin = TooltipPlugin::with_config(config);

    assert_eq!(plugin.config().activation_key, None);
}

#[test]
fn test_tooltip_positioning_preferences() {
    let config_prefer_left_top = TooltipConfig::new()
        .prefer_right(false)
        .prefer_bottom(false);

    let plugin = TooltipPlugin::with_config(config_prefer_left_top);

    assert!(!plugin.config().prefer_right);
    assert!(!plugin.config().prefer_bottom);
}

#[test]
fn test_tooltip_auto_hide_configuration() {
    let config = TooltipConfig::new().with_auto_hide_timeout_ms(3000);
    let plugin = TooltipPlugin::with_config(config);

    assert_eq!(plugin.config().auto_hide_timeout_ms, 3000);
}

#[test]
fn test_tooltip_with_long_title() {
    let content = TooltipContent::new("Short body")
        .with_title("This is a very long title that exceeds the body width");

    // Title should influence max_line_width
    assert!(content.max_line_width() > 10);
    assert_eq!(content.line_count(), 2); // title + body
}

#[test]
fn test_tooltip_empty_body() {
    let content = TooltipContent::new("").with_title("Title Only");

    assert_eq!(content.line_count(), 2); // title + 1 (empty body counts as 1)
    assert_eq!(content.body, "");
}

#[test]
fn test_tooltip_plugin_priority_between_omnibar_and_nav() {
    let plugin = TooltipPlugin::new();
    let priority = <TooltipPlugin as LocustPlugin<TestBackend>>::priority(&plugin);

    // Should be between omnibar (40) and nav (50)
    assert!(priority > 40);
    assert!(priority < 50);
    assert_eq!(priority, 45);
}

#[test]
fn test_multiple_tooltips_on_screen() {
    let mut ctx = LocustContext::default();

    // Register multiple targets with tooltips
    for i in 1..=10 {
        ctx.targets.register(NavTarget::new(
            i,
            Rect::new(0, (i as u16 - 1) * 3, 30, 2),
        ));
        ctx.tooltips.register(
            i,
            TooltipContent::new(format!("Tooltip for target {}", i))
                .with_title(format!("Target {}", i)),
        );
    }

    assert_eq!(ctx.targets.len(), 10);
    assert_eq!(ctx.tooltips.len(), 10);

    // All tooltips should be retrievable
    for i in 1..=10 {
        assert!(ctx.tooltips.get(i).is_some());
    }
}

#[test]
fn test_tooltip_replace_existing() {
    let mut ctx = LocustContext::default();

    ctx.tooltips
        .register(1, TooltipContent::new("Original tooltip"));
    assert_eq!(ctx.tooltips.get(1).unwrap().body, "Original tooltip");

    ctx.tooltips
        .register(1, TooltipContent::new("Updated tooltip"));
    assert_eq!(ctx.tooltips.get(1).unwrap().body, "Updated tooltip");
    assert_eq!(ctx.tooltips.len(), 1);
}
