//! Unit tests for tour management system.

use locust::plugins::highlight::{MessagePosition, Tour, TourState, TourStep};
use ratatui::layout::Rect;

#[test]
fn test_tour_step_builder() {
    let step = TourStep::new("Welcome", "Welcome to the app!")
        .with_target(42)
        .with_position(MessagePosition::Center)
        .with_auto_advance(3000)
        .with_metadata("category", "onboarding");

    assert_eq!(step.title, "Welcome");
    assert_eq!(step.message, "Welcome to the app!");
    assert_eq!(step.target_id, Some(42));
    assert_eq!(step.position, MessagePosition::Center);
    assert_eq!(step.auto_advance_ms, 3000);
    assert_eq!(step.metadata.get("category"), Some(&"onboarding".to_string()));
}

#[test]
fn test_tour_step_with_area() {
    let area = Rect::new(10, 10, 50, 20);
    let step = TourStep::new("Step", "Message").with_area(area);

    assert_eq!(step.highlight_area, Some(area));
    assert_eq!(step.target_id, None);
}

#[test]
fn test_tour_step_target_overrides_area() {
    let area = Rect::new(10, 10, 50, 20);
    let step = TourStep::new("Step", "Message")
        .with_area(area)
        .with_target(99);

    assert_eq!(step.target_id, Some(99));
    assert_eq!(step.highlight_area, None);
}

#[test]
fn test_tour_creation_and_setup() {
    let tour = Tour::new("onboarding")
        .add_step(TourStep::new("Step 1", "First step"))
        .add_step(TourStep::new("Step 2", "Second step"))
        .add_step(TourStep::new("Step 3", "Third step"))
        .with_description("User onboarding tour")
        .with_skippable(true)
        .with_loop(false);

    assert_eq!(tour.id, "onboarding");
    assert_eq!(tour.steps.len(), 3);
    assert_eq!(tour.description, Some("User onboarding tour".to_string()));
    assert!(tour.skippable);
    assert!(!tour.loop_tour);
}

#[test]
fn test_tour_lifecycle() {
    let mut tour = Tour::new("test")
        .add_step(TourStep::new("Step 1", "First"));

    assert_eq!(tour.state, TourState::Inactive);
    assert!(!tour.is_active());

    tour.start();
    assert_eq!(tour.state, TourState::Active);
    assert!(tour.is_active());

    tour.pause();
    assert_eq!(tour.state, TourState::Paused);
    assert!(!tour.is_active());

    tour.resume();
    assert_eq!(tour.state, TourState::Active);

    tour.stop();
    assert_eq!(tour.state, TourState::Inactive);
}

#[test]
fn test_tour_forward_navigation() {
    let mut tour = Tour::new("test")
        .add_step(TourStep::new("Step 1", "First"))
        .add_step(TourStep::new("Step 2", "Second"))
        .add_step(TourStep::new("Step 3", "Third"));

    tour.start();

    assert_eq!(tour.current_index(), 0);
    assert!(tour.is_first_step());
    assert!(!tour.is_last_step());

    assert!(tour.next_step());
    assert_eq!(tour.current_index(), 1);
    assert!(!tour.is_first_step());
    assert!(!tour.is_last_step());

    assert!(tour.next_step());
    assert_eq!(tour.current_index(), 2);
    assert!(!tour.is_first_step());
    assert!(tour.is_last_step());

    // At end without loop, should stop
    assert!(!tour.next_step());
    assert_eq!(tour.state, TourState::Inactive);
}

#[test]
fn test_tour_backward_navigation() {
    let mut tour = Tour::new("test")
        .add_step(TourStep::new("Step 1", "First"))
        .add_step(TourStep::new("Step 2", "Second"))
        .add_step(TourStep::new("Step 3", "Third"));

    tour.start();
    tour.jump_to(2);

    assert_eq!(tour.current_index(), 2);

    assert!(tour.previous_step());
    assert_eq!(tour.current_index(), 1);

    assert!(tour.previous_step());
    assert_eq!(tour.current_index(), 0);

    // At beginning without loop, should stay
    assert!(!tour.previous_step());
    assert_eq!(tour.current_index(), 0);
}

