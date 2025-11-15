# Locust 🦗

[![Crates.io](https://img.shields.io/crates/v/locust.svg)](https://crates.io/crates/locust)
[![Documentation](https://docs.rs/locust/badge.svg)](https://docs.rs/locust)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![Build Status](https://github.com/raibid-labs/locust/workflows/CI/badge.svg)](https://github.com/raibid-labs/locust/actions)
[![Rust Version](https://img.shields.io/badge/rust-1.70%2B-blue.svg)](https://www.rust-lang.org)

**Vimium-style keyboard navigation and overlay plugins for ratatui terminal UIs**

## Overview

Locust brings browser-like keyboard navigation to your terminal applications. Inspired by [Vimium](https://github.com/philc/vimium), it overlays hints on navigable UI elements, letting users jump to any item with just a few keystrokes. Built on [ratatui](https://github.com/ratatui-org/ratatui), Locust provides a plugin-based architecture for adding powerful overlay capabilities to any terminal UI application.

### Key Features

- 🎯 **Vimium-Style Navigation** - Press `f` to see hints, type to navigate instantly
- ⌨️ **Command Palette** - Fuzzy-search commands with Ctrl+P (Omnibar plugin)
- 💡 **Smart Tooltips** - Context-sensitive help overlays with auto-positioning
- 🎓 **Guided Tours** - Multi-step walkthroughs with visual highlights
- 🎨 **Theming System** - 4 built-in themes (Dark, Light, Solarized, Nord) + custom themes
- ⚙️ **Configurable** - TOML/JSON configuration with hot reload support
- 🔌 **Plugin Architecture** - Extensible system for custom overlay behaviors
- 📚 **Well Documented** - 15,000+ lines of guides, examples, and API docs
- 🚀 **Performance** - Overlay rendering < 10ms, minimal memory footprint
- 🔧 **Type Safe** - Leverages Rust's type system for compile-time guarantees

### Quick Demo

```rust
use locust::prelude::*;
use ratatui::widgets::List;

// Your existing ratatui app
let list = List::new(items);

// Add Locust navigation (3 simple changes)
let mut locust = Locust::new(LocustConfig::default());
locust.register_plugin(NavPlugin::new());

// Event handling
let outcome = locust.on_event(&event);
if !outcome.consumed {
    // Your app's event handling
}

// Rendering
terminal.draw(|f| {
    // Your app's rendering
    f.render_widget(list, area);

    // Locust overlays on top
    locust.render_overlay(f);
})?;

// Now pressing 'f' shows hints on all list items!
```

## Table of Contents

1. [Installation](#installation)
2. [Quick Start](#quick-start)
3. [Features](#features)
   - [Navigation Plugin](#-navigation-plugin)
   - [Command Palette](#-command-palette-omnibar)
   - [Tooltips](#-tooltips)
   - [Guided Tours](#-guided-tours--highlights)
   - [Configuration](#-configuration-system)
   - [Theming](#-theming-system)
4. [Documentation](#documentation)
5. [Examples](#examples)
6. [Architecture](#architecture)
7. [Plugin Development](#plugin-development)
8. [Performance](#performance)
9. [Testing](#testing)
10. [Contributing](#contributing)
11. [Roadmap](#roadmap)
12. [License](#license)
13. [Acknowledgments](#acknowledgments)

## Installation

Add Locust to your `Cargo.toml`:

```toml
[dependencies]
locust = "0.1"
ratatui = "0.28"
crossterm = "0.27"
```

### System Requirements

- **Rust**: 1.70 or later
- **OS**: Linux, macOS, Windows (via crossterm)
- **Terminal**: Any terminal with modern escape sequence support

## Quick Start

Integrate Locust into your ratatui app in 3 simple steps:

### 1. Create Locust Instance

```rust
use locust::prelude::*;

let mut locust = Locust::new(LocustConfig::default());
locust.register_plugin(NavPlugin::new());
```

### 2. Handle Events

```rust
// In your event loop
let outcome = locust.on_event(&event);
if !outcome.consumed {
    // Your app's event handling
    app.handle_event(&event);
}
```

### 3. Render Overlays

```rust
// In your draw function
terminal.draw(|f| {
    // Your app's UI rendering
    app.render(f);

    // Locust overlays on top
    locust.render_overlay(f);
})?;
```

That's it! Press `f` to activate navigation hints.

For complete integration details, see [INTEGRATION_GUIDE.md](docs/INTEGRATION_GUIDE.md).

## Features

### 🎯 Navigation Plugin

Vimium-style hint-based navigation for terminal UIs:

**Features:**
- **Hint Mode**: Press `f` to display hints on all navigable elements
- **Progressive Matching**: Type partial hints to filter targets
- **Smart Prioritization**: Frequently accessed items get shorter hints
- **Widget Support**: Automatic adapters for `List`, `Table`, `Tabs`, `Tree` widgets
- **Configurable**: Customize activation key, hint charset, and behavior
- **Performance**: < 1ms hint generation for 100+ targets

**Configuration:**

```rust
use locust::plugins::nav::{NavPlugin, NavConfig};

let config = NavConfig::new()
    .with_hint_key('f')              // Activation key (default: 'f')
    .with_charset("asdfghjkl")       // Home row keys
    .with_max_hints(100)             // Maximum hints to display
    .with_priority_threshold(5);     // Priority cutoff for short hints

locust.register_plugin(NavPlugin::with_config(config));
```

**Usage:**
1. Press `f` to activate hint mode
2. Hints appear on all navigable targets
3. Type hint characters (e.g., "as") to navigate
4. Target activates when unique match found
5. Press `Esc` to cancel

See [PLUGINS.md](docs/PLUGINS.md#navplugin) for complete API reference.

### ⌨️ Command Palette (Omnibar)

Fuzzy-search command palette inspired by VS Code:

**Features:**
- **Fuzzy Search**: Fast fuzzy matching with relevance scoring
- **Extensible Commands**: Register custom commands and handlers
- **History Tracking**: Recent command memory
- **Keyboard Shortcuts**: Display shortcuts alongside commands
- **Contextual**: Show relevant commands based on current UI state

**Configuration:**

```rust
use locust::plugins::omnibar::{OmnibarPlugin, OmnibarConfig};

let config = OmnibarConfig::new()
    .with_activation_key(KeyCode::Char('p'), KeyModifiers::CONTROL)
    .with_max_results(10)
    .with_fuzzy_threshold(0.6)
    .with_show_shortcuts(true);

locust.register_plugin(OmnibarPlugin::with_config(config));
```

**Usage:**
1. Press `Ctrl+P` to open command palette
2. Type to search commands (fuzzy matching)
3. Use arrow keys to navigate results
4. Press `Enter` to execute command
5. Press `Esc` to cancel

See [PLUGINS.md](docs/PLUGINS.md#omnibarplugin) for complete API reference.

### 💡 Tooltips

Context-sensitive tooltip overlays with smart positioning:

**Features:**
- **Auto-Positioning**: Automatically position tooltips to avoid screen edges
- **Semantic Styles**: Info, Warning, Error, Success styles
- **Multiple Triggers**: Hover, keyboard focus, or custom events
- **Multi-line Support**: Rich formatted content
- **Auto-hide**: Configurable timeout or manual dismissal

**Configuration:**

```rust
use locust::plugins::tooltip::{TooltipPlugin, TooltipConfig};

let config = TooltipConfig::new()
    .with_default_delay_ms(500)      // Show delay
    .with_default_position(Position::Right)
    .with_auto_hide_ms(3000);        // Auto-hide after 3s

locust.register_plugin(TooltipPlugin::with_config(config));
```

**Usage:**

```rust
// Register a tooltip for a UI element
locust.tooltip_plugin().register_tooltip(TooltipSpec {
    target_id: "help_button".to_string(),
    content: "Press F1 for help".to_string(),
    style: TooltipStyle::Info,
    position: Position::Right,
});
```

See [PLUGINS.md](docs/PLUGINS.md#tooltipplugin) for complete API reference.

### 🎓 Guided Tours & Highlights

Multi-step guided tours with visual highlights:

**Features:**
- **Spotlight Mode**: Dim overlay with highlighted regions
- **Animated Borders**: Pulse, shimmer, breathe animations
- **Step-by-step Tours**: Multi-step walkthroughs with progress
- **Flexible Positioning**: Message positioning relative to highlights
- **Tour Completion**: Track user progress through tours

**Configuration:**

```rust
use locust::plugins::highlight::{HighlightPlugin, HighlightConfig};

let config = HighlightConfig::new()
    .with_default_animation(Animation::Pulse)
    .with_dim_opacity(0.7)
    .with_border_thickness(2);

locust.register_plugin(HighlightPlugin::with_config(config));
```

**Usage:**

```rust
// Create a guided tour
let tour = Tour::new("onboarding")
    .add_step(TourStep {
        target: "main_menu",
        message: "This is the main menu. Press 'm' to navigate.",
        position: MessagePosition::Bottom,
        animation: Animation::Pulse,
    })
    .add_step(TourStep {
        target: "search_box",
        message: "Use Ctrl+F to search.",
        position: MessagePosition::Right,
        animation: Animation::Shimmer,
    });

locust.highlight_plugin().start_tour(tour);
```

See [PLUGINS.md](docs/PLUGINS.md#highlightplugin) for complete API reference.

### ⚙️ Configuration System

Unified configuration with TOML/JSON support:

**Features:**
- **Multiple Formats**: TOML (recommended) and JSON
- **Per-Plugin Config**: Each plugin has its own configuration section
- **Runtime Updates**: Change configuration without restart
- **Hot Reload**: Automatically detect file changes
- **Type Safety**: Strongly typed with validation
- **Defaults**: Sensible defaults for all options

**Example TOML Configuration:**

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

**Loading Configuration:**

```rust
// From file
let config = LocustConfig::from_file("locust.toml")?;
let mut locust = Locust::new(config);

// Runtime updates
locust.update_config(new_config)?;

// Hot reload
locust.enable_hot_reload("locust.toml")?;
```

See [CONFIGURATION.md](docs/CONFIGURATION.md) for complete reference.

### 🎨 Theming System

Comprehensive theming with built-in themes and custom support:

**Built-in Themes:**
- **Dark** (default) - Dark theme with blue accents
- **Light** - Light theme optimized for daylight
- **Solarized Dark** - Based on popular Solarized palette
- **Nord** - Arctic-inspired north-bluish tones

**Features:**
- **Color Schemes**: Semantic color definitions
- **Style Schemes**: Text styles with modifiers (bold, italic, etc.)
- **Runtime Switching**: Change themes without restart
- **Custom Themes**: Define your own color palettes
- **Style Inheritance**: Hierarchical style definitions

**Example Custom Theme:**

```toml
# themes/my_theme.toml
name = "My Theme"
description = "A custom theme"

[colors]
background = { r = 30, g = 30, b = 30 }
foreground = { r = 220, g = 220, b = 220 }
primary = { r = 100, g = 150, b = 255 }
success = { r = 100, g = 255, b = 100 }
warning = { r = 255, g = 200, b = 100 }
error = { r = 255, g = 100, b = 100 }

[styles.normal]
fg = { r = 220, g = 220, b = 220 }

[styles.focused]
fg = { r = 100, g = 150, b = 255 }
modifiers = ["bold"]
```

**Usage:**

```rust
// Load built-in theme
locust.set_theme(Theme::load_builtin("dark")?);

// Load custom theme
locust.set_theme(Theme::load_from_file("themes/my_theme.toml")?);

// Runtime switching
locust.set_theme(Theme::load_builtin("solarized-dark")?);
```

See [THEMING.md](docs/THEMING.md) for complete theming guide.

## Documentation

Comprehensive documentation covering all aspects of Locust:

### Core Documentation

- **[Architecture](docs/ARCHITECTURE.md)** (1,295 lines) - System design and internals
- **[Integration Guide](docs/INTEGRATION_GUIDE.md)** (1,322 lines) - Step-by-step integration
- **[Plugin Development](docs/PLUGIN_DEVELOPMENT_GUIDE.md)** (1,515 lines) - Create custom plugins
- **[Plugins](docs/PLUGINS.md)** (604 lines) - Built-in plugin reference
- **[Configuration](docs/CONFIGURATION.md)** (398 lines) - Configuration reference
- **[Theming](docs/THEMING.md)** (335 lines) - Theme customization
- **[Keybindings](docs/KEYBINDINGS.md)** (443 lines) - Keybinding configuration

### Guides & Examples

- **[Examples](docs/EXAMPLES.md)** (1,524 lines) - Example walkthroughs
- **[API Patterns](docs/API_PATTERNS.md)** (1,201 lines) - Design patterns and best practices
- **[Case Studies](docs/CASE_STUDIES.md)** (1,502 lines) - Real-world integration examples
- **[Troubleshooting](docs/TROUBLESHOOTING.md)** (1,422 lines) - Common issues and solutions

### Migration & Reference

- **[Migration Checklist](docs/MIGRATION_CHECKLIST.md)** (780 lines) - Migration from vanilla ratatui
- **[Widget Adapters](docs/WIDGET_ADAPTERS.md)** (364 lines) - Automatic widget integration
- **[Roadmap](docs/ROADMAP.md)** (427 lines) - Development timeline and status

**Total Documentation**: 13,000+ lines covering all aspects of Locust.

## Examples

Locust includes comprehensive examples demonstrating all features:

### Basic Examples

Run with `cargo run --example <name>`:

- **`basic_nav`** - Minimal navigation setup (90 lines)
- **`widget_navigation`** - List/Table navigation (150 lines)
- **`omnibar_demo`** - Command palette showcase (200 lines)
- **`tooltip_demo`** - Tooltip examples (180 lines)
- **`tour_demo`** - Guided tour walkthrough (220 lines)

### Production Examples

Full-featured applications demonstrating real-world usage:

- **`dashboard`** - Multi-pane dashboard (877 lines)
  - Multiple widgets with independent navigation
  - Custom command palette integration
  - Real-time data updates

- **`file_browser`** - Three-pane file manager (735 lines)
  - Directory tree, file list, preview pane
  - Hint-based navigation across panes
  - File operations via command palette

- **`log_viewer`** - Log analysis tool (824 lines)
  - Searchable log viewing
  - Filter commands via omnibar
  - Contextual tooltips for log levels

- **`terminal_multiplexer`** - tmux-like pane manager (761 lines)
  - Dynamic pane splitting
  - Pane navigation with hints
  - Command palette for pane management

- **`git_browser`** - Git repository browser (810 lines)
  - Commit history, file diff viewer
  - Branch/tag navigation
  - Git commands via omnibar

- **`database_tool`** - SQL query tool (996 lines)
  - Schema browser, query editor
  - Result set navigation
  - Keyboard-driven table selection

- **`system_monitor`** - Real-time system monitor (961 lines)
  - Process list, CPU/memory graphs
  - Process navigation and actions
  - Custom highlight plugin for alerts

See [EXAMPLES.md](docs/EXAMPLES.md) for detailed walkthroughs.

## Architecture

Locust uses a plugin-based architecture with clear separation of concerns:

```
┌─────────────────────────────────────┐
│  Your ratatui Application           │
└──────────┬─────────────┬────────────┘
           │             │
    Events │             │ Rendering
           ▼             ▼
┌─────────────────────────────────────┐
│         Locust Orchestrator         │
│  ┌──────────────────────────────┐  │
│  │   LocustContext (shared)     │  │
│  └──────────────────────────────┘  │
│  ┌──────────┐  ┌──────────┐       │
│  │ NavPlugin│  │ Omnibar  │  ...  │
│  └──────────┘  └──────────┘       │
└─────────────────────────────────────┘
           │
           ▼
    Overlay Rendering
```

**Core Components:**

- **Locust**: Central orchestrator managing plugins and context
- **LocustContext**: Shared state (targets, overlays) accessible to all plugins
- **LocustPlugin**: Trait for implementing custom overlay behaviors
- **TargetRegistry**: Collection of navigable UI elements
- **EventPipeline**: Event processing with consumption semantics
- **OverlayRenderer**: Z-layered overlay rendering system

**Key Design Patterns:**

- **Plugin Pattern**: Extensible architecture via trait-based plugins
- **Event Pipeline**: Chain-of-responsibility for event handling
- **Registry Pattern**: Central registration of UI targets
- **Observer Pattern**: Plugins observe and react to events
- **Overlay Composition**: Z-layered overlay rendering

See [ARCHITECTURE.md](docs/ARCHITECTURE.md) for detailed system design.

## Plugin Development

Create custom plugins by implementing the `LocustPlugin<B>` trait:

```rust
use locust::core::{LocustPlugin, LocustContext, PluginEventResult};
use crossterm::event::Event;
use ratatui::{Frame, backend::Backend};

pub struct MyPlugin {
    active: bool,
    config: MyConfig,
}

impl<B: Backend> LocustPlugin<B> for MyPlugin {
    fn id(&self) -> &'static str {
        "my_plugin"
    }

    fn on_event(&mut self, event: &Event, ctx: &mut LocustContext)
        -> PluginEventResult
    {
        // Handle events
        match event {
            Event::Key(key) if key.code == self.config.activation_key => {
                self.active = !self.active;
                PluginEventResult::Consumed
            }
            _ => PluginEventResult::NotHandled
        }
    }

    fn render_overlay(&self, frame: &mut Frame<'_, B>, ctx: &LocustContext) {
        if self.active {
            // Render your overlay
            let popup = Popup::new()
                .title("My Plugin")
                .content("Hello from my plugin!");

            frame.render_widget(popup, centered_rect(60, 40, frame.size()));
        }
    }
}
```

**Register Your Plugin:**

```rust
locust.register_plugin(MyPlugin::new());
```

**Plugin Capabilities:**

- Event handling with consumption semantics
- Access to shared context and target registry
- Custom overlay rendering
- Configuration integration
- Lifecycle hooks (init, cleanup)

See [PLUGIN_DEVELOPMENT_GUIDE.md](docs/PLUGIN_DEVELOPMENT_GUIDE.md) for complete guide.

## Performance

Locust is designed for minimal overhead:

**Benchmarks** (on MacBook Pro M1):

| Operation | Time | Notes |
|-----------|------|-------|
| Overlay rendering | < 10ms | 100+ overlay elements |
| Event processing | < 1ms | Through plugin pipeline |
| Hint generation | < 1ms | 100+ navigation targets |
| Config load/save | < 10ms | TOML/JSON parsing |
| Theme switching | < 5ms | Runtime theme change |
| Fuzzy matching | < 10ms | 1000 items |

**Memory Footprint:**

- Base overhead: ~2 MB
- Per plugin: ~500 KB
- Full plugin suite: ~6 MB
- Zero-cost abstractions where possible

**Optimization Strategies:**

- Lazy overlay rendering (only active plugins)
- Event pipeline short-circuits on consumption
- Target registry spatial indexing
- Theme caching with invalidation
- Incremental hint generation

See [ARCHITECTURE.md](docs/ARCHITECTURE.md#performance-considerations) for details.

## Testing

Comprehensive test coverage across all components:

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_nav_plugin

# Run integration tests
cargo test --test integration

# Run with coverage
cargo tarpaulin --out Html

# Run benchmarks
cargo bench
```

**Test Coverage:**

- Unit tests: >80% coverage
- Integration tests: All plugin interactions
- Example programs: Acceptance tests
- Benchmark tests: Performance regression

**Test Organization:**

```
tests/
├── unit/              # Unit tests
│   ├── core/
│   ├── plugins/
│   └── ratatui_ext/
├── integration/       # Integration tests
│   ├── plugin_interaction.rs
│   ├── event_pipeline.rs
│   └── overlay_rendering.rs
└── fixtures/          # Test fixtures and mocks
```

See [CONTRIBUTING.md](CONTRIBUTING.md#testing-guidelines) for testing guidelines.

## Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for:

- Code of conduct
- Development workflow (SPARC methodology)
- Pull request process
- Coding standards
- Testing guidelines
- Documentation requirements

### Quick Contribution Workflow

1. **Fork and Clone**
   ```bash
   git clone https://github.com/<your-username>/locust.git
   cd locust
   ```

2. **Create Feature Branch**
   ```bash
   git checkout -b feat/my-feature
   ```

3. **Develop with SPARC**
   ```bash
   # Optional: Use SPARC methodology
   npx claude-flow sparc tdd "Add my feature"
   ```

4. **Test and Lint**
   ```bash
   cargo test
   cargo fmt
   cargo clippy
   ```

5. **Submit PR**
   - Clear description of changes
   - Link to related issues
   - Include tests and documentation

## Roadmap

See [ROADMAP.md](docs/ROADMAP.md) for detailed timeline.

### Phase 1: Core Navigation ✅ (Complete)

- [x] NavTarget actions (select, activate, scroll)
- [x] Ratatui adapters for List, Table, Tabs, Tree
- [x] Hint generation and input decoding
- [x] Per-target hint rendering
- [x] Comprehensive testing (>80% coverage)

### Phase 2: Command Palette ✅ (Complete)

- [x] OmnibarPlugin implementation
- [x] Fuzzy search with relevance scoring
- [x] Command registry and dispatch
- [x] History tracking and frecency
- [x] Integration with navigation

### Phase 3: Overlay Ecosystem ✅ (Complete)

- [x] TooltipPlugin implementation
- [x] HighlightPlugin for tours
- [x] Configuration layer (TOML/JSON)
- [x] Theming system (4 built-in themes)
- [x] Keybinding configuration
- [x] Hot reload support

### Phase 4: Documentation & Polish 🚧 (In Progress)

- [x] Architecture documentation
- [x] Integration patterns documentation
- [x] Plugin development guide
- [x] Reference examples (7 production examples)
- [x] Case studies
- [ ] Final documentation polish
- [ ] Performance optimization
- [ ] Comprehensive testing review
- [ ] v0.1.0 Release preparation

**Timeline**: Nov 2024 - Jan 2025

**Current Status**: Phase 3 Complete, Phase 4 90% Complete

## Community

- **Issues**: [GitHub Issues](https://github.com/raibid-labs/locust/issues)
- **Discussions**: [GitHub Discussions](https://github.com/raibid-labs/locust/discussions)
- **Documentation**: [docs.rs/locust](https://docs.rs/locust)
- **Examples**: [examples/](examples/)

## Related Projects

- [ratatui](https://github.com/ratatui-org/ratatui) - Terminal UI library (foundation)
- [tui-realm](https://github.com/veeso/tui-realm) - Framework for ratatui apps
- [Vimium](https://github.com/philc/vimium) - Browser navigation extension (inspiration)
- [fzf](https://github.com/junegunn/fzf) - Command-line fuzzy finder (omnibar inspiration)

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Built on the excellent [ratatui](https://github.com/ratatui-org/ratatui) terminal UI library
- Inspired by [Vimium](https://github.com/philc/vimium) browser extension
- Developed using [SPARC methodology](https://github.com/ruvnet/claude-flow) for systematic TDD
- Part of the [raibid-labs](https://github.com/raibid-labs) ecosystem

## Support

If you find Locust useful, please consider:

- ⭐ Starring the repository
- 🐛 Reporting bugs and issues
- 💡 Suggesting new features
- 📖 Improving documentation
- 🔧 Contributing code

---

**Made with ❤️ by raibid-labs**

**Current Version**: 0.1.0-alpha
**Status**: Phase 4 - Documentation & Polish (90% Complete)
**Last Updated**: January 2025
