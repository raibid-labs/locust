//! Configuration for the highlight plugin.

use ratatui::style::{Color, Modifier, Style};

/// Border style for the highlight cutout.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HighlightBorderStyle {
    /// No border around highlight
    None,
    /// Simple single-line border
    Single,
    /// Double-line border
    Double,
    /// Thick border
    Thick,
    /// Rounded corners border
    Rounded,
}

impl Default for HighlightBorderStyle {
    fn default() -> Self {
        Self::Rounded
    }
}

/// Animation type for the highlight effect.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HighlightAnimation {
    /// No animation
    None,
    /// Pulsing glow effect
    Pulse,
    /// Shimmer effect around border
    Shimmer,
    /// Breathing effect (scale in/out)
    Breathe,
}

impl Default for HighlightAnimation {
    fn default() -> Self {
        Self::Pulse
    }
}

/// Configuration for the highlight plugin.
#[derive(Debug, Clone)]
pub struct HighlightConfig {
    /// Activation key to start a tour (default: '?')
    pub activation_key: char,

    /// Opacity of the dim overlay (0.0 = transparent, 1.0 = opaque)
    /// Represented as a u8 from 0-255 for simplicity
    pub dim_opacity: u8,

    /// Color of the dim overlay
    pub dim_color: Color,

    /// Border style for highlighted area
    pub border_style: HighlightBorderStyle,

    /// Border color
    pub border_color: Color,

    /// Highlight padding (extra space around highlighted area)
    pub highlight_padding: u16,

    /// Animation type
    pub animation: HighlightAnimation,

    /// Animation speed in milliseconds per frame
    pub animation_speed_ms: u64,

    /// Message box background color
    pub message_bg_color: Color,

    /// Message box text color
    pub message_text_color: Color,

    /// Message box border color
    pub message_border_color: Color,

    /// Message title style
    pub message_title_style: Style,

    /// Message body style
    pub message_body_style: Style,

    /// Step indicator style (e.g., "2/5")
    pub step_indicator_style: Style,

    /// Maximum width of message box as percentage of screen width
    pub message_max_width_percent: u16,

    /// Whether to show navigation hints (Next, Previous, Skip)
    pub show_navigation_hints: bool,

    /// Navigation hints style
    pub navigation_hints_style: Style,

    /// Z-index for overlay rendering (higher = on top)
    pub z_index: i32,

    /// Whether to auto-save tour progress
    pub save_progress: bool,
}

impl Default for HighlightConfig {
    fn default() -> Self {
        Self {
            activation_key: '?',
            dim_opacity: 180, // ~70% opacity
            dim_color: Color::Black,
            border_style: HighlightBorderStyle::default(),
            border_color: Color::Yellow,
            highlight_padding: 1,
            animation: HighlightAnimation::default(),
            animation_speed_ms: 500,
            message_bg_color: Color::Rgb(30, 30, 40),
            message_text_color: Color::White,
            message_border_color: Color::Cyan,
            message_title_style: Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
            message_body_style: Style::default().fg(Color::White),
            step_indicator_style: Style::default().fg(Color::Gray).add_modifier(Modifier::DIM),
            message_max_width_percent: 60,
            show_navigation_hints: true,
            navigation_hints_style: Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::ITALIC),
            z_index: 250, // Modal-level overlay
            save_progress: true,
        }
    }
}

impl HighlightConfig {
    /// Creates a new configuration with defaults.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the activation key.
    pub fn with_activation_key(mut self, key: char) -> Self {
        self.activation_key = key;
        self
    }

    /// Sets the dim overlay opacity (0-255).
    pub fn with_dim_opacity(mut self, opacity: u8) -> Self {
        self.dim_opacity = opacity;
        self
    }

    /// Sets the dim overlay color.
    pub fn with_dim_color(mut self, color: Color) -> Self {
        self.dim_color = color;
        self
    }

    /// Sets the border style.
    pub fn with_border_style(mut self, style: HighlightBorderStyle) -> Self {
        self.border_style = style;
        self
    }

    /// Sets the border color.
    pub fn with_border_color(mut self, color: Color) -> Self {
        self.border_color = color;
        self
    }

    /// Sets the highlight padding.
    pub fn with_highlight_padding(mut self, padding: u16) -> Self {
        self.highlight_padding = padding;
        self
    }

    /// Sets the animation type.
    pub fn with_animation(mut self, animation: HighlightAnimation) -> Self {
        self.animation = animation;
        self
    }

    /// Sets the animation speed in milliseconds.
    pub fn with_animation_speed(mut self, ms: u64) -> Self {
        self.animation_speed_ms = ms;
        self
    }

    /// Sets the message box background color.
    pub fn with_message_bg_color(mut self, color: Color) -> Self {
        self.message_bg_color = color;
        self
    }

    /// Sets the message text color.
    pub fn with_message_text_color(mut self, color: Color) -> Self {
        self.message_text_color = color;
        self
    }

    /// Sets whether to show navigation hints.
    pub fn with_navigation_hints(mut self, show: bool) -> Self {
        self.show_navigation_hints = show;
        self
    }

    /// Sets the z-index for overlay rendering.
    pub fn with_z_index(mut self, z_index: i32) -> Self {
        self.z_index = z_index;
        self
    }

    /// Sets whether to save tour progress.
    pub fn with_save_progress(mut self, save: bool) -> Self {
        self.save_progress = save;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = HighlightConfig::default();
        assert_eq!(config.activation_key, '?');
        assert_eq!(config.dim_opacity, 180);
        assert_eq!(config.border_style, HighlightBorderStyle::Rounded);
        assert!(config.show_navigation_hints);
        assert!(config.save_progress);
    }

    #[test]
    fn test_config_builder() {
        let config = HighlightConfig::new()
            .with_activation_key('h')
            .with_dim_opacity(200)
            .with_border_style(HighlightBorderStyle::Double)
            .with_navigation_hints(false)
            .with_z_index(300);

        assert_eq!(config.activation_key, 'h');
        assert_eq!(config.dim_opacity, 200);
        assert_eq!(config.border_style, HighlightBorderStyle::Double);
        assert!(!config.show_navigation_hints);
        assert_eq!(config.z_index, 300);
    }

    #[test]
    fn test_border_styles() {
        assert_eq!(
            HighlightBorderStyle::default(),
            HighlightBorderStyle::Rounded
        );
    }

    #[test]
    fn test_animation_types() {
        assert_eq!(HighlightAnimation::default(), HighlightAnimation::Pulse);
    }
}
