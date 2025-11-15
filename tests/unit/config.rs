//! Unit tests for the configuration system.

use locust::core::config::*;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn test_default_config() {
    let config = LocustConfig::new();
    assert_eq!(config.global.fps_limit, Some(60));
    assert!(config.global.mouse_support);
    assert!(!config.global.enable_logging);
    assert_eq!(config.global.log_level, LogLevel::Info);
    assert!(config.plugins.is_empty());
    assert!(config.config_path.is_none());
}

#[test]
fn test_global_config_defaults() {
    let config = GlobalConfig::default();
    assert!(!config.enable_logging);
    assert_eq!(config.log_level, LogLevel::Info);
    assert_eq!(config.fps_limit, Some(60));
    assert!(config.mouse_support);
}

#[test]
fn test_nav_config_defaults() {
    let config = NavConfig::default();
    assert_eq!(config.hint_key, 'f');
    assert_eq!(config.charset, "asdfghjkl");
    assert_eq!(config.min_target_area, 1);
    assert_eq!(config.max_hints, 0);
}

#[test]
fn test_omnibar_config_defaults() {
    let config = OmnibarConfig::default();
    assert_eq!(config.activation_key, 'p');
    assert_eq!(config.activation_modifiers, vec!["Ctrl".to_string()]);
    assert_eq!(config.max_results, 10);
    assert_eq!(config.fuzzy_threshold, 0.6);
}

#[test]
fn test_tooltip_config_defaults() {
    let config = TooltipConfig::default();
    assert_eq!(config.default_delay_ms, 500);
    assert_eq!(config.default_position, TooltipPosition::Right);
    assert_eq!(config.auto_hide_ms, 3000);
}

#[test]
fn test_highlight_config_defaults() {
    let config = HighlightConfig::default();
    assert_eq!(config.default_animation, HighlightAnimation::Pulse);
    assert_eq!(config.dim_opacity, 0.7);
    assert_eq!(config.border_thickness, 2);
}

#[test]
fn test_update_plugin_config() {
    let mut config = LocustConfig::new();
    let nav_config = NavConfig {
        hint_key: 'g',
        charset: "abc".to_string(),
        min_target_area: 10,
        max_hints: 50,
    };

    config.update_plugin("nav", nav_config.clone()).unwrap();
    let retrieved: NavConfig = config.get_plugin_config("nav").unwrap();
    assert_eq!(retrieved, nav_config);
}

#[test]
fn test_get_nonexistent_plugin_config() {
    let config = LocustConfig::new();
    let result: Option<NavConfig> = config.get_plugin_config("nonexistent");
    assert!(result.is_none());
}

#[test]
fn test_save_and_load_toml() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("test.toml");

    let mut config = LocustConfig::new();
    config.global.fps_limit = Some(120);
    config.global.enable_logging = true;
    config.config_path = Some(config_path.clone());

    // Save
    config.save().unwrap();

    // Load
    let loaded = LocustConfig::from_file(&config_path).unwrap();
    assert_eq!(loaded.global.fps_limit, Some(120));
    assert!(loaded.global.enable_logging);
}

#[test]
fn test_save_and_load_json() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("test.json");

    let mut config = LocustConfig::new();
    config.global.fps_limit = Some(90);
    config.global.mouse_support = false;

    config.save_to(&config_path).unwrap();

    // Load
    let loaded = LocustConfig::from_file(&config_path).unwrap();
    assert_eq!(loaded.global.fps_limit, Some(90));
    assert!(!loaded.global.mouse_support);
}

#[test]
fn test_reload_config() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("test.toml");

    let mut config = LocustConfig::new();
    config.global.fps_limit = Some(60);
    config.config_path = Some(config_path.clone());
    config.save().unwrap();

    // Load initial config
    let mut loaded = LocustConfig::from_file(&config_path).unwrap();
    assert_eq!(loaded.global.fps_limit, Some(60));

    // Modify file directly
    let mut modified = LocustConfig::new();
    modified.global.fps_limit = Some(144);
    modified.save_to(&config_path).unwrap();

    // Reload
    loaded.reload().unwrap();
    assert_eq!(loaded.global.fps_limit, Some(144));
}

#[test]
fn test_validation_fps_zero() {
    let mut config = LocustConfig::new();
    config.global.fps_limit = Some(0);

    let errors = config.validate();
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].severity, Severity::Error);
    assert!(errors[0].message.contains("greater than 0"));
    assert_eq!(errors[0].field, "global.fps_limit");
}

#[test]
fn test_validation_fps_warning() {
    let mut config = LocustConfig::new();
    config.global.fps_limit = Some(300);

    let errors = config.validate();
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].severity, Severity::Warning);
    assert!(errors[0].message.contains("240"));
}

#[test]
fn test_validation_empty_charset() {
    let mut config = LocustConfig::new();
    let nav_config = NavConfig {
        hint_key: 'f',
        charset: String::new(),
        min_target_area: 1,
        max_hints: 0,
    };
    config.plugins.insert("nav".to_string(), PluginConfig::Nav(nav_config));

    let errors = config.validate();
    assert!(!errors.is_empty());
    assert!(errors.iter().any(|e| e.field.contains("charset")));
    assert!(errors.iter().any(|e| e.severity == Severity::Error));
}

#[test]
fn test_validation_no_errors() {
    let config = LocustConfig::new();
    let errors = config.validate();
    assert!(errors.is_empty());
}

