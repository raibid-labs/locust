//! Built-in commands for the Omnibar plugin.
//!
//! This module provides standard commands that are useful in most applications:
//! - Quit: Exit the application
//! - Help: Show available commands
//! - ClearHistory: Clear the omnibar command history
//! - Echo: Echo back a message (for testing)

use super::registry::{Command, CommandResult};
use crate::core::context::LocustContext;

/// Command to exit the application.
///
/// This command sets a quit flag that applications can check to initiate
/// a graceful shutdown.
///
/// # Example
///
/// ```rust
/// use locust::plugins::omnibar::commands::QuitCommand;
/// use locust::plugins::omnibar::registry::Command;
/// use std::sync::Arc;
///
/// let cmd = Arc::new(QuitCommand::new());
/// assert_eq!(cmd.name(), "quit");
/// ```
pub struct QuitCommand {
    quit_flag: std::sync::Arc<std::sync::atomic::AtomicBool>,
}

impl QuitCommand {
    /// Creates a new quit command.
    ///
    /// # Arguments
    ///
    /// * `quit_flag` - Shared atomic boolean to set when quitting
    pub fn new(quit_flag: std::sync::Arc<std::sync::atomic::AtomicBool>) -> Self {
        Self { quit_flag }
    }
}

impl Command for QuitCommand {
    fn name(&self) -> &str {
        "quit"
    }

    fn description(&self) -> &str {
        "Exit the application"
    }

    fn category(&self) -> Option<&str> {
        Some("system")
    }

    fn aliases(&self) -> Vec<&str> {
        vec!["q", "exit"]
    }

    fn execute(&self, _ctx: &mut LocustContext) -> CommandResult {
        self.quit_flag
            .store(true, std::sync::atomic::Ordering::Relaxed);
        eprintln!("Locust: Quit command executed");
        Ok(())
    }
}

/// Command to display help information.
///
/// This command shows all available commands and their descriptions.
/// It requires access to the command registry to list commands.
pub struct HelpCommand {
    registry_ref: std::sync::Arc<std::sync::Mutex<super::registry::CommandRegistry>>,
}

impl HelpCommand {
    /// Creates a new help command.
    ///
    /// # Arguments
    ///
    /// * `registry_ref` - Shared reference to the command registry
    pub fn new(
        registry_ref: std::sync::Arc<std::sync::Mutex<super::registry::CommandRegistry>>,
    ) -> Self {
        Self { registry_ref }
    }
}

impl Command for HelpCommand {
    fn name(&self) -> &str {
        "help"
    }

    fn description(&self) -> &str {
        "Show available commands"
    }

    fn category(&self) -> Option<&str> {
        Some("system")
    }

    fn aliases(&self) -> Vec<&str> {
        vec!["?", "h"]
    }

    fn execute(&self, _ctx: &mut LocustContext) -> CommandResult {
        if let Ok(registry) = self.registry_ref.lock() {
            eprintln!("\nAvailable Commands:");
            eprintln!("{}", "=".repeat(50));

            let mut suggestions = registry.search("");
            suggestions.sort_by(|a, b| a.name.cmp(&b.name));

            for suggestion in suggestions {
                let category = suggestion
                    .category
                    .as_ref()
                    .map(|c| format!(" [{}]", c))
                    .unwrap_or_default();
                eprintln!(
                    "  {:12} - {}{}",
                    suggestion.name, suggestion.description, category
                );
            }

            eprintln!("{}", "=".repeat(50));
            Ok(())
        } else {
            Err("Failed to access command registry".to_string())
        }
    }
}

/// Command to clear the omnibar command history.
pub struct ClearHistoryCommand;

impl ClearHistoryCommand {
    /// Creates a new clear history command.
    pub fn new() -> Self {
        Self
    }
}

impl Default for ClearHistoryCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl Command for ClearHistoryCommand {
    fn name(&self) -> &str {
        "clear-history"
    }

    fn description(&self) -> &str {
        "Clear omnibar command history"
    }

    fn category(&self) -> Option<&str> {
        Some("omnibar")
    }

    fn aliases(&self) -> Vec<&str> {
        vec!["ch", "clear"]
    }

    fn execute(&self, _ctx: &mut LocustContext) -> CommandResult {
        // Note: The actual clearing happens in OmnibarPlugin when this command
        // is detected. This is a marker command.
        eprintln!("Locust: Command history cleared");
        Ok(())
    }
}

/// Command to echo a message (for testing and demos).
pub struct EchoCommand;

