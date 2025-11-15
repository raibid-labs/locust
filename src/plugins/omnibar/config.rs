//! Configuration for the Omnibar plugin.
//!
//! Provides customization options for appearance, behavior, and key bindings.

use ratatui::style::{Color, Modifier, Style};

/// Configuration for the Omnibar plugin.
///
/// # Example
///
/// ```rust
/// use locust::plugins::omnibar::OmnibarConfig;
///
/// let config = OmnibarConfig::new()
///     .with_activation_key('/')
///     .with_max_width(80)
///     .with_placeholder("Type a command...");
/// ```
#[derive(Debug, Clone)]
pub struct OmnibarConfig {
    /// Key that activates the omnibar (default: '/')
    pub activation_key: char,

    /// Maximum width of the omnibar popup as percentage of screen width (0-100)
    pub max_width_percent: u16,

    /// Maximum height of the omnibar popup in lines
    pub max_height: u16,

    /// Placeholder text shown when input is empty
    pub placeholder_text: String,

    /// Maximum number of commands to keep in history
    pub max_history: usize,

    /// Style for the popup border
    pub border_style: Style,

    /// Style for the title
    pub title_style: Style,

    /// Style for the input text
    pub input_style: Style,

    /// Style for the placeholder text
    pub placeholder_style: Style,

    /// Style for the cursor
    pub cursor_style: Style,

    /// Border type (can be extended with ratatui::widgets::BorderType)
    pub border_type: BorderType,
}

/// Border style for the omnibar popup
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BorderType {
    /// Plain border (default)
    Plain,
    /// Rounded corners
    Rounded,
    /// Double-line border
    Double,
    /// Thick border
    Thick,
}

impl Default for OmnibarConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl OmnibarConfig {
    /// Creates a new configuration with sensible defaults.
    pub fn new() -> Self {
        Self {
            activation_key: '/',
            max_width_percent: 60,
            max_height: 3,
            placeholder_text: "Type a command...".to_string(),
            max_history: 10,
            border_style: Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
            title_style: Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
            input_style: Style::default().fg(Color::White),
            placeholder_style: Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::ITALIC),
            cursor_style: Style::default().bg(Color::White).fg(Color::Black),
            border_type: BorderType::Rounded,
        }
    }

    /// Sets the activation key.
    pub fn with_activation_key(mut self, key: char) -> Self {
        self.activation_key = key;
        self
    }

    /// Sets the maximum width as a percentage (0-100).
    pub fn with_max_width(mut self, percent: u16) -> Self {
        self.max_width_percent = percent.min(100);
        self
    }

    /// Sets the maximum height in lines.
    pub fn with_max_height(mut self, height: u16) -> Self {
        self.max_height = height;
        self
    }

    /// Sets the placeholder text.
    pub fn with_placeholder(mut self, text: impl Into<String>) -> Self {
        self.placeholder_text = text.into();
        self
    }

    /// Sets the maximum history size.
    pub fn with_max_history(mut self, size: usize) -> Self {
        self.max_history = size;
        self
    }

    /// Sets the border style.
    pub fn with_border_style(mut self, style: Style) -> Self {
        self.border_style = style;
        self
    }

    /// Sets the title style.
    pub fn with_title_style(mut self, style: Style) -> Self {
        self.title_style = style;
        self
    }

    /// Sets the input text style.
    pub fn with_input_style(mut self, style: Style) -> Self {
        self.input_style = style;
        self
    }

    /// Sets the placeholder text style.
    pub fn with_placeholder_style(mut self, style: Style) -> Self {
        self.placeholder_style = style;
        self
    }

    /// Sets the cursor style.
    pub fn with_cursor_style(mut self, style: Style) -> Self {
        self.cursor_style = style;
        self
    }

    /// Sets the border type.
    pub fn with_border_type(mut self, border_type: BorderType) -> Self {
        self.border_type = border_type;
        self
    }
}

impl BorderType {
    /// Converts to ratatui's BorderType
    pub fn to_ratatui_border(&self) -> ratatui::widgets::BorderType {
        match self {
            BorderType::Plain => ratatui::widgets::BorderType::Plain,
            BorderType::Rounded => ratatui::widgets::BorderType::Rounded,
            BorderType::Double => ratatui::widgets::BorderType::Double,
            BorderType::Thick => ratatui::widgets::BorderType::Thick,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = OmnibarConfig::default();
        assert_eq!(config.activation_key, '/');
        assert_eq!(config.max_width_percent, 60);
        assert_eq!(config.max_height, 3);
        assert_eq!(config.max_history, 10);
    }

    #[test]
    fn test_config_builder() {
        let config = OmnibarConfig::new()
            .with_activation_key(':')
            .with_max_width(80)
            .with_max_height(5)
            .with_placeholder("Enter command")
            .with_max_history(20);

        assert_eq!(config.activation_key, ':');
        assert_eq!(config.max_width_percent, 80);
        assert_eq!(config.max_height, 5);
        assert_eq!(config.placeholder_text, "Enter command");
        assert_eq!(config.max_history, 20);
    }

    #[test]
    fn test_max_width_clamping() {
        let config = OmnibarConfig::new().with_max_width(150);
        assert_eq!(config.max_width_percent, 100);
    }
}