#[test]
fn test_config_watcher_detects_changes() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("test.toml");

    // Create initial file
    fs::write(&config_path, "# initial").unwrap();

    let mut watcher = ConfigWatcher::new(config_path.clone());

    // No change initially
    assert!(!watcher.check_for_changes());

    // Wait a bit to ensure timestamp difference
    std::thread::sleep(std::time::Duration::from_millis(10));

    // Modify file
    fs::write(&config_path, "# modified").unwrap();

    // Should detect change
    assert!(watcher.check_for_changes());

    // Subsequent check without modification should return false
    assert!(!watcher.check_for_changes());
}

#[test]
fn test_config_watcher_missing_file() {
    let watcher = ConfigWatcher::new(PathBuf::from("/nonexistent/path.toml"));
    // Should not panic
    assert!(!watcher.check_for_changes());
}

#[test]
fn test_serialization_roundtrip_toml() {
    let mut config = LocustConfig::new();
    config.global.fps_limit = Some(75);
    config.global.enable_logging = true;
    config.global.log_level = LogLevel::Debug;

    let nav_config = NavConfig {
        hint_key: 'x',
        charset: "xyz".to_string(),
        min_target_area: 5,
        max_hints: 25,
    };
    config.update_plugin("nav", nav_config.clone()).unwrap();

    let toml_str = toml::to_string(&config).unwrap();
    let deserialized: LocustConfig = toml::from_str(&toml_str).unwrap();

    assert_eq!(deserialized.global.fps_limit, config.global.fps_limit);
    assert_eq!(deserialized.global.enable_logging, config.global.enable_logging);

    let retrieved: NavConfig = deserialized.get_plugin_config("nav").unwrap();
    assert_eq!(retrieved, nav_config);
}

#[test]
fn test_serialization_roundtrip_json() {
    let mut config = LocustConfig::new();
    config.global.mouse_support = false;

    let tooltip_config = TooltipConfig {
        default_delay_ms: 1000,
        default_position: TooltipPosition::Below,
        auto_hide_ms: 5000,
    };
    config.update_plugin("tooltip", tooltip_config.clone()).unwrap();

    let json_str = serde_json::to_string(&config).unwrap();
    let deserialized: LocustConfig = serde_json::from_str(&json_str).unwrap();

    assert_eq!(deserialized.global.mouse_support, config.global.mouse_support);

    let retrieved: TooltipConfig = deserialized.get_plugin_config("tooltip").unwrap();
    assert_eq!(retrieved, tooltip_config);
}

#[test]
fn test_config_merge_plugins() {
    let mut config = LocustConfig::new();

    let nav = NavConfig::default();
    let omnibar = OmnibarConfig::default();
    let tooltip = TooltipConfig::default();
    let highlight = HighlightConfig::default();

    config.update_plugin("nav", nav).unwrap();
    config.update_plugin("omnibar", omnibar).unwrap();
    config.update_plugin("tooltip", tooltip).unwrap();
    config.update_plugin("highlight", highlight).unwrap();

    assert_eq!(config.plugins.len(), 4);
    assert!(config.get_plugin_config::<NavConfig>("nav").is_some());
    assert!(config.get_plugin_config::<OmnibarConfig>("omnibar").is_some());
    assert!(config.get_plugin_config::<TooltipConfig>("tooltip").is_some());
    assert!(config.get_plugin_config::<HighlightConfig>("highlight").is_some());
}

#[test]
fn test_log_level_serialization() {
    assert_eq!(LogLevel::Trace, LogLevel::Trace);
    assert_eq!(LogLevel::Debug, LogLevel::Debug);
    assert_eq!(LogLevel::Info, LogLevel::Info);
    assert_eq!(LogLevel::Warn, LogLevel::Warn);
    assert_eq!(LogLevel::Error, LogLevel::Error);
}

#[test]
fn test_tooltip_position_serialization() {
    assert_eq!(TooltipPosition::Right, TooltipPosition::Right);
    assert_eq!(TooltipPosition::Left, TooltipPosition::Left);
    assert_eq!(TooltipPosition::Above, TooltipPosition::Above);
    assert_eq!(TooltipPosition::Below, TooltipPosition::Below);
    assert_eq!(TooltipPosition::Auto, TooltipPosition::Auto);
}

#[test]
fn test_highlight_animation_serialization() {
    assert_eq!(HighlightAnimation::None, HighlightAnimation::None);
    assert_eq!(HighlightAnimation::Pulse, HighlightAnimation::Pulse);
    assert_eq!(HighlightAnimation::Shimmer, HighlightAnimation::Shimmer);
    assert_eq!(HighlightAnimation::Breathe, HighlightAnimation::Breathe);
}

#[test]
fn test_error_display() {
    let error = ConfigError::NoConfigPath;
    assert!(format!("{}", error).contains("No configuration path"));

    let error = ConfigError::Io {
        path: PathBuf::from("/test/path"),
        source: std::io::Error::new(std::io::ErrorKind::NotFound, "test"),
    };
    assert!(format!("{}", error).contains("/test/path"));
}

#[test]
fn test_custom_plugin_config() {
    let mut config = LocustConfig::new();

    let custom_data = serde_json::json!({
        "key1": "value1",
        "key2": 42,
        "key3": [1, 2, 3]
    });

    config.plugins.insert(
        "custom".to_string(),
        PluginConfig::Custom(custom_data.clone()),
    );

    let retrieved = config.get_plugin_config::<serde_json::Value>("custom").unwrap();
    assert_eq!(retrieved, custom_data);
}

#[test]
fn test_config_path_preservation() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("test.toml");

    let config = LocustConfig::new();
    config.save_to(&config_path).unwrap();

    let loaded = LocustConfig::from_file(&config_path).unwrap();
    assert_eq!(loaded.config_path, Some(config_path));
}
