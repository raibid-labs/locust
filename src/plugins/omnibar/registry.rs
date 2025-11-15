//! Command registry for the Omnibar plugin.
//!
//! This module provides a command registry system that allows registering,
//! searching, and executing commands. Commands can be categorized and filtered
//! for easy discovery.

use crate::core::context::LocustContext;
use crate::core::fuzzy::FuzzyMatcher;
use std::collections::HashMap;
use std::sync::Arc;

/// Result type for command execution.
pub type CommandResult = Result<(), String>;

/// A command that can be executed via the omnibar.
///
/// Commands provide a name, description, optional category, and an execution
/// callback. They can be registered with the CommandRegistry and executed
/// when the user submits a matching command in the omnibar.
///
/// # Example
///
/// ```rust
/// use locust::plugins::omnibar::registry::{Command, CommandResult};
/// use locust::core::context::LocustContext;
///
/// struct QuitCommand;
///
/// impl Command for QuitCommand {
///     fn name(&self) -> &str {
///         "quit"
///     }
///
///     fn description(&self) -> &str {
///         "Exit the application"
///     }
///
///     fn execute(&self, _ctx: &mut LocustContext) -> CommandResult {
///         Ok(())
///     }
/// }
/// ```
pub trait Command: Send + Sync {
    /// Returns the command name (used for matching input).
    fn name(&self) -> &str;

    /// Returns a human-readable description of what the command does.
    fn description(&self) -> &str;

    /// Returns an optional category for grouping related commands.
    fn category(&self) -> Option<&str> {
        None
    }

    /// Returns optional aliases for this command.
    fn aliases(&self) -> Vec<&str> {
        Vec::new()
    }

    /// Executes the command with the given context.
    ///
    /// # Arguments
    ///
    /// * `ctx` - Mutable reference to the Locust context
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success or `Err(message)` with an error description.
    fn execute(&self, ctx: &mut LocustContext) -> CommandResult;
}

/// A suggestion for a command that matches user input.
#[derive(Debug, Clone)]
pub struct CommandSuggestion {
    /// The command name
    pub name: String,
    /// Command description
    pub description: String,
    /// Optional category
    pub category: Option<String>,
    /// Match score (higher = better match)
    pub score: f32,
    /// Byte positions of matched characters (for highlighting)
    pub match_positions: Vec<usize>,
}

/// Registry for managing commands.
///
/// The registry maintains a collection of commands indexed by name and provides
/// functionality for searching, filtering, and executing commands.
///
/// # Thread Safety
///
/// Commands are stored as `Arc<dyn Command>` to allow safe sharing across threads
/// while maintaining the registry's single-threaded interface.
///
/// # Example
///
/// ```rust
/// use locust::plugins::omnibar::registry::{CommandRegistry, Command, CommandResult};
/// use locust::core::context::LocustContext;
/// use std::sync::Arc;
///
/// struct HelloCommand;
///
/// impl Command for HelloCommand {
///     fn name(&self) -> &str { "hello" }
///     fn description(&self) -> &str { "Say hello" }
///     fn execute(&self, _ctx: &mut LocustContext) -> CommandResult {
///         eprintln!("Hello, world!");
///         Ok(())
///     }
/// }
///
/// let mut registry = CommandRegistry::new();
/// registry.register(Arc::new(HelloCommand));
/// ```
pub struct CommandRegistry {
    /// Map of command name to command implementation
    commands: HashMap<String, Arc<dyn Command>>,
    /// Map of aliases to command names
    aliases: HashMap<String, String>,
    /// Fuzzy matcher for command search
    fuzzy_matcher: FuzzyMatcher,
}

impl Default for CommandRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl CommandRegistry {
    /// Creates a new empty command registry.
    pub fn new() -> Self {
        Self {
            commands: HashMap::new(),
            aliases: HashMap::new(),
            fuzzy_matcher: FuzzyMatcher::new(),
        }
    }

