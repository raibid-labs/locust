# Locust Configuration Guide

This guide covers the complete configuration system in Locust, including file formats, runtime updates, hot reload, and plugin configuration.

## Table of Contents

- [Overview](#overview)
- [Configuration File Format](#configuration-file-format)
- [Global Settings](#global-settings)
- [Plugin Configuration](#plugin-configuration)
- [Loading and Saving](#loading-and-saving)
- [Runtime Updates](#runtime-updates)
- [Hot Reload](#hot-reload)
- [Validation](#validation)
- [Examples](#examples)

## Overview

Locust uses a unified configuration system that supports:

- **TOML and JSON formats** - Human-readable configuration files
- **Per-plugin configuration** - Each plugin can have its own settings
- **Runtime updates** - Change configuration without restarting
- **Hot reload** - Automatically detect file changes
- **Validation** - Comprehensive error checking with helpful messages
- **Type safety** - Strongly typed configuration with Rust types

## Configuration File Format

Configuration files can be in TOML or JSON format. TOML is recommended for readability.

### Example TOML Configuration

```toml
[global]
enable_logging = true
log_level = "Info"
fps_limit = 60
mouse_support = true

[plugins.nav]
hint_key = 'f'
charset = "asdfghjkl"
min_target_area = 4
max_hints = 100

[plugins.omnibar]
activation_key = 'p'
activation_modifiers = ["Ctrl"]
max_results = 10
fuzzy_threshold = 0.6

[plugins.tooltip]
default_delay_ms = 500
default_position = "Right"
auto_hide_ms = 3000

[plugins.highlight]
default_animation = "Pulse"
dim_opacity = 0.7
border_thickness = 2
```

### Example JSON Configuration

```json
{
  "global": {
    "enable_logging": true,
    "log_level": "Info",
    "fps_limit": 60,
    "mouse_support": true
  },
  "plugins": {
    "nav": {
      "hint_key": "f",
      "charset": "asdfghjkl",
      "min_target_area": 4,
      "max_hints": 100
    }
  }
}
```

## Global Settings

Global settings affect the entire Locust instance.

| Setting | Type | Default | Description |
|---------|------|---------|-------------|
| `enable_logging` | `bool` | `false` | Enable logging to stderr |
| `log_level` | `LogLevel` | `Info` | Logging level (Trace, Debug, Info, Warn, Error) |
| `fps_limit` | `Option<u32>` | `Some(60)` | Frame rate limit (None = unlimited) |
| `mouse_support` | `bool` | `true` | Enable mouse support for hover interactions |

### Log Levels

- **Trace** - Most verbose, shows all messages
- **Debug** - Debug information for development
- **Info** - Informational messages
- **Warn** - Warnings only
- **Error** - Errors only

## Plugin Configuration

Each plugin can have its own configuration section under `[plugins.<plugin_id>]`.

### Navigation Plugin (nav)

```toml
[plugins.nav]
hint_key = 'f'              # Key to activate hints
charset = "asdfghjkl"       # Characters for hint generation
min_target_area = 4         # Minimum target size (pixels)
max_hints = 100             # Maximum number of hints (0 = unlimited)
```

### Omnibar Plugin (omnibar)

```toml
[plugins.omnibar]
activation_key = 'p'                    # Activation key
activation_modifiers = ["Ctrl"]         # Required modifiers
max_results = 10                        # Maximum results to show
fuzzy_threshold = 0.6                   # Fuzzy matching threshold (0.0-1.0)
```

### Tooltip Plugin (tooltip)

```toml
[plugins.tooltip]
default_delay_ms = 500                  # Hover delay (milliseconds)
default_position = "Right"              # Preferred position (Right, Left, Above, Below, Auto)
auto_hide_ms = 3000                     # Auto-hide timeout (0 = never)
```

### Highlight Plugin (highlight)

```toml
[plugins.highlight]
default_animation = "Pulse"             # Animation type (None, Pulse, Shimmer, Breathe)
dim_opacity = 0.7                       # Dim overlay opacity (0.0-1.0)
border_thickness = 2                    # Border thickness (pixels)
```

## Loading and Saving

### Loading from File

```rust
use locust::core::config::LocustConfig;
use std::path::Path;

// Auto-detect format by extension
let config = LocustConfig::from_file(Path::new("locust.toml"))?;

// Or explicitly
let config = LocustConfig::from_file(Path::new("config.json"))?;
```

### Saving to File

```rust
use locust::core::config::LocustConfig;
use std::path::Path;

let mut config = LocustConfig::new();
config.global.fps_limit = Some(120);

// Save with path
config.save_to(Path::new("locust.toml"))?;

// Or save to remembered path
config.config_path = Some("locust.toml".into());
config.save()?;
```

### Creating Programmatically

```rust
use locust::core::config::*;

let mut config = LocustConfig::new();

// Update global settings
config.global.fps_limit = Some(144);
config.global.enable_logging = true;

// Add plugin configuration
let nav_config = NavConfig {
    hint_key: 'g',
    charset: "abc".to_string(),
    min_target_area: 10,
    max_hints: 50,
};
config.update_plugin("nav", nav_config)?;
```

## Runtime Updates

Configuration can be updated while the application is running.

```rust
use locust::core::config::LocustConfig;
use std::path::Path;

// Load new configuration
let new_config = LocustConfig::from_file(Path::new("locust.toml"))?;

// Update Locust instance
locust.update_config(new_config)?;
// This automatically calls reload_config() on all plugins
```

### Plugin Hot Reload

Plugins can react to configuration changes by implementing the `reload_config` method:

```rust
use locust::prelude::*;

impl<B: Backend> LocustPlugin<B> for MyPlugin {
    fn reload_config(&mut self, ctx: &LocustContext) {
        if let Some(config) = ctx.get_plugin_config::<MyPluginConfig>("my_plugin") {
            // Update internal state based on new config
            self.setting = config.setting;
        }
    }
}
```

## Hot Reload

The `ConfigWatcher` can detect file changes for automatic reloading.

```rust
use locust::core::config::ConfigWatcher;
use std::path::PathBuf;

let mut watcher = ConfigWatcher::new(PathBuf::from("locust.toml"));

// In your event loop
if watcher.check_for_changes() {
    // File has changed, reload configuration
    let new_config = LocustConfig::from_file(watcher.path())?;
    locust.update_config(new_config)?;
}
```

### Example Hot Reload Loop

```rust
use std::time::{Duration, Instant};

let mut watcher = ConfigWatcher::new(config_path);
let mut last_check = Instant::now();
let check_interval = Duration::from_secs(1);

loop {
    // ... handle events ...

    if last_check.elapsed() >= check_interval {
        if watcher.check_for_changes() {
            match LocustConfig::from_file(&config_path) {
                Ok(new_config) => {
                    locust.update_config(new_config)?;
                    eprintln!("Configuration reloaded");
                }
                Err(e) => eprintln!("Failed to reload config: {}", e),
            }
        }
        last_check = Instant::now();
    }
}
```

## Validation

Configuration is validated automatically when loaded or updated.

### Validation Errors

```rust
let mut config = LocustConfig::new();
config.global.fps_limit = Some(0);  // Invalid!

let errors = config.validate();
for error in errors {
    match error.severity {
        Severity::Error => eprintln!("ERROR: {} - {}", error.field, error.message),
        Severity::Warning => eprintln!("WARNING: {} - {}", error.field, error.message),
    }
}
```

### Common Validation Rules

| Rule | Severity | Description |
|------|----------|-------------|
| FPS limit > 0 | Error | FPS must be positive if set |
| FPS limit ≤ 240 | Warning | High FPS may impact performance |
| Charset not empty | Error | Nav plugin requires non-empty charset |
| Min target area ≥ 1 | Warning | Very small targets may be hard to click |

## Examples

### Complete Configuration File

See `locust.example.toml` in the repository root for a complete, commented example.

### Minimal Configuration

```toml
[global]
fps_limit = 60

[plugins.nav]
hint_key = 'f'
charset = "asdf"
```

### High Performance Configuration

```toml
[global]
fps_limit = 144
mouse_support = true

[plugins.nav]
max_hints = 50  # Limit hints for better performance

[plugins.omnibar]
max_results = 5  # Reduce results for faster rendering
```

### Accessibility Configuration

```toml
[global]
mouse_support = true

[plugins.tooltip]
default_delay_ms = 200       # Faster tooltip display
auto_hide_ms = 0             # Never auto-hide

[plugins.nav]
charset = "123456789"        # Numbers instead of letters
```

### Development Configuration

```toml
[global]
enable_logging = true
log_level = "Debug"
fps_limit = 30               # Lower FPS for easier debugging

[plugins.nav]
max_hints = 10               # Fewer hints for clarity
```

## Error Handling

Configuration errors are well-typed and include context:

```rust
use locust::core::config::{ConfigError, LocustConfig};

match LocustConfig::from_file(path) {
    Ok(config) => { /* use config */ }
    Err(ConfigError::Io { path, source }) => {
        eprintln!("Failed to read {}: {}", path.display(), source);
    }
    Err(ConfigError::ParseToml { path, source }) => {
        eprintln!("TOML parse error in {}: {}", path.display(), source);
    }
    Err(ConfigError::ParseJson { path, source }) => {
        eprintln!("JSON parse error in {}: {}", path.display(), source);
    }
    Err(e) => eprintln!("Configuration error: {}", e),
}
```

## Best Practices

1. **Use TOML for readability** - Comments and structure make it easier to maintain
2. **Validate early** - Check configuration before starting the application
3. **Set reasonable defaults** - Don't require configuration for basic usage
4. **Document custom plugins** - Include example configuration in plugin docs
5. **Use hot reload for development** - Faster iteration without restarting
6. **Version your config files** - Track configuration changes in version control
7. **Test edge cases** - Validate unusual configurations in tests

## See Also

- [Plugin Development Guide](PLUGINS.md) - How to make plugins configurable
- [Examples](../examples/config_demo.rs) - Working code examples
- [API Documentation](https://docs.rs/locust) - Full API reference
