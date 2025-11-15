//! Tooltip content types and styling.
//!
//! This module defines the content structure and visual styles for tooltips,
//! supporting rich text with titles, bodies, and multiple style variants.

use ratatui::style::{Color, Modifier, Style};

/// Visual style variant for tooltips.
///
/// Each variant provides different visual styling to convey
/// the semantic meaning of the tooltip content.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TooltipStyle {
    /// Informational tooltip (blue/neutral theme).
    Info,

    /// Warning tooltip (yellow/amber theme).
    Warning,

    /// Error tooltip (red theme).
    Error,

    /// Success tooltip (green theme).
    Success,
}

impl Default for TooltipStyle {
    fn default() -> Self {
        Self::Info
    }
}

impl TooltipStyle {
    /// Returns the background color for this style.
    pub fn bg_color(&self) -> Color {
        match self {
            Self::Info => Color::Rgb(30, 58, 138),    // Dark blue
            Self::Warning => Color::Rgb(146, 64, 14), // Dark amber
            Self::Error => Color::Rgb(127, 29, 29),   // Dark red
            Self::Success => Color::Rgb(20, 83, 45),  // Dark green
        }
    }

    /// Returns the foreground (text) color for this style.
    pub fn fg_color(&self) -> Color {
        match self {
            Self::Info => Color::Rgb(219, 234, 254),    // Light blue
            Self::Warning => Color::Rgb(254, 243, 199), // Light amber
            Self::Error => Color::Rgb(254, 226, 226),   // Light red
            Self::Success => Color::Rgb(220, 252, 231), // Light green
        }
    }

    /// Returns the border color for this style.
    pub fn border_color(&self) -> Color {
        match self {
            Self::Info => Color::Rgb(59, 130, 246),    // Blue
            Self::Warning => Color::Rgb(251, 191, 36), // Amber
            Self::Error => Color::Rgb(239, 68, 68),    // Red
            Self::Success => Color::Rgb(34, 197, 94),  // Green
        }
    }

    /// Returns the title style for this tooltip style.
    pub fn title_style(&self) -> Style {
        Style::default()
            .fg(self.fg_color())
            .bg(self.bg_color())
            .add_modifier(Modifier::BOLD)
    }

    /// Returns the body style for this tooltip style.
    pub fn body_style(&self) -> Style {
        Style::default().fg(self.fg_color()).bg(self.bg_color())
    }

    /// Returns the border style for this tooltip style.
    pub fn border_style(&self) -> Style {
        Style::default().fg(self.border_color()).bg(self.bg_color())
    }
}

/// Content of a tooltip with optional title and styled body.
///
/// Tooltips can contain:
/// - Optional title (rendered in bold)
/// - Multi-line body text
/// - Visual style variant (Info, Warning, Error, Success)
///
/// # Examples
///
/// ```rust
/// use locust::plugins::tooltip::{TooltipContent, TooltipStyle};
///
/// // Simple tooltip
/// let tip = TooltipContent::new("Press 'f' to activate navigation hints");
///
/// // Tooltip with title
/// let tip = TooltipContent::new("Navigate to any visible target quickly")
///     .with_title("Navigation Mode");
///
/// // Warning tooltip
/// let tip = TooltipContent::new("This action cannot be undone")
///     .with_title("Warning")
///     .with_style(TooltipStyle::Warning);
///
/// // Multi-line tooltip
/// let tip = TooltipContent::new("Line 1\nLine 2\nLine 3")
///     .with_style(TooltipStyle::Info);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TooltipContent {
    /// Optional title text (displayed in bold).
    pub title: Option<String>,

    /// Main body text (can be multi-line with \n).
    pub body: String,

    /// Visual style variant.
    pub style: TooltipStyle,
}

impl TooltipContent {
    /// Creates a new tooltip with the given body text.
    ///
    /// The tooltip will use the default Info style with no title.
    pub fn new(body: impl Into<String>) -> Self {
        Self {
            title: None,
            body: body.into(),
            style: TooltipStyle::default(),
        }
    }

    /// Sets the title for this tooltip.
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Sets the visual style for this tooltip.
    pub fn with_style(mut self, style: TooltipStyle) -> Self {
        self.style = style;
        self
    }

    /// Returns the total number of lines in this tooltip.
    ///
    /// This includes the title (if present) and all body lines.
    pub fn line_count(&self) -> usize {
        let title_lines = if self.title.is_some() { 1 } else { 0 };
        let body_lines = self.body.lines().count().max(1);
        title_lines + body_lines
    }

    /// Returns the maximum line width in this tooltip.
    ///
    /// Used for calculating the tooltip dimensions.
    pub fn max_line_width(&self) -> usize {
        let title_width = self.title.as_ref().map(|t| t.len()).unwrap_or(0);
        let body_width = self.body.lines().map(|line| line.len()).max().unwrap_or(0);
        title_width.max(body_width)
    }

    /// Splits the body into individual lines for rendering.
    pub fn body_lines(&self) -> Vec<&str> {
        self.body.lines().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tooltip_content_creation() {
        let content = TooltipContent::new("Test tooltip");
        assert_eq!(content.body, "Test tooltip");
        assert_eq!(content.title, None);
        assert_eq!(content.style, TooltipStyle::Info);
    }

    #[test]
    fn test_tooltip_with_title() {
        let content = TooltipContent::new("Body text").with_title("Title");
        assert_eq!(content.title, Some("Title".to_string()));
        assert_eq!(content.body, "Body text");
    }

    #[test]
    fn test_tooltip_with_style() {
        let content = TooltipContent::new("Warning!").with_style(TooltipStyle::Warning);
        assert_eq!(content.style, TooltipStyle::Warning);
    }

    #[test]
    fn test_line_count() {
        let content = TooltipContent::new("Line 1\nLine 2\nLine 3");
        assert_eq!(content.line_count(), 3);

        let content = TooltipContent::new("Single line").with_title("Title");
        assert_eq!(content.line_count(), 2);

        let content = TooltipContent::new("");
        assert_eq!(content.line_count(), 1); // Empty body counts as 1 line
    }

    #[test]
    fn test_max_line_width() {
        let content = TooltipContent::new("Short\nMedium line\nVery long line here");
        assert_eq!(content.max_line_width(), 19); // "Very long line here"

        let content = TooltipContent::new("Body").with_title("Very long title");
        assert_eq!(content.max_line_width(), 15); // "Very long title"
    }

    #[test]
    fn test_body_lines() {
        let content = TooltipContent::new("Line 1\nLine 2\nLine 3");
        assert_eq!(content.body_lines(), vec!["Line 1", "Line 2", "Line 3"]);

        let content = TooltipContent::new("Single line");
        assert_eq!(content.body_lines(), vec!["Single line"]);
    }

    #[test]
    fn test_tooltip_style_colors() {
        assert_eq!(TooltipStyle::Info.bg_color(), Color::Rgb(30, 58, 138));
        assert_eq!(TooltipStyle::Warning.bg_color(), Color::Rgb(146, 64, 14));
        assert_eq!(TooltipStyle::Error.bg_color(), Color::Rgb(127, 29, 29));
        assert_eq!(TooltipStyle::Success.bg_color(), Color::Rgb(20, 83, 45));
    }
}
