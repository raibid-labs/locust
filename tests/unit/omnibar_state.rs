//! Unit tests for OmnibarState.
//!
//! These tests verify:
//! - Input buffer management
//! - Cursor position tracking
//! - Mode transitions
//! - History management
//! - Unicode handling

use locust::plugins::omnibar::state::{OmnibarMode, OmnibarState};

#[test]
fn test_new_state_is_inactive() {
    let state = OmnibarState::new(10);
    assert_eq!(state.mode(), OmnibarMode::Inactive);
    assert!(!state.is_active());
    assert_eq!(state.buffer(), "");
    assert_eq!(state.cursor(), 0);
    assert_eq!(state.history().len(), 0);
}

#[test]
fn test_activation_sets_mode() {
    let mut state = OmnibarState::new(10);
    state.activate();
    assert_eq!(state.mode(), OmnibarMode::Input);
    assert!(state.is_active());
}

#[test]
fn test_deactivation_clears_buffer() {
    let mut state = OmnibarState::new(10);
    state.activate();
    state.insert_char('a');
    state.insert_char('b');
    assert_eq!(state.buffer(), "ab");

    state.deactivate();
    assert_eq!(state.mode(), OmnibarMode::Inactive);
    assert_eq!(state.buffer(), "");
    assert_eq!(state.cursor(), 0);
}

#[test]
fn test_insert_char_appends() {
    let mut state = OmnibarState::new(10);
    state.activate();
    state.insert_char('h');
    state.insert_char('e');
    state.insert_char('l');
    state.insert_char('l');
    state.insert_char('o');
    assert_eq!(state.buffer(), "hello");
    assert_eq!(state.cursor(), 5);
}

#[test]
fn test_delete_char_removes_before_cursor() {
    let mut state = OmnibarState::new(10);
    state.activate();
    state.insert_char('a');
    state.insert_char('b');
    state.insert_char('c');
    assert_eq!(state.buffer(), "abc");

    state.delete_char();
    assert_eq!(state.buffer(), "ab");
    assert_eq!(state.cursor(), 2);

    state.delete_char();
    assert_eq!(state.buffer(), "a");
    assert_eq!(state.cursor(), 1);
}

#[test]
fn test_delete_char_at_start_noop() {
    let mut state = OmnibarState::new(10);
    state.activate();
    state.delete_char();
    assert_eq!(state.buffer(), "");
    assert_eq!(state.cursor(), 0);
}

#[test]
fn test_cursor_left() {
    let mut state = OmnibarState::new(10);
    state.activate();
    state.insert_char('a');
    state.insert_char('b');
    state.insert_char('c');
    assert_eq!(state.cursor(), 3);

    state.move_cursor_left();
    assert_eq!(state.cursor(), 2);

    state.move_cursor_left();
    assert_eq!(state.cursor(), 1);

    state.move_cursor_left();
    assert_eq!(state.cursor(), 0);

    // Should not go below 0
    state.move_cursor_left();
    assert_eq!(state.cursor(), 0);
}

#[test]
fn test_cursor_right() {
    let mut state = OmnibarState::new(10);
    state.activate();
    state.insert_char('a');
    state.insert_char('b');
    state.insert_char('c');

    state.move_cursor_home();
    assert_eq!(state.cursor(), 0);

    state.move_cursor_right();
    assert_eq!(state.cursor(), 1);

    state.move_cursor_right();
    assert_eq!(state.cursor(), 2);

    state.move_cursor_right();
    assert_eq!(state.cursor(), 3);

    // Should not go beyond buffer length
    state.move_cursor_right();
    assert_eq!(state.cursor(), 3);
}

#[test]
fn test_cursor_home_end() {
    let mut state = OmnibarState::new(10);
    state.activate();
    state.insert_char('h');
    state.insert_char('e');
    state.insert_char('l');
    state.insert_char('l');
    state.insert_char('o');

    state.move_cursor_home();
    assert_eq!(state.cursor(), 0);

    state.move_cursor_end();
    assert_eq!(state.cursor(), 5);
}

#[test]
fn test_submit_non_empty() {
    let mut state = OmnibarState::new(10);
    state.activate();
    state.insert_char('t');
    state.insert_char('e');
    state.insert_char('s');
    state.insert_char('t');

    let result = state.submit();
    assert_eq!(result, Some("test".to_string()));
    assert_eq!(state.mode(), OmnibarMode::Inactive);
    assert_eq!(state.buffer(), "");
    assert_eq!(state.history().len(), 1);
    assert_eq!(state.history()[0], "test");
}

#[test]
fn test_submit_empty_returns_none() {
    let mut state = OmnibarState::new(10);
    state.activate();

    let result = state.submit();
    assert_eq!(result, None);
    assert_eq!(state.history().len(), 0);
}

#[test]
fn test_history_stores_commands() {
    let mut state = OmnibarState::new(10);

    // Submit first command
    state.activate();
    state.insert_char('c');
    state.insert_char('m');
    state.insert_char('d');
    state.insert_char('1');
    state.submit();

    // Submit second command
    state.activate();
    state.insert_char('c');
    state.insert_char('m');
    state.insert_char('d');
    state.insert_char('2');
    state.submit();

    assert_eq!(state.history().len(), 2);
    assert_eq!(state.history()[0], "cmd2"); // Most recent first
    assert_eq!(state.history()[1], "cmd1");
}

