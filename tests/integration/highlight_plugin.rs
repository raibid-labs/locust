//! Integration tests for the highlight plugin.

use locust::core::context::{Locust, LocustConfig, LocustContext};
use locust::core::plugin::LocustPlugin;
use locust::core::targets::NavTarget;
use locust::plugins::highlight::{HighlightConfig, HighlightPlugin, MessagePosition, Tour, TourStep};
use ratatui::backend::TestBackend;
use ratatui::layout::Rect;
use ratatui::Terminal;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};

#[test]
fn test_plugin_registration_and_init() {
    let mut ctx = LocustContext::default();
    let mut plugin = HighlightPlugin::new();

    LocustPlugin::<TestBackend>::init(&mut plugin, &mut ctx);

    assert_eq!(LocustPlugin::<TestBackend>::id(&plugin), "locust.highlight");
    assert_eq!(LocustPlugin::<TestBackend>::priority(&plugin), 30);
}

#[test]
fn test_plugin_with_locust_integration() {
    let config = LocustConfig::default();
    let mut locust: Locust<TestBackend> = Locust::new(config);

    let highlight = HighlightPlugin::new();
    locust.register_plugin(highlight);

    assert_eq!(locust.plugin_count(), 1);
    assert!(locust.has_plugin("locust.highlight"));
}

#[test]
fn test_tour_activation_via_event() {
    let mut plugin = HighlightPlugin::new();
    let mut ctx = LocustContext::default();

    // Register a tour
    let tour = Tour::new("test")
        .add_step(TourStep::new("Step 1", "First step"));
    plugin.register_tour(tour);

    LocustPlugin::<TestBackend>::init(&mut plugin, &mut ctx);

    // Activate with configured key
    let event = Event::Key(KeyEvent::new(KeyCode::Char('?'), KeyModifiers::NONE));
    let result = LocustPlugin::<TestBackend>::on_event(&mut plugin, &event, &mut ctx);

    assert!(result.is_consumed());
    assert!(result.requests_redraw());
    assert!(ctx.overlay.has_overlay);
}

#[test]
fn test_tour_navigation_via_events() {
    let mut plugin = HighlightPlugin::new();
    let mut ctx = LocustContext::default();

    let tour = Tour::new("test")
        .add_step(TourStep::new("Step 1", "First"))
        .add_step(TourStep::new("Step 2", "Second"))
        .add_step(TourStep::new("Step 3", "Third"));
    plugin.register_tour(tour);

    LocustPlugin::<TestBackend>::init(&mut plugin, &mut ctx);

    // Start tour
    plugin.start_tour("test", &mut ctx);

    // Navigate forward with arrow key
    let event = Event::Key(KeyEvent::new(KeyCode::Right, KeyModifiers::NONE));
    let result = LocustPlugin::<TestBackend>::on_event(&mut plugin, &event, &mut ctx);
    assert!(result.is_consumed());

    // Navigate backward
    let event = Event::Key(KeyEvent::new(KeyCode::Left, KeyModifiers::NONE));
    let result = LocustPlugin::<TestBackend>::on_event(&mut plugin, &event, &mut ctx);
    assert!(result.is_consumed());

    // Advance with 'n'
    let event = Event::Key(KeyEvent::new(KeyCode::Char('n'), KeyModifiers::NONE));
    let _result = LocustPlugin::<TestBackend>::on_event(&mut plugin, &event, &mut ctx);

    // Previous with 'p'
    let event = Event::Key(KeyEvent::new(KeyCode::Char('p'), KeyModifiers::NONE));
    let _result = LocustPlugin::<TestBackend>::on_event(&mut plugin, &event, &mut ctx);
}

#[test]
fn test_tour_skip_via_escape() {
    let mut plugin = HighlightPlugin::new();
    let mut ctx = LocustContext::default();

    let tour = Tour::new("test")
        .add_step(TourStep::new("Step 1", "First"))
        .with_skippable(true);
    plugin.register_tour(tour);

    plugin.start_tour("test", &mut ctx);
    assert!(ctx.overlay.has_overlay);

    // Press escape to skip
    let event = Event::Key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
    let result = LocustPlugin::<TestBackend>::on_event(&mut plugin, &event, &mut ctx);

    assert!(result.is_consumed());
    // Tour should be stopped
    assert!(plugin.active_tour().is_none());
}

#[test]
fn test_non_skippable_tour() {
    let mut plugin = HighlightPlugin::new();
    let mut ctx = LocustContext::default();

    let tour = Tour::new("test")
        .add_step(TourStep::new("Step 1", "First"))
        .with_skippable(false);
    plugin.register_tour(tour);

    plugin.start_tour("test", &mut ctx);

    // Escape should not skip
    let event = Event::Key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
    let result = LocustPlugin::<TestBackend>::on_event(&mut plugin, &event, &mut ctx);

    // Event not consumed for non-skippable tours
    assert!(!result.is_consumed());
    assert!(plugin.active_tour().is_some());
}

