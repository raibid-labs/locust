use ratatui::style::{Color, Modifier, Style};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, thiserror::Error)]
pub enum ThemeError {
    #[error("Failed to load theme: {0}")]
    LoadError(String),
    #[error("Failed to parse theme: {0}")]
    ParseError(String),
    #[error("Theme not found: {0}")]
    NotFound(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    pub name: String,
    pub description: String,
    pub colors: ColorScheme,
    pub styles: StyleScheme,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorScheme {
    // Base colors
    pub background: ColorDef,
    pub foreground: ColorDef,
    pub primary: ColorDef,
    pub secondary: ColorDef,
    pub accent: ColorDef,

    // Semantic colors
    pub success: ColorDef,
    pub warning: ColorDef,
    pub error: ColorDef,
    pub info: ColorDef,

    // UI element colors
    pub border: ColorDef,
    pub border_focused: ColorDef,
    pub selection: ColorDef,
    pub highlight: ColorDef,

    // Text colors
    pub text_normal: ColorDef,
    pub text_muted: ColorDef,
    pub text_emphasis: ColorDef,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ColorDef {
    Named(String),
    Rgb { r: u8, g: u8, b: u8 },
}

impl ColorDef {
    pub fn to_color(&self) -> Color {
        match self {
            ColorDef::Named(name) => match name.to_lowercase().as_str() {
                "black" => Color::Black,
                "red" => Color::Red,
                "green" => Color::Green,
                "yellow" => Color::Yellow,
                "blue" => Color::Blue,
                "magenta" => Color::Magenta,
                "cyan" => Color::Cyan,
                "gray" | "grey" => Color::Gray,
                "darkgray" | "darkgrey" => Color::DarkGray,
                "lightred" => Color::LightRed,
                "lightgreen" => Color::LightGreen,
                "lightyellow" => Color::LightYellow,
                "lightblue" => Color::LightBlue,
                "lightmagenta" => Color::LightMagenta,
                "lightcyan" => Color::LightCyan,
                "white" => Color::White,
                _ => Color::Reset,
            },
            ColorDef::Rgb { r, g, b } => Color::Rgb(*r, *g, *b),
        }
    }
}

impl From<Color> for ColorDef {
    fn from(color: Color) -> Self {
        match color {
            Color::Rgb(r, g, b) => ColorDef::Rgb { r, g, b },
            Color::Black => ColorDef::Named("black".to_string()),
            Color::Red => ColorDef::Named("red".to_string()),
            Color::Green => ColorDef::Named("green".to_string()),
            Color::Yellow => ColorDef::Named("yellow".to_string()),
            Color::Blue => ColorDef::Named("blue".to_string()),
            Color::Magenta => ColorDef::Named("magenta".to_string()),
            Color::Cyan => ColorDef::Named("cyan".to_string()),
            Color::Gray => ColorDef::Named("gray".to_string()),
            Color::DarkGray => ColorDef::Named("darkgray".to_string()),
            Color::LightRed => ColorDef::Named("lightred".to_string()),
            Color::LightGreen => ColorDef::Named("lightgreen".to_string()),
            Color::LightYellow => ColorDef::Named("lightyellow".to_string()),
            Color::LightBlue => ColorDef::Named("lightblue".to_string()),
            Color::LightMagenta => ColorDef::Named("lightmagenta".to_string()),
            Color::LightCyan => ColorDef::Named("lightcyan".to_string()),
            Color::White => ColorDef::Named("white".to_string()),
            _ => ColorDef::Named("reset".to_string()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StyleScheme {
    #[serde(default)]
    pub normal: StyleDef,
    #[serde(default)]
    pub focused: StyleDef,
    #[serde(default)]
    pub selected: StyleDef,
    #[serde(default)]
    pub disabled: StyleDef,
    #[serde(default)]
    pub hint: StyleDef,
    #[serde(default)]
    pub hint_matched: StyleDef,
    #[serde(default)]
    pub tooltip: StyleDef,
    #[serde(default)]
    pub highlight_border: StyleDef,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StyleDef {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fg: Option<ColorDef>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bg: Option<ColorDef>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub modifiers: Vec<String>,
}

impl StyleDef {
    pub fn to_style(&self) -> Style {
        let mut style = Style::default();

        if let Some(ref fg) = self.fg {
            style = style.fg(fg.to_color());
        }

        if let Some(ref bg) = self.bg {
            style = style.bg(bg.to_color());
        }

        for modifier_name in &self.modifiers {
            let modifier = match modifier_name.to_lowercase().as_str() {
                "bold" => Modifier::BOLD,
                "dim" => Modifier::DIM,
                "italic" => Modifier::ITALIC,
                "underlined" => Modifier::UNDERLINED,
                "slowblink" => Modifier::SLOW_BLINK,
                "rapidblink" => Modifier::RAPID_BLINK,
                "reversed" => Modifier::REVERSED,
                "hidden" => Modifier::HIDDEN,
                "crossedout" => Modifier::CROSSED_OUT,
                _ => continue,
            };
            style = style.add_modifier(modifier);
        }

        style
    }
}

impl Theme {
    pub fn default_dark() -> Self {
        Theme {
            name: "Dark".to_string(),
            description: "Default dark theme with blue accents".to_string(),
            colors: ColorScheme {
                background: ColorDef::Rgb {
                    r: 30,
                    g: 30,
                    b: 30,
                },
                foreground: ColorDef::Rgb {
                    r: 220,
                    g: 220,
                    b: 220,
                },
                primary: ColorDef::Rgb {
                    r: 100,
                    g: 150,
                    b: 255,
                },
                secondary: ColorDef::Rgb {
                    r: 150,
                    g: 100,
                    b: 255,
                },
                accent: ColorDef::Rgb {
                    r: 255,
                    g: 200,
                    b: 100,
                },
                success: ColorDef::Rgb {
                    r: 100,
                    g: 255,
                    b: 100,
                },
                warning: ColorDef::Rgb {
                    r: 255,
                    g: 200,
                    b: 100,
                },
                error: ColorDef::Rgb {
                    r: 255,
                    g: 100,
                    b: 100,
                },
                info: ColorDef::Rgb {
                    r: 100,
                    g: 200,
                    b: 255,
                },
                border: ColorDef::Rgb {
                    r: 80,
                    g: 80,
                    b: 80,
                },
                border_focused: ColorDef::Rgb {
                    r: 100,
                    g: 150,
                    b: 255,
                },
                selection: ColorDef::Rgb {
                    r: 60,
                    g: 90,
                    b: 150,
                },
                highlight: ColorDef::Rgb {
                    r: 255,
                    g: 200,
                    b: 100,
                },
                text_normal: ColorDef::Rgb {
                    r: 220,
                    g: 220,
                    b: 220,
                },
                text_muted: ColorDef::Rgb {
                    r: 140,
                    g: 140,
                    b: 140,
                },
                text_emphasis: ColorDef::Rgb {
                    r: 255,
                    g: 255,
                    b: 255,
                },
            },
            styles: StyleScheme {
                normal: StyleDef {
                    fg: Some(ColorDef::Rgb {
                        r: 220,
                        g: 220,
                        b: 220,
                    }),
                    bg: None,
                    modifiers: vec![],
                },
                focused: StyleDef {
                    fg: Some(ColorDef::Rgb {
                        r: 100,
                        g: 150,
                        b: 255,
                    }),
                    bg: None,
                    modifiers: vec!["bold".to_string()],
                },
                selected: StyleDef {
                    fg: Some(ColorDef::Rgb {
                        r: 255,
                        g: 255,
                        b: 255,
                    }),
                    bg: Some(ColorDef::Rgb {
                        r: 60,
                        g: 90,
                        b: 150,
                    }),
                    modifiers: vec![],
                },
                disabled: StyleDef {
                    fg: Some(ColorDef::Rgb {
                        r: 100,
                        g: 100,
                        b: 100,
                    }),
                    bg: None,
                    modifiers: vec!["dim".to_string()],
                },
                hint: StyleDef {
                    fg: Some(ColorDef::Rgb {
                        r: 140,
                        g: 140,
                        b: 140,
                    }),
                    bg: None,
                    modifiers: vec!["italic".to_string()],
                },
                hint_matched: StyleDef {
                    fg: Some(ColorDef::Rgb {
                        r: 100,
                        g: 150,
                        b: 255,
                    }),
                    bg: None,
                    modifiers: vec!["bold".to_string()],
                },
                tooltip: StyleDef {
                    fg: Some(ColorDef::Rgb {
                        r: 220,
                        g: 220,
                        b: 220,
                    }),
                    bg: Some(ColorDef::Rgb {
                        r: 50,
                        g: 50,
                        b: 50,
                    }),
                    modifiers: vec![],
                },
                highlight_border: StyleDef {
                    fg: Some(ColorDef::Rgb {
                        r: 255,
                        g: 200,
                        b: 100,
                    }),
                    bg: None,
                    modifiers: vec!["bold".to_string()],
                },
            },
        }
    }

    pub fn default_light() -> Self {
        Theme {
            name: "Light".to_string(),
            description: "Light theme with dark text".to_string(),
            colors: ColorScheme {
                background: ColorDef::Rgb {
                    r: 250,
                    g: 250,
                    b: 250,
                },
                foreground: ColorDef::Rgb {
                    r: 40,
                    g: 40,
                    b: 40,
                },
                primary: ColorDef::Rgb {
                    r: 50,
                    g: 100,
                    b: 200,
                },
                secondary: ColorDef::Rgb {
                    r: 100,
                    g: 50,
                    b: 200,
                },
                accent: ColorDef::Rgb {
                    r: 200,
                    g: 100,
                    b: 0,
                },
                success: ColorDef::Rgb { r: 0, g: 150, b: 0 },
                warning: ColorDef::Rgb {
                    r: 200,
                    g: 120,
                    b: 0,
                },
                error: ColorDef::Rgb { r: 200, g: 0, b: 0 },
                info: ColorDef::Rgb {
                    r: 0,
                    g: 120,
                    b: 200,
                },
                border: ColorDef::Rgb {
                    r: 200,
                    g: 200,
                    b: 200,
                },
                border_focused: ColorDef::Rgb {
                    r: 50,
                    g: 100,
                    b: 200,
                },
                selection: ColorDef::Rgb {
                    r: 200,
                    g: 220,
                    b: 255,
                },
                highlight: ColorDef::Rgb {
                    r: 255,
                    g: 220,
                    b: 150,
                },
                text_normal: ColorDef::Rgb {
                    r: 40,
                    g: 40,
                    b: 40,
                },
                text_muted: ColorDef::Rgb {
                    r: 140,
                    g: 140,
                    b: 140,
                },
                text_emphasis: ColorDef::Rgb { r: 0, g: 0, b: 0 },
            },
            styles: StyleScheme {
                normal: StyleDef {
                    fg: Some(ColorDef::Rgb {
                        r: 40,
                        g: 40,
                        b: 40,
                    }),
                    bg: None,
                    modifiers: vec![],
                },
                focused: StyleDef {
                    fg: Some(ColorDef::Rgb {
                        r: 50,
                        g: 100,
                        b: 200,
                    }),
                    bg: None,
                    modifiers: vec!["bold".to_string()],
                },
                selected: StyleDef {
                    fg: Some(ColorDef::Rgb { r: 0, g: 0, b: 0 }),
                    bg: Some(ColorDef::Rgb {
                        r: 200,
                        g: 220,
                        b: 255,
                    }),
                    modifiers: vec![],
                },
                disabled: StyleDef {
                    fg: Some(ColorDef::Rgb {
                        r: 180,
                        g: 180,
                        b: 180,
                    }),
                    bg: None,
                    modifiers: vec!["dim".to_string()],
                },
                hint: StyleDef {
                    fg: Some(ColorDef::Rgb {
                        r: 140,
                        g: 140,
                        b: 140,
                    }),
                    bg: None,
                    modifiers: vec!["italic".to_string()],
                },
                hint_matched: StyleDef {
                    fg: Some(ColorDef::Rgb {
                        r: 50,
                        g: 100,
                        b: 200,
                    }),
                    bg: None,
                    modifiers: vec!["bold".to_string()],
                },
                tooltip: StyleDef {
                    fg: Some(ColorDef::Rgb {
                        r: 40,
                        g: 40,
                        b: 40,
                    }),
                    bg: Some(ColorDef::Rgb {
                        r: 240,
                        g: 240,
                        b: 240,
                    }),
                    modifiers: vec![],
                },
                highlight_border: StyleDef {
                    fg: Some(ColorDef::Rgb {
                        r: 200,
                        g: 100,
                        b: 0,
                    }),
                    bg: None,
                    modifiers: vec!["bold".to_string()],
                },
            },
        }
    }

    pub fn from_file(path: &Path) -> Result<Self, ThemeError> {
        let content = fs::read_to_string(path)?;
        toml::from_str(&content).map_err(|e| ThemeError::ParseError(e.to_string()))
    }

    pub fn to_file(&self, path: &Path) -> Result<(), ThemeError> {
        let content =
            toml::to_string_pretty(self).map_err(|e| ThemeError::ParseError(e.to_string()))?;
        fs::write(path, content)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_dark_theme() {
        let theme = Theme::default_dark();
        assert_eq!(theme.name, "Dark");
        assert!(!theme.description.is_empty());
    }

    #[test]
    fn test_default_light_theme() {
        let theme = Theme::default_light();
        assert_eq!(theme.name, "Light");
        assert!(!theme.description.is_empty());
    }

    #[test]
    fn test_color_def_named() {
        let color = ColorDef::Named("blue".to_string());
        assert!(matches!(color.to_color(), Color::Blue));
    }

    #[test]
    fn test_color_def_rgb() {
        let color = ColorDef::Rgb {
            r: 100,
            g: 150,
            b: 200,
        };
        match color.to_color() {
            Color::Rgb(r, g, b) => {
                assert_eq!(r, 100);
                assert_eq!(g, 150);
                assert_eq!(b, 200);
            }
            _ => panic!("Expected RGB color"),
        }
    }

    #[test]
    fn test_style_def_basic() {
        let style_def = StyleDef {
            fg: Some(ColorDef::Named("blue".to_string())),
            bg: None,
            modifiers: vec![],
        };
        let style = style_def.to_style();
        assert_eq!(style.fg, Some(Color::Blue));
    }

    #[test]
    fn test_style_def_with_modifiers() {
        let style_def = StyleDef {
            fg: Some(ColorDef::Named("blue".to_string())),
            bg: None,
            modifiers: vec!["bold".to_string(), "italic".to_string()],
        };
        let style = style_def.to_style();
        assert!(style.add_modifier.contains(Modifier::BOLD));
        assert!(style.add_modifier.contains(Modifier::ITALIC));
    }

    #[test]
    fn test_theme_serialization() {
        let theme = Theme::default_dark();
        let toml_str = toml::to_string(&theme).unwrap();
        let deserialized: Theme = toml::from_str(&toml_str).unwrap();
        assert_eq!(theme.name, deserialized.name);
    }
}
