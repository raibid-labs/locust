# Locust Development Justfile
# Usage: just <command>
# List all commands: just --list

# Default recipe - show available commands
default:
    @just --list

# Build the project
build:
    cargo build

# Build in release mode
build-release:
    cargo build --release

# Run all tests
test:
    cargo test --all-features

# Run unit tests only
test-unit:
    cargo test --lib --all-features

# Run integration tests only
test-integration:
    cargo test --test '*' --all-features

# Run a specific test by name
test-one TEST:
    cargo test {{TEST}} --all-features -- --nocapture

# Run all benchmarks
bench:
    cargo bench

# Run a specific benchmark
bench-one BENCH:
    cargo bench --bench {{BENCH}}

# Generate and open test coverage report
coverage:
    cargo llvm-cov --all-features --workspace --open

# Generate coverage in lcov format
coverage-lcov:
    cargo llvm-cov --all-features --workspace --lcov --output-path lcov.info

# Run clippy lints
lint:
    cargo clippy --all-features --all-targets -- -D warnings

# Format code
fmt:
    cargo fmt --all

# Check formatting without modifying files
fmt-check:
    cargo fmt --all -- --check

# Run cargo check
check:
    cargo check --all-features --all-targets

# Build documentation
doc:
    cargo doc --all-features --no-deps --open

# Clean build artifacts
clean:
    cargo clean

# Run a specific example by name
demo EXAMPLE:
    cargo run --example {{EXAMPLE}}

# List all available examples
demos:
    @echo "Available examples:"
    @ls examples/*.rs | sed 's/examples\//  - /' | sed 's/\.rs$//'

# Run the basic example
demo-basic:
    cargo run --example basic_usage

# Run the dashboard example
demo-dashboard:
    cargo run --example dashboard

# Run the file browser example
demo-file-browser:
    cargo run --example file_browser

# Run the terminal multiplexer example
demo-terminal-multiplexer:
    cargo run --example terminal_multiplexer

# Run all examples in sequence
demos-all:
    @echo "Running all examples..."
    @for example in examples/*.rs; do \
        name=$$(basename $$example .rs); \
        echo "\n=== Running $$name ==="; \
        cargo run --example $$name || true; \
    done

# Full CI check (what CI runs)
ci: fmt-check lint test

# Quick development check (fast feedback)
dev: check test-unit

# Pre-commit check
pre-commit: fmt lint test

# Full validation (everything)
all: clean build test lint doc bench coverage-lcov
    @echo "✅ All checks passed!"

# Watch for changes and run tests
watch:
    cargo watch -x 'test --all-features'

# Watch for changes and run a specific test
watch-one TEST:
    cargo watch -x 'test {{TEST}} --all-features -- --nocapture'

# Install development tools
install-tools:
    cargo install cargo-watch
    cargo install cargo-llvm-cov
    rustup component add clippy rustfmt

# Update dependencies
update:
    cargo update

# Audit dependencies for security issues
audit:
    cargo audit

# Run cargo tree to visualize dependencies
tree:
    cargo tree --all-features

# Check for outdated dependencies
outdated:
    cargo outdated

# Generate CHANGELOG from git history
changelog:
    git log --oneline --decorate --color > CHANGELOG.txt
    @echo "CHANGELOG.txt generated"

# Show project statistics
stats:
    @echo "=== Locust Project Statistics ==="
    @echo ""
    @echo "Source code:"
    @find src -name '*.rs' | xargs wc -l | tail -1
    @echo ""
    @echo "Tests:"
    @find tests -name '*.rs' | xargs wc -l | tail -1
    @echo ""
    @echo "Examples:"
    @find examples -name '*.rs' | xargs wc -l | tail -1
    @echo ""
    @echo "Documentation:"
    @find docs -name '*.md' | xargs wc -l | tail -1
    @echo ""
    @echo "Total Rust files:"
    @find . -name '*.rs' -not -path './target/*' | wc -l

# Prepare for release
release-prepare VERSION:
    @echo "Preparing release {{VERSION}}..."
    sed -i '' 's/^version = .*/version = "{{VERSION}}"/' Cargo.toml
    @echo "Updated Cargo.toml to version {{VERSION}}"
    @echo "Next steps:"
    @echo "  1. Review changes: git diff"
    @echo "  2. Commit: git commit -am 'chore: Bump version to {{VERSION}}'"
    @echo "  3. Tag: git tag -a v{{VERSION}} -m 'Release v{{VERSION}}'"
    @echo "  4. Push: git push && git push --tags"

# Publish to crates.io (dry run)
publish-dry:
    cargo publish --dry-run --allow-dirty

# Publish to crates.io
publish:
    cargo publish

# Show git status
status:
    git status

# Quick commit with message
commit MSG:
    git add -A
    git commit -m "{{MSG}}"

# Quick commit and push
push MSG:
    git add -A
    git commit -m "{{MSG}}"
    git push

# Performance profiling with flamegraph
flamegraph EXAMPLE:
    cargo flamegraph --example {{EXAMPLE}}

# Memory profiling with valgrind (Linux only)
valgrind EXAMPLE:
    cargo build --example {{EXAMPLE}}
    valgrind --leak-check=full --show-leak-kinds=all ./target/debug/examples/{{EXAMPLE}}

# Run examples with timing
time-demo EXAMPLE:
    @echo "Timing example: {{EXAMPLE}}"
    time cargo run --example {{EXAMPLE}}

# Check for unused dependencies
unused-deps:
    cargo +nightly udeps --all-targets

# Verify all documentation links work
doc-check:
    cargo doc --all-features --no-deps
    @echo "Documentation built successfully"

# Run examples in release mode for performance testing
demo-release EXAMPLE:
    cargo run --release --example {{EXAMPLE}}

# Create a new example file
new-example NAME:
    @echo "use locust::prelude::*;" > examples/{{NAME}}.rs
    @echo "use ratatui::{backend::TestBackend, Terminal};" >> examples/{{NAME}}.rs
    @echo "" >> examples/{{NAME}}.rs
    @echo "fn main() -> anyhow::Result<()> {" >> examples/{{NAME}}.rs
    @echo "    // TODO: Implement {{NAME}} example" >> examples/{{NAME}}.rs
    @echo "    Ok(())" >> examples/{{NAME}}.rs
    @echo "}" >> examples/{{NAME}}.rs
    @echo "Created examples/{{NAME}}.rs"

# Run security audit and update
secure:
    cargo audit
    cargo update
    cargo audit

# Complete pre-release checklist
pre-release: clean all ci
    @echo ""
    @echo "✅ Pre-release checklist complete!"
    @echo ""
    @echo "Manual checks:"
    @echo "  [ ] Update CHANGELOG.md"
    @echo "  [ ] Update README.md version references"
    @echo "  [ ] Review all documentation"
    @echo "  [ ] Test examples manually"
    @echo "  [ ] Review git log since last release"
