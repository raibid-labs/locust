use super::theme::{Theme, ThemeError};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug)]
pub struct ThemeManager {
    current_theme: Theme,
    available_themes: HashMap<String, Theme>,
}

impl ThemeManager {
    pub fn new() -> Self {
        let mut manager = ThemeManager {
            current_theme: Theme::default_dark(),
            available_themes: HashMap::new(),
        };

        // Add built-in themes
        let dark = Theme::default_dark();
        let light = Theme::default_light();
        manager.available_themes.insert(dark.name.clone(), dark.clone());
        manager.available_themes.insert(light.name.clone(), light);
        manager.current_theme = dark;

        manager
    }

    pub fn load_themes_from_dir(dir: &Path) -> Result<Self, ThemeError> {
        let mut manager = Self::new();

        if !dir.exists() {
            return Ok(manager);
        }

        let entries = fs::read_dir(dir)
            .map_err(|e| ThemeError::LoadError(format!("Failed to read themes directory: {}", e)))?;

        for entry in entries {
            let entry = entry
                .map_err(|e| ThemeError::LoadError(format!("Failed to read directory entry: {}", e)))?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("toml") {
                match Theme::from_file(&path) {
                    Ok(theme) => {
                        manager.available_themes.insert(theme.name.clone(), theme);
                    }
                    Err(e) => {
                        eprintln!("Warning: Failed to load theme from {:?}: {}", path, e);
                    }
                }
            }
        }

        Ok(manager)
    }

    pub fn set_theme(&mut self, name: &str) -> Result<(), ThemeError> {
        let theme = self.available_themes
            .get(name)
            .ok_or_else(|| ThemeError::NotFound(name.to_string()))?;
        self.current_theme = theme.clone();
        Ok(())
    }

    pub fn get_current_theme(&self) -> &Theme {
        &self.current_theme
    }

    pub fn list_themes(&self) -> Vec<&str> {
        self.available_themes.keys().map(|s| s.as_str()).collect()
    }

    pub fn add_theme(&mut self, theme: Theme) {
        self.available_themes.insert(theme.name.clone(), theme);
    }

    pub fn remove_theme(&mut self, name: &str) -> Result<(), ThemeError> {
        if name == self.current_theme.name {
            return Err(ThemeError::LoadError(
                "Cannot remove the currently active theme".to_string(),
            ));
        }
        self.available_themes.remove(name);
        Ok(())
    }
}

impl Default for ThemeManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_new_theme_manager() {
        let manager = ThemeManager::new();
        assert_eq!(manager.current_theme.name, "Dark");
        assert!(manager.available_themes.len() >= 2);
    }

    #[test]
    fn test_set_theme() {
        let mut manager = ThemeManager::new();
        manager.set_theme("Light").unwrap();
        assert_eq!(manager.current_theme.name, "Light");
    }

    #[test]
    fn test_set_nonexistent_theme() {
        let mut manager = ThemeManager::new();
        let result = manager.set_theme("NonExistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_list_themes() {
        let manager = ThemeManager::new();
        let themes = manager.list_themes();
        assert!(themes.contains(&"Dark"));
        assert!(themes.contains(&"Light"));
    }

    #[test]
    fn test_add_theme() {
        let mut manager = ThemeManager::new();
        let custom = Theme {
            name: "Custom".to_string(),
            description: "Custom theme".to_string(),
            colors: Theme::default_dark().colors,
            styles: Theme::default_dark().styles,
        };
        manager.add_theme(custom);
        assert!(manager.available_themes.contains_key("Custom"));
    }

    #[test]
    fn test_remove_theme() {
        let mut manager = ThemeManager::new();
        manager.remove_theme("Light").unwrap();
        assert!(!manager.available_themes.contains_key("Light"));
    }

    #[test]
    fn test_cannot_remove_current_theme() {
        let mut manager = ThemeManager::new();
        let result = manager.remove_theme("Dark");
        assert!(result.is_err());
    }

    #[test]
    fn test_load_themes_from_dir() {
        let temp_dir = TempDir::new().unwrap();
        let theme_path = temp_dir.path().join("test.toml");

        let theme_content = r#"
name = "Test"
description = "Test theme"

[colors]
background = { r = 0, g = 0, b = 0 }
foreground = { r = 255, g = 255, b = 255 }
primary = { r = 100, g = 150, b = 255 }
secondary = { r = 150, g = 100, b = 255 }
accent = { r = 255, g = 200, b = 100 }
success = { r = 100, g = 255, b = 100 }
warning = { r = 255, g = 200, b = 100 }
error = { r = 255, g = 100, b = 100 }
info = { r = 100, g = 200, b = 255 }
border = { r = 80, g = 80, b = 80 }
border_focused = { r = 100, g = 150, b = 255 }
selection = { r = 60, g = 90, b = 150 }
highlight = { r = 255, g = 200, b = 100 }
text_normal = { r = 220, g = 220, b = 220 }
text_muted = { r = 140, g = 140, b = 140 }
text_emphasis = { r = 255, g = 255, b = 255 }

[styles.normal]
[styles.focused]
[styles.selected]
[styles.disabled]
[styles.hint]
[styles.hint_matched]
[styles.tooltip]
[styles.highlight_border]
"#;

        let mut file = fs::File::create(&theme_path).unwrap();
        file.write_all(theme_content.as_bytes()).unwrap();

        let manager = ThemeManager::load_themes_from_dir(temp_dir.path()).unwrap();
        assert!(manager.available_themes.contains_key("Test"));
    }
}
