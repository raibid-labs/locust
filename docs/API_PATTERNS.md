# Locust API Design Patterns

## Table of Contents

1. [Plugin Development Patterns](#plugin-development-patterns)
2. [State Management](#state-management)
3. [Resource Cleanup](#resource-cleanup)
4. [Error Handling](#error-handling)
5. [Thread Safety](#thread-safety)
6. [Extension Points](#extension-points)
7. [Composition Patterns](#composition-patterns)
8. [Performance Patterns](#performance-patterns)
9. [Testing Patterns](#testing-patterns)
10. [Best Practices](#best-practices)

---

## Plugin Development Patterns

### Pattern 1: Simple Event Handler

**Use Case**: Plugin that responds to specific events

```rust
use locust::prelude::*;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};

struct SimplePlugin {
    enabled: bool,
}

impl SimplePlugin {
    pub fn new() -> Self {
        Self { enabled: true }
    }

    pub fn toggle(&mut self) {
        self.enabled = !self.enabled;
    }
}

impl<B: Backend> LocustPlugin<B> for SimplePlugin {
    fn id(&self) -> &'static str {
        "app.simple_plugin"
    }

    fn priority(&self) -> i32 {
        100  // Default priority
    }

    fn init(&mut self, ctx: &mut LocustContext) {
        println!("Simple plugin initialized");
    }

    fn on_event(&mut self, event: &Event, ctx: &mut LocustContext) -> PluginEventResult {
        if !self.enabled {
            return PluginEventResult::NotHandled;
        }

        match event {
            Event::Key(KeyEvent {
                code: KeyCode::Char('t'),
                modifiers: KeyModifiers::CONTROL,
                ..
            }) => {
                self.toggle();
                PluginEventResult::ConsumedRequestRedraw
            }
            _ => PluginEventResult::NotHandled,
        }
    }

    fn render_overlay(&self, frame: &mut Frame, ctx: &LocustContext) {
        if !self.enabled {
            return;
        }

        let status = Paragraph::new(format!("Plugin: {}", if self.enabled { "ON" } else { "OFF" }))
            .style(Style::default().fg(Color::Green));

        let area = Rect::new(frame.area().width - 20, frame.area().height - 1, 20, 1);
        frame.render_widget(status, area);
    }

    fn cleanup(&mut self, ctx: &mut LocustContext) {
        println!("Simple plugin cleaned up");
    }
}
```

### Pattern 2: Stateful Plugin with Context Data

**Use Case**: Plugin that maintains state across frames

```rust
use std::collections::VecDeque;
use std::time::Instant;

struct StatefulPlugin {
    history: VecDeque<(Instant, Event)>,
    max_history: usize,
    show_history: bool,
}

impl StatefulPlugin {
    pub fn new(max_history: usize) -> Self {
        Self {
            history: VecDeque::with_capacity(max_history),
            max_history,
            show_history: false,
        }
    }

    fn add_to_history(&mut self, event: Event) {
        if self.history.len() >= self.max_history {
            self.history.pop_front();
        }
        self.history.push_back((Instant::now(), event));
    }

    fn recent_events(&self, count: usize) -> Vec<String> {
        self.history
            .iter()
            .rev()
            .take(count)
            .map(|(time, event)| {
                format!("{:?} ago: {:?}", time.elapsed(), event)
            })
            .collect()
    }
}

impl<B: Backend> LocustPlugin<B> for StatefulPlugin {
    fn id(&self) -> &'static str {
        "app.stateful_plugin"
    }

    fn on_event(&mut self, event: &Event, ctx: &mut LocustContext) -> PluginEventResult {
        // Record all events
        self.add_to_history(event.clone());

        // Toggle history display
        if let Event::Key(KeyEvent {
            code: KeyCode::Char('h'),
            modifiers: KeyModifiers::CONTROL,
            ..
        }) = event {
            self.show_history = !self.show_history;
            return PluginEventResult::ConsumedRequestRedraw;
        }

        // Store state in context for other plugins
        ctx.store_data("event_history_count", Box::new(self.history.len()));

        PluginEventResult::NotHandled
    }

    fn render_overlay(&self, frame: &mut Frame, ctx: &LocustContext) {
        if !self.show_history {
            return;
        }

        let events = self.recent_events(10);
        let height = events.len().min(10) as u16;
        let width = 60;

        let area = Rect::new(
            frame.area().width.saturating_sub(width),
            frame.area().height.saturating_sub(height + 2),
            width,
            height + 2,
        );

        let block = Block::default()
            .title("Event History")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));

        let paragraph = Paragraph::new(events.join("\n"))
            .block(block)
            .style(Style::default().fg(Color::White));

        frame.render_widget(paragraph, area);
    }
}
```

### Pattern 3: Plugin with Custom Actions

**Use Case**: Plugin that provides actions for targets

```rust
#[derive(Clone, Debug)]
pub enum CustomAction {
    Execute,
    Delete,
    Rename,
    Copy,
}

struct ActionablePlugin {
    pending_action: Option<(String, CustomAction)>,  // (target_id, action)
}

impl ActionablePlugin {
    pub fn new() -> Self {
        Self {
            pending_action: None,
        }
    }

    fn execute_action(&mut self, target_id: String, action: CustomAction, ctx: &mut LocustContext) {
        println!("Executing {:?} on target {}", action, target_id);

        // Store action result in context
        ctx.store_data("last_action", Box::new((target_id.clone(), action.clone())));

        // Clear pending action
        self.pending_action = None;
    }
}

impl<B: Backend> LocustPlugin<B> for ActionablePlugin {
    fn id(&self) -> &'static str {
        "app.actionable_plugin"
    }

    fn on_event(&mut self, event: &Event, ctx: &mut LocustContext) -> PluginEventResult {
        // Check if there's a pending action
        if let Some((target_id, action)) = &self.pending_action {
            if let Event::Key(KeyEvent { code: KeyCode::Enter, .. }) = event {
                let id = target_id.clone();
                let act = action.clone();
                self.execute_action(id, act, ctx);
                return PluginEventResult::ConsumedRequestRedraw;
            }

            if let Event::Key(KeyEvent { code: KeyCode::Esc, .. }) = event {
                self.pending_action = None;
                return PluginEventResult::ConsumedRequestRedraw;
            }
        }

        // Set up actions for different keys
        if let Event::Key(KeyEvent { code, modifiers, .. }) = event {
            if modifiers.contains(KeyModifiers::CONTROL) {
                // Get currently selected target from context
                if let Some(target_id) = ctx.get_data::<String>("selected_target") {
                    let action = match code {
                        KeyCode::Char('e') => Some(CustomAction::Execute),
                        KeyCode::Char('d') => Some(CustomAction::Delete),
                        KeyCode::Char('r') => Some(CustomAction::Rename),
                        KeyCode::Char('c') => Some(CustomAction::Copy),
                        _ => None,
                    };

                    if let Some(action) = action {
                        self.pending_action = Some((target_id.clone(), action));
                        return PluginEventResult::ConsumedRequestRedraw;
                    }
                }
            }
        }

        PluginEventResult::NotHandled
    }

    fn render_overlay(&self, frame: &mut Frame, ctx: &LocustContext) {
        if let Some((target_id, action)) = &self.pending_action {
            let text = format!("Execute {:?} on {}? [Enter] Yes [Esc] Cancel", action, target_id);
            let width = text.len() as u16 + 4;
            let area = Rect::new(
                (frame.area().width.saturating_sub(width)) / 2,
                frame.area().height / 2,
                width,
                3,
            );

            let block = Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow))
                .style(Style::default().bg(Color::DarkGray));

            let paragraph = Paragraph::new(text)
                .block(block)
                .style(Style::default().fg(Color::Yellow));

            frame.render_widget(paragraph, area);
        }
    }
}
```

---

## State Management

### Pattern: Shared State with Arc

```rust
use std::sync::{Arc, RwLock};

#[derive(Clone, Debug)]
struct AppState {
    current_view: String,
    selection: Vec<usize>,
    filters: HashMap<String, String>,
}

struct StateAwarePlugin {
    state: Arc<RwLock<AppState>>,
}

impl StateAwarePlugin {
    pub fn new(state: Arc<RwLock<AppState>>) -> Self {
        Self { state }
    }

    fn read_state<F, R>(&self, f: F) -> Option<R>
    where
        F: FnOnce(&AppState) -> R,
    {
        self.state.read().ok().map(|state| f(&*state))
    }

    fn write_state<F, R>(&self, f: F) -> Option<R>
    where
        F: FnOnce(&mut AppState) -> R,
    {
        self.state.write().ok().map(|mut state| f(&mut *state))
    }
}

impl<B: Backend> LocustPlugin<B> for StateAwarePlugin {
    fn id(&self) -> &'static str {
        "app.state_aware"
    }

    fn on_event(&mut self, event: &Event, ctx: &mut LocustContext) -> PluginEventResult {
        if let Event::Key(KeyEvent { code, .. }) = event {
            match code {
                KeyCode::Char('1'..='9') => {
                    let index = (*code as u8 - b'1') as usize;
                    self.write_state(|state| {
                        if state.selection.contains(&index) {
                            state.selection.retain(|&i| i != index);
                        } else {
                            state.selection.push(index);
                        }
                    });
                    return PluginEventResult::ConsumedRequestRedraw;
                }
                _ => {}
            }
        }

        PluginEventResult::NotHandled
    }

    fn render_overlay(&self, frame: &mut Frame, ctx: &LocustContext) {
        if let Some(selection_count) = self.read_state(|state| state.selection.len()) {
            let text = format!("Selected: {}", selection_count);
            let area = Rect::new(0, 0, text.len() as u16 + 2, 1);
            frame.render_widget(
                Paragraph::new(text).style(Style::default().fg(Color::Cyan)),
                area
            );
        }
    }
}
```

### Pattern: Message Passing

```rust
use std::sync::mpsc::{channel, Sender, Receiver};

#[derive(Debug, Clone)]
enum PluginMessage {
    SelectItem(usize),
    ClearSelection,
    UpdateFilter(String, String),
}

struct MessagePlugin {
    tx: Sender<PluginMessage>,
    rx: Receiver<PluginMessage>,
}

impl MessagePlugin {
    pub fn new() -> Self {
        let (tx, rx) = channel();
        Self { tx, rx }
    }

    pub fn sender(&self) -> Sender<PluginMessage> {
        self.tx.clone()
    }

    fn process_messages(&mut self, ctx: &mut LocustContext) {
        while let Ok(msg) = self.rx.try_recv() {
            match msg {
                PluginMessage::SelectItem(index) => {
                    ctx.store_data("selected_item", Box::new(index));
                }
                PluginMessage::ClearSelection => {
                    ctx.remove_data("selected_item");
                }
                PluginMessage::UpdateFilter(key, value) => {
                    ctx.store_data(&format!("filter_{}", key), Box::new(value));
                }
            }
        }
    }
}

impl<B: Backend> LocustPlugin<B> for MessagePlugin {
    fn id(&self) -> &'static str {
        "app.message_plugin"
    }

    fn on_frame_begin(&mut self, ctx: &mut LocustContext) {
        self.process_messages(ctx);
    }

    fn on_event(&mut self, event: &Event, ctx: &mut LocustContext) -> PluginEventResult {
        // Events trigger messages
        if let Event::Key(KeyEvent { code: KeyCode::Char('c'), modifiers, .. }) = event {
            if modifiers.contains(KeyModifiers::CONTROL) {
                let _ = self.tx.send(PluginMessage::ClearSelection);
                return PluginEventResult::ConsumedRequestRedraw;
            }
        }

        PluginEventResult::NotHandled
    }

    fn render_overlay(&self, frame: &mut Frame, ctx: &LocustContext) {
        // Render based on context data updated by messages
    }
}
```

---

## Resource Cleanup

### Pattern: RAII Resources

```rust
struct ResourcePlugin {
    temp_file: Option<std::fs::File>,
    cache_dir: Option<tempfile::TempDir>,
}

impl ResourcePlugin {
    pub fn new() -> io::Result<Self> {
        Ok(Self {
            temp_file: Some(tempfile::tempfile()?),
            cache_dir: Some(tempfile::tempdir()?),
        })
    }
}

impl<B: Backend> LocustPlugin<B> for ResourcePlugin {
    fn id(&self) -> &'static str {
        "app.resource_plugin"
    }

    fn on_event(&mut self, event: &Event, ctx: &mut LocustContext) -> PluginEventResult {
        // Use resources
        PluginEventResult::NotHandled
    }

    fn render_overlay(&self, frame: &mut Frame, ctx: &LocustContext) {
        // Use resources
    }

    fn cleanup(&mut self, ctx: &mut LocustContext) {
        // Explicit cleanup (also happens automatically via Drop)
        self.temp_file = None;
        self.cache_dir = None;
        println!("Resources cleaned up");
    }
}

impl Drop for ResourcePlugin {
    fn drop(&mut self) {
        println!("ResourcePlugin dropped");
    }
}
```

### Pattern: Cleanup Hooks

```rust
struct CleanupPlugin {
    cleanup_hooks: Vec<Box<dyn FnOnce(&mut LocustContext) + Send>>,
}

impl CleanupPlugin {
    pub fn new() -> Self {
        Self {
            cleanup_hooks: Vec::new(),
        }
    }

    pub fn register_cleanup<F>(&mut self, f: F)
    where
        F: FnOnce(&mut LocustContext) + Send + 'static,
    {
        self.cleanup_hooks.push(Box::new(f));
    }
}

impl<B: Backend> LocustPlugin<B> for CleanupPlugin {
    fn id(&self) -> &'static str {
        "app.cleanup_plugin"
    }

    fn on_event(&mut self, event: &Event, ctx: &mut LocustContext) -> PluginEventResult {
        PluginEventResult::NotHandled
    }

    fn render_overlay(&self, frame: &mut Frame, ctx: &LocustContext) {}

    fn cleanup(&mut self, ctx: &mut LocustContext) {
        // Execute all cleanup hooks
        for hook in self.cleanup_hooks.drain(..) {
            hook(ctx);
        }
    }
}
```

---

## Error Handling

### Pattern: Result-based Event Handling

```rust
use thiserror::Error;

#[derive(Error, Debug)]
enum PluginError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid state: {0}")]
    InvalidState(String),

    #[error("Configuration error: {0}")]
    Config(String),
}

struct FalliblePlugin {
    last_error: Option<PluginError>,
}

impl FalliblePlugin {
    pub fn new() -> Self {
        Self { last_error: None }
    }

    fn try_handle_event(&mut self, event: &Event, ctx: &mut LocustContext) -> Result<PluginEventResult, PluginError> {
        // Logic that might fail
        if let Event::Key(KeyEvent { code: KeyCode::Char('s'), modifiers, .. }) = event {
            if modifiers.contains(KeyModifiers::CONTROL) {
                self.try_save_state(ctx)?;
                return Ok(PluginEventResult::ConsumedRequestRedraw);
            }
        }

        Ok(PluginEventResult::NotHandled)
    }

    fn try_save_state(&self, ctx: &LocustContext) -> Result<(), PluginError> {
        std::fs::write("/tmp/state.json", "{}").map_err(PluginError::Io)?;
        Ok(())
    }
}

impl<B: Backend> LocustPlugin<B> for FalliblePlugin {
    fn id(&self) -> &'static str {
        "app.fallible_plugin"
    }

    fn on_event(&mut self, event: &Event, ctx: &mut LocustContext) -> PluginEventResult {
        match self.try_handle_event(event, ctx) {
            Ok(result) => {
                self.last_error = None;
                result
            }
            Err(e) => {
                eprintln!("Plugin error: {}", e);
                self.last_error = Some(e);
                PluginEventResult::NotHandled
            }
        }
    }

    fn render_overlay(&self, frame: &mut Frame, ctx: &LocustContext) {
        if let Some(ref error) = self.last_error {
            let text = format!("Error: {}", error);
            let area = Rect::new(0, frame.area().height - 1, frame.area().width, 1);
            frame.render_widget(
                Paragraph::new(text)
                    .style(Style::default().bg(Color::Red).fg(Color::White)),
                area
            );
        }
    }
}
```

---

## Thread Safety

### Pattern: Send + Sync Plugin

```rust
use std::sync::Arc;
use parking_lot::RwLock;

struct ThreadSafePlugin {
    state: Arc<RwLock<PluginState>>,
}

#[derive(Debug, Clone)]
struct PluginState {
    counter: usize,
    data: HashMap<String, String>,
}

impl ThreadSafePlugin {
    pub fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(PluginState {
                counter: 0,
                data: HashMap::new(),
            })),
        }
    }

    pub fn clone_state(&self) -> Arc<RwLock<PluginState>> {
        self.state.clone()
    }

    // Can be called from other threads
    pub fn increment_from_thread(&self) {
        let mut state = self.state.write();
        state.counter += 1;
    }
}

// Manually implement Send + Sync
unsafe impl Send for ThreadSafePlugin {}
unsafe impl Sync for ThreadSafePlugin {}

impl<B: Backend> LocustPlugin<B> for ThreadSafePlugin {
    fn id(&self) -> &'static str {
        "app.threadsafe_plugin"
    }

    fn on_event(&mut self, event: &Event, ctx: &mut LocustContext) -> PluginEventResult {
        let mut state = self.state.write();
        state.counter += 1;
        PluginEventResult::NotHandled
    }

    fn render_overlay(&self, frame: &mut Frame, ctx: &LocustContext) {
        let state = self.state.read();
        let text = format!("Counter: {}", state.counter);
        frame.render_widget(
            Paragraph::new(text),
            Rect::new(0, 0, 20, 1)
        );
    }
}
```

---

## Extension Points

### Pattern: Custom Target Actions

```rust
use locust::core::targets::{NavTarget, TargetAction};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum CustomTargetAction {
    Standard(TargetAction),
    Preview,
    Edit,
    Share,
}

struct CustomActionPlugin {
    action_handlers: HashMap<CustomTargetAction, Box<dyn Fn(&str, &mut LocustContext)>>,
}

impl CustomActionPlugin {
    pub fn new() -> Self {
        let mut plugin = Self {
            action_handlers: HashMap::new(),
        };

        // Register default handlers
        plugin.register_action(CustomTargetAction::Preview, |target_id, ctx| {
            println!("Previewing {}", target_id);
            ctx.store_data("preview_target", Box::new(target_id.to_string()));
        });

        plugin.register_action(CustomTargetAction::Edit, |target_id, ctx| {
            println!("Editing {}", target_id);
            ctx.store_data("edit_target", Box::new(target_id.to_string()));
        });

        plugin
    }

    pub fn register_action<F>(&mut self, action: CustomTargetAction, handler: F)
    where
        F: Fn(&str, &mut LocustContext) + 'static,
    {
        self.action_handlers.insert(action, Box::new(handler));
    }

    fn execute_action(&self, target_id: &str, action: &CustomTargetAction, ctx: &mut LocustContext) {
        if let Some(handler) = self.action_handlers.get(action) {
            handler(target_id, ctx);
        }
    }
}

impl<B: Backend> LocustPlugin<B> for CustomActionPlugin {
    fn id(&self) -> &'static str {
        "app.custom_actions"
    }

    fn on_event(&mut self, event: &Event, ctx: &mut LocustContext) -> PluginEventResult {
        if let Event::Key(KeyEvent { code, .. }) = event {
            if let Some(selected) = ctx.get_data::<String>("selected_target") {
                let action = match code {
                    KeyCode::Char('p') => Some(CustomTargetAction::Preview),
                    KeyCode::Char('e') => Some(CustomTargetAction::Edit),
                    KeyCode::Char('s') => Some(CustomTargetAction::Share),
                    _ => None,
                };

                if let Some(action) = action {
                    self.execute_action(&selected, &action, ctx);
                    return PluginEventResult::ConsumedRequestRedraw;
                }
            }
        }

        PluginEventResult::NotHandled
    }

    fn render_overlay(&self, frame: &mut Frame, ctx: &LocustContext) {
        // Show available actions
        let help_text = "[P]review [E]dit [S]hare";
        let area = Rect::new(
            0,
            frame.area().height - 1,
            help_text.len() as u16,
            1
        );
        frame.render_widget(
            Paragraph::new(help_text).style(Style::default().fg(Color::Gray)),
            area
        );
    }
}
```

### Pattern: Custom Hint Generator

```rust
trait HintGenerator: Send + Sync {
    fn generate_hint(&self, index: usize) -> String;
}

struct AlphabeticHints;

impl HintGenerator for AlphabeticHints {
    fn generate_hint(&self, index: usize) -> String {
        let mut hint = String::new();
        let mut n = index;

        loop {
            let ch = (b'a' + (n % 26) as u8) as char;
            hint.insert(0, ch);
            n /= 26;
            if n == 0 {
                break;
            }
            n -= 1;
        }

        hint
    }
}

struct NumericHints;

impl HintGenerator for NumericHints {
    fn generate_hint(&self, index: usize) -> String {
        (index + 1).to_string()
    }
}

struct CustomHintPlugin {
    generator: Box<dyn HintGenerator>,
}

impl CustomHintPlugin {
    pub fn with_generator(generator: Box<dyn HintGenerator>) -> Self {
        Self { generator }
    }

    pub fn alphabetic() -> Self {
        Self::with_generator(Box::new(AlphabeticHints))
    }

    pub fn numeric() -> Self {
        Self::with_generator(Box::new(NumericHints))
    }
}

impl<B: Backend> LocustPlugin<B> for CustomHintPlugin {
    fn id(&self) -> &'static str {
        "app.custom_hints"
    }

    fn on_event(&mut self, event: &Event, ctx: &mut LocustContext) -> PluginEventResult {
        PluginEventResult::NotHandled
    }

    fn render_overlay(&self, frame: &mut Frame, ctx: &LocustContext) {
        for (i, target) in ctx.targets.all().iter().enumerate() {
            let hint = self.generator.generate_hint(i);
            let hint_area = Rect::new(
                target.area.x,
                target.area.y,
                hint.len() as u16,
                1
            );

            frame.render_widget(
                Paragraph::new(hint)
                    .style(Style::default().bg(Color::Yellow).fg(Color::Black)),
                hint_area
            );
        }
    }
}
```

---

## Composition Patterns

### Pattern: Plugin Chain

```rust
struct PluginChain<B: Backend> {
    plugins: Vec<Box<dyn LocustPlugin<B>>>,
}

impl<B: Backend> PluginChain<B> {
    pub fn new() -> Self {
        Self {
            plugins: Vec::new(),
        }
    }

    pub fn add(mut self, plugin: impl LocustPlugin<B> + 'static) -> Self {
        self.plugins.push(Box::new(plugin));
        self
    }
}

impl<B: Backend> LocustPlugin<B> for PluginChain<B> {
    fn id(&self) -> &'static str {
        "app.plugin_chain"
    }

    fn init(&mut self, ctx: &mut LocustContext) {
        for plugin in &mut self.plugins {
            plugin.init(ctx);
        }
    }

    fn on_event(&mut self, event: &Event, ctx: &mut LocustContext) -> PluginEventResult {
        for plugin in &mut self.plugins {
            let result = plugin.on_event(event, ctx);
            if result.is_consumed() {
                return result;
            }
        }
        PluginEventResult::NotHandled
    }

    fn render_overlay(&self, frame: &mut Frame, ctx: &LocustContext) {
        for plugin in &self.plugins {
            plugin.render_overlay(frame, ctx);
        }
    }

    fn cleanup(&mut self, ctx: &mut LocustContext) {
        for plugin in &mut self.plugins {
            plugin.cleanup(ctx);
        }
    }
}

// Usage
let chain = PluginChain::new()
    .add(LoggingPlugin::new())
    .add(ValidationPlugin::new())
    .add(ActionPlugin::new());

locust.register_plugin(chain);
```

### Pattern: Conditional Plugin

```rust
struct ConditionalPlugin<B: Backend, P: LocustPlugin<B>> {
    inner: P,
    condition: Box<dyn Fn(&LocustContext) -> bool>,
    _phantom: std::marker::PhantomData<B>,
}

impl<B: Backend, P: LocustPlugin<B>> ConditionalPlugin<B, P> {
    pub fn new(plugin: P, condition: impl Fn(&LocustContext) -> bool + 'static) -> Self {
        Self {
            inner: plugin,
            condition: Box::new(condition),
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<B: Backend, P: LocustPlugin<B>> LocustPlugin<B> for ConditionalPlugin<B, P> {
    fn id(&self) -> &'static str {
        self.inner.id()
    }

    fn priority(&self) -> i32 {
        self.inner.priority()
    }

    fn init(&mut self, ctx: &mut LocustContext) {
        self.inner.init(ctx);
    }

    fn on_event(&mut self, event: &Event, ctx: &mut LocustContext) -> PluginEventResult {
        if (self.condition)(ctx) {
            self.inner.on_event(event, ctx)
        } else {
            PluginEventResult::NotHandled
        }
    }

    fn render_overlay(&self, frame: &mut Frame, ctx: &LocustContext) {
        if (self.condition)(ctx) {
            self.inner.render_overlay(frame, ctx);
        }
    }

    fn cleanup(&mut self, ctx: &mut LocustContext) {
        self.inner.cleanup(ctx);
    }
}

// Usage
let debug_plugin = ConditionalPlugin::new(
    DebugPlugin::new(),
    |ctx| cfg!(debug_assertions)
);

let feature_plugin = ConditionalPlugin::new(
    FeaturePlugin::new(),
    |ctx| ctx.get_data::<bool>("feature_enabled").copied().unwrap_or(false)
);
```

---

## Performance Patterns

### Pattern: Lazy Rendering

```rust
struct LazyRenderPlugin {
    cached_overlay: Option<Buffer>,
    cache_valid: bool,
    last_frame: u64,
}

impl LazyRenderPlugin {
    pub fn new() -> Self {
        Self {
            cached_overlay: None,
            cache_valid: false,
            last_frame: 0,
        }
    }

    fn invalidate_cache(&mut self) {
        self.cache_valid = false;
    }

    fn rebuild_cache(&mut self, area: Rect, ctx: &LocustContext) {
        let mut buffer = Buffer::empty(area);

        // Expensive rendering logic here
        for target in ctx.targets.all() {
            // ... render to buffer
        }

        self.cached_overlay = Some(buffer);
        self.cache_valid = true;
    }
}

impl<B: Backend> LocustPlugin<B> for LazyRenderPlugin {
    fn id(&self) -> &'static str {
        "app.lazy_render"
    }

    fn on_event(&mut self, event: &Event, ctx: &mut LocustContext) -> PluginEventResult {
        // Invalidate cache on relevant events
        self.invalidate_cache();
        PluginEventResult::NotHandled
    }

    fn render_overlay(&mut self, frame: &mut Frame, ctx: &LocustContext) {
        // Only rebuild if necessary
        if !self.cache_valid || self.last_frame != ctx.frame_count {
            self.rebuild_cache(frame.area(), ctx);
            self.last_frame = ctx.frame_count;
        }

        // Render from cache
        if let Some(ref buffer) = self.cached_overlay {
            // Copy buffer to frame
            // (in real implementation, you'd merge the buffer)
        }
    }
}
```

### Pattern: Batch Processing

```rust
struct BatchPlugin {
    pending_operations: Vec<Operation>,
    batch_size: usize,
}

#[derive(Debug)]
enum Operation {
    UpdateTarget(String, Rect),
    RemoveTarget(String),
    AddTarget(NavTarget),
}

impl BatchPlugin {
    pub fn new(batch_size: usize) -> Self {
        Self {
            pending_operations: Vec::with_capacity(batch_size),
            batch_size,
        }
    }

    fn queue_operation(&mut self, op: Operation) {
        self.pending_operations.push(op);
    }

    fn process_batch(&mut self, ctx: &mut LocustContext) {
        if self.pending_operations.len() < self.batch_size {
            return;  // Wait for more operations
        }

        for op in self.pending_operations.drain(..) {
            match op {
                Operation::UpdateTarget(id, area) => {
                    // Update target
                }
                Operation::RemoveTarget(id) => {
                    // Remove target
                }
                Operation::AddTarget(target) => {
                    ctx.targets.register(target);
                }
            }
        }
    }
}

impl<B: Backend> LocustPlugin<B> for BatchPlugin {
    fn id(&self) -> &'static str {
        "app.batch"
    }

    fn on_frame_begin(&mut self, ctx: &mut LocustContext) {
        self.process_batch(ctx);
    }

    fn on_event(&mut self, event: &Event, ctx: &mut LocustContext) -> PluginEventResult {
        // Queue operations instead of executing immediately
        PluginEventResult::NotHandled
    }

    fn render_overlay(&self, frame: &mut Frame, ctx: &LocustContext) {}
}
```

---

## Testing Patterns

### Pattern: Mock Plugin

```rust
#[cfg(test)]
mod tests {
    use super::*;

    struct MockPlugin {
        event_count: usize,
        render_count: usize,
    }

    impl MockPlugin {
        fn new() -> Self {
            Self {
                event_count: 0,
                render_count: 0,
            }
        }
    }

    impl<B: Backend> LocustPlugin<B> for MockPlugin {
        fn id(&self) -> &'static str {
            "test.mock"
        }

        fn on_event(&mut self, event: &Event, ctx: &mut LocustContext) -> PluginEventResult {
            self.event_count += 1;
            PluginEventResult::NotHandled
        }

        fn render_overlay(&mut self, frame: &mut Frame, ctx: &LocustContext) {
            self.render_count += 1;
        }
    }

    #[test]
    fn test_plugin_lifecycle() {
        let backend = TestBackend::new(80, 24);
        let mut locust = Locust::new(LocustConfig::default());
        let mut plugin = MockPlugin::new();

        locust.register_plugin(plugin);

        // Test event handling
        let event = Event::Key(KeyEvent::new(KeyCode::Char('a'), KeyModifiers::empty()));
        locust.on_event(&event);

        // Verify
        // (In real test, you'd have access to plugin state via Arc/Rc)
    }
}
```

---

## Best Practices

1. **Always implement cleanup**: Release resources in `cleanup()`
2. **Use appropriate priorities**: System plugins < 50, User plugins >= 100
3. **Handle errors gracefully**: Don't panic in plugin code
4. **Document your plugins**: Include usage examples
5. **Keep plugins focused**: One responsibility per plugin
6. **Test thoroughly**: Unit tests and integration tests
7. **Consider performance**: Profile hot paths
8. **Be thread-safe**: Use Arc/Mutex when needed
9. **Version your APIs**: Use semantic versioning
10. **Provide examples**: Show common use cases

## Related Documentation

This API patterns guide connects with other Locust development documentation:

### Development Guides
- **[PLUGIN_DEVELOPMENT_GUIDE.md](PLUGIN_DEVELOPMENT_GUIDE.md)** - Apply patterns to plugin development
- **[INTEGRATION_GUIDE.md](INTEGRATION_GUIDE.md)** - Integration patterns in practice
- **[ARCHITECTURE.md](ARCHITECTURE.md)** - Architectural patterns and design

### Pattern Applications
- **[CASE_STUDIES.md](CASE_STUDIES.md)** - Real-world pattern implementations
- **[EXAMPLES.md](EXAMPLES.md)** - Pattern usage in examples
- **[PLUGINS.md](PLUGINS.md)** - Built-in plugin patterns

### Configuration Patterns
- **[CONFIGURATION.md](CONFIGURATION.md)** - Configuration design patterns
- **[THEMING.md](THEMING.md)** - Theme system patterns
- **[KEYBINDINGS.md](KEYBINDINGS.md)** - Keybinding patterns

### Reference
- **[WIDGET_ADAPTERS.md](WIDGET_ADAPTERS.md)** - Adapter pattern implementations
- **[TROUBLESHOOTING.md](TROUBLESHOOTING.md)** - Anti-patterns to avoid
- **[MIGRATION_CHECKLIST.md](MIGRATION_CHECKLIST.md)** - Migration patterns

### Project Documentation
- **[README.md](../README.md)** - Pattern overview
- **[CONTRIBUTING.md](../CONTRIBUTING.md)** - Contributing pattern documentation

---

*For more patterns and examples, see [Plugin Development Guide](PLUGIN_DEVELOPMENT_GUIDE.md) and [Case Studies](CASE_STUDIES.md).*
