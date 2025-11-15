use locust::core::theme::{ColorDef, ColorScheme, StyleDef, Theme, ThemeError};
use ratatui::style::{Color, Modifier};
use std::fs;
use std::io::Write;
use tempfile::TempDir;

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
fn test_color_def_named_blue() {
    let color = ColorDef::Named("blue".to_string());
    assert_eq!(color.to_color(), Color::Blue);
}

#[test]
fn test_color_def_named_red() {
    let color = ColorDef::Named("red".to_string());
    assert_eq!(color.to_color(), Color::Red);
}

#[test]
fn test_color_def_named_case_insensitive() {
    let color = ColorDef::Named("BLUE".to_string());
    assert_eq!(color.to_color(), Color::Blue);
}

#[test]
fn test_color_def_rgb() {
    let color = ColorDef::Rgb { r: 100, g: 150, b: 200 };
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
fn test_color_def_from_color() {
    let color = Color::Blue;
    let def: ColorDef = color.into();
    match def {
        ColorDef::Named(name) => assert_eq!(name, "blue"),
        _ => panic!("Expected named color"),
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
fn test_style_def_with_background() {
    let style_def = StyleDef {
        fg: Some(ColorDef::Named("blue".to_string())),
        bg: Some(ColorDef::Named("red".to_string())),
        modifiers: vec![],
    };
    let style = style_def.to_style();
    assert_eq!(style.fg, Some(Color::Blue));
    assert_eq!(style.bg, Some(Color::Red));
}

#[test]
fn test_style_def_with_bold() {
    let style_def = StyleDef {
        fg: None,
        bg: None,
        modifiers: vec!["bold".to_string()],
    };
    let style = style_def.to_style();
    assert!(style.add_modifier.contains(Modifier::BOLD));
}

#[test]
fn test_style_def_with_multiple_modifiers() {
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

#[test]
fn test_theme_from_file() {
    let temp_dir = TempDir::new().unwrap();
    let theme_path = temp_dir.path().join("test.toml");

    let theme = Theme::default_dark();
    theme.to_file(&theme_path).unwrap();

    let loaded = Theme::from_file(&theme_path).unwrap();
    assert_eq!(loaded.name, "Dark");
}

#[test]
fn test_theme_to_file() {
    let temp_dir = TempDir::new().unwrap();
    let theme_path = temp_dir.path().join("test.toml");

    let theme = Theme::default_light();
    let result = theme.to_file(&theme_path);
    assert!(result.is_ok());
    assert!(theme_path.exists());
}

#[test]
fn test_color_scheme_has_all_colors() {
    let theme = Theme::default_dark();
    let _ = theme.colors.background.to_color();
    let _ = theme.colors.foreground.to_color();
    let _ = theme.colors.primary.to_color();
    let _ = theme.colors.secondary.to_color();
    let _ = theme.colors.accent.to_color();
    let _ = theme.colors.success.to_color();
    let _ = theme.colors.warning.to_color();
    let _ = theme.colors.error.to_color();
    let _ = theme.colors.info.to_color();
    let _ = theme.colors.border.to_color();
    let _ = theme.colors.border_focused.to_color();
    let _ = theme.colors.selection.to_color();
    let _ = theme.colors.highlight.to_color();
    let _ = theme.colors.text_normal.to_color();
    let _ = theme.colors.text_muted.to_color();
    let _ = theme.colors.text_emphasis.to_color();
}

#[test]
fn test_style_scheme_has_all_styles() {
    let theme = Theme::default_dark();
    let _ = theme.styles.normal.to_style();
    let _ = theme.styles.focused.to_style();
    let _ = theme.styles.selected.to_style();
    let _ = theme.styles.disabled.to_style();
    let _ = theme.styles.hint.to_style();
    let _ = theme.styles.hint_matched.to_style();
    let _ = theme.styles.tooltip.to_style();
    let _ = theme.styles.highlight_border.to_style();
}