#[test]
fn test_tour_completion_tracking() {
    let mut plugin = HighlightPlugin::new();
    let mut ctx = LocustContext::default();

    let tour = Tour::new("onboarding")
        .add_step(TourStep::new("Step 1", "First"));
    plugin.register_tour(tour);

    assert!(!plugin.is_tour_completed("onboarding"));

    plugin.start_tour("onboarding", &mut ctx);

    // Complete the tour by advancing past last step
    let event = Event::Key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
    LocustPlugin::<TestBackend>::on_event(&mut plugin, &event, &mut ctx);

    // Should be marked as completed
    assert!(plugin.is_tour_completed("onboarding"));
}

#[test]
fn test_multiple_tours() {
    let mut plugin = HighlightPlugin::new();

    let tour1 = Tour::new("tour1")
        .add_step(TourStep::new("T1S1", "Tour 1 Step 1"));
    let tour2 = Tour::new("tour2")
        .add_step(TourStep::new("T2S1", "Tour 2 Step 1"));

    plugin.register_tour(tour1);
    plugin.register_tour(tour2);

    let tour_ids = plugin.tour_ids();
    assert_eq!(tour_ids.len(), 2);
    assert!(tour_ids.contains(&"tour1".to_string()));
    assert!(tour_ids.contains(&"tour2".to_string()));
}

#[test]
fn test_tour_with_nav_targets() {
    let mut plugin = HighlightPlugin::new();
    let mut ctx = LocustContext::default();

    // Register a nav target
    ctx.targets.register(
        NavTarget::new(42, Rect::new(10, 10, 30, 5))
            .with_label("Submit Button")
    );

    // Create tour that highlights the target
    let tour = Tour::new("test")
        .add_step(
            TourStep::new("Click Submit", "Click this button to submit")
                .with_target(42)
                .with_position(MessagePosition::Bottom)
        );

    plugin.register_tour(tour);
    plugin.start_tour("test", &mut ctx);

    let active_tour = plugin.active_tour().unwrap();
    let step = active_tour.current_step().unwrap();
    let highlight_rect = step.highlight_rect(Some(&ctx.targets));

    assert!(highlight_rect.is_some());
    assert_eq!(highlight_rect.unwrap(), Rect::new(10, 10, 30, 5));
}

#[test]
fn test_tour_with_explicit_area() {
    let mut plugin = HighlightPlugin::new();
    let mut ctx = LocustContext::default();

    let area = Rect::new(20, 20, 40, 10);
    let tour = Tour::new("test")
        .add_step(
            TourStep::new("Focus Here", "Pay attention to this area")
                .with_area(area)
        );

    plugin.register_tour(tour);
    plugin.start_tour("test", &mut ctx);

    let active_tour = plugin.active_tour().unwrap();
    let step = active_tour.current_step().unwrap();
    let highlight_rect = step.highlight_rect(Some(&ctx.targets));

    assert_eq!(highlight_rect, Some(area));
}

#[test]
fn test_plugin_cleanup() {
    let mut plugin = HighlightPlugin::new();
    let mut ctx = LocustContext::default();

    let tour = Tour::new("test")
        .add_step(TourStep::new("Step 1", "First"));
    plugin.register_tour(tour);

    plugin.start_tour("test", &mut ctx);
    assert!(plugin.active_tour().is_some());

    LocustPlugin::<TestBackend>::cleanup(&mut plugin, &mut ctx);
    assert!(plugin.active_tour().is_none());
}

#[test]
fn test_rendering_does_not_crash() {
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();

    let mut plugin = HighlightPlugin::new();
    let mut ctx = LocustContext::default();

    let tour = Tour::new("test")
        .add_step(
            TourStep::new("Welcome", "Welcome to the app!")
                .with_area(Rect::new(10, 5, 50, 10))
                .with_position(MessagePosition::Center)
        );

    plugin.register_tour(tour);
    plugin.start_tour("test", &mut ctx);

    // Render should not crash
    terminal
        .draw(|frame| {
            LocustPlugin::<TestBackend>::render_overlay(&plugin, frame, &ctx);
        })
        .unwrap();
}

#[test]
fn test_custom_config_integration() {
    let config = HighlightConfig::new()
        .with_activation_key('h')
        .with_dim_opacity(150)
        .with_navigation_hints(false);

    let plugin = HighlightPlugin::with_config(config);

    assert_eq!(plugin.config().activation_key, 'h');
    assert_eq!(plugin.config().dim_opacity, 150);
    assert!(!plugin.config().show_navigation_hints);
}

#[test]
fn test_tour_replace_existing() {
    let mut plugin = HighlightPlugin::new();

    let tour1 = Tour::new("test")
        .add_step(TourStep::new("Original", "Original step"));

    let tour2 = Tour::new("test")
        .add_step(TourStep::new("Replaced", "Replaced step"));

    plugin.register_tour(tour1);
    plugin.register_tour(tour2);

    // Should only have one tour with id "test"
    assert_eq!(plugin.tour_ids().len(), 1);
}
