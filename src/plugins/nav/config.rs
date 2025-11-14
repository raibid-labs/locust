//! Configuration for the navigation plugin.
//!
//! This module defines configuration options for hint generation,
//! rendering styles, and keybindings.

use ratatui::style::{Color, Modifier, Style};

/// Configuration for the navigation plugin.
///
/// Controls hint generation algorithm, visual styling, and keybindings.
#[derive(Debug, Clone)]
pub struct NavConfig {
    /// The key that activates hint mode.
    /// Default: 'f' (like Vimium)
    pub hint_key: char,

    /// Character set used for generating hints.
    /// Should be ordered from most convenient to least convenient.
    /// Default: "asdfghjkl" (home row keys)
    pub hint_charset: String,

    /// Style for hint overlays (background box).
    pub hint_background_style: Style,

    /// Style for hint text (unmatched characters).
    pub hint_text_style: Style,

    /// Style for matched hint characters.
    pub hint_matched_style: Style,

    /// Style for dimmed hints (when other hints are being matched).
    pub hint_dimmed_style: Style,

    /// Whether to show hint labels when no targets are visible.
    pub show_empty_hints: bool,

    /// Minimum target area (width * height) to receive a hint.
    /// Targets smaller than this are ignored.
    pub min_target_area: u32,

    /// Maximum number of hints to generate.
    /// Set to 0 for unlimited.
    pub max_hints: usize,
}

impl Default for NavConfig {
    fn default() -> Self {
        Self {
            hint_key: 'f',
            hint_charset: "asdfghjkl".to_string(),
            hint_background_style: Style::default()
                .bg(Color::Yellow)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
            hint_text_style: Style::default()
                .bg(Color::Yellow)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
            hint_matched_style: Style::default()
                .bg(Color::Green)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
            hint_dimmed_style: Style::default()
                .bg(Color::Gray)
                .fg(Color::DarkGray)
                .add_modifier(Modifier::DIM),
            show_empty_hints: false,
            min_target_area: 1,
            max_hints: 0,
        }
    }
}

impl NavConfig {
    /// Creates a new navigation configuration with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the hint activation key.
    pub fn with_hint_key(mut self, key: char) -> Self {
        self.hint_key = key;
        self
    }

    /// Sets the hint character set.
    ///
    /// # Panics
    ///
    /// Panics if the charset is empty.
    pub fn with_charset(mut self, charset: impl Into<String>) -> Self {
        let charset = charset.into();
        assert!(!charset.is_empty(), "Hint charset cannot be empty");
        self.hint_charset = charset;
        self
    }

    /// Sets the hint background style.
    pub fn with_background_style(mut self, style: Style) -> Self {
        self.hint_background_style = style;
        self
    }

    /// Sets the hint text style.
    pub fn with_text_style(mut self, style: Style) -> Self {
        self.hint_text_style = style;
        self
    }

    /// Sets the matched character style.
    pub fn with_matched_style(mut self, style: Style) -> Self {
        self.hint_matched_style = style;
        self
    }

    /// Sets the dimmed hint style.
    pub fn with_dimmed_style(mut self, style: Style) -> Self {
        self.hint_dimmed_style = style;
        self
    }

    /// Sets whether to show hints when no targets are visible.
    pub fn with_show_empty_hints(mut self, show: bool) -> Self {
        self.show_empty_hints = show;
        self
    }

    /// Sets the minimum target area for hint generation.
    pub fn with_min_target_area(mut self, area: u32) -> Self {
        self.min_target_area = area;
        self
    }

    /// Sets the maximum number of hints to generate.
    pub fn with_max_hints(mut self, max: usize) -> Self {
        self.max_hints = max;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = NavConfig::default();
        assert_eq!(config.hint_key, 'f');
        assert_eq!(config.hint_charset, "asdfghjkl");
        assert_eq!(config.min_target_area, 1);
        assert_eq!(config.max_hints, 0);
        assert!(!config.show_empty_hints);
    }

    #[test]
    fn test_config_builder() {
        let config = NavConfig::new()
            .with_hint_key('g')
            .with_charset("abcdef")
            .with_min_target_area(10)
            .with_max_hints(50)
            .with_show_empty_hints(true);

        assert_eq!(config.hint_key, 'g');
        assert_eq!(config.hint_charset, "abcdef");
        assert_eq!(config.min_target_area, 10);
        assert_eq!(config.max_hints, 50);
        assert!(config.show_empty_hints);
    }

    #[test]
    #[should_panic(expected = "Hint charset cannot be empty")]
    fn test_empty_charset_panics() {
        NavConfig::new().with_charset("");
    }
}