#[test]
fn test_tour_loop_behavior() {
    let mut tour = Tour::new("test")
        .add_step(TourStep::new("Step 1", "First"))
        .add_step(TourStep::new("Step 2", "Second"))
        .with_loop(true);

    tour.start();
    tour.jump_to(1);

    // At end with loop, should wrap to start
    assert!(tour.next_step());
    assert_eq!(tour.current_index(), 0);
    assert_eq!(tour.state, TourState::Active);

    // At beginning with loop, should wrap to end
    assert!(tour.previous_step());
    assert_eq!(tour.current_index(), 1);
}

#[test]
fn test_tour_jump_to() {
    let mut tour = Tour::new("test")
        .add_step(TourStep::new("Step 1", "First"))
        .add_step(TourStep::new("Step 2", "Second"))
        .add_step(TourStep::new("Step 3", "Third"));

    tour.start();

    assert!(tour.jump_to(2));
    assert_eq!(tour.current_index(), 2);

    assert!(tour.jump_to(0));
    assert_eq!(tour.current_index(), 0);

    assert!(!tour.jump_to(10));
    assert_eq!(tour.current_index(), 0);
}

#[test]
fn test_tour_progress() {
    let mut tour = Tour::new("test")
        .add_step(TourStep::new("Step 1", "First"))
        .add_step(TourStep::new("Step 2", "Second"))
        .add_step(TourStep::new("Step 3", "Third"));

    tour.start();

    assert_eq!(tour.progress(), (1, 3));
    assert_eq!(tour.total_steps(), 3);

    tour.next_step();
    assert_eq!(tour.progress(), (2, 3));

    tour.next_step();
    assert_eq!(tour.progress(), (3, 3));
}

#[test]
fn test_tour_current_step() {
    let mut tour = Tour::new("test")
        .add_step(TourStep::new("Step 1", "First"))
        .add_step(TourStep::new("Step 2", "Second"));

    // Inactive tour has no current step
    assert!(tour.current_step().is_none());

    tour.start();
    let step = tour.current_step().unwrap();
    assert_eq!(step.title, "Step 1");

    tour.next_step();
    let step = tour.current_step().unwrap();
    assert_eq!(step.title, "Step 2");
}

#[test]
fn test_tour_empty_steps() {
    let mut tour = Tour::new("empty");

    tour.start();
    // Should remain inactive if no steps
    assert_eq!(tour.state, TourState::Inactive);
    assert_eq!(tour.total_steps(), 0);
}

#[test]
fn test_navigation_only_works_when_active() {
    let mut tour = Tour::new("test")
        .add_step(TourStep::new("Step 1", "First"))
        .add_step(TourStep::new("Step 2", "Second"));

    // Navigation should fail when inactive
    assert!(!tour.next_step());
    assert!(!tour.previous_step());

    tour.start();
    assert!(tour.next_step());

    tour.pause();
    // Navigation should fail when paused
    assert!(!tour.next_step());
    assert!(!tour.previous_step());
}

#[test]
fn test_message_positions() {
    let positions = vec![
        MessagePosition::Top,
        MessagePosition::Bottom,
        MessagePosition::Left,
        MessagePosition::Right,
        MessagePosition::Center,
    ];

    for pos in positions {
        let step = TourStep::new("Test", "Message").with_position(pos);
        assert_eq!(step.position, pos);
    }
}

#[test]
fn test_tour_step_metadata() {
    let step = TourStep::new("Test", "Message")
        .with_metadata("key1", "value1")
        .with_metadata("key2", "value2")
        .with_metadata("importance", "high");

    assert_eq!(step.metadata.len(), 3);
    assert_eq!(step.metadata.get("key1"), Some(&"value1".to_string()));
    assert_eq!(step.metadata.get("importance"), Some(&"high".to_string()));
}
