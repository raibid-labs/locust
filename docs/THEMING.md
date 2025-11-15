# Theming Guide

Locust provides a comprehensive theming system that allows you to customize the appearance of your TUI applications.

## Table of Contents

- [Overview](#overview)
- [Built-in Themes](#built-in-themes)
- [Creating Custom Themes](#creating-custom-themes)
- [Color Scheme](#color-scheme)
- [Style Scheme](#style-scheme)
- [Runtime Theme Switching](#runtime-theme-switching)
- [Loading Themes](#loading-themes)
- [Best Practices](#best-practices)

## Overview

Locust's theming system separates colors and styles into reusable components:

- **Color Scheme**: Defines the color palette (background, foreground, semantic colors, etc.)
- **Style Scheme**: Defines text styles with colors and modifiers (bold, italic, etc.)

## Built-in Themes

Locust includes four built-in themes:

### Dark (Default)
A dark theme with blue accents, optimized for reduced eye strain in low-light environments.

### Light
A light theme with dark text, ideal for well-lit environments.

### Solarized Dark
Based on Ethan Schoonover's popular Solarized color scheme, designed for optimal readability.

### Nord
An arctic-inspired color palette with north-bluish tones.

## Creating Custom Themes

Themes are defined in TOML format. Create a new file in the `themes/` directory:

```toml
# themes/my_theme.toml
name = "My Theme"
description = "A custom theme for my application"

[colors]
# Base colors
background = { r = 30, g = 30, b = 30 }
foreground = { r = 220, g = 220, b = 220 }
primary = { r = 100, g = 150, b = 255 }
secondary = { r = 150, g = 100, b = 255 }
accent = { r = 255, g = 200, b = 100 }

# Semantic colors
success = { r = 100, g = 255, b = 100 }
warning = { r = 255, g = 200, b = 100 }
error = { r = 255, g = 100, b = 100 }
info = { r = 100, g = 200, b = 255 }

# UI element colors
border = { r = 80, g = 80, b = 80 }
border_focused = { r = 100, g = 150, b = 255 }
selection = { r = 60, g = 90, b = 150 }
highlight = { r = 255, g = 200, b = 100 }

# Text colors
text_normal = { r = 220, g = 220, b = 220 }
text_muted = { r = 140, g = 140, b = 140 }
text_emphasis = { r = 255, g = 255, b = 255 }

[styles.normal]
fg = { r = 220, g = 220, b = 220 }

[styles.focused]
fg = { r = 100, g = 150, b = 255 }
modifiers = ["bold"]

[styles.selected]
fg = { r = 255, g = 255, b = 255 }
bg = { r = 60, g = 90, b = 150 }

[styles.disabled]
fg = { r = 100, g = 100, b = 100 }
modifiers = ["dim"]

[styles.hint]
fg = { r = 140, g = 140, b = 140 }
modifiers = ["italic"]

[styles.hint_matched]
fg = { r = 100, g = 150, b = 255 }
modifiers = ["bold"]

[styles.tooltip]
fg = { r = 220, g = 220, b = 220 }
bg = { r = 50, g = 50, b = 50 }

[styles.highlight_border]
fg = { r = 255, g = 200, b = 100 }
modifiers = ["bold"]
```

## Color Scheme

### Color Definition

Colors can be defined in two ways:

1. **Named colors**: Use standard terminal color names
   ```toml
   foreground = "blue"
   background = "black"
   ```

2. **RGB colors**: Specify exact RGB values
   ```toml
   foreground = { r = 100, g = 150, b = 255 }
   ```

### Available Named Colors

- `black`, `red`, `green`, `yellow`, `blue`, `magenta`, `cyan`, `white`
- `gray`/`grey`, `darkgray`/`darkgrey`
- `lightred`, `lightgreen`, `lightyellow`, `lightblue`, `lightmagenta`, `lightcyan`

### Color Categories

#### Base Colors
- `background`: Primary background color
- `foreground`: Primary text color
- `primary`: Primary accent color
- `secondary`: Secondary accent color
- `accent`: Highlight accent color

#### Semantic Colors
- `success`: Indicates success states (green)
- `warning`: Indicates warning states (yellow/orange)
- `error`: Indicates error states (red)
- `info`: Indicates informational states (blue)

#### UI Element Colors
- `border`: Default border color
- `border_focused`: Focused element border color
- `selection`: Selected item background
- `highlight`: Highlighted element color

#### Text Colors
- `text_normal`: Normal text
- `text_muted`: De-emphasized text
- `text_emphasis`: Emphasized text

## Style Scheme

Styles combine colors with text modifiers.

### Available Modifiers

- `bold`: Bold text
- `dim`: Dimmed text
- `italic`: Italic text
- `underlined`: Underlined text
- `slowblink`: Slow blinking text
- `rapidblink`: Rapid blinking text
- `reversed`: Reversed foreground/background
- `hidden`: Hidden text
- `crossedout`: Crossed-out text

### Predefined Styles

- `normal`: Default text style
- `focused`: Style for focused elements
- `selected`: Style for selected items
- `disabled`: Style for disabled elements
- `hint`: Style for hint text
- `hint_matched`: Style for matched hint characters
- `tooltip`: Style for tooltips
- `highlight_border`: Style for highlight borders

## Runtime Theme Switching

### In Your Application

```rust
use locust::core::{LocustContext, ThemeManager};

let mut ctx = LocustContext::default();

// Switch to a different theme
ctx.set_theme("Light").unwrap();

// Get the current theme
let theme = ctx.get_theme();

// Use theme colors in your widgets
let style = theme.styles.focused.to_style();
```

### Using ThemeManager

```rust
use locust::core::theme_manager::ThemeManager;
use std::path::Path;

// Load themes from a directory
let mut manager = ThemeManager::load_themes_from_dir(Path::new("themes"))?;

// Switch themes
manager.set_theme("Nord")?;

// List available themes
let themes = manager.list_themes();

// Add a custom theme
manager.add_theme(custom_theme);
```

## Loading Themes

Themes are automatically loaded from the `themes/` directory at application startup.

You can also load themes programmatically:

```rust
use locust::core::theme::Theme;
use std::path::Path;

// Load a single theme
let theme = Theme::from_file(Path::new("themes/my_theme.toml"))?;

// Save a theme
theme.to_file(Path::new("themes/custom.toml"))?;
```

## Best Practices

### 1. Color Contrast

Ensure sufficient contrast between foreground and background colors for readability:
- **Light themes**: Dark text on light backgrounds
- **Dark themes**: Light text on dark backgrounds
- Use WCAG AA guidelines (4.5:1 contrast ratio for normal text)

### 2. Semantic Consistency

Use semantic colors consistently:
- `success`: Green tones for positive actions
- `warning`: Yellow/orange for caution
- `error`: Red tones for errors
- `info`: Blue tones for information

### 3. Accessibility

- Avoid relying solely on color to convey information
- Test themes with color blindness simulators
- Provide sufficient contrast ratios
- Consider using text modifiers (bold, underline) alongside color

### 4. Color Palette Size

Keep your color palette manageable:
- 5-7 base colors
- 4-5 semantic colors
- 2-3 accent colors

### 5. Testing

Test your theme in different terminals:
- Modern terminals (iTerm2, Alacritty, Windows Terminal)
- Classic terminals (xterm, Terminal.app)
- Screen readers and accessibility tools

### 6. Theme Naming

Use descriptive names that indicate:
- Color scheme (dark, light, high-contrast)
- Inspiration (solarized, nord, monokai)
- Purpose (presentation, coding, reading)

## Example: Using Themes in Plugins

```rust
use locust::core::plugin::LocustPlugin;
use ratatui::Frame;

impl<B: Backend> LocustPlugin<B> for MyPlugin {
    fn render_overlay(&self, frame: &mut Frame, ctx: &LocustContext) {
        let theme = ctx.get_theme();

        // Use theme styles
        let border_style = theme.styles.focused.to_style();
        let text_style = theme.styles.normal.to_style();
        let highlight_style = theme.styles.selected.to_style();

        // Use theme colors
        let primary_color = theme.colors.primary.to_color();
        let error_color = theme.colors.error.to_color();

        // Apply to widgets...
    }
}
```

## Troubleshooting

### Colors Don't Match

- Ensure your terminal supports true color (24-bit color)
- Check terminal color settings
- Verify TOML syntax in theme files

### Theme Not Loading

- Check file permissions
- Verify TOML syntax with `toml lint`
- Ensure theme file is in `themes/` directory
- Check for naming conflicts

### Performance Issues

- Themes are cached after loading
- Theme switching is < 5ms overhead
- Use built-in themes for best performance

## Resources

- [Solarized](https://ethanschoonover.com/solarized/)
- [Nord](https://www.nordtheme.com/)
- [Color Contrast Checker](https://webaim.org/resources/contrastchecker/)
- [Color Blindness Simulator](https://www.color-blindness.com/coblis-color-blindness-simulator/)

## Related Documentation

This theming guide connects with other Locust customization documentation:

### Configuration
- **[CONFIGURATION.md](CONFIGURATION.md)** - Configure theme settings
- **[KEYBINDINGS.md](KEYBINDINGS.md)** - Keybinding visual styles

### Plugin Customization
- **[PLUGINS.md](PLUGINS.md)** - Theme plugin overlays
- **[PLUGIN_DEVELOPMENT_GUIDE.md](PLUGIN_DEVELOPMENT_GUIDE.md)** - Create themeable plugins

### Implementation
- **[INTEGRATION_GUIDE.md](INTEGRATION_GUIDE.md)** - Integrate themes into your application
- **[ARCHITECTURE.md](ARCHITECTURE.md#theme-system)** - Theme system architecture
- **[API_PATTERNS.md](API_PATTERNS.md)** - Theme design patterns

### Examples
- **[EXAMPLES.md](EXAMPLES.md)** - Themed example applications
- **[CASE_STUDIES.md](CASE_STUDIES.md)** - Real-world theme usage
- **[TROUBLESHOOTING.md](TROUBLESHOOTING.md#theming-issues)** - Theme troubleshooting

### Project Documentation
- **[README.md](../README.md#theming)** - Theming overview
- **[CONTRIBUTING.md](../CONTRIBUTING.md)** - Contributing themes

---

For more information, see the [Locust Documentation](../README.md).

---

## Related Documentation

- **[Configuration](CONFIGURATION.md)** - Configuration system overview
- **[Keybindings](KEYBINDINGS.md)** - Keybinding customization
- **[Integration Guide](INTEGRATION_GUIDE.md)** - Integrating themes
- **[Plugin Development](PLUGIN_DEVELOPMENT_GUIDE.md)** - Using themes in plugins
- **[Examples](EXAMPLES.md#theme-switcher)** - Theme switcher example

---

*For complete API reference, see [docs.rs/locust](https://docs.rs/locust)*
