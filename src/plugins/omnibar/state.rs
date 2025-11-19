//! State management for the Omnibar plugin.
//!
//! Handles input buffer, cursor position, mode tracking, and command history.

use std::time::Instant;

/// Current mode of the omnibar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OmnibarMode {
    /// Omnibar is not visible
    Inactive,
    /// Omnibar is visible and accepting input
    Input,
    /// Omnibar is filtering/matching commands (future feature)
    Filtered,
}

/// State for the Omnibar plugin.
///
/// Manages:
/// - Input buffer and cursor position
/// - Current mode (inactive, input, filtered)
/// - Command history
#[derive(Debug, Clone)]
pub struct OmnibarState {
    /// Current mode
    mode: OmnibarMode,

    /// Input buffer
    buffer: String,

    /// Cursor position in the buffer (byte offset)
    cursor: usize,

    /// Command history (most recent first)
    history: Vec<String>,

    /// Maximum history size
    max_history: usize,

    /// Current position in history navigation (None = not navigating)
    history_index: Option<usize>,

    /// Temporary buffer when navigating history
    temp_buffer: Option<String>,

    /// Temporary message to display (e.g., error, success)
    pub message: Option<(String, Instant)>,
}

impl OmnibarState {
    /// Creates a new omnibar state.
    pub fn new(max_history: usize) -> Self {
        Self {
            mode: OmnibarMode::Inactive,
            buffer: String::new(),
            cursor: 0,
            history: Vec::new(),
            max_history,
            history_index: None,
            temp_buffer: None,
            message: None,
        }
    }

    /// Returns the current mode.
    pub fn mode(&self) -> OmnibarMode {
        self.mode
    }

    /// Checks if the omnibar is active.
    pub fn is_active(&self) -> bool {
        self.mode != OmnibarMode::Inactive
    }

    /// Returns the current input buffer.
    pub fn buffer(&self) -> &str {
        &self.buffer
    }

    /// Returns the cursor position.
    pub fn cursor(&self) -> usize {
        self.cursor
    }

    /// Returns the command history.
    pub fn history(&self) -> &[String] {
        &self.history
    }

    /// Activates the omnibar in input mode.
    pub fn activate(&mut self) {
        self.mode = OmnibarMode::Input;
        self.buffer.clear();
        self.cursor = 0;
        self.history_index = None;
        self.temp_buffer = None;
    }

    /// Deactivates the omnibar and clears input.
    pub fn deactivate(&mut self) {
        self.mode = OmnibarMode::Inactive;
        self.buffer.clear();
        self.cursor = 0;
        self.history_index = None;
        self.temp_buffer = None;
    }

    /// Inserts a character at the cursor position.
    pub fn insert_char(&mut self, c: char) {
        // Exit history navigation mode
        if self.history_index.is_some() {
            self.history_index = None;
            self.temp_buffer = None;
        }

        self.buffer.insert(self.cursor, c);
        self.cursor += c.len_utf8();
    }

    /// Deletes the character before the cursor (backspace).
    pub fn delete_char(&mut self) {
        if self.cursor == 0 {
            return;
        }

        // Exit history navigation mode
        if self.history_index.is_some() {
            self.history_index = None;
            self.temp_buffer = None;
        }

        // Find the previous character boundary
        let prev_idx = self.buffer[..self.cursor]
            .char_indices()
            .next_back()
            .map(|(i, _)| i)
            .unwrap_or(0);

        self.buffer.remove(prev_idx);
        self.cursor = prev_idx;
    }

    /// Moves the cursor left by one character.
    pub fn move_cursor_left(&mut self) {
        if self.cursor > 0 {
            let prev_idx = self.buffer[..self.cursor]
                .char_indices()
                .next_back()
                .map(|(i, _)| i)
                .unwrap_or(0);
            self.cursor = prev_idx;
        }
    }

    /// Moves the cursor right by one character.
    pub fn move_cursor_right(&mut self) {
        if self.cursor < self.buffer.len() {
            let next_idx = self.buffer[self.cursor..]
                .char_indices()
                .nth(1)
                .map(|(i, _)| self.cursor + i)
                .unwrap_or(self.buffer.len());
            self.cursor = next_idx;
        }
    }

    /// Moves the cursor to the start of the buffer.
    pub fn move_cursor_home(&mut self) {
        self.cursor = 0;
    }

    /// Moves the cursor to the end of the buffer.
    pub fn move_cursor_end(&mut self) {
        self.cursor = self.buffer.len();
    }

    /// Submits the current input and adds it to history.
    ///
    /// Returns the submitted command if non-empty.
    pub fn submit(&mut self) -> Option<String> {
        if self.buffer.is_empty() {
            return None;
        }

        let command = self.buffer.clone();

        // Add to history (avoid duplicates at the front)
        if self.history.first() != Some(&command) {
            self.history.insert(0, command.clone());

            // Trim history to max size
            if self.history.len() > self.max_history {
                self.history.truncate(self.max_history);
            }
        }

        self.deactivate();
        Some(command)
    }

