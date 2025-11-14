# Claude Code Configuration - Locust Development

## Project Overview

**Locust** is a plugin-based overlay framework for ratatui (Rust TUI library) that provides Vimium-style navigation, omnibar, tooltips, and other overlay features for terminal user interfaces.

**Language**: Rust 1.70+
**Framework**: ratatui 0.28+
**Development Methodology**: SPARC + Claude Flow orchestration

## Quick Commands

```bash
# Build
cargo build
cargo build --release

# Test
cargo test
cargo test --all-features

# Lint
cargo fmt
cargo clippy

# Run example
cargo run --example basic_nav

# Documentation
cargo doc --open
```

## Project Structure

```
locust/
├── src/
│   ├── core/          # Core framework (Locust, Context, Plugin trait)
│   ├── plugins/       # Built-in plugins (NavPlugin, etc.)
│   └── ratatui_ext/   # Ratatui widget adapters
├── examples/          # Example applications
├── tests/             # Integration tests
├── docs/              # Documentation
│   ├── orchestration/ # Meta-orchestrator and workstream specs
│   ├── ARCHITECTURE.md
│   ├── PLUGINS.md
│   └── ROADMAP.md
└── Cargo.toml
```

## Development Workflow

### SPARC Integration

```bash
# Install Claude Flow
npm install -g claude-flow@alpha

# Run full SPARC workflow for features
npx claude-flow sparc tdd "Implement hint-based navigation"

# Run specific phases
npx claude-flow sparc run spec-pseudocode "Add tooltip plugin"
npx claude-flow sparc run architect "Design omnibar system"
```

### Parallel Development

This project uses meta-orchestrator pattern for coordinated parallel development:

```bash
# Initialize meta orchestrator
npx claude-flow spawn orchestrator meta-locust \
  --spec docs/orchestration/meta-orchestrator.md

# Spawn domain orchestrators
npx claude-flow spawn orchestrator core-framework
npx claude-flow spawn orchestrator plugin-development
npx claude-flow spawn orchestrator integration
```

## Coding Standards

### Rust Guidelines

- Follow [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use `cargo fmt` (enforced in CI)
- Use `cargo clippy` (zero warnings enforced)
- Maximum function length: 50 lines (guidance, not strict)
- Prefer composition over inheritance

### Documentation

```rust
/// Public APIs must have rustdoc comments
///
/// # Examples
///
/// ```
/// use locust::core::Locust;
/// let locust = Locust::new(Default::default());
/// ```
pub struct Locust<B> { /* ... */ }
```

### Testing

```rust
// Unit tests in same file
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature() {
        // Arrange, Act, Assert
    }
}
```

## Orchestration Patterns

### Meta-Orchestrator

Coordinates 3 domain orchestrators across development lifecycle:

1. **Core Framework Orchestrator** - Plugin system, context, events
2. **Plugin Development Orchestrator** - NavPlugin, Omnibar, Tooltips
3. **Integration Orchestrator** - Examples, docs, testing, CI/CD

### Workstreams

Development organized into ~12-15 workstreams aligned with roadmap phases:

- **Phase 1**: Real Navigation (WS-01 to WS-04)
- **Phase 2**: Omnibar/Command Palette (WS-05 to WS-07)
- **Phase 3**: Overlay Ecosystem (WS-08 to WS-11)
- **Phase 4**: Integration & Polish (WS-12 to WS-15)

See `docs/orchestration/` for detailed specifications.

## Git Workflow

### Branches

- `main` - Stable, tested code
- `feat/*` - New features
- `fix/*` - Bug fixes
- `docs/*` - Documentation
- `refactor/*` - Refactoring

### Commits

Use conventional commits:

```
feat(nav): add NavTarget actions for selection
fix(plugin): resolve overlay z-ordering issue
docs(architecture): update plugin registration flow
test(nav): add hint generation unit tests
```

## CI/CD Pipeline

### GitHub Actions

- **Build**: Compile on Linux, macOS, Windows
- **Test**: Run unit and integration tests
- **Lint**: cargo fmt --check, cargo clippy
- **Docs**: Build and deploy rustdoc to GitHub Pages
- **Release**: Automated crates.io publishing on tags

### Quality Gates

- All tests must pass
- Zero clippy warnings
- Code coverage >80%
- Documentation builds successfully

## Plugin Development

### Creating a Plugin

```rust
use locust::core::{LocustPlugin, LocustContext, PluginEventResult};
use crossterm::event::Event;
use ratatui::Frame;

pub struct MyPlugin;

impl<B> LocustPlugin<B> for MyPlugin {
    fn id(&self) -> &'static str {
        "my-plugin"
    }

    fn on_event(&mut self, event: &Event, ctx: &mut LocustContext)
        -> PluginEventResult
    {
        // Handle events
        PluginEventResult::NotHandled
    }

    fn render_overlay(&self, frame: &mut Frame<'_, B>, ctx: &LocustContext) {
        // Render overlay
    }
}
```

### Plugin Guidelines

- Plugins are stateful (can store internal state)
- Use `LocustContext` for cross-plugin shared state
- Event consumption stops propagation (use carefully)
- Overlays compose (render in registration order)

## Roadmap Alignment

Current milestone: **Phase 0 Complete** (Scaffold)

Next priorities:
1. Complete Phase 1 (Real Navigation with hint system)
2. Parallel development of Phase 2 (Omnibar) and Phase 3 (Tooltips)
3. Integration examples and comprehensive documentation

See `docs/ROADMAP.md` and `docs/orchestration/workstream-plan.md` for details.

## raibid-labs Conventions

### File Organization

- No files in root except essential project files
- Documentation in `/docs`
- Tests in `/tests`
- Examples in `/examples`
- Source organized by domain in `/src`

### .gitignore

Excludes Claude Flow artifacts:
- `.swarm/`, `.hive-mind/`, `.claude-flow/`
- `memory/`, `coordination/`
- `.orchestration/` (runtime tracking)

## Resources

- **Repository**: https://github.com/raibid-labs/locust
- **Documentation**: https://raibid-labs.github.io/locust
- **Issues**: https://github.com/raibid-labs/locust/issues
- **Discussions**: https://github.com/raibid-labs/locust/discussions

---

**Development Philosophy**: Modular, composable, well-tested, thoroughly documented.