    /// Registers a command in the registry.
    ///
    /// If a command with the same name already exists, it will be replaced.
    /// All aliases for the command are also registered.
    ///
    /// # Arguments
    ///
    /// * `command` - The command to register
    pub fn register(&mut self, command: Arc<dyn Command>) {
        let name = command.name().to_string();

        // Register aliases
        for alias in command.aliases() {
            self.aliases.insert(alias.to_string(), name.clone());
        }

        // Register the command
        self.commands.insert(name, command);
    }

    /// Unregisters a command by name.
    ///
    /// This removes both the command and all its aliases.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the command to unregister
    ///
    /// # Returns
    ///
    /// Returns `true` if the command was removed, `false` if it didn't exist.
    pub fn unregister(&mut self, name: &str) -> bool {
        if let Some(command) = self.commands.remove(name) {
            // Remove all aliases
            for alias in command.aliases() {
                self.aliases.remove(alias);
            }
            true
        } else {
            false
        }
    }

    /// Gets a command by name or alias.
    ///
    /// # Arguments
    ///
    /// * `name` - The command name or alias to look up
    ///
    /// # Returns
    ///
    /// Returns `Some(command)` if found, `None` otherwise.
    pub fn get(&self, name: &str) -> Option<Arc<dyn Command>> {
        // Try direct lookup first
        if let Some(command) = self.commands.get(name) {
            return Some(Arc::clone(command));
        }

        // Try alias lookup
        if let Some(real_name) = self.aliases.get(name) {
            return self.commands.get(real_name).map(Arc::clone);
        }

        None
    }

    /// Checks if a command exists by name or alias.
    pub fn contains(&self, name: &str) -> bool {
        self.commands.contains_key(name) || self.aliases.contains_key(name)
    }

    /// Returns the number of registered commands.
    pub fn len(&self) -> usize {
        self.commands.len()
    }

    /// Returns true if the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }

    /// Returns all registered command names.
    pub fn command_names(&self) -> Vec<String> {
        self.commands.keys().cloned().collect()
    }

