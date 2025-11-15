# Contributing to Locust

Thank you for your interest in contributing to Locust! This document provides guidelines and instructions for contributing.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Workflow](#development-workflow)
- [Pull Request Process](#pull-request-process)
- [Coding Standards](#coding-standards)
- [Testing Guidelines](#testing-guidelines)
- [Documentation](#documentation)

## Code of Conduct

This project follows the raibid-labs code of conduct. Be respectful, collaborative, and constructive in all interactions.

## Getting Started

### Prerequisites

- Rust 1.70 or later
- Cargo
- Git

### Setup

```bash
# Clone the repository
git clone https://github.com/raibid-labs/locust.git
cd locust

# Build the project
cargo build

# Run tests
cargo test

# Run the example
cargo run --example basic_nav
```

## Development Workflow

### Using SPARC Methodology

This project uses SPARC (Specification, Pseudocode, Architecture, Refinement, Completion) for systematic Test-Driven Development. SPARC provides a structured approach to feature development with AI assistance.

#### SPARC Phases

1. **Specification** - Define requirements and acceptance criteria
2. **Pseudocode** - Design algorithms and data structures
3. **Architecture** - Plan system integration and component design
4. **Refinement** - TDD implementation with iterative testing
5. **Completion** - Integration testing and documentation

#### Quick Start with SPARC

```bash
# Install Claude Flow (optional but recommended)
npm install -g claude-flow@alpha

# Run complete SPARC TDD workflow for a feature
npx claude-flow sparc tdd "Add tooltip plugin with auto-positioning"

# Run specific SPARC phases
npx claude-flow sparc run spec-pseudocode "Design hint algorithm"
npx claude-flow sparc run architect "Plan plugin architecture"

# View available SPARC modes
npx claude-flow sparc modes
```

#### SPARC Workflow Examples

**Adding a New Plugin:**

```bash
# 1. Specification phase - Define plugin requirements
npx claude-flow sparc run spec-pseudocode "Create notification plugin with toast messages"

# 2. Architecture phase - Design integration points
npx claude-flow sparc run architect "Design notification plugin architecture"

# 3. TDD implementation - Write tests first, then implement
npx claude-flow sparc tdd "Implement notification plugin with configurable duration"

# 4. Integration - Test with existing plugins
npx claude-flow sparc run integration "Integrate notification plugin with existing overlays"
```

**Enhancing Existing Features:**

```bash
# Run parallel batch processing for multiple related tasks
npx claude-flow sparc batch "spec-pseudocode,architect" "Add keyboard shortcuts to omnibar"

# Run full pipeline for complex features
npx claude-flow sparc pipeline "Implement theme hot reload with file watcher"
```

#### Benefits of SPARC

- **Systematic Development**: Structured approach reduces overlooked requirements
- **Test-First**: TDD methodology ensures high test coverage (>80%)
- **Documentation**: Each phase generates documentation artifacts
- **AI Assistance**: Claude Flow provides intelligent code generation
- **Quality Assurance**: Built-in checkpoints for validation

For detailed SPARC methodology, see [CLAUDE.md](CLAUDE.md).

### Branch Strategy

- `main` - Stable, production-ready code
- `feat/*` - New features
- `fix/*` - Bug fixes
- `docs/*` - Documentation improvements
- `refactor/*` - Code refactoring

### Commit Messages

Follow conventional commits format:

```
<type>(<scope>): <description>

[optional body]

[optional footer]
```

Types: `feat`, `fix`, `docs`, `style`, `refactor`, `test`, `chore`

Examples:
- `feat(nav): add hint generation for List widgets`
- `fix(plugin): resolve event consumption race condition`
- `docs(readme): update plugin registration example`

## Pull Request Process

1. **Fork and Branch**: Fork the repository and create a feature branch
2. **Develop**: Make your changes following our coding standards
3. **Test**: Ensure all tests pass and add new tests for your changes
4. **Document**: Update documentation for any API changes
5. **Commit**: Use conventional commit messages
6. **Pull Request**: Open a PR with a clear description

### PR Template

```markdown
## Description
Brief description of changes

## Type of Change
- [ ] Bug fix
- [ ] New feature
- [ ] Breaking change
- [ ] Documentation update

## Testing
- [ ] Unit tests pass
- [ ] Integration tests pass
- [ ] Example programs work
- [ ] Documentation builds

## Checklist
- [ ] Code follows project style guidelines
- [ ] Self-review completed
- [ ] Documentation updated
- [ ] No new warnings
```

## Coding Standards

### Rust Style

- Follow [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use `cargo fmt` for formatting (enforced in CI)
- Use `cargo clippy` for linting (enforced in CI)
- Maximum line length: 100 characters
- Prefer explicit over implicit
- Document all public APIs with rustdoc

### Code Organization

```
src/
├── core/           # Core framework types
├── plugins/        # Built-in plugins
└── ratatui_ext/    # Ratatui extensions
```

### Naming Conventions

- **Types**: `PascalCase` (e.g., `LocustPlugin`, `NavTarget`)
- **Functions**: `snake_case` (e.g., `on_event`, `render_overlay`)
- **Constants**: `SCREAMING_SNAKE_CASE` (e.g., `MAX_TARGETS`)
- **Modules**: `snake_case` (e.g., `nav`, `context`)

## Testing Guidelines

### Test Organization

```
tests/
├── unit/          # Unit tests
├── integration/   # Integration tests
└── fixtures/      # Test fixtures and mocks
```

### Writing Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_consumes_event() {
        let mut plugin = NavPlugin::new();
        let event = Event::Key(/* ... */);
        let result = plugin.on_event(&event, &mut ctx);
        assert!(result.consumed);
    }
}
```

### Test Coverage

- Aim for >80% code coverage
- All public APIs must have tests
- Integration tests for plugin interactions
- Example programs serve as acceptance tests

## Documentation

### Rustdoc

All public items must have documentation:

```rust
/// Represents a navigation target in the overlay.
///
/// # Examples
///
/// ```
/// use locust::core::targets::NavTarget;
///
/// let target = NavTarget::new("a", Rect::default());
/// ```
pub struct NavTarget {
    // ...
}
```

### Architecture Documentation

Update relevant documentation in `docs/`:

- `ARCHITECTURE.md` - System design changes
- `PLUGINS.md` - New plugin patterns or APIs
- `ROADMAP.md` - Milestone progress

### Examples

Provide working examples for new features:

```rust
// examples/my_feature.rs
fn main() -> Result<()> {
    // Demonstrate the feature
}
```

## Plugin Development

### Creating a New Plugin

1. Create plugin module in `src/plugins/your_plugin/`
2. Implement `LocustPlugin<B>` trait
3. Add tests in `tests/plugins/`
4. Document in `docs/PLUGINS.md`
5. Add example in `examples/`

### Plugin Guidelines

- Plugins should be self-contained
- Use `LocustContext` for shared state
- Event consumption should be intentional
- Overlays should be composable
- Provide configuration options

## Release Process

Releases are managed by maintainers:

1. Version bump in `Cargo.toml`
2. Update `CHANGELOG.md`
3. Tag release: `git tag v0.x.0`
4. CI automatically publishes to crates.io

## Getting Help

- **Issues**: Open a GitHub issue for bugs or feature requests
- **Discussions**: Use GitHub Discussions for questions
- **Discord**: Join our Discord server (link in README)

## License

By contributing, you agree that your contributions will be licensed under the MIT License.

## Related Documentation

Before contributing, please familiarize yourself with these key documents:

### Core Documentation
- **[README.md](README.md)** - Project overview and quick start
- **[ARCHITECTURE.md](docs/ARCHITECTURE.md)** - System design and internals
- **[ROADMAP.md](docs/ROADMAP.md)** - Development timeline and priorities

### Development Guides
- **[PLUGIN_DEVELOPMENT_GUIDE.md](docs/PLUGIN_DEVELOPMENT_GUIDE.md)** - Comprehensive plugin development guide
- **[INTEGRATION_GUIDE.md](docs/INTEGRATION_GUIDE.md)** - Integration patterns and examples
- **[API_PATTERNS.md](docs/API_PATTERNS.md)** - Design patterns and best practices

### Reference Documentation
- **[PLUGINS.md](docs/PLUGINS.md)** - Built-in plugin reference
- **[CONFIGURATION.md](docs/CONFIGURATION.md)** - Configuration system reference
- **[THEMING.md](docs/THEMING.md)** - Theming and styling guide
- **[KEYBINDINGS.md](docs/KEYBINDINGS.md)** - Keybinding configuration
- **[WIDGET_ADAPTERS.md](docs/WIDGET_ADAPTERS.md)** - Widget adapter system

### Examples and Troubleshooting
- **[EXAMPLES.md](docs/EXAMPLES.md)** - Detailed example walkthroughs
- **[CASE_STUDIES.md](docs/CASE_STUDIES.md)** - Real-world integration examples
- **[TROUBLESHOOTING.md](docs/TROUBLESHOOTING.md)** - Common issues and solutions
- **[MIGRATION_CHECKLIST.md](docs/MIGRATION_CHECKLIST.md)** - Migration from vanilla ratatui

---

**Thank you for contributing to Locust!**
