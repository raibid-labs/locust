//! Tour system for multi-step guided highlights.

use ratatui::layout::Rect;
use std::collections::HashMap;

/// Position for message display relative to highlighted area.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessagePosition {
    /// Display message above the highlighted area
    Top,
    /// Display message below the highlighted area
    Bottom,
    /// Display message to the left of the highlighted area
    Left,
    /// Display message to the right of the highlighted area
    Right,
    /// Display message in the center of the screen
    Center,
}

impl Default for MessagePosition {
    fn default() -> Self {
        Self::Bottom
    }
}

/// A single step in a guided tour.
#[derive(Debug, Clone)]
pub struct TourStep {
    /// Optional target ID to highlight (from NavTarget registry)
    pub target_id: Option<u64>,

    /// Optional explicit highlight area (if target_id is None)
    pub highlight_area: Option<Rect>,

    /// Title of this step
    pub title: String,

    /// Detailed message/instructions for this step
    pub message: String,

    /// Where to position the message box
    pub position: MessagePosition,

    /// Optional custom metadata for this step
    pub metadata: HashMap<String, String>,

    /// Auto-advance to next step after this many milliseconds (0 = manual only)
    pub auto_advance_ms: u64,
}

impl TourStep {
    /// Creates a new tour step with the given title and message.
    pub fn new(title: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            target_id: None,
            highlight_area: None,
            title: title.into(),
            message: message.into(),
            position: MessagePosition::default(),
            metadata: HashMap::new(),
            auto_advance_ms: 0,
        }
    }

    /// Sets the target ID to highlight (from NavTarget registry).
    pub fn with_target(mut self, target_id: u64) -> Self {
        self.target_id = Some(target_id);
        self.highlight_area = None; // Clear explicit area
        self
    }

    /// Sets an explicit highlight area.
    pub fn with_area(mut self, area: Rect) -> Self {
        self.highlight_area = Some(area);
        self.target_id = None; // Clear target ID
        self
    }

    /// Sets the message position.
    pub fn with_position(mut self, position: MessagePosition) -> Self {
        self.position = position;
        self
    }

    /// Adds metadata to this step.
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Sets auto-advance timing in milliseconds.
    pub fn with_auto_advance(mut self, ms: u64) -> Self {
        self.auto_advance_ms = ms;
        self
    }

    /// Gets the highlight rect for this step, either from target or explicit area.
    pub fn highlight_rect(
        &self,
        registry: Option<&crate::core::targets::TargetRegistry>,
    ) -> Option<Rect> {
        if let Some(target_id) = self.target_id {
            if let Some(registry) = registry {
                return registry.by_id(target_id).map(|t| t.rect);
            }
        }
        self.highlight_area
    }
}

/// State of a tour.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TourState {
    /// Tour is not running
    Inactive,
    /// Tour is running and displaying a step
    Active,
    /// Tour is paused
    Paused,
}

/// A guided tour consisting of multiple steps.
#[derive(Debug, Clone)]
pub struct Tour {
    /// Unique identifier for this tour
    pub id: String,

    /// All steps in this tour
    pub steps: Vec<TourStep>,

    /// Current step index (0-based)
    pub current_step: usize,

    /// Current state of the tour
    pub state: TourState,

    /// Optional description of this tour
    pub description: Option<String>,

    /// Whether this tour can be skipped
    pub skippable: bool,

    /// Whether to loop back to start after last step
    pub loop_tour: bool,
}

