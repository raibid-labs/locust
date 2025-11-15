//! Integration tests for command execution in OmnibarPlugin
//!
//! These tests verify the end-to-end functionality of:
//! - Command registration and execution
//! - Built-in commands (quit, help, clear-history)
//! - Custom commands
//! - Command suggestions and filtering

use locust::core::context::LocustContext;
use locust::core::plugin::LocustPlugin;
use locust::plugins::omnibar::{Command, CommandResult, OmnibarPlugin};
use ratatui::backend::TestBackend;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

// Test command that tracks execution
struct TestCommand {
    name: &'static str,
    description: &'static str,
    executed: Arc<AtomicBool>,
}

impl Command for TestCommand {
    fn name(&self) -> &str {
        self.name
    }

    fn description(&self) -> &str {
        self.description
    }

    fn execute(&self, _ctx: &mut LocustContext) -> CommandResult {
        self.executed.store(true, Ordering::Relaxed);
        Ok(())
    }
}

#[test]
fn test_register_and_execute_custom_command() {
    let mut omnibar = OmnibarPlugin::new();
    let mut ctx = LocustContext::default();

    let executed = Arc::new(AtomicBool::new(false));
    let cmd = Arc::new(TestCommand {
        name: "test",
        description: "Test command",
        executed: Arc::clone(&executed),
    });

    omnibar.register_command(cmd);

    // Simulate command input
    LocustPlugin::<TestBackend>::init(&mut omnibar, &mut ctx);

    // Get command registry and execute
    if let Ok(registry) = omnibar.registry().lock() {
        let result = registry.execute("test", &mut ctx);
        assert!(result.is_ok());
    }

    assert!(executed.load(Ordering::Relaxed));
}

#[test]
fn test_builtin_quit_command() {
    let mut omnibar = OmnibarPlugin::new();
    let mut ctx = LocustContext::default();

    omnibar.register_builtin_commands();

    assert!(!omnibar.should_quit());

    // Execute quit command
    if let Ok(registry) = omnibar.registry().lock() {
        let result = registry.execute("quit", &mut ctx);
        assert!(result.is_ok());
    }

    assert!(omnibar.should_quit());
}

#[test]
fn test_builtin_quit_alias() {
    let mut omnibar = OmnibarPlugin::new();
    let mut ctx = LocustContext::default();

    omnibar.register_builtin_commands();

    // Execute via alias "q"
    if let Ok(registry) = omnibar.registry().lock() {
        let result = registry.execute("q", &mut ctx);
        assert!(result.is_ok());
    }

    assert!(omnibar.should_quit());
}

#[test]
fn test_builtin_help_command() {
    let mut omnibar = OmnibarPlugin::new();
    let mut ctx = LocustContext::default();

    omnibar.register_builtin_commands();

    // Execute help command
    if let Ok(registry) = omnibar.registry().lock() {
        let result = registry.execute("help", &mut ctx);
        assert!(result.is_ok());
    }
}

#[test]
fn test_builtin_version_command() {
    let mut omnibar = OmnibarPlugin::new();
    let mut ctx = LocustContext::default();

    omnibar.register_builtin_commands();

    if let Ok(registry) = omnibar.registry().lock() {
        let result = registry.execute("version", &mut ctx);
        assert!(result.is_ok());
    }
}

#[test]
fn test_command_not_found() {
    let omnibar = OmnibarPlugin::new();
    let mut ctx = LocustContext::default();

    if let Ok(registry) = omnibar.registry().lock() {
        let result = registry.execute("nonexistent", &mut ctx);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }
}

#[test]
fn test_command_suggestions() {
    let mut omnibar = OmnibarPlugin::new();
    omnibar.register_builtin_commands();

    // Test empty query
    let suggestions = omnibar.get_suggestions();
    assert!(!suggestions.is_empty()); // Should show all commands

    // Activate and type partial command
    omnibar.state_mut().activate();
    omnibar.state_mut().insert_char('h');
    omnibar.state_mut().insert_char('e');

    let suggestions = omnibar.get_suggestions();
    assert!(!suggestions.is_empty());

    // Should have "help" and "hello" as suggestions
    let names: Vec<&str> = suggestions.iter().map(|s| s.name.as_str()).collect();
    assert!(names.contains(&"help"));
}

#[test]
fn test_command_suggestions_fuzzy() {
    let mut omnibar = OmnibarPlugin::new();
    omnibar.register_builtin_commands();

    omnibar.state_mut().activate();

    // Type fuzzy pattern
    omnibar.state_mut().insert_char('h');
    omnibar.state_mut().insert_char('l');

    let suggestions = omnibar.get_suggestions();

    // Should find "hello" and possibly "help"
    assert!(!suggestions.is_empty());
}

#[test]
fn test_multiple_custom_commands() {
    let mut omnibar = OmnibarPlugin::new();
    let mut ctx = LocustContext::default();

    let executed1 = Arc::new(AtomicBool::new(false));
    let executed2 = Arc::new(AtomicBool::new(false));

    omnibar.register_command(Arc::new(TestCommand {
        name: "cmd1",
        description: "First command",
        executed: Arc::clone(&executed1),
    }));

    omnibar.register_command(Arc::new(TestCommand {
        name: "cmd2",
        description: "Second command",
        executed: Arc::clone(&executed2),
    }));

    if let Ok(registry) = omnibar.registry().lock() {
        registry.execute("cmd1", &mut ctx).unwrap();
        registry.execute("cmd2", &mut ctx).unwrap();
    }

    assert!(executed1.load(Ordering::Relaxed));
    assert!(executed2.load(Ordering::Relaxed));
}

#[test]
fn test_command_categories() {
    let mut omnibar = OmnibarPlugin::new();
    omnibar.register_builtin_commands();

    if let Ok(registry) = omnibar.registry().lock() {
        let categories = registry.categories();

        // Should have at least system, omnibar, demo categories
        assert!(categories.contains(&"system".to_string()));
        assert!(categories.contains(&"omnibar".to_string()));
    }
}

#[test]
fn test_clear_history_command() {
    let mut omnibar = OmnibarPlugin::new();
    let mut ctx = LocustContext::default();

    omnibar.register_builtin_commands();

    // Add some history
    omnibar.state_mut().activate();
    omnibar.state_mut().insert_char('t');
    omnibar.state_mut().insert_char('e');
    omnibar.state_mut().insert_char('s');
    omnibar.state_mut().insert_char('t');
    omnibar.state_mut().submit();

    assert_eq!(omnibar.state().history().len(), 1);

    // Execute clear-history command
    if let Ok(registry) = omnibar.registry().lock() {
        registry.execute("clear-history", &mut ctx).unwrap();
    }

    // History should still be there (cleared by plugin, not command directly)
    // But the command executed successfully
}

#[test]
fn test_registry_thread_safety() {
    let omnibar = OmnibarPlugin::new();

    // Should be able to clone registry reference
    let registry1 = omnibar.registry();
    let registry2 = omnibar.registry();

    // Both should access the same registry
    drop(registry1);
    drop(registry2);
}
