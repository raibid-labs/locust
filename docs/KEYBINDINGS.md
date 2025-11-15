# Keybindings Guide

Locust provides a flexible keybinding system that allows you to customize keyboard shortcuts for your TUI applications.

## Table of Contents

- [Overview](#overview)
- [Default Keybindings](#default-keybindings)
- [Custom Keybindings](#custom-keybindings)
- [Keybinding Format](#keybinding-format)
- [Conflict Detection](#conflict-detection)
- [Runtime Keybinding Changes](#runtime-keybinding-changes)
- [Plugin-Specific Bindings](#plugin-specific-bindings)
- [Best Practices](#best-practices)

## Overview

Locust's keybinding system provides:

- **Global keybindings**: Actions available across the entire application
- **Plugin-specific keybindings**: Actions scoped to individual plugins
- **Conflict detection**: Automatic detection of duplicate bindings
- **Runtime rebinding**: Change keybindings while the application is running
- **TOML configuration**: Easy-to-read configuration format

## Default Keybindings

### Global Bindings

| Key | Action | Description |
|-----|--------|-------------|
| `q` | quit | Exit the application |
| `F1` | help | Show help overlay |

### Nav Plugin

| Key | Action | Description |
|-----|--------|-------------|
| `f` | activate | Activate fuzzy navigation |
| `Esc` | cancel | Cancel navigation |

### Omnibar Plugin

| Key | Action | Description |
|-----|--------|-------------|
| `Ctrl+P` | activate | Open omnibar |
| `Esc` | cancel | Close omnibar |

### Tooltip Plugin

| Key | Action | Description |
|-----|--------|-------------|
| `h` | show | Show tooltip |
| `Esc` | hide | Hide tooltip |

### Highlight Plugin

| Key | Action | Description |
|-----|--------|-------------|
| `n` | next_step | Next tour step |
| `p` | previous_step | Previous tour step |
| `s` | skip_tour | Skip the tour |

## Custom Keybindings

Create a custom keymap in TOML format:

```toml
# keymaps/my_keymap.toml

[global]
quit = { key = 'q' }
help = { key = { F = 1 } }
save = { key = 's', modifiers = 1 }  # Ctrl+S

[plugins.nav]
activate = { key = 'f' }
cancel = { key = "esc" }

[plugins.omnibar]
activate = { key = 'p', modifiers = 1 }  # Ctrl+P
cancel = { key = "esc" }

[plugins.custom]
my_action = { key = 'x', modifiers = 2 }  # Alt+X
```

## Keybinding Format

### Key Types

#### Character Keys

```toml
key = 'a'  # Single character
key = 'Z'  # Case-sensitive
```

#### Function Keys

```toml
key = { F = 1 }   # F1
key = { F = 12 }  # F12
```

#### Named Keys

```toml
key = "esc"       # Escape
key = "enter"     # Enter
key = "tab"       # Tab
key = "backspace" # Backspace
key = "delete"    # Delete
key = "insert"    # Insert
key = "home"      # Home
key = "end"       # End
key = "pageup"    # Page Up
key = "pagedown"  # Page Down
key = "left"      # Left Arrow
key = "right"     # Right Arrow
key = "up"        # Up Arrow
key = "down"      # Down Arrow
```

### Modifiers

Modifiers are specified as integer flags:

| Value | Modifier | Description |
|-------|----------|-------------|
| 0 | None | No modifiers |
| 1 | Control | Ctrl key |
| 2 | Alt | Alt key |
| 4 | Shift | Shift key |

Combine modifiers by adding values:
```toml
modifiers = 3  # Ctrl+Alt (1 + 2)
modifiers = 5  # Ctrl+Shift (1 + 4)
```

### Complete Examples

```toml
# Simple character
quit = { key = 'q' }

# Character with modifier
save = { key = 's', modifiers = 1 }  # Ctrl+S

# Function key
help = { key = { F = 1 } }

# Named key
cancel = { key = "esc" }

# Named key with modifier
paste = { key = "insert", modifiers = 4 }  # Shift+Insert
```

## Conflict Detection

Locust automatically detects keybinding conflicts:

```rust
use locust::core::keybindings::{KeyMap, detect_conflicts};

let keymap = KeyMap::from_file("keymaps/my_keymap.toml")?;

// Check for conflicts
let conflicts = detect_conflicts(&keymap);

if !conflicts.is_empty() {
    for conflict in conflicts {
        println!("Key {:?} is bound to multiple actions:", conflict.binding.key);
        for action in conflict.actions {
            println!("  - {}", action);
        }
    }
}

// Or use validate()
if let Err(conflicts) = keymap.validate() {
    // Handle conflicts
}
```

## Runtime Keybinding Changes

### In Your Application

```rust
use locust::core::{LocustContext, KeyBinding, KeyCodeDef};
use crossterm::event::KeyModifiers;

let mut ctx = LocustContext::default();

// Bind a new key
let binding = KeyBinding {
    key: KeyCodeDef::Char('x'),
    modifiers: KeyModifiers::CONTROL,
};
ctx.bind_key("custom.action", binding)?;

// Unbind a key
ctx.unbind_key("custom.action");

// Get current binding
if let Some(binding) = ctx.get_keymap().get_binding("quit") {
    println!("Quit key: {:?}", binding.key);
}
```

### Programmatic Changes

```rust
use locust::core::keybindings::{KeyMap, KeyBinding, KeyCodeDef};

let mut keymap = KeyMap::default();

// Add a global binding
keymap.bind("save", KeyBinding {
    key: KeyCodeDef::Char('s'),
    modifiers: KeyModifiers::CONTROL,
})?;

// Add a plugin binding
keymap.bind("editor.format", KeyBinding {
    key: KeyCodeDef::Char('f'),
    modifiers: KeyModifiers::CONTROL | KeyModifiers::ALT,
})?;

// Remove a binding
keymap.unbind("quit");

// Get action for a binding
let binding = KeyBinding {
    key: KeyCodeDef::Char('q'),
    modifiers: KeyModifiers::empty(),
};
if let Some(action) = keymap.get_action(&binding) {
    println!("Action: {}", action);
}
```

## Plugin-Specific Bindings

Plugins can define their own keybindings:

```rust
use locust::core::plugin::LocustPlugin;
use crossterm::event::{Event, KeyCode, KeyEvent};

impl<B: Backend> LocustPlugin<B> for MyPlugin {
    fn on_event(&mut self, event: &Event, ctx: &mut LocustContext) -> LocustEventOutcome {
        if let Event::Key(key) = event {
            // Check if this key matches the plugin's action
            if let Some(binding) = ctx.get_keymap().get_binding("myplugin.action") {
                if matches_key(key, binding) {
                    // Handle the action
                    return LocustEventOutcome::CONSUMED;
                }
            }
        }
        LocustEventOutcome::NOT_HANDLED
    }
}
```

## Best Practices

### 1. Use Conventional Bindings

Follow common conventions:
- `Ctrl+S`: Save
- `Ctrl+Q`: Quit
- `Ctrl+C`: Copy
- `Ctrl+V`: Paste
- `F1`: Help
- `Esc`: Cancel/Close

### 2. Avoid Conflicts

- Check for conflicts before deploying
- Use plugin namespaces to avoid global conflicts
- Provide fallback bindings if primary binding is taken

### 3. Document Your Keybindings

Always document custom keybindings:
```rust
/// Custom keybindings:
/// - Ctrl+S: Save document
/// - Ctrl+O: Open file
/// - F5: Refresh view
```

### 4. Provide Help Overlay

Show available keybindings in a help screen:
```rust
fn show_help(ctx: &LocustContext) {
    let keymap = ctx.get_keymap();
    for (action, binding) in &keymap.global {
        println!("{}: {:?}", action, binding.key);
    }
}
```

### 5. Platform Considerations

- **macOS**: Consider `Cmd` key alternatives
- **Windows**: Test with different keyboard layouts
- **Linux**: Verify terminal emulator compatibility

### 6. Modifier Usage

- **None**: Frequently used actions
- **Ctrl**: Primary shortcuts
- **Alt**: Alternative actions
- **Shift**: Reverse or opposite actions (e.g., Shift+Tab)

### 7. Ergonomics

Prioritize frequently used actions:
- Place on home row keys (a, s, d, f, j, k, l)
- Avoid awkward key combinations
- Consider left/right hand balance

## Conflict Resolution

If conflicts are detected:

1. **Remove less important binding**:
   ```rust
   keymap.unbind("rarely_used_action");
   ```

2. **Use different modifier**:
   ```toml
   # Instead of Ctrl+P
   activate = { key = 'p', modifiers = 2 }  # Alt+P
   ```

3. **Use different key**:
   ```toml
   # Instead of 'p'
   activate = { key = 'o' }
   ```

4. **Scope to plugin**:
   ```toml
   # Move from global to plugin-specific
   [plugins.myplugin]
   activate = { key = 'p' }
   ```

## Loading Keymaps

Load keymaps from files:

```rust
use locust::core::keybindings::KeyMap;
use std::path::Path;

// Load from file
let keymap = KeyMap::from_file(Path::new("keymaps/default.toml"))?;

// Save to file
keymap.to_file(Path::new("keymaps/custom.toml"))?;

// Use in context
ctx.keymap = keymap;
```

## Advanced Usage

### Conditional Bindings

```rust
fn get_binding_for_mode(mode: &str, ctx: &LocustContext) -> Option<&KeyBinding> {
    match mode {
        "normal" => ctx.get_keymap().get_binding("normal.action"),
        "insert" => ctx.get_keymap().get_binding("insert.action"),
        _ => None,
    }
}
```

### Dynamic Binding Updates

```rust
fn update_bindings_for_plugin(plugin_id: &str, ctx: &mut LocustContext) {
    // Add plugin-specific bindings
    for (action, key) in get_plugin_defaults(plugin_id) {
        let action_name = format!("{}.{}", plugin_id, action);
        ctx.bind_key(&action_name, key).ok();
    }
}
```

## Troubleshooting

### Keybinding Not Working

1. Check for conflicts: `keymap.validate()`
2. Verify TOML syntax
3. Ensure plugin is loaded
4. Check terminal key capture

### Wrong Key Detected

- Some terminals intercept certain keys (Ctrl+S, Ctrl+Q)
- Test in different terminals
- Use alternative bindings for problematic keys

### Modifiers Not Working

- Verify modifier values (1=Ctrl, 2=Alt, 4=Shift)
- Check terminal modifier support
- Test with simple bindings first

## Examples

See the `examples/custom_keybindings.rs` file for a complete demonstration of:
- Loading keymaps
- Detecting conflicts
- Adding/removing bindings
- Displaying current bindings

Run the example:
```bash
cargo run --example custom_keybindings
```

## Resources

- [Crossterm Key Events](https://docs.rs/crossterm/latest/crossterm/event/struct.KeyEvent.html)
- [Ratatui Input Handling](https://ratatui.rs/how-to/input/)

## Related Documentation

This keybindings guide connects with other Locust configuration documentation:

### Configuration
- **[CONFIGURATION.md](CONFIGURATION.md)** - General configuration system
- **[THEMING.md](THEMING.md)** - Visual styling for keybindings
- **[PLUGINS.md](PLUGINS.md)** - Plugin keybinding configuration

### Implementation
- **[INTEGRATION_GUIDE.md](INTEGRATION_GUIDE.md)** - Integrate keybindings into your app
- **[PLUGIN_DEVELOPMENT_GUIDE.md](PLUGIN_DEVELOPMENT_GUIDE.md)** - Custom plugin keybindings
- **[ARCHITECTURE.md](ARCHITECTURE.md#keybinding-system)** - Keybinding architecture

### Examples
- **[EXAMPLES.md](EXAMPLES.md)** - Keybinding usage examples
- **[CASE_STUDIES.md](CASE_STUDIES.md)** - Real-world keybinding configurations
- **[API_PATTERNS.md](API_PATTERNS.md)** - Keybinding design patterns

### Troubleshooting
- **[TROUBLESHOOTING.md](TROUBLESHOOTING.md#keybinding-issues)** - Keybinding troubleshooting

### Project Documentation
- **[README.md](../README.md#keybindings)** - Keybindings overview
- **[CONTRIBUTING.md](../CONTRIBUTING.md)** - Contributing keybinding features

---

For more information, see the [Locust Documentation](../README.md).

---

## Related Documentation

- **[Configuration](CONFIGURATION.md)** - Configuration system overview
- **[Theming](THEMING.md)** - Theme customization
- **[Integration Guide](INTEGRATION_GUIDE.md)** - Integrating keybindings
- **[Plugin Development](PLUGIN_DEVELOPMENT_GUIDE.md)** - Plugin keybindings
- **[Troubleshooting](TROUBLESHOOTING.md#keybinding-conflicts)** - Conflict resolution

---

*For complete API reference, see [docs.rs/locust](https://docs.rs/locust)*
