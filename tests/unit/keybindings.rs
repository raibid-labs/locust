use crossterm::event::{KeyCode, KeyModifiers};
use locust::core::keybindings::{detect_conflicts, KeyBinding, KeyCodeDef, KeyMap};
use tempfile::TempDir;

#[test]
fn test_default_keymap() {
    let keymap = KeyMap::default();
    assert!(keymap.global.contains_key("quit"));
    assert!(keymap.plugins.contains_key("nav"));
}

#[test]
fn test_global_bindings_present() {
    let keymap = KeyMap::default();
    assert!(keymap.get_binding("quit").is_some());
    assert!(keymap.get_binding("help").is_some());
}

#[test]
fn test_plugin_bindings_present() {
    let keymap = KeyMap::default();
    assert!(keymap.get_binding("nav.activate").is_some());
    assert!(keymap.get_binding("omnibar.activate").is_some());
    assert!(keymap.get_binding("tooltip.show").is_some());
    assert!(keymap.get_binding("highlight.next_step").is_some());
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
fn test_unbind_plugin_action() {
    let mut keymap = KeyMap::default();
    keymap.unbind("nav.activate");
    assert_eq!(keymap.get_binding("nav.activate"), None);
}

#[test]
fn test_get_action_by_binding() {
    let keymap = KeyMap::default();
    let quit_binding = keymap.get_binding("quit").unwrap().clone();
    assert_eq!(keymap.get_action(&quit_binding), Some("quit".to_string()));
}

#[test]
fn test_get_action_by_binding_plugin() {
    let keymap = KeyMap::default();
    let nav_binding = keymap.get_binding("nav.activate").unwrap().clone();
    assert_eq!(keymap.get_action(&nav_binding), Some("nav.activate".to_string()));
}

#[test]
fn test_detect_conflicts() {
    let mut keymap = KeyMap::default();
    let binding = KeyBinding::new(KeyCodeDef::Char('q'));
    keymap.bind("custom", binding).unwrap();

    let conflicts = detect_conflicts(&keymap);
    assert!(!conflicts.is_empty());
    assert!(conflicts.iter().any(|c| c.actions.contains(&"quit".to_string())));
}

#[test]
fn test_no_conflicts_different_keys() {
    let mut keymap = KeyMap::default();
    let binding = KeyBinding::new(KeyCodeDef::Char('x'));
    keymap.bind("custom", binding).unwrap();

    let conflicts = detect_conflicts(&keymap);
    // Should not have conflicts with 'x' since it's unique
    assert!(!conflicts.iter().any(|c|
        c.binding.key == KeyCodeDef::Char('x') && c.actions.contains(&"custom".to_string())
    ));
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
fn test_keycode_def_named_esc() {
    let code = KeyCodeDef::Named("esc".to_string());
    assert_eq!(code.to_keycode(), KeyCode::Esc);
}

#[test]
fn test_keycode_def_named_enter() {
    let code = KeyCodeDef::Named("enter".to_string());
    assert_eq!(code.to_keycode(), KeyCode::Enter);
}

#[test]
fn test_keycode_def_from_keycode() {
    let code = KeyCode::Char('x');
    let def: KeyCodeDef = code.into();
    assert_eq!(def, KeyCodeDef::Char('x'));
}

#[test]
fn test_keymap_serialization() {
    let keymap = KeyMap::default();
    let toml_str = toml::to_string(&keymap).unwrap();
    let deserialized: KeyMap = toml::from_str(&toml_str).unwrap();
    assert_eq!(keymap.global.len(), deserialized.global.len());
}

#[test]
fn test_keymap_from_file() {
    let temp_dir = TempDir::new().unwrap();
    let keymap_path = temp_dir.path().join("test.toml");

    let keymap = KeyMap::default();
    keymap.to_file(&keymap_path).unwrap();

    let loaded = KeyMap::from_file(&keymap_path).unwrap();
    assert_eq!(loaded.global.len(), keymap.global.len());
}

#[test]
fn test_keymap_to_file() {
    let temp_dir = TempDir::new().unwrap();
    let keymap_path = temp_dir.path().join("test.toml");

    let keymap = KeyMap::default();
    let result = keymap.to_file(&keymap_path);
    assert!(result.is_ok());
    assert!(keymap_path.exists());
}

#[test]
fn test_validate_no_conflicts() {
    let mut keymap = KeyMap::default();
    // Remove all default bindings
    keymap.global.clear();
    keymap.plugins.clear();

    // Add unique bindings
    keymap.bind("action1", KeyBinding::new(KeyCodeDef::Char('a'))).unwrap();
    keymap.bind("action2", KeyBinding::new(KeyCodeDef::Char('b'))).unwrap();

    let result = keymap.validate();
    assert!(result.is_ok());
}

#[test]
fn test_validate_with_conflicts() {
    let mut keymap = KeyMap::default();
    let binding = KeyBinding::new(KeyCodeDef::Char('q'));
    keymap.bind("custom", binding).unwrap();

    let result = keymap.validate();
    assert!(result.is_err());
}

#[test]
fn test_modifiers_control() {
    let binding = KeyBinding::with_modifiers(KeyCodeDef::Char('p'), KeyModifiers::CONTROL);
    assert_ne!(binding.modifiers, 0);
}

#[test]
fn test_modifiers_empty() {
    let binding = KeyBinding::new(KeyCodeDef::Char('p'));
    assert_eq!(binding.modifiers, 0);
}