impl EchoCommand {
    /// Creates a new echo command.
    pub fn new() -> Self {
        Self
    }
}

impl Default for EchoCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl Command for EchoCommand {
    fn name(&self) -> &str {
        "echo"
    }

    fn description(&self) -> &str {
        "Echo back the command (for testing)"
    }

    fn category(&self) -> Option<&str> {
        Some("utility")
    }

    fn aliases(&self) -> Vec<&str> {
        vec!["e"]
    }

    fn execute(&self, _ctx: &mut LocustContext) -> CommandResult {
        eprintln!("Locust: Echo command executed");
        Ok(())
    }
}

/// Command to display version information.
pub struct VersionCommand;

impl VersionCommand {
    /// Creates a new version command.
    pub fn new() -> Self {
        Self
    }
}

impl Default for VersionCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl Command for VersionCommand {
    fn name(&self) -> &str {
        "version"
    }

    fn description(&self) -> &str {
        "Show Locust version information"
    }

    fn category(&self) -> Option<&str> {
        Some("system")
    }

    fn aliases(&self) -> Vec<&str> {
        vec!["v"]
    }

    fn execute(&self, _ctx: &mut LocustContext) -> CommandResult {
        eprintln!("Locust Framework");
        eprintln!("Version: {}", env!("CARGO_PKG_VERSION"));
        eprintln!("A ratatui plugin framework for overlay management");
        Ok(())
    }
}

/// Example custom command for demonstrations.
pub struct HelloCommand;

impl HelloCommand {
    /// Creates a new hello command.
    pub fn new() -> Self {
        Self
    }
}

impl Default for HelloCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl Command for HelloCommand {
    fn name(&self) -> &str {
        "hello"
    }

    fn description(&self) -> &str {
        "Say hello to the world"
    }

    fn category(&self) -> Option<&str> {
        Some("demo")
    }

    fn aliases(&self) -> Vec<&str> {
        vec!["hi", "greet"]
    }

    fn execute(&self, _ctx: &mut LocustContext) -> CommandResult {
        eprintln!("Hello, Locust user! 🦀");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicBool;
    use std::sync::{Arc, Mutex};

    #[test]
    fn test_quit_command() {
        let quit_flag = Arc::new(AtomicBool::new(false));
        let cmd = QuitCommand::new(Arc::clone(&quit_flag));
        let mut ctx = LocustContext::default();

        assert_eq!(cmd.name(), "quit");
        assert!(cmd.aliases().contains(&"q"));
        assert!(!quit_flag.load(std::sync::atomic::Ordering::Relaxed));

        let result = cmd.execute(&mut ctx);
        assert!(result.is_ok());
        assert!(quit_flag.load(std::sync::atomic::Ordering::Relaxed));
    }

    #[test]
    fn test_help_command() {
        let registry = Arc::new(Mutex::new(super::super::registry::CommandRegistry::new()));
        let cmd = HelpCommand::new(Arc::clone(&registry));
        let mut ctx = LocustContext::default();

        assert_eq!(cmd.name(), "help");
        assert!(cmd.aliases().contains(&"?"));

        let result = cmd.execute(&mut ctx);
        assert!(result.is_ok());
    }

    #[test]
    fn test_clear_history_command() {
        let cmd = ClearHistoryCommand::new();
        let mut ctx = LocustContext::default();

        assert_eq!(cmd.name(), "clear-history");
        assert!(cmd.aliases().contains(&"clear"));

        let result = cmd.execute(&mut ctx);
        assert!(result.is_ok());
    }

    #[test]
    fn test_echo_command() {
        let cmd = EchoCommand::new();
        let mut ctx = LocustContext::default();

        assert_eq!(cmd.name(), "echo");
        assert!(cmd.aliases().contains(&"e"));

        let result = cmd.execute(&mut ctx);
        assert!(result.is_ok());
    }

    #[test]
    fn test_version_command() {
        let cmd = VersionCommand::new();
        let mut ctx = LocustContext::default();

        assert_eq!(cmd.name(), "version");
        assert!(cmd.aliases().contains(&"v"));

        let result = cmd.execute(&mut ctx);
        assert!(result.is_ok());
    }

    #[test]
    fn test_hello_command() {
        let cmd = HelloCommand::new();
        let mut ctx = LocustContext::default();

        assert_eq!(cmd.name(), "hello");
        assert!(cmd.aliases().contains(&"hi"));
        assert_eq!(cmd.category(), Some("demo"));

        let result = cmd.execute(&mut ctx);
        assert!(result.is_ok());
    }
}
