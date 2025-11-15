//! Unit tests for CommandRegistry
//!
//! These tests verify the command registry functionality including:
//! - Command registration and unregistration
//! - Command lookup and execution
//! - Alias handling
//! - Search and filtering
//! - Category management

use locust::core::context::LocustContext;
use locust::plugins::omnibar::registry::{Command, CommandRegistry, CommandResult};
use std::sync::Arc;

// Test command implementation
struct TestCommand {
    name: String,
    description: String,
    category: Option<String>,
    aliases: Vec<String>,
    executed: std::sync::Arc<std::sync::atomic::AtomicBool>,
}

impl Command for TestCommand {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn category(&self) -> Option<&str> {
        self.category.as_deref()
    }

    fn aliases(&self) -> Vec<&str> {
        self.aliases.iter().map(|s| s.as_str()).collect()
    }

    fn execute(&self, _ctx: &mut LocustContext) -> CommandResult {
        self.executed
            .store(true, std::sync::atomic::Ordering::Relaxed);
        Ok(())
    }
}

fn create_test_command(name: &str, desc: &str) -> Arc<dyn Command> {
    Arc::new(TestCommand {
        name: name.to_string(),
        description: desc.to_string(),
        category: None,
        aliases: Vec::new(),
        executed: Arc::new(std::sync::atomic::AtomicBool::new(false)),
    })
}

fn create_test_command_with_category(
    name: &str,
    desc: &str,
    category: &str,
) -> Arc<dyn Command> {
    Arc::new(TestCommand {
        name: name.to_string(),
        description: desc.to_string(),
        category: Some(category.to_string()),
        aliases: Vec::new(),
        executed: Arc::new(std::sync::atomic::AtomicBool::new(false)),
    })
}

#[test]
fn test_registry_creation() {
    let registry = CommandRegistry::new();
    assert!(registry.is_empty());
    assert_eq!(registry.len(), 0);
}

#[test]
fn test_register_and_get_command() {
    let mut registry = CommandRegistry::new();
    let cmd = create_test_command("test", "A test command");

    registry.register(Arc::clone(&cmd));

    assert_eq!(registry.len(), 1);
    assert!(registry.contains("test"));

    let retrieved = registry.get("test");
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().name(), "test");
}

#[test]
fn test_unregister_command() {
    let mut registry = CommandRegistry::new();
    registry.register(create_test_command("test", "Test"));

    assert!(registry.contains("test"));
    assert!(registry.unregister("test"));
    assert!(!registry.contains("test"));
    assert!(registry.is_empty());
}

#[test]
fn test_unregister_nonexistent() {
    let mut registry = CommandRegistry::new();
    assert!(!registry.unregister("nonexistent"));
}

#[test]
fn test_command_aliases() {
    let mut registry = CommandRegistry::new();

    let cmd = Arc::new(TestCommand {
        name: "quit".to_string(),
        description: "Exit".to_string(),
        category: None,
        aliases: vec!["q".to_string(), "exit".to_string()],
        executed: Arc::new(std::sync::atomic::AtomicBool::new(false)),
    });

    registry.register(cmd);

    // Should be accessible via main name and aliases
    assert!(registry.contains("quit"));
    assert!(registry.contains("q"));
    assert!(registry.contains("exit"));

    // Getting via alias should return the same command
    let via_alias = registry.get("q");
    assert!(via_alias.is_some());
    assert_eq!(via_alias.unwrap().name(), "quit");
}

#[test]
fn test_search_empty_query() {
    let mut registry = CommandRegistry::new();
    registry.register(create_test_command("hello", "Say hello"));
    registry.register(create_test_command("help", "Show help"));
    registry.register(create_test_command("quit", "Exit"));

    let results = registry.search("");

    // Empty query should return all commands
    assert_eq!(results.len(), 3);
}

#[test]
fn test_search_exact_match() {
    let mut registry = CommandRegistry::new();
    registry.register(create_test_command("hello", "Say hello"));
    registry.register(create_test_command("help", "Show help"));

    let results = registry.search("hello");

    assert!(!results.is_empty());
    // Best match should be first
    assert_eq!(results[0].name, "hello");
}

#[test]
fn test_search_fuzzy_match() {
    let mut registry = CommandRegistry::new();
    registry.register(create_test_command("hello", "Say hello"));
    registry.register(create_test_command("help", "Show help"));

    // Fuzzy search should find matches
    let results = registry.search("hlo");
    assert!(!results.is_empty());
}