#[test]
fn test_history_respects_max_size() {
    let mut state = OmnibarState::new(3);

    // Add 5 commands
    for i in 1..=5 {
        state.activate();
        state.insert_char((b'0' + i) as char);
        state.submit();
    }

    // Should only keep last 3
    assert_eq!(state.history().len(), 3);
    assert_eq!(state.history()[0], "5");
    assert_eq!(state.history()[1], "4");
    assert_eq!(state.history()[2], "3");
}

#[test]
fn test_history_avoids_consecutive_duplicates() {
    let mut state = OmnibarState::new(10);

    state.activate();
    state.insert_char('t');
    state.insert_char('e');
    state.insert_char('s');
    state.insert_char('t');
    state.submit();

    state.activate();
    state.insert_char('t');
    state.insert_char('e');
    state.insert_char('s');
    state.insert_char('t');
    state.submit();

    // Should only have one entry
    assert_eq!(state.history().len(), 1);
    assert_eq!(state.history()[0], "test");
}

#[test]
fn test_history_prev_navigation() {
    let mut state = OmnibarState::new(10);

    // Add history
    state.activate();
    state.insert_char('a');
    state.submit();
    state.activate();
    state.insert_char('b');
    state.submit();
    state.activate();
    state.insert_char('c');
    state.submit();

    // Navigate history
    state.activate();
    state.insert_char('n');
    state.insert_char('e');
    state.insert_char('w');

    state.history_prev();
    assert_eq!(state.buffer(), "c");

    state.history_prev();
    assert_eq!(state.buffer(), "b");

    state.history_prev();
    assert_eq!(state.buffer(), "a");

    // Should not go beyond oldest
    state.history_prev();
    assert_eq!(state.buffer(), "a");
}

#[test]
fn test_history_next_navigation() {
    let mut state = OmnibarState::new(10);

    // Add history
    state.activate();
    state.insert_char('a');
    state.submit();
    state.activate();
    state.insert_char('b');
    state.submit();

    // Navigate to oldest
    state.activate();
    state.insert_char('n');
    state.insert_char('e');
    state.insert_char('w');

    state.history_prev();
    state.history_prev();
    assert_eq!(state.buffer(), "a");

    // Navigate back
    state.history_next();
    assert_eq!(state.buffer(), "b");

    state.history_next();
    assert_eq!(state.buffer(), "new"); // Restores original buffer
}

#[test]
fn test_history_navigation_preserves_temp_buffer() {
    let mut state = OmnibarState::new(10);

    state.activate();
    state.insert_char('h');
    state.insert_char('i');
    state.submit();

    state.activate();
    state.insert_char('t');
    state.insert_char('y');
    state.insert_char('p');
    state.insert_char('i');
    state.insert_char('n');
    state.insert_char('g');

    state.history_prev();
    assert_eq!(state.buffer(), "hi");

    state.history_next();
    assert_eq!(state.buffer(), "typing");
}

#[test]
fn test_clear_history() {
    let mut state = OmnibarState::new(10);

    state.activate();
    state.insert_char('a');
    state.submit();
    state.activate();
    state.insert_char('b');
    state.submit();

    assert_eq!(state.history().len(), 2);

    state.clear_history();
    assert_eq!(state.history().len(), 0);
}

#[test]
fn test_unicode_insertion() {
    let mut state = OmnibarState::new(10);
    state.activate();
    state.insert_char('🦀');
    state.insert_char('🚀');
    state.insert_char('✨');

    assert_eq!(state.buffer(), "🦀🚀✨");
    assert!(state.cursor() > 3); // Multi-byte characters
}

#[test]
fn test_unicode_deletion() {
    let mut state = OmnibarState::new(10);
    state.activate();
    state.insert_char('🦀');
    state.insert_char('🚀');

    state.delete_char();
    assert_eq!(state.buffer(), "🦀");

    state.delete_char();
    assert_eq!(state.buffer(), "");
}

#[test]
fn test_unicode_cursor_movement() {
    let mut state = OmnibarState::new(10);
    state.activate();
    state.insert_char('a');
    state.insert_char('🦀');
    state.insert_char('b');

    assert!(state.cursor() > 3);

    state.move_cursor_left();
    state.move_cursor_left();

    // Should be after 'a'
    let cursor_after_a = "a".len();
    assert_eq!(state.cursor(), cursor_after_a);
}

#[test]
fn test_insert_at_cursor_position() {
    let mut state = OmnibarState::new(10);
    state.activate();
    state.insert_char('a');
    state.insert_char('c');

    state.move_cursor_left(); // Between 'a' and 'c'
    state.insert_char('b');

    assert_eq!(state.buffer(), "abc");
}

#[test]
fn test_delete_at_middle_position() {
    let mut state = OmnibarState::new(10);
    state.activate();
    state.insert_char('a');
    state.insert_char('b');
    state.insert_char('c');

    state.move_cursor_left(); // Before 'c'
    state.delete_char(); // Delete 'b'

    assert_eq!(state.buffer(), "ac");
    assert_eq!(state.cursor(), 1);
}

#[test]
fn test_multiple_activations_reset_state() {
    let mut state = OmnibarState::new(10);

    state.activate();
    state.insert_char('t');
    state.insert_char('e');
    state.insert_char('s');
    state.insert_char('t');

    state.activate(); // Reactivate
    assert_eq!(state.buffer(), "");
    assert_eq!(state.cursor(), 0);
}
