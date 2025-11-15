//! Tooltip plugin configuration.
//!
//! This module defines configuration options for the tooltip plugin,
//! including activation behavior, display timing, and visual styling.

/// Configuration for the tooltip plugin.
///
/// Controls tooltip activation, timing, sizing, and positioning behavior.
///
/// # Examples
///
/// ```rust
/// use locust::plugins::tooltip::TooltipConfig;
///
/// // Default configuration
/// let config = TooltipConfig::default();
///
/// // Custom configuration
/// let config = TooltipConfig::new()
///     .with_activation_key('h')
///     .with_hover_delay_ms(500)
///     .with_auto_hide_timeout_ms(5000)
///     .with_max_width(60);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TooltipConfig {
    /// Key to activate tooltip when target is focused (None = hover only).
    ///
    /// If set, tooltips can be activated by pressing this key when a target
    /// is focused. If None, tooltips only activate on hover (mouse support required).
    pub activation_key: Option<char>,

    /// Delay in milliseconds before showing tooltip on hover.
    ///
    /// This prevents tooltips from appearing immediately when the cursor
    /// moves over a target. Default: 300ms.
    pub hover_delay_ms: u64,

    /// Timeout in milliseconds to auto-hide tooltip (0 = disabled).
    ///
    /// If > 0, tooltips will automatically hide after this duration.
    /// If 0, tooltips remain visible until dismissed. Default: 0 (disabled).
    pub auto_hide_timeout_ms: u64,

    /// Maximum width for tooltip content (in characters).
    ///
    /// Long lines will be wrapped or truncated to fit this width.
    /// Default: 50.
    pub max_width: u16,

    /// Maximum height for tooltip content (in lines).
    ///
    /// If content exceeds this, it will be truncated with "..." indicator.
    /// Default: 10.
    pub max_height: u16,

    /// Horizontal offset from target (in cells).
    ///
    /// Positive values move tooltip right, negative values move left.
    /// Default: 1.
    pub offset_x: i16,

    /// Vertical offset from target (in cells).
    ///
    /// Positive values move tooltip down, negative values move up.
    /// Default: 1.
    pub offset_y: i16,

    /// Padding inside tooltip border (in cells).
    ///
    /// Space between border and content. Default: 1.
    pub padding: u16,

    /// Whether to show a border around tooltips.
    ///
    /// Default: true.
    pub show_border: bool,

    /// Whether to prefer positioning tooltip on the right side of target.
    ///
    /// If true, tries to place tooltip to the right, falling back to left
    /// if there's no space. Default: true.
    pub prefer_right: bool,

    /// Whether to prefer positioning tooltip below the target.
    ///
    /// If true, tries to place tooltip below, falling back to above
    /// if there's no space. Default: true.
    pub prefer_bottom: bool,

    /// Whether to show an arrow pointing to the target.
    ///
    /// If true, renders a small arrow (▸, ◂, ▴, ▾) pointing at the target.
    /// Default: true.
    pub show_arrow: bool,
}

impl Default for TooltipConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl TooltipConfig {
    /// Creates a new tooltip configuration with sensible defaults.
    pub fn new() -> Self {
        Self {
            activation_key: Some('h'),
            hover_delay_ms: 300,
            auto_hide_timeout_ms: 0, // Disabled by default
            max_width: 50,
            max_height: 10,
            offset_x: 1,
            offset_y: 1,
            padding: 1,
            show_border: true,
            prefer_right: true,
            prefer_bottom: true,
            show_arrow: true,
        }
    }

    /// Sets the activation key.
    pub fn with_activation_key(mut self, key: char) -> Self {
        self.activation_key = Some(key);
        self
    }

    /// Disables keyboard activation (hover only).
    pub fn hover_only(mut self) -> Self {
        self.activation_key = None;
        self
    }

    /// Sets the hover delay in milliseconds.
    pub fn with_hover_delay_ms(mut self, delay: u64) -> Self {
        self.hover_delay_ms = delay;
        self
    }

    /// Sets the auto-hide timeout in milliseconds (0 to disable).
    pub fn with_auto_hide_timeout_ms(mut self, timeout: u64) -> Self {
        self.auto_hide_timeout_ms = timeout;
        self
    }

    /// Sets the maximum tooltip width.
    pub fn with_max_width(mut self, width: u16) -> Self {
        self.max_width = width;
        self
    }

    /// Sets the maximum tooltip height.
    pub fn with_max_height(mut self, height: u16) -> Self {
        self.max_height = height;
        self
    }

    /// Sets the horizontal offset from the target.
    pub fn with_offset_x(mut self, offset: i16) -> Self {
        self.offset_x = offset;
        self
    }

    /// Sets the vertical offset from the target.
    pub fn with_offset_y(mut self, offset: i16) -> Self {
        self.offset_y = offset;
        self
    }

    /// Sets the padding inside the tooltip.
    pub fn with_padding(mut self, padding: u16) -> Self {
        self.padding = padding;
        self
    }

    /// Sets whether to show a border.
    pub fn with_border(mut self, show: bool) -> Self {
        self.show_border = show;
        self
    }

    /// Sets whether to prefer right positioning.
    pub fn prefer_right(mut self, prefer: bool) -> Self {
        self.prefer_right = prefer;
        self
    }

    /// Sets whether to prefer bottom positioning.
    pub fn prefer_bottom(mut self, prefer: bool) -> Self {
        self.prefer_bottom = prefer;
        self
    }

    /// Sets whether to show an arrow pointing to the target.
    pub fn with_arrow(mut self, show: bool) -> Self {
        self.show_arrow = show;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = TooltipConfig::default();
        assert_eq!(config.activation_key, Some('h'));
        assert_eq!(config.hover_delay_ms, 300);
        assert_eq!(config.auto_hide_timeout_ms, 0);
        assert_eq!(config.max_width, 50);
        assert_eq!(config.max_height, 10);
        assert!(config.show_border);
        assert!(config.show_arrow);
    }

    #[test]
    fn test_custom_config() {
        let config = TooltipConfig::new()
            .with_activation_key('?')
            .with_hover_delay_ms(500)
            .with_auto_hide_timeout_ms(3000)
            .with_max_width(80)
            .with_border(false);

        assert_eq!(config.activation_key, Some('?'));
        assert_eq!(config.hover_delay_ms, 500);
        assert_eq!(config.auto_hide_timeout_ms, 3000);
        assert_eq!(config.max_width, 80);
        assert!(!config.show_border);
    }

    #[test]
    fn test_hover_only() {
        let config = TooltipConfig::new().hover_only();
        assert_eq!(config.activation_key, None);
    }

    #[test]
    fn test_positioning_preferences() {
        let config = TooltipConfig::new()
            .prefer_right(false)
            .prefer_bottom(false);
        assert!(!config.prefer_right);
        assert!(!config.prefer_bottom);
    }
}
