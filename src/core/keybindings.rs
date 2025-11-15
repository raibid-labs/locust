use crossterm::event::{KeyCode, KeyModifiers};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, thiserror::Error)]
pub enum KeyMapError {
    #[error("Failed to load keymap: {0}")]
    LoadError(String),
    #[error("Failed to parse keymap: {0}")]
    ParseError(String),
    #[error("Action not found: {0}")]
    ActionNotFound(String),
    #[error("Key binding conflicts detected")]
    Conflicts(Vec<ConflictError>),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct KeyBinding {
    pub key: KeyCodeDef,
    #[serde(default, skip_serializing_if = "is_empty_modifiers")]
    pub modifiers: u8,
}

fn is_empty_modifiers(modifiers: &u8) -> bool {
    *modifiers == 0
}

impl KeyBinding {
    pub fn new(key: KeyCodeDef) -> Self {
        Self {
            key,
            modifiers: 0,
        }
    }

    pub fn with_modifiers(key: KeyCodeDef, modifiers: KeyModifiers) -> Self {
        Self {
            key,
            modifiers: modifiers.bits(),
        }
    }

    pub fn get_modifiers(&self) -> KeyModifiers {
        KeyModifiers::from_bits_truncate(self.modifiers)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum KeyCodeDef {
    Char(char),
    F(u8),
    Named(String),
}

impl KeyCodeDef {
    pub fn to_keycode(&self) -> KeyCode {
        match self {
            KeyCodeDef::Char(c) => KeyCode::Char(*c),
            KeyCodeDef::F(n) => KeyCode::F(*n),
            KeyCodeDef::Named(name) => match name.to_lowercase().as_str() {
                "backspace" => KeyCode::Backspace,
                "enter" => KeyCode::Enter,
                "left" => KeyCode::Left,
                "right" => KeyCode::Right,
                "up" => KeyCode::Up,
                "down" => KeyCode::Down,
                "home" => KeyCode::Home,
                "end" => KeyCode::End,
                "pageup" => KeyCode::PageUp,
                "pagedown" => KeyCode::PageDown,
                "tab" => KeyCode::Tab,
                "backtab" => KeyCode::BackTab,
                "delete" => KeyCode::Delete,
                "insert" => KeyCode::Insert,
                "esc" | "escape" => KeyCode::Esc,
                _ => KeyCode::Null,
            },
        }
    }
}

impl From<KeyCode> for KeyCodeDef {
    fn from(code: KeyCode) -> Self {
        match code {
            KeyCode::Char(c) => KeyCodeDef::Char(c),
            KeyCode::F(n) => KeyCodeDef::F(n),
            KeyCode::Backspace => KeyCodeDef::Named("backspace".to_string()),
            KeyCode::Enter => KeyCodeDef::Named("enter".to_string()),
            KeyCode::Left => KeyCodeDef::Named("left".to_string()),
            KeyCode::Right => KeyCodeDef::Named("right".to_string()),
            KeyCode::Up => KeyCodeDef::Named("up".to_string()),
            KeyCode::Down => KeyCodeDef::Named("down".to_string()),
            KeyCode::Home => KeyCodeDef::Named("home".to_string()),
            KeyCode::End => KeyCodeDef::Named("end".to_string()),
            KeyCode::PageUp => KeyCodeDef::Named("pageup".to_string()),
            KeyCode::PageDown => KeyCodeDef::Named("pagedown".to_string()),
            KeyCode::Tab => KeyCodeDef::Named("tab".to_string()),
            KeyCode::BackTab => KeyCodeDef::Named("backtab".to_string()),
            KeyCode::Delete => KeyCodeDef::Named("delete".to_string()),
            KeyCode::Insert => KeyCodeDef::Named("insert".to_string()),
            KeyCode::Esc => KeyCodeDef::Named("esc".to_string()),
            _ => KeyCodeDef::Named("null".to_string()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ConflictError {
    pub binding: KeyBinding,
    pub actions: Vec<String>,
}

impl std::fmt::Display for ConflictError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Key binding {:?} conflicts between actions: {}",
            self.binding,
            self.actions.join(", ")
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyMap {
    /// Global keybindings
    #[serde(default)]
    pub global: HashMap<String, KeyBinding>,

    /// Per-plugin keybindings
    #[serde(default)]
    pub plugins: HashMap<String, HashMap<String, KeyBinding>>,
}

impl Default for KeyMap {
    fn default() -> Self {
        let mut keymap = KeyMap {
            global: HashMap::new(),
            plugins: HashMap::new(),
        };

        // Global bindings
        keymap.global.insert(
            "quit".to_string(),
            KeyBinding::new(KeyCodeDef::Char('q')),
        );
        keymap.global.insert(
            "help".to_string(),
            KeyBinding::new(KeyCodeDef::F(1)),
        );

        // Nav plugin bindings
        let mut nav_bindings = HashMap::new();
        nav_bindings.insert(
            "activate".to_string(),
            KeyBinding::new(KeyCodeDef::Char('f')),
        );
        nav_bindings.insert(
            "cancel".to_string(),
            KeyBinding::new(KeyCodeDef::Named("esc".to_string())),
        );
        keymap.plugins.insert("nav".to_string(), nav_bindings);

        // Omnibar plugin bindings
        let mut omnibar_bindings = HashMap::new();
        omnibar_bindings.insert(
            "activate".to_string(),
            KeyBinding::with_modifiers(KeyCodeDef::Char('p'), KeyModifiers::CONTROL),
        );
        omnibar_bindings.insert(
            "cancel".to_string(),
            KeyBinding::new(KeyCodeDef::Named("esc".to_string())),
        );
        keymap.plugins.insert("omnibar".to_string(), omnibar_bindings);

        // Tooltip plugin bindings
        let mut tooltip_bindings = HashMap::new();
        tooltip_bindings.insert(
            "show".to_string(),
            KeyBinding::new(KeyCodeDef::Char('h')),
        );
        tooltip_bindings.insert(
            "hide".to_string(),
            KeyBinding::new(KeyCodeDef::Named("esc".to_string())),
        );
        keymap.plugins.insert("tooltip".to_string(), tooltip_bindings);

        // Highlight plugin bindings
        let mut highlight_bindings = HashMap::new();
        highlight_bindings.insert(
            "next_step".to_string(),
            KeyBinding::new(KeyCodeDef::Char('n')),
        );
        highlight_bindings.insert(
            "previous_step".to_string(),
            KeyBinding::new(KeyCodeDef::Char('p')),
        );
        highlight_bindings.insert(
            "skip_tour".to_string(),
            KeyBinding::new(KeyCodeDef::Char('s')),
        );
        keymap.plugins.insert("highlight".to_string(), highlight_bindings);

        keymap
    }
}

impl KeyMap {
    pub fn from_file(path: &Path) -> Result<Self, KeyMapError> {
        let content = fs::read_to_string(path)?;
        toml::from_str(&content)
            .map_err(|e| KeyMapError::ParseError(e.to_string()))
    }

    pub fn to_file(&self, path: &Path) -> Result<(), KeyMapError> {
        let content = toml::to_string_pretty(self)
            .map_err(|e| KeyMapError::ParseError(e.to_string()))?;
        fs::write(path, content)?;
        Ok(())
    }

    pub fn bind(&mut self, action: &str, binding: KeyBinding) -> Result<(), KeyMapError> {
        // Parse action to determine if it's global or plugin-specific
        if let Some((plugin, action_name)) = action.split_once('.') {
            self.plugins
                .entry(plugin.to_string())
                .or_default()
                .insert(action_name.to_string(), binding);
        } else {
            self.global.insert(action.to_string(), binding);
        }
        Ok(())
    }

    pub fn unbind(&mut self, action: &str) {
        if let Some((plugin, action_name)) = action.split_once('.') {
            if let Some(bindings) = self.plugins.get_mut(plugin) {
                bindings.remove(action_name);
            }
        } else {
            self.global.remove(action);
        }
    }

    pub fn get_binding(&self, action: &str) -> Option<&KeyBinding> {
        if let Some((plugin, action_name)) = action.split_once('.') {
            self.plugins
                .get(plugin)
                .and_then(|bindings| bindings.get(action_name))
        } else {
            self.global.get(action)
        }
    }

    pub fn get_action(&self, binding: &KeyBinding) -> Option<String> {
        // Check global bindings
        for (action, bound) in &self.global {
            if bound == binding {
                return Some(action.clone());
            }
        }

        // Check plugin bindings
        for (plugin, bindings) in &self.plugins {
            for (action, bound) in bindings {
                if bound == binding {
                    return Some(format!("{}.{}", plugin, action));
                }
            }
        }

        None
    }

    pub fn validate(&self) -> Result<(), Vec<ConflictError>> {
        let conflicts = detect_conflicts(self);
        if conflicts.is_empty() {
            Ok(())
        } else {
            Err(conflicts)
        }
    }
}

pub fn detect_conflicts(keymap: &KeyMap) -> Vec<ConflictError> {
    let mut binding_map: HashMap<KeyBinding, Vec<String>> = HashMap::new();

    // Collect all bindings
    for (action, binding) in &keymap.global {
        binding_map
            .entry(binding.clone())
            .or_default()
            .push(action.clone());
    }

    for (plugin, bindings) in &keymap.plugins {
        for (action, binding) in bindings {
            binding_map
                .entry(binding.clone())
                .or_default()
                .push(format!("{}.{}", plugin, action));
        }
    }

    // Find conflicts (bindings with multiple actions)
    binding_map
        .into_iter()
        .filter(|(_, actions)| actions.len() > 1)
        .map(|(binding, actions)| ConflictError { binding, actions })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_keymap() {
        let keymap = KeyMap::default();
        assert!(keymap.global.contains_key("quit"));
        assert!(keymap.plugins.contains_key("nav"));
    }

    #[test]
    fn test_bind_global_action() {
        let mut keymap = KeyMap::default();
        let binding = KeyBinding::new(KeyCodeDef::Char('x'));
        keymap.bind("custom", binding.clone()).unwrap();
        assert_eq!(keymap.get_binding("custom"), Some(&binding));
    }

    #[test]
    fn test_bind_plugin_action() {
        let mut keymap = KeyMap::default();
        let binding = KeyBinding::new(KeyCodeDef::Char('x'));
        keymap.bind("myplugin.action", binding.clone()).unwrap();
        assert_eq!(keymap.get_binding("myplugin.action"), Some(&binding));
    }

    #[test]
    fn test_unbind_action() {
        let mut keymap = KeyMap::default();
        keymap.unbind("quit");
        assert_eq!(keymap.get_binding("quit"), None);
    }

    #[test]
    fn test_get_action_by_binding() {
        let keymap = KeyMap::default();
        let quit_binding = keymap.get_binding("quit").unwrap().clone();
        assert_eq!(keymap.get_action(&quit_binding), Some("quit".to_string()));
    }

    #[test]
    fn test_detect_conflicts() {
        let mut keymap = KeyMap::default();
        let binding = KeyBinding::new(KeyCodeDef::Char('q'));
        keymap.bind("custom", binding).unwrap();

        let conflicts = detect_conflicts(&keymap);
        assert!(!conflicts.is_empty());
        // The 'q' key now has conflicts between 'quit' and 'custom'
        // There may also be ESC conflicts from multiple plugins
        let q_conflict = conflicts.iter().find(|c| c.binding.key == KeyCodeDef::Char('q'));
        assert!(q_conflict.is_some());
        assert!(q_conflict.unwrap().actions.contains(&"quit".to_string()));
        assert!(q_conflict.unwrap().actions.contains(&"custom".to_string()));
    }

    #[test]
    fn test_detect_esc_conflicts() {
        let keymap = KeyMap::default();
        let conflicts = detect_conflicts(&keymap);
        // Default keymap has ESC conflicts across multiple plugins
        // (nav.cancel, omnibar.cancel, tooltip.hide all use ESC)
        let esc_conflict = conflicts.iter().find(|c|
            c.binding.key == KeyCodeDef::Named("esc".to_string())
        );
        assert!(esc_conflict.is_some());
    }

    #[test]
    fn test_keycode_def_char() {
        let code = KeyCodeDef::Char('a');
        assert_eq!(code.to_keycode(), KeyCode::Char('a'));
    }

    #[test]
    fn test_keycode_def_f_key() {
        let code = KeyCodeDef::F(1);
        assert_eq!(code.to_keycode(), KeyCode::F(1));
    }

    #[test]
    fn test_keycode_def_named() {
        let code = KeyCodeDef::Named("esc".to_string());
        assert_eq!(code.to_keycode(), KeyCode::Esc);
    }

    #[test]
    fn test_keymap_serialization() {
        let keymap = KeyMap::default();
        let toml_str = toml::to_string(&keymap).unwrap();
        let deserialized: KeyMap = toml::from_str(&toml_str).unwrap();
        assert_eq!(keymap.global.len(), deserialized.global.len());
    }
}