    /// Searches for commands matching the given query.
    ///
    /// This performs a fuzzy search across command names, aliases, and descriptions.
    /// Results are sorted by relevance (best matches first).
    ///
    /// # Arguments
    ///
    /// * `query` - The search query (can be empty)
    ///
    /// # Returns
    ///
    /// A vector of command suggestions sorted by match score.
    pub fn search(&self, query: &str) -> Vec<CommandSuggestion> {
        // If query is empty, return all commands with base score
        if query.is_empty() {
            let mut suggestions: Vec<CommandSuggestion> = self
                .commands
                .iter()
                .map(|(name, command)| CommandSuggestion {
                    name: name.clone(),
                    description: command.description().to_string(),
                    category: command.category().map(|s| s.to_string()),
                    score: 0.0,
                    match_positions: Vec::new(),
                })
                .collect();

            suggestions.sort_by(|a, b| a.name.cmp(&b.name));
            return suggestions;
        }

        let mut suggestions: Vec<CommandSuggestion> = Vec::new();

        for (name, command) in &self.commands {
            let mut best_score = 0.0;
            let mut best_positions = Vec::new();

            // Try fuzzy matching on command name
            if let Some((score, positions)) = self.fuzzy_matcher.score(query, name) {
                if score > best_score {
                    best_score = score;
                    best_positions = positions;
                }
            }

            // Try fuzzy matching on aliases (with slightly lower priority)
            for alias in command.aliases() {
                if let Some((score, positions)) = self.fuzzy_matcher.score(query, alias) {
                    let adjusted_score = score * 0.9; // Slight penalty for alias matches
                    if adjusted_score > best_score {
                        best_score = adjusted_score;
                        best_positions = positions;
                    }
                }
            }

            // Try fuzzy matching on description (with lower priority)
            if best_score < 10.0 {
                if let Some((score, _)) = self.fuzzy_matcher.score(query, command.description()) {
                    let adjusted_score = score * 0.5; // Larger penalty for description matches
                    if adjusted_score > best_score {
                        best_score = adjusted_score;
                        best_positions = Vec::new(); // Don't highlight description
                    }
                }
            }

            if best_score > 0.0 {
                suggestions.push(CommandSuggestion {
                    name: name.clone(),
                    description: command.description().to_string(),
                    category: command.category().map(|s| s.to_string()),
                    score: best_score,
                    match_positions: best_positions,
                });
            }
        }

        // Sort by score (descending), then by name (ascending)
        suggestions.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.name.cmp(&b.name))
        });

        suggestions
    }

    /// Filters commands by category.
    ///
    /// # Arguments
    ///
    /// * `category` - The category to filter by
    ///
    /// # Returns
    ///
    /// A vector of command suggestions in the given category.
    pub fn filter_by_category(&self, category: &str) -> Vec<CommandSuggestion> {
        self.commands
            .iter()
            .filter_map(|(name, command)| {
                if command.category() == Some(category) {
                    Some(CommandSuggestion {
                        name: name.clone(),
                        description: command.description().to_string(),
                        category: Some(category.to_string()),
                        score: 50.0,
                        match_positions: Vec::new(),
                    })
                } else {
                    None
                }
            })
            .collect()
    }

    /// Returns all unique categories.
    pub fn categories(&self) -> Vec<String> {
        let mut cats: Vec<String> = self
            .commands
            .values()
            .filter_map(|cmd| cmd.category().map(|s| s.to_string()))
            .collect();
        cats.sort();
        cats.dedup();
        cats
    }

    /// Executes a command by name.
    ///
    /// # Arguments
    ///
    /// * `name` - The command name or alias to execute
    /// * `ctx` - Mutable reference to the Locust context
    ///
    /// # Returns
    ///
    /// Returns the result of command execution, or an error if the command
    /// was not found.
    pub fn execute(&self, name: &str, ctx: &mut LocustContext) -> CommandResult {
        match self.get(name) {
            Some(command) => command.execute(ctx),
            None => Err(format!("Command not found: '{}'", name)),
        }
    }

    /// Clears all registered commands.
    pub fn clear(&mut self) {
        self.commands.clear();
        self.aliases.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestCommand {
        name: String,
        description: String,
        category: Option<String>,
        aliases: Vec<String>,
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
            Ok(())
        }
    }

    fn create_test_command(name: &str, desc: &str) -> Arc<dyn Command> {
        Arc::new(TestCommand {
            name: name.to_string(),
            description: desc.to_string(),
            category: None,
            aliases: Vec::new(),
        })
    }

    #[test]
    fn test_registry_creation() {
        let registry = CommandRegistry::new();
        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);
    }

    #[test]
    fn test_register_command() {
        let mut registry = CommandRegistry::new();
        let cmd = create_test_command("test", "A test command");

        registry.register(cmd);
        assert_eq!(registry.len(), 1);
        assert!(registry.contains("test"));
    }

    #[test]
    fn test_unregister_command() {
        let mut registry = CommandRegistry::new();
        registry.register(create_test_command("test", "Test"));

        assert!(registry.unregister("test"));
        assert!(!registry.contains("test"));
        assert!(registry.is_empty());
    }

    #[test]
    fn test_get_command() {
        let mut registry = CommandRegistry::new();
        registry.register(create_test_command("hello", "Say hello"));

        let cmd = registry.get("hello");
        assert!(cmd.is_some());
        assert_eq!(cmd.unwrap().name(), "hello");

        let missing = registry.get("goodbye");
        assert!(missing.is_none());
    }

    #[test]
    fn test_command_aliases() {
        let mut registry = CommandRegistry::new();
        let cmd = Arc::new(TestCommand {
            name: "quit".to_string(),
            description: "Exit".to_string(),
            category: None,
            aliases: vec!["q".to_string(), "exit".to_string()],
        });

        registry.register(cmd);

        assert!(registry.contains("quit"));
        assert!(registry.contains("q"));
        assert!(registry.contains("exit"));

        let via_alias = registry.get("q");
        assert!(via_alias.is_some());
        assert_eq!(via_alias.unwrap().name(), "quit");
    }

    #[test]
    fn test_search_empty_query() {
        let mut registry = CommandRegistry::new();
        registry.register(create_test_command("hello", "Say hello"));
        registry.register(create_test_command("help", "Show help"));

        let results = registry.search("");
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_search_exact_match() {
        let mut registry = CommandRegistry::new();
        registry.register(create_test_command("hello", "Say hello"));
        registry.register(create_test_command("help", "Show help"));

        let results = registry.search("hello");
        assert!(!results.is_empty());
        assert_eq!(results[0].name, "hello");
        assert!(results[0].score > results.get(1).map(|r| r.score).unwrap_or(0.0));
    }

    #[test]
    fn test_search_prefix_match() {
        let mut registry = CommandRegistry::new();
        registry.register(create_test_command("hello", "Say hello"));
        registry.register(create_test_command("help", "Show help"));

        let results = registry.search("hel");
        assert_eq!(results.len(), 2);
        // Both should match as they start with "hel"
    }

    #[test]
    fn test_search_description() {
        let mut registry = CommandRegistry::new();
        registry.register(create_test_command("greet", "Say hello to everyone"));

        let results = registry.search("hello");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "greet");
    }

    #[test]
    fn test_filter_by_category() {
        let mut registry = CommandRegistry::new();

        let cmd1 = Arc::new(TestCommand {
            name: "quit".to_string(),
            description: "Exit".to_string(),
            category: Some("system".to_string()),
            aliases: Vec::new(),
        });

        let cmd2 = Arc::new(TestCommand {
            name: "help".to_string(),
            description: "Help".to_string(),
            category: Some("system".to_string()),
            aliases: Vec::new(),
        });

        let cmd3 = Arc::new(TestCommand {
            name: "hello".to_string(),
            description: "Greet".to_string(),
            category: Some("fun".to_string()),
            aliases: Vec::new(),
        });

        registry.register(cmd1);
        registry.register(cmd2);
        registry.register(cmd3);

        let system_cmds = registry.filter_by_category("system");
        assert_eq!(system_cmds.len(), 2);

        let fun_cmds = registry.filter_by_category("fun");
        assert_eq!(fun_cmds.len(), 1);
    }

    #[test]
    fn test_categories() {
        let mut registry = CommandRegistry::new();

        registry.register(Arc::new(TestCommand {
            name: "cmd1".to_string(),
            description: "".to_string(),
            category: Some("cat1".to_string()),
            aliases: Vec::new(),
        }));

        registry.register(Arc::new(TestCommand {
            name: "cmd2".to_string(),
            description: "".to_string(),
            category: Some("cat2".to_string()),
            aliases: Vec::new(),
        }));

        registry.register(Arc::new(TestCommand {
            name: "cmd3".to_string(),
            description: "".to_string(),
            category: Some("cat1".to_string()),
            aliases: Vec::new(),
        }));

        let cats = registry.categories();
        assert_eq!(cats.len(), 2);
        assert!(cats.contains(&"cat1".to_string()));
        assert!(cats.contains(&"cat2".to_string()));
    }

    #[test]
    fn test_execute_success() {
        let mut registry = CommandRegistry::new();
        registry.register(create_test_command("test", "Test"));

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
    fn test_clear() {
        let mut registry = CommandRegistry::new();
        registry.register(create_test_command("cmd1", "Test 1"));
        registry.register(create_test_command("cmd2", "Test 2"));

        assert_eq!(registry.len(), 2);

        registry.clear();
        assert!(registry.is_empty());
    }
}
