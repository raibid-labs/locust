# Locust 🦗

[![Crates.io](https://img.shields.io/crates/v/locust.svg)](https://crates.io/crates/locust)
[![Documentation](https://docs.rs/locust/badge.svg)](https://docs.rs/locust)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Build Status](https://github.com/raibid-labs/locust/workflows/CI/badge.svg)](https://github.com/raibid-labs/locust/actions)

**Plugin-based overlay framework for ratatui with Vimium-style navigation and overlay management**

Locust extends [ratatui](https://github.com/ratatui-org/ratatui) terminal UIs with powerful overlay capabilities:

- 🎯 **Vimium-style Navigation** - Keyboard-driven hint-based UI element selection
- 🔍 **Command Palette** - Omnibar for quick command execution
- 💡 **Tooltips & Tours** - Contextual help and onboarding overlays
- 🔌 **Plugin Architecture** - Extensible system for custom overlay behaviors
- 🎨 **Zero Configuration** - Drop into existing ratatui apps with minimal changes

## Table of Contents

- [Features](#features)
- [Quick Start](#quick-start)
- [Installation](#installation)
- [Usage](#usage)
- [Architecture](#architecture)
- [Plugin Development](#plugin-development)
- [Examples](#examples)
- [Roadmap](#roadmap)
- [Contributing](#contributing)
- [License](#license)

## Features

### Navigation Plugin

Vimium-style hint-based navigation for terminal UIs:

- **Hint Mode**: Press a key to show hints on all interactive elements
- **Target Selection**: Type hint characters to select and activate UI elements
- **Ratatui Integration**: Works with `List`, `Table`, `Tabs`, and custom widgets
- **Customizable**: Configure hint style, keybindings, and behavior

### Omnibar (Coming in Phase 2)

Command palette for quick actions:

- **Fuzzy Search**: Find commands and items quickly
- **Contextual**: Show relevant commands based on current UI state
- **Extensible**: Register custom commands and handlers

### Tooltip System (Coming in Phase 3)

Contextual help overlays:

- **Smart Positioning**: Automatically position tooltips near target elements
- **Rich Content**: Support for formatted text and multi-line tooltips
- **Event-driven**: Show tooltips on hover, focus, or custom triggers

## Quick Start

```rust
use locust::prelude::*;
use locust::plugins::nav::NavPlugin;
use ratatui::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create Locust instance
    let mut locust = Locust::new(LocustConfig::default());

    // Register navigation plugin
    locust.register_plugin(NavPlugin::new());

    // Your ratatui event loop
    loop {
        // Pre-handle events through Locust
        let outcome = locust.on_event(&event);
        if !outcome.consumed {
            // Handle in your app
            handle_app_event(&event);
        }

        // Draw UI
        terminal.draw(|f| {
            // Your app rendering
            app.render(f);

            // Locust overlays on top
            locust.render_overlay(f);
        })?;
    }

    Ok(())
}
```

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
locust = "0.1"
ratatui = "0.28"
crossterm = "0.27"
```

## Usage

### Basic Integration

Locust integrates with ratatui at two points:

1. **Event Loop**: Pre-process events before your app
2. **Draw Loop**: Render overlays on top of your UI

```rust
use locust::{Locust, LocustConfig};
use locust::plugins::nav::NavPlugin;

// Initialize
let mut locust = Locust::new(LocustConfig::default());
locust.register_plugin(NavPlugin::new());

// Event handling
let outcome = locust.on_event(&event);
if !outcome.consumed {
    // Your app's event handling
}

// Rendering
terminal.draw(|f| {
    // Your app's UI
    app.render(f);

    // Locust overlays
    locust.render_overlay(f);
})?;
```

### Navigation Plugin

Enable Vimium-style navigation:

```rust
use locust::plugins::nav::NavPlugin;

let mut locust = Locust::new(LocustConfig::default());

// Register navigation with default config
locust.register_plugin(NavPlugin::new());

// Or customize
let nav_config = NavConfig {
    hint_key: KeyCode::Char('f'),
    hint_chars: "asdfghjkl".to_string(),
    ..Default::default()
};
locust.register_plugin(NavPlugin::with_config(nav_config));
```

### Registering Navigation Targets

Tell Locust which UI elements are navigable:

```rust
// Manual target registration
locust.begin_frame();
locust.register_target(NavTarget {
    id: "button_1".to_string(),
    area: button_rect,
    action: TargetAction::Select,
});

// Or use ratatui adapters (coming in Phase 1)
use locust::ratatui_ext::ListExt;

let list = List::new(items)
    .with_nav_targets(&mut locust); // Automatic target registration
```

## Architecture

Locust uses a plugin-based architecture:

```
┌─────────────────────────────────────┐
│  Your ratatui Application          │
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

**Core Components**:

- **Locust**: Central orchestrator managing plugins and context
- **LocustContext**: Shared state (targets, flags) accessible to all plugins
- **LocustPlugin**: Trait for implementing custom overlay behaviors
- **TargetRegistry**: Collection of navigable UI elements

See [ARCHITECTURE.md](docs/ARCHITECTURE.md) for details.

## Plugin Development

Create custom plugins by implementing `LocustPlugin<B>`:

```rust
use locust::core::{LocustPlugin, LocustContext, PluginEventResult};
use crossterm::event::Event;
use ratatui::{Frame, backend::Backend};

pub struct TooltipPlugin {
    active: bool,
    content: String,
}

impl<B: Backend> LocustPlugin<B> for TooltipPlugin {
    fn id(&self) -> &'static str {
        "tooltip"
    }

    fn on_event(&mut self, event: &Event, ctx: &mut LocustContext)
        -> PluginEventResult
    {
        // Handle events
        PluginEventResult::NotHandled
    }

    fn render_overlay(&self, frame: &mut Frame<'_, B>, ctx: &LocustContext) {
        if self.active {
            // Render tooltip
        }
    }
}
```

Register your plugin:

```rust
locust.register_plugin(TooltipPlugin::new());
```

See [PLUGINS.md](docs/PLUGINS.md) for plugin development guide.

## Examples

### Basic Navigation

```bash
cargo run --example basic_nav
```

Demonstrates:
- Navigation plugin setup
- Hint display and selection
- Event handling

### Multi-pane Dashboard (Coming Soon)

```bash
cargo run --example dashboard
```

Features:
- Multiple panes with independent navigation
- Omnibar for quick pane switching
- Tooltips for interactive elements

### File Browser (Coming Soon)

```bash
cargo run --example file_browser
```

Showcases:
- Hint-based file selection
- Command palette for operations
- Custom plugin integration

## Roadmap

### Phase 0: Scaffold ✅ (Complete)

- [x] Core types: `Locust`, `LocustPlugin`, `LocustContext`
- [x] Event pipeline and overlay rendering
- [x] Basic NavPlugin stub
- [x] Example application

### Phase 1: Real Navigation 🚧 (In Progress)

- [ ] NavTarget actions (select, activate, scroll)
- [ ] Ratatui adapters for List, Table, Tabs
- [ ] Hint generation and input decoding
- [ ] Per-target hint rendering

**Timeline**: 4-6 weeks with parallel development

### Phase 2: Omnibar / Command Palette

- [ ] Input capture and filtering
- [ ] Command registry and dispatch
- [ ] Fuzzy matching
- [ ] Integration with navigation

**Timeline**: 3-4 weeks

### Phase 3: Overlay Ecosystem

- [ ] Tooltip plugin
- [ ] Highlight region plugin (tours)
- [ ] Configuration layer (keymaps, themes)

**Timeline**: 4-5 weeks

### Phase 4: Integration & Documentation

- [ ] Integration patterns documentation
- [ ] Reference examples (dashboard, log viewer, file browser)
- [ ] Performance optimization
- [ ] Comprehensive testing

**Timeline**: 3-4 weeks

**Total Timeline**: ~14-19 weeks (with parallel development: ~8-12 weeks)

See [ROADMAP.md](docs/ROADMAP.md) and [orchestration docs](docs/orchestration/) for detailed workstream breakdown.

## Development

### Prerequisites

- Rust 1.70+
- Cargo

### Building

```bash
# Debug build
cargo build

# Release build
cargo build --release

# Run tests
cargo test

# Run clippy
cargo clippy

# Format code
cargo fmt
```

### Using SPARC Methodology

This project uses SPARC (Specification, Pseudocode, Architecture, Refinement, Completion) for development:

```bash
# Install Claude Flow
npm install -g claude-flow@alpha

# Run SPARC TDD workflow
npx claude-flow sparc tdd "Implement hint generation for List widgets"

# Spawn orchestrators for parallel development
npx claude-flow spawn orchestrator core-framework
```

See [CLAUDE.md](CLAUDE.md) for development workflow details.

## Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for:

- Code of conduct
- Development workflow
- Pull request process
- Coding standards
- Testing guidelines

## Community

- **Issues**: [GitHub Issues](https://github.com/raibid-labs/locust/issues)
- **Discussions**: [GitHub Discussions](https://github.com/raibid-labs/locust/discussions)
- **Documentation**: [docs.rs/locust](https://docs.rs/locust)

## Related Projects

- [ratatui](https://github.com/ratatui-org/ratatui) - Terminal UI library
- [tui-realm](https://github.com/veeso/tui-realm) - Framework for ratatui apps
- [Vimium](https://github.com/philc/vimium) - Browser navigation (inspiration)

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Built on [ratatui](https://github.com/ratatui-org/ratatui)
- Inspired by [Vimium](https://github.com/philc/vimium) browser extension
- Part of the [raibid-labs](https://github.com/raibid-labs) ecosystem

---

**Status**: Phase 0 Complete, Phase 1 In Progress

Made with 🦗 by raibid-labs