#[test]
fn test_search_description() {
    let mut registry = CommandRegistry::new();
    registry.register(create_test_command("greet", "Say hello to everyone"));

    let results = registry.search("hello");

    // Should match based on description
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].name, "greet");
}

#[test]
fn test_filter_by_category() {
    let mut registry = CommandRegistry::new();

    registry.register(create_test_command_with_category(
        "quit", "Exit", "system",
    ));
    registry.register(create_test_command_with_category(
        "help", "Help", "system",
    ));
    registry.register(create_test_command_with_category(
        "hello", "Greet", "fun",
    ));

    let system_cmds = registry.filter_by_category("system");
    assert_eq!(system_cmds.len(), 2);

    let fun_cmds = registry.filter_by_category("fun");
    assert_eq!(fun_cmds.len(), 1);

    let empty = registry.filter_by_category("nonexistent");
    assert_eq!(empty.len(), 0);
}

#[test]
fn test_categories_list() {
    let mut registry = CommandRegistry::new();

    registry.register(create_test_command_with_category(
        "cmd1", "Test 1", "cat1",
    ));
    registry.register(create_test_command_with_category(
        "cmd2", "Test 2", "cat2",
    ));
    registry.register(create_test_command_with_category(
        "cmd3", "Test 3", "cat1",
    ));

    let cats = registry.categories();

    assert_eq!(cats.len(), 2);
    assert!(cats.contains(&"cat1".to_string()));
    assert!(cats.contains(&"cat2".to_string()));
}

#[test]
fn test_execute_success() {
    let mut registry = CommandRegistry::new();
    let cmd = create_test_command("test", "Test");
    registry.register(cmd);

    let mut ctx = LocustContext::default();
    let result = registry.execute("test", &mut ctx);

    assert!(result.is_ok());
}

#[test]
fn test_execute_not_found() {
    let registry = CommandRegistry::new();
    let mut ctx = LocustContext::default();

    let result = registry.execute("missing", &mut ctx);

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("not found"));
}

#[test]
fn test_execute_via_alias() {
    let mut registry = CommandRegistry::new();

    let cmd = Arc::new(TestCommand {
        name: "quit".to_string(),
        description: "Exit".to_string(),
        category: None,
        aliases: vec!["q".to_string()],
        executed: Arc::new(std::sync::atomic::AtomicBool::new(false)),
    });

    registry.register(cmd);

    let mut ctx = LocustContext::default();
    let result = registry.execute("q", &mut ctx);

    assert!(result.is_ok());
}

#[test]
fn test_clear_registry() {
    let mut registry = CommandRegistry::new();

    registry.register(create_test_command("cmd1", "Test 1"));
    registry.register(create_test_command("cmd2", "Test 2"));
    registry.register(create_test_command("cmd3", "Test 3"));

    assert_eq!(registry.len(), 3);

    registry.clear();

    assert!(registry.is_empty());
    assert_eq!(registry.len(), 0);
}

#[test]
fn test_command_names() {
    let mut registry = CommandRegistry::new();

    registry.register(create_test_command("alpha", "First"));
    registry.register(create_test_command("beta", "Second"));
    registry.register(create_test_command("gamma", "Third"));

    let names = registry.command_names();

    assert_eq!(names.len(), 3);
    assert!(names.contains(&"alpha".to_string()));
    assert!(names.contains(&"beta".to_string()));
    assert!(names.contains(&"gamma".to_string()));
}

#[test]
fn test_replace_command() {
    let mut registry = CommandRegistry::new();

    registry.register(create_test_command("test", "Old description"));
    assert_eq!(registry.len(), 1);

    registry.register(create_test_command("test", "New description"));

    // Should still have only one command
    assert_eq!(registry.len(), 1);

    let cmd = registry.get("test").unwrap();
    assert_eq!(cmd.description(), "New description");
}

#[test]
fn test_search_scoring() {
    let mut registry = CommandRegistry::new();

    registry.register(create_test_command("help", "Show help"));
    registry.register(create_test_command("helper", "Helper utility"));
    registry.register(create_test_command("superhero", "Be a hero"));

    let results = registry.search("help");

    // "help" should score higher than "helper" or "superhero"
    assert!(!results.is_empty());
    assert_eq!(results[0].name, "help");
    assert!(results[0].score > results[1].score);
}