impl Tour {
    /// Creates a new tour with the given ID.
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            steps: Vec::new(),
            current_step: 0,
            state: TourState::Inactive,
            description: None,
            skippable: true,
            loop_tour: false,
        }
    }

    /// Adds a step to the tour.
    pub fn add_step(mut self, step: TourStep) -> Self {
        self.steps.push(step);
        self
    }

    /// Sets the tour description.
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Sets whether the tour can be skipped.
    pub fn with_skippable(mut self, skippable: bool) -> Self {
        self.skippable = skippable;
        self
    }

    /// Sets whether the tour loops back to start.
    pub fn with_loop(mut self, loop_tour: bool) -> Self {
        self.loop_tour = loop_tour;
        self
    }

    /// Starts the tour from the beginning.
    pub fn start(&mut self) {
        if !self.steps.is_empty() {
            self.current_step = 0;
            self.state = TourState::Active;
        }
    }

    /// Stops the tour.
    pub fn stop(&mut self) {
        self.state = TourState::Inactive;
    }

    /// Pauses the tour.
    pub fn pause(&mut self) {
        if self.state == TourState::Active {
            self.state = TourState::Paused;
        }
    }

    /// Resumes a paused tour.
    pub fn resume(&mut self) {
        if self.state == TourState::Paused {
            self.state = TourState::Active;
        }
    }

    /// Advances to the next step.
    ///
    /// Returns `true` if advanced, `false` if at end.
    pub fn next_step(&mut self) -> bool {
        if self.state != TourState::Active {
            return false;
        }

        if self.current_step + 1 < self.steps.len() {
            self.current_step += 1;
            true
        } else if self.loop_tour {
            self.current_step = 0;
            true
        } else {
            self.stop();
            false
        }
    }

    /// Goes back to the previous step.
    ///
    /// Returns `true` if went back, `false` if at beginning.
    pub fn previous_step(&mut self) -> bool {
        if self.state != TourState::Active {
            return false;
        }

        if self.current_step > 0 {
            self.current_step -= 1;
            true
        } else if self.loop_tour && !self.steps.is_empty() {
            self.current_step = self.steps.len() - 1;
            true
        } else {
            false
        }
    }

    /// Jumps to a specific step by index.
    ///
    /// Returns `true` if jump was successful.
    pub fn jump_to(&mut self, step: usize) -> bool {
        if step < self.steps.len() {
            self.current_step = step;
            true
        } else {
            false
        }
    }

    /// Gets the current step.
    pub fn current_step(&self) -> Option<&TourStep> {
        if self.state != TourState::Inactive {
            self.steps.get(self.current_step)
        } else {
            None
        }
    }

    /// Gets the current step index.
    pub fn current_index(&self) -> usize {
        self.current_step
    }

    /// Gets the total number of steps.
    pub fn total_steps(&self) -> usize {
        self.steps.len()
    }

    /// Checks if the tour is active.
    pub fn is_active(&self) -> bool {
        self.state == TourState::Active
    }

    /// Checks if this is the last step.
    pub fn is_last_step(&self) -> bool {
        self.current_step + 1 >= self.steps.len()
    }

    /// Checks if this is the first step.
    pub fn is_first_step(&self) -> bool {
        self.current_step == 0
    }

    /// Gets the progress as a tuple (current, total).
    pub fn progress(&self) -> (usize, usize) {
        (self.current_step + 1, self.steps.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tour_creation() {
        let tour = Tour::new("onboarding")
            .add_step(TourStep::new("Welcome", "Welcome to the app!"))
            .add_step(TourStep::new("Features", "Here are the features"))
            .with_description("User onboarding tour");

        assert_eq!(tour.id, "onboarding");
        assert_eq!(tour.steps.len(), 2);
        assert_eq!(tour.current_step, 0);
        assert_eq!(tour.state, TourState::Inactive);
        assert!(tour.skippable);
    }

    #[test]
    fn test_tour_navigation() {
        let mut tour = Tour::new("test")
            .add_step(TourStep::new("Step 1", "First step"))
            .add_step(TourStep::new("Step 2", "Second step"))
            .add_step(TourStep::new("Step 3", "Third step"));

        // Start tour
        tour.start();
        assert_eq!(tour.state, TourState::Active);
        assert_eq!(tour.current_step, 0);

        // Navigate forward
        assert!(tour.next_step());
        assert_eq!(tour.current_step, 1);
        assert!(tour.next_step());
        assert_eq!(tour.current_step, 2);

        // At end, should return false and stop
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
        tour.next_step();
        tour.next_step();
        assert_eq!(tour.current_step, 2);

        assert!(tour.previous_step());
        assert_eq!(tour.current_step, 1);
        assert!(tour.previous_step());
        assert_eq!(tour.current_step, 0);

        // At beginning, should return false
        assert!(!tour.previous_step());
        assert_eq!(tour.current_step, 0);
    }

    #[test]
    fn test_tour_loop() {
        let mut tour = Tour::new("test")
            .add_step(TourStep::new("Step 1", "First"))
            .add_step(TourStep::new("Step 2", "Second"))
            .with_loop(true);

        tour.start();
        tour.next_step();
        assert_eq!(tour.current_step, 1);

        // Should loop back to start
        assert!(tour.next_step());
        assert_eq!(tour.current_step, 0);
        assert_eq!(tour.state, TourState::Active);
    }

    #[test]
    fn test_tour_pause_resume() {
        let mut tour = Tour::new("test").add_step(TourStep::new("Step 1", "First"));

        tour.start();
        assert_eq!(tour.state, TourState::Active);

        tour.pause();
        assert_eq!(tour.state, TourState::Paused);

        tour.resume();
        assert_eq!(tour.state, TourState::Active);
    }

    #[test]
    fn test_tour_jump_to() {
        let mut tour = Tour::new("test")
            .add_step(TourStep::new("Step 1", "First"))
            .add_step(TourStep::new("Step 2", "Second"))
            .add_step(TourStep::new("Step 3", "Third"));

        tour.start();
        assert!(tour.jump_to(2));
        assert_eq!(tour.current_step, 2);

        assert!(!tour.jump_to(5));
        assert_eq!(tour.current_step, 2);
    }

    #[test]
    fn test_tour_progress() {
        let mut tour = Tour::new("test")
            .add_step(TourStep::new("Step 1", "First"))
            .add_step(TourStep::new("Step 2", "Second"))
            .add_step(TourStep::new("Step 3", "Third"));

        tour.start();
        assert_eq!(tour.progress(), (1, 3));

        tour.next_step();
        assert_eq!(tour.progress(), (2, 3));

        assert!(tour.is_first_step() == false);
        assert!(tour.is_last_step() == false);

        tour.next_step();
        assert_eq!(tour.progress(), (3, 3));
        assert!(tour.is_last_step());
    }

    #[test]
    fn test_step_creation() {
        let step = TourStep::new("Title", "Message")
            .with_target(42)
            .with_position(MessagePosition::Top)
            .with_auto_advance(5000)
            .with_metadata("key", "value");

        assert_eq!(step.title, "Title");
        assert_eq!(step.message, "Message");
        assert_eq!(step.target_id, Some(42));
        assert_eq!(step.position, MessagePosition::Top);
        assert_eq!(step.auto_advance_ms, 5000);
        assert_eq!(step.metadata.get("key"), Some(&"value".to_string()));
    }

    #[test]
    fn test_step_with_area() {
        let rect = Rect::new(10, 10, 20, 10);
        let step = TourStep::new("Title", "Message").with_area(rect);

        assert_eq!(step.highlight_area, Some(rect));
        assert_eq!(step.target_id, None);
    }
}