    /// Navigates to the previous command in history.
    pub fn history_prev(&mut self) {
        if self.history.is_empty() {
            return;
        }

        // Save current buffer on first history navigation
        if self.history_index.is_none() {
            self.temp_buffer = Some(self.buffer.clone());
            self.history_index = Some(0);
        } else if let Some(idx) = self.history_index {
            // Move to next older entry if available
            if idx < self.history.len() - 1 {
                self.history_index = Some(idx + 1);
            }
        }

        // Load history entry
        if let Some(idx) = self.history_index {
            if let Some(cmd) = self.history.get(idx) {
                self.buffer = cmd.clone();
                self.cursor = self.buffer.len();
            }
        }
    }

    /// Navigates to the next command in history.
    pub fn history_next(&mut self) {
        if self.history_index.is_none() {
            return;
        }

        if let Some(idx) = self.history_index {
            if idx == 0 {
                // Restore original buffer
                if let Some(temp) = self.temp_buffer.take() {
                    self.buffer = temp;
                    self.cursor = self.buffer.len();
                }
                self.history_index = None;
            } else {
                // Move to next newer entry
                self.history_index = Some(idx - 1);
                if let Some(cmd) = self.history.get(idx - 1) {
                    self.buffer = cmd.clone();
                    self.cursor = self.buffer.len();
                }
            }
        }
    }

    /// Clears the command history.
    pub fn clear_history(&mut self) {
        self.history.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_state() {
        let state = OmnibarState::new(10);
        assert_eq!(state.mode(), OmnibarMode::Inactive);
        assert!(!state.is_active());
        assert_eq!(state.buffer(), "");
        assert_eq!(state.cursor(), 0);
        assert!(state.history().is_empty());
    }

    #[test]
    fn test_activation() {
        let mut state = OmnibarState::new(10);
        state.activate();
        assert_eq!(state.mode(), OmnibarMode::Input);
        assert!(state.is_active());
    }

    #[test]
    fn test_deactivation() {
        let mut state = OmnibarState::new(10);
        state.activate();
        state.insert_char('a');
        state.deactivate();
        assert_eq!(state.mode(), OmnibarMode::Inactive);
        assert_eq!(state.buffer(), "");
    }

    #[test]
    fn test_insert_char() {
        let mut state = OmnibarState::new(10);
        state.activate();
        state.insert_char('h');
        state.insert_char('i');
        assert_eq!(state.buffer(), "hi");
        assert_eq!(state.cursor(), 2);
    }

    #[test]
    fn test_delete_char() {
        let mut state = OmnibarState::new(10);
        state.activate();
        state.insert_char('h');
        state.insert_char('i');
        state.delete_char();
        assert_eq!(state.buffer(), "h");
        assert_eq!(state.cursor(), 1);
        state.delete_char();
        assert_eq!(state.buffer(), "");
        assert_eq!(state.cursor(), 0);
    }

    #[test]
    fn test_cursor_movement() {
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

        state.move_cursor_right();
        assert_eq!(state.cursor(), 2);

        state.move_cursor_home();
        assert_eq!(state.cursor(), 0);

        state.move_cursor_end();
        assert_eq!(state.cursor(), 3);
    }

    #[test]
    fn test_submit() {
        let mut state = OmnibarState::new(10);
        state.activate();
        state.insert_char('t');
        state.insert_char('e');
        state.insert_char('s');
        state.insert_char('t');

        let result = state.submit();
        assert_eq!(result, Some("test".to_string()));
        assert_eq!(state.mode(), OmnibarMode::Inactive);
        assert_eq!(state.history().len(), 1);
        assert_eq!(state.history()[0], "test");
    }

    #[test]
    fn test_submit_empty() {
        let mut state = OmnibarState::new(10);
        state.activate();
        let result = state.submit();
        assert_eq!(result, None);
        assert!(state.history().is_empty());
    }

    #[test]
    fn test_history_navigation() {
        let mut state = OmnibarState::new(10);

        // Add some history
        state.activate();
        state.insert_char('c');
        state.insert_char('m');
        state.insert_char('d');
        state.insert_char('1');
        state.submit();

        state.activate();
        state.insert_char('c');
        state.insert_char('m');
        state.insert_char('d');
        state.insert_char('2');
        state.submit();

        // Navigate history
        state.activate();
        state.insert_char('n');
        state.insert_char('e');
        state.insert_char('w');

        state.history_prev();
        assert_eq!(state.buffer(), "cmd2");

        state.history_prev();
        assert_eq!(state.buffer(), "cmd1");

        state.history_next();
        assert_eq!(state.buffer(), "cmd2");

        state.history_next();
        assert_eq!(state.buffer(), "new");
    }

    #[test]
    fn test_history_max_size() {
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
    fn test_history_no_duplicates() {
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

        // Should only have one "test" entry
        assert_eq!(state.history().len(), 1);
        assert_eq!(state.history()[0], "test");
    }

    #[test]
    fn test_unicode_handling() {
        let mut state = OmnibarState::new(10);
        state.activate();
        state.insert_char('🦀');
        state.insert_char('🚀');
        assert_eq!(state.buffer(), "🦀🚀");
        assert!(state.cursor() > 2); // Multi-byte characters

        state.move_cursor_left();
        state.delete_char();
        assert_eq!(state.buffer(), "🚀");
    }
}
