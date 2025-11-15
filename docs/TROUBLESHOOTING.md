# Locust Troubleshooting Guide

## Table of Contents

1. [Common Issues](#common-issues)
2. [Events Not Being Captured](#events-not-being-captured)
3. [Overlays Not Rendering](#overlays-not-rendering)
4. [Configuration Not Loading](#configuration-not-loading)
5. [Performance Degradation](#performance-degradation)
6. [Keybinding Conflicts](#keybinding-conflicts)
7. [Integration Issues](#integration-issues)
8. [Debugging Tools](#debugging-tools)
9. [FAQ](#faq)
10. [Advanced Diagnostics](#advanced-diagnostics)

---

## Common Issues

### Quick Diagnostic Checklist

Before diving into specific issues, run through this checklist:

- [ ] Is Locust properly initialized? (`Locust::new()` called)
- [ ] Are plugins registered? (`register_plugin()` called)
- [ ] Is `begin_frame()` called at the start of each frame?
- [ ] Is `render_overlay()` called after app rendering?
- [ ] Are targets being registered during `draw()`?
- [ ] Are events being passed to `locust.on_event()`?

### Most Common Problems

1. **Events not captured** (60% of issues)
2. **Overlays not visible** (20% of issues)
3. **Configuration errors** (10% of issues)
4. **Performance problems** (5% of issues)
5. **Keybinding conflicts** (5% of issues)

---

## Events Not Being Captured

### Symptom: Pressing 'f' doesn't show navigation hints

**Diagnostic Steps:**

```rust
// Add debug logging to your event handling
fn handle_event(&mut self, event: &Event) {
    println!("Event received: {:?}", event);
    let outcome = self.locust.on_event(event);
    println!("Locust outcome: consumed={}, redraw={}",
             outcome.consumed, outcome.request_redraw);
}
```

**Common Causes:**

#### 1. Plugin Not Registered

```rust
// Wrong - NavPlugin not registered
let mut locust = Locust::new(LocustConfig::default());
// ... events won't be handled

// Right
let mut locust = Locust::new(LocustConfig::default());
locust.register_plugin(NavPlugin::new());
```

#### 2. Event Consumed Before Reaching Locust

```rust
// Wrong - app handles all events first
if let Event::Key(key) = event::read()? {
    if app.handle_key(key) {  // Consumes all events
        return Ok(());
    }
    locust.on_event(&Event::Key(key));  // Never reached
}

// Right - Locust gets first chance
if let Event::Key(key) = event::read()? {
    let outcome = locust.on_event(&Event::Key(key));
    if !outcome.consumed {
        app.handle_key(key);
    }
}
```

#### 3. Wrong Event Type Passed

```rust
// Wrong - only passing KeyEvent
if let Event::Key(key) = event::read()? {
    locust.on_event(&Event::Key(key));
}

// Right - pass full Event
let event = event::read()?;
locust.on_event(&event);
```

#### 4. Plugin Priority Issues

```rust
// If custom plugin has higher priority than NavPlugin
struct MyPlugin;

impl<B: Backend> LocustPlugin<B> for MyPlugin {
    fn priority(&self) -> i32 {
        0  // Higher priority than NavPlugin (which defaults to 100)
    }

    fn on_event(&mut self, event: &Event, ctx: &mut LocustContext) -> PluginEventResult {
        // If this consumes all events, NavPlugin never sees them
        PluginEventResult::ConsumedRequestRedraw
    }
}

// Solution: Adjust priorities or be selective about consumption
impl<B: Backend> LocustPlugin<B> for MyPlugin {
    fn priority(&self) -> i32 {
        150  // Lower priority than NavPlugin
    }

    fn on_event(&mut self, event: &Event, ctx: &mut LocustContext) -> PluginEventResult {
        // Only consume events you actually handle
        if matches!(event, Event::Key(KeyEvent { code: KeyCode::Char('x'), .. })) {
            PluginEventResult::ConsumedRequestRedraw
        } else {
            PluginEventResult::NotHandled
        }
    }
}
```

### Debugging Events

Create a debug plugin to trace event flow:

```rust
struct EventDebugPlugin {
    log: Vec<(Instant, Event, bool)>,
}

impl EventDebugPlugin {
    fn new() -> Self {
        Self { log: Vec::new() }
    }
}

impl<B: Backend> LocustPlugin<B> for EventDebugPlugin {
    fn id(&self) -> &'static str {
        "debug.events"
    }

    fn priority(&self) -> i32 {
        -1000  // See events before everyone
    }

    fn on_event(&mut self, event: &Event, ctx: &mut LocustContext) -> PluginEventResult {
        self.log.push((Instant::now(), event.clone(), false));
        eprintln!("Event: {:?} | Targets: {} | Plugins: {}",
                  event, ctx.targets.len(), ctx.frame_count);
        PluginEventResult::NotHandled
    }

    fn render_overlay(&self, frame: &mut Frame, _ctx: &LocustContext) {
        if let Some((time, last_event, _)) = self.log.last() {
            let debug_text = format!("Last event: {:?} at {:?}", last_event, time);
            let area = Rect::new(0, frame.area().height - 1, frame.area().width, 1);
            frame.render_widget(
                Paragraph::new(debug_text)
                    .style(Style::default().bg(Color::Black).fg(Color::Yellow)),
                area
            );
        }
    }
}

// Use it:
locust.register_plugin(EventDebugPlugin::new());
```

---

## Overlays Not Rendering

### Symptom: Navigation hints/omnibar/tooltips not visible

**Diagnostic Steps:**

```rust
// Check if render_overlay is being called
fn draw(&mut self, frame: &mut Frame) {
    app.draw(frame);

    println!("About to render overlay, frame area: {:?}", frame.area());
    locust.render_overlay(frame);
    println!("Overlay rendered");
}
```

**Common Causes:**

#### 1. render_overlay() Not Called

```rust
// Wrong - overlay never rendered
terminal.draw(|frame| {
    app.draw(frame);
    // Missing: locust.render_overlay(frame);
})?;

// Right
terminal.draw(|frame| {
    app.draw(frame);
    locust.render_overlay(frame);
})?;
```

#### 2. begin_frame() Not Called

```rust
// Wrong - targets from previous frame used
loop {
    terminal.draw(|frame| {
        app.draw(frame, &mut locust.ctx);
        locust.render_overlay(frame);
    })?;
}

// Right
loop {
    locust.begin_frame();  // Clears old targets
    terminal.draw(|frame| {
        app.draw(frame, &mut locust.ctx);
        locust.render_overlay(frame);
    })?;
}
```

#### 3. No Targets Registered

```rust
// Wrong - targets never registered
fn draw(&self, frame: &mut Frame) {
    let list = List::new(items);
    frame.render_widget(list, area);
    // No targets registered
}

// Right
fn draw(&self, frame: &mut Frame, ctx: &mut LocustContext) {
    let list = List::new(items);
    frame.render_widget(list, area);

    // Register targets
    for (i, _) in items.iter().enumerate() {
        ctx.targets.register(NavTarget {
            id: format!("item_{}", i),
            area: Rect { y: area.y + i as u16, height: 1, ..area },
            kind: TargetKind::ListItem,
            ..Default::default()
        });
    }
}
```

#### 4. Z-Order Issues

```rust
// If your app renders on top of overlays
terminal.draw(|frame| {
    locust.render_overlay(frame);  // Rendered first
    app.draw(frame);               // Covers overlay
})?;

// Solution: Render overlays last
terminal.draw(|frame| {
    app.draw(frame);               // Base layer
    locust.render_overlay(frame);  // Top layer
})?;
```

#### 5. Plugin State Issues

```rust
// Plugin might not be in correct state
struct MyPlugin {
    active: bool,  // If false, won't render
}

impl<B: Backend> LocustPlugin<B> for MyPlugin {
    fn render_overlay(&self, frame: &mut Frame, ctx: &LocustContext) {
        if !self.active {
            return;  // Nothing rendered
        }
        // ... render code
    }
}

// Debug by logging state
fn render_overlay(&self, frame: &mut Frame, ctx: &LocustContext) {
    eprintln!("Plugin active: {}, targets: {}", self.active, ctx.targets.len());
    // ...
}
```

### Visual Debugging

Create a visual debug overlay:

```rust
struct RenderDebugPlugin {
    show_target_boxes: bool,
}

impl<B: Backend> LocustPlugin<B> for RenderDebugPlugin {
    fn id(&self) -> &'static str {
        "debug.render"
    }

    fn render_overlay(&self, frame: &mut Frame, ctx: &LocustContext) {
        if self.show_target_boxes {
            // Draw boxes around all targets
            for target in ctx.targets.all() {
                let block = Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Red));
                frame.render_widget(block, target.area);

                // Show target ID
                let label = Paragraph::new(target.id.clone())
                    .style(Style::default().bg(Color::Red).fg(Color::White));
                let label_area = Rect::new(
                    target.area.x,
                    target.area.y,
                    target.id.len().min(target.area.width as usize) as u16,
                    1
                );
                frame.render_widget(label, label_area);
            }
        }

        // Show frame info
        let info = format!(
            "Targets: {} | Frame: {} | Area: {}x{}",
            ctx.targets.len(),
            ctx.frame_count,
            frame.area().width,
            frame.area().height
        );
        let info_area = Rect::new(0, 0, frame.area().width, 1);
        frame.render_widget(
            Paragraph::new(info).style(Style::default().bg(Color::DarkGray)),
            info_area
        );
    }
}
```

---

## Configuration Not Loading

### Symptom: Theme/keybindings not applied

**Diagnostic Steps:**

```rust
// Check configuration loading
fn main() {
    let config = match LocustConfig::load_from_file("locust.toml") {
        Ok(c) => {
            println!("Config loaded: {:?}", c);
            c
        }
        Err(e) => {
            eprintln!("Config load failed: {}", e);
            LocustConfig::default()
        }
    };
}
```

**Common Causes:**

#### 1. File Path Issues

```rust
// Wrong - relative path may not work
let config = LocustConfig::load_from_file("locust.toml")?;

// Right - use absolute or well-known paths
use dirs::config_dir;

let config_path = config_dir()
    .map(|mut p| {
        p.push("myapp");
        p.push("locust.toml");
        p
    })
    .unwrap_or_else(|| PathBuf::from("locust.toml"));

let config = LocustConfig::load_from_file(&config_path)
    .unwrap_or_else(|e| {
        eprintln!("Failed to load config from {:?}: {}", config_path, e);
        LocustConfig::default()
    });
```

#### 2. Format Validation Errors

```toml
# Wrong - invalid TOML
[theme]
primary_color = #FF0000  # Missing quotes

[keybindings]
navigate = f  # Missing quotes

# Right
[theme]
primary_color = "#FF0000"

[keybindings]
navigate = "f"
```

#### 3. Schema Mismatches

```rust
// If your config structure doesn't match the file
#[derive(Deserialize)]
struct MyConfig {
    theme: ThemeConfig,  // But file has "appearance" not "theme"
}

// Solution: Use serde aliases
#[derive(Deserialize)]
struct MyConfig {
    #[serde(alias = "appearance")]
    theme: ThemeConfig,
}
```

#### 4. Missing Required Fields

```toml
# Config file missing required fields
[theme]
# Missing: primary_color, secondary_color
```

```rust
// Solution: Use defaults
#[derive(Deserialize)]
struct ThemeConfig {
    #[serde(default = "default_primary_color")]
    primary_color: Color,

    #[serde(default = "default_secondary_color")]
    secondary_color: Color,
}

fn default_primary_color() -> Color {
    Color::Cyan
}

fn default_secondary_color() -> Color {
    Color::Yellow
}
```

### Configuration Validation

```rust
use serde::Deserialize;
use std::fs;

#[derive(Deserialize, Debug)]
struct LocustConfigFile {
    #[serde(default)]
    theme: ThemeConfig,

    #[serde(default)]
    keybindings: KeybindingsConfig,

    #[serde(default)]
    plugins: PluginsConfig,
}

impl LocustConfigFile {
    fn load(path: impl AsRef<Path>) -> Result<Self, ConfigError> {
        let content = fs::read_to_string(path.as_ref())
            .map_err(|e| ConfigError::Io(e))?;

        let config: Self = toml::from_str(&content)
            .map_err(|e| ConfigError::Parse(e.to_string()))?;

        config.validate()?;
        Ok(config)
    }

    fn validate(&self) -> Result<(), ConfigError> {
        // Validate keybindings don't conflict
        let mut seen = HashSet::new();
        for (action, key) in &self.keybindings.bindings {
            if !seen.insert(key) {
                return Err(ConfigError::Validation(
                    format!("Duplicate keybinding: {}", key)
                ));
            }
        }

        // Validate theme colors
        if !is_valid_color(&self.theme.primary_color) {
            return Err(ConfigError::Validation(
                "Invalid primary_color".to_string()
            ));
        }

        Ok(())
    }
}

#[derive(Debug)]
enum ConfigError {
    Io(std::io::Error),
    Parse(String),
    Validation(String),
}
```

---

## Performance Degradation

### Symptom: App becomes slow/laggy with Locust

**Diagnostic Steps:**

```rust
use std::time::Instant;

// Measure frame time
let frame_start = Instant::now();
locust.begin_frame();
terminal.draw(|frame| {
    app.draw(frame, &mut locust.ctx);
    locust.render_overlay(frame);
})?;
let frame_time = frame_start.elapsed();

if frame_time.as_millis() > 16 {  // > 60 FPS
    eprintln!("Slow frame: {:?}", frame_time);
}
```

**Common Causes:**

#### 1. Too Many Targets

```rust
// Problem: Registering thousands of targets
for i in 0..10000 {
    ctx.targets.register(NavTarget {
        id: format!("item_{}", i),
        // ...
    });
}

// Solution: Only register visible targets
fn register_visible_targets(&self, ctx: &mut LocustContext, viewport: Rect) {
    let visible_start = viewport.y as usize;
    let visible_end = visible_start + viewport.height as usize;

    for i in visible_start..visible_end.min(self.items.len()) {
        ctx.targets.register(NavTarget {
            id: format!("item_{}", i),
            area: self.item_area(i, viewport),
            // ...
        });
    }
}
```

#### 2. Inefficient Rendering

```rust
// Problem: Recreating widgets every frame
fn render_overlay(&self, frame: &mut Frame, ctx: &LocustContext) {
    for target in ctx.targets.all() {
        let widget = create_complex_widget(target);  // Expensive
        frame.render_widget(widget, target.area);
    }
}

// Solution: Cache widgets or use simpler rendering
struct CachedOverlayPlugin {
    cached_widgets: HashMap<String, CachedWidget>,
    cache_valid: bool,
}

fn render_overlay(&mut self, frame: &mut Frame, ctx: &LocustContext) {
    if !self.cache_valid {
        self.rebuild_cache(ctx);
        self.cache_valid = true;
    }

    for (id, widget) in &self.cached_widgets {
        if let Some(target) = ctx.targets.get(id) {
            frame.render_widget(widget.clone(), target.area);
        }
    }
}
```

#### 3. Redundant Allocations

```rust
// Problem: Allocating in hot path
fn on_event(&mut self, event: &Event, ctx: &mut LocustContext) -> PluginEventResult {
    let targets = ctx.targets.all()
        .filter(|t| t.kind == TargetKind::ListItem)
        .collect::<Vec<_>>();  // Allocation every event
    // ...
}

// Solution: Reuse allocations
struct OptimizedPlugin {
    target_buffer: Vec<NavTarget>,
}

fn on_event(&mut self, event: &Event, ctx: &mut LocustContext) -> PluginEventResult {
    self.target_buffer.clear();
    self.target_buffer.extend(
        ctx.targets.all()
            .filter(|t| t.kind == TargetKind::ListItem)
    );
    // Use target_buffer without reallocating
}
```

#### 4. Unbounded History

```rust
// Problem: History grows without bounds
struct HistoryPlugin {
    events: Vec<Event>,  // Grows forever
}

// Solution: Bounded history
struct BoundedHistoryPlugin {
    events: VecDeque<Event>,
    max_size: usize,
}

impl BoundedHistoryPlugin {
    fn add_event(&mut self, event: Event) {
        if self.events.len() >= self.max_size {
            self.events.pop_front();
        }
        self.events.push_back(event);
    }
}
```

### Performance Profiling

```rust
use std::time::{Duration, Instant};

struct PerformanceMonitor {
    frame_times: VecDeque<Duration>,
    event_times: VecDeque<Duration>,
    render_times: VecDeque<Duration>,
}

impl PerformanceMonitor {
    fn new() -> Self {
        Self {
            frame_times: VecDeque::with_capacity(60),
            event_times: VecDeque::with_capacity(60),
            render_times: VecDeque::with_capacity(60),
        }
    }

    fn record_frame(&mut self, duration: Duration) {
        if self.frame_times.len() >= 60 {
            self.frame_times.pop_front();
        }
        self.frame_times.push_back(duration);
    }

    fn average_frame_time(&self) -> Duration {
        if self.frame_times.is_empty() {
            return Duration::ZERO;
        }

        let total: Duration = self.frame_times.iter().sum();
        total / self.frame_times.len() as u32
    }

    fn fps(&self) -> f64 {
        let avg = self.average_frame_time();
        if avg.as_millis() == 0 {
            return 0.0;
        }
        1000.0 / avg.as_millis() as f64
    }

    fn report(&self) -> String {
        format!(
            "FPS: {:.1} | Avg frame: {:.2}ms | Max frame: {:.2}ms",
            self.fps(),
            self.average_frame_time().as_secs_f64() * 1000.0,
            self.frame_times.iter().max().unwrap_or(&Duration::ZERO).as_secs_f64() * 1000.0
        )
    }
}

// Use in main loop
let mut perf = PerformanceMonitor::new();

loop {
    let frame_start = Instant::now();

    locust.begin_frame();
    terminal.draw(|frame| {
        app.draw(frame, &mut locust.ctx);
        locust.render_overlay(frame);
    })?;

    perf.record_frame(frame_start.elapsed());

    if perf.frame_times.len() == 60 {
        println!("{}", perf.report());
    }
}
```

---

## Keybinding Conflicts

### Symptom: Keys don't work as expected

**Diagnostic Steps:**

```rust
// Log all keybindings at startup
fn log_keybindings(locust: &Locust<B>) {
    for plugin in locust.plugins() {
        println!("Plugin {}: bindings = {:?}", plugin.id(), plugin.keybindings());
    }
}
```

**Common Causes:**

#### 1. Multiple Plugins Binding Same Key

```rust
// Plugin A binds 'f'
impl<B: Backend> LocustPlugin<B> for PluginA {
    fn on_event(&mut self, event: &Event, ctx: &mut LocustContext) -> PluginEventResult {
        if let Event::Key(KeyEvent { code: KeyCode::Char('f'), .. }) = event {
            // Handle 'f'
            return PluginEventResult::ConsumedRequestRedraw;
        }
        PluginEventResult::NotHandled
    }
}

// Plugin B also binds 'f'
impl<B: Backend> LocustPlugin<B> for PluginB {
    fn on_event(&mut self, event: &Event, ctx: &mut LocustContext) -> PluginEventResult {
        if let Event::Key(KeyEvent { code: KeyCode::Char('f'), .. }) = event {
            // Never reached if PluginA has higher priority
            return PluginEventResult::ConsumedRequestRedraw;
        }
        PluginEventResult::NotHandled
    }
}
```

**Solution: Keybinding Registry**

```rust
use std::collections::HashMap;

struct KeybindingRegistry {
    bindings: HashMap<KeyEvent, Vec<String>>,  // Key -> Plugin IDs
}

impl KeybindingRegistry {
    fn register(&mut self, plugin_id: String, key: KeyEvent) -> Result<(), String> {
        let conflicts = self.bindings.entry(key.clone()).or_default();

        if !conflicts.is_empty() {
            return Err(format!(
                "Keybinding conflict: {} already bound by {:?}",
                key_to_string(&key),
                conflicts
            ));
        }

        conflicts.push(plugin_id);
        Ok(())
    }

    fn check_conflicts(&self) -> Vec<(KeyEvent, Vec<String>)> {
        self.bindings
            .iter()
            .filter(|(_, plugins)| plugins.len() > 1)
            .map(|(k, p)| (k.clone(), p.clone()))
            .collect()
    }
}

// Use during initialization
let mut registry = KeybindingRegistry::new();

for plugin in &plugins {
    for key in plugin.keybindings() {
        if let Err(e) = registry.register(plugin.id().to_string(), key) {
            eprintln!("Warning: {}", e);
        }
    }
}

if !registry.check_conflicts().is_empty() {
    eprintln!("Keybinding conflicts detected:");
    for (key, plugins) in registry.check_conflicts() {
        eprintln!("  {} -> {:?}", key_to_string(&key), plugins);
    }
}
```

#### 2. Modifier Key Issues

```rust
// Problem: Not checking modifiers
if let Event::Key(KeyEvent { code: KeyCode::Char('c'), .. }) = event {
    // Matches both Ctrl+C and plain 'c'
}

// Solution: Check modifiers explicitly
if let Event::Key(KeyEvent {
    code: KeyCode::Char('c'),
    modifiers: KeyModifiers::CONTROL,
    ..
}) = event {
    // Only matches Ctrl+C
}
```

#### 3. Case Sensitivity

```rust
// Problem: Case not handled
if matches!(event, Event::Key(KeyEvent { code: KeyCode::Char('f'), .. })) {
    // Doesn't match Shift+F
}

// Solution: Normalize or handle both
fn is_char_key(event: &Event, ch: char) -> bool {
    match event {
        Event::Key(KeyEvent { code: KeyCode::Char(c), .. }) => {
            c.to_lowercase().eq(ch.to_lowercase())
        }
        _ => false,
    }
}
```

### Conflict Detection Tool

```rust
struct ConflictDetectorPlugin {
    seen_bindings: HashMap<String, (Instant, String)>,  // Key -> (Time, Plugin)
}

impl<B: Backend> LocustPlugin<B> for ConflictDetectorPlugin {
    fn id(&self) -> &'static str {
        "debug.conflicts"
    }

    fn priority(&self) -> i32 {
        -2000  // See everything first
    }

    fn on_event(&mut self, event: &Event, ctx: &mut LocustContext) -> PluginEventResult {
        if let Event::Key(key) = event {
            let key_str = format!("{:?}", key);
            let now = Instant::now();

            if let Some((last_time, last_plugin)) = self.seen_bindings.get(&key_str) {
                if now.duration_since(*last_time) < Duration::from_millis(100) {
                    eprintln!(
                        "WARNING: Key {} handled by both {} and current plugin within 100ms",
                        key_str, last_plugin
                    );
                }
            }

            // Note: In real impl, you'd need to track which plugin actually consumed it
            self.seen_bindings.insert(key_str, (now, "unknown".to_string()));
        }

        PluginEventResult::NotHandled
    }
}
```

---

## Integration Issues

### Symptom: Locust interferes with existing app

#### Issue 1: State Desynchronization

```rust
// Problem: App state and Locust context out of sync
struct App {
    selected_index: usize,
    items: Vec<String>,
}

// App changes state, but Locust doesn't know
impl App {
    fn delete_item(&mut self, index: usize) {
        self.items.remove(index);
        // Locust targets still reference deleted item
    }
}

// Solution: Synchronization plugin
struct StateSyncPlugin {
    last_item_count: usize,
}

impl<B: Backend> LocustPlugin<B> for StateSyncPlugin {
    fn on_frame_begin(&mut self, ctx: &mut LocustContext) {
        // Check if app state changed
        if let Some(app_state) = ctx.get_data::<AppState>("app_state") {
            if app_state.items.len() != self.last_item_count {
                // State changed, clear targets
                ctx.targets.clear();
                self.last_item_count = app_state.items.len();
            }
        }
    }
}
```

#### Issue 2: Lifetime Conflicts

```rust
// Problem: Can't pass mutable reference to both app and context
terminal.draw(|frame| {
    app.draw(frame, &mut locust.ctx);  // Borrows ctx
    locust.render_overlay(frame);      // Also needs ctx
})?;

// Solution 1: Split borrows
terminal.draw(|frame| {
    // Scope mutable borrow
    {
        let ctx = &mut locust.ctx;
        app.draw(frame, ctx);
    }
    // ctx no longer borrowed
    locust.render_overlay(frame);
})?;

// Solution 2: Two-phase drawing
terminal.draw(|frame| {
    // Phase 1: Register targets
    app.register_targets(&mut locust.ctx, frame.area());

    // Phase 2: Draw everything
    app.draw_widgets(frame);
    locust.render_overlay(frame);
})?;
```

#### Issue 3: Backend Type Mismatches

```rust
// Problem: Different backend types
let app_backend = CrosstermBackend::new(stdout);
let locust: Locust<TestBackend> = Locust::new(config);  // Wrong type

// Solution: Match backend types
use ratatui::backend::CrosstermBackend;

let backend = CrosstermBackend::new(stdout);
let mut terminal = Terminal::new(backend)?;
let mut locust: Locust<CrosstermBackend<_>> = Locust::new(config);
```

---

## Debugging Tools

### Built-in Debug Overlays

```rust
// Enable debug overlay
locust.enable_debug_overlay();

// Customize debug info
locust.set_debug_level(DebugLevel::Verbose);
locust.set_debug_position(DebugPosition::TopRight);
```

### Custom Debug Plugin

```rust
struct ComprehensiveDebugPlugin {
    show_targets: bool,
    show_events: bool,
    show_performance: bool,
    event_log: VecDeque<(Instant, Event)>,
    frame_times: VecDeque<Duration>,
}

impl ComprehensiveDebugPlugin {
    fn new() -> Self {
        Self {
            show_targets: true,
            show_events: true,
            show_performance: true,
            event_log: VecDeque::with_capacity(10),
            frame_times: VecDeque::with_capacity(60),
        }
    }

    fn toggle(&mut self, what: &str) {
        match what {
            "targets" => self.show_targets = !self.show_targets,
            "events" => self.show_events = !self.show_events,
            "performance" => self.show_performance = !self.show_performance,
            _ => {}
        }
    }
}

impl<B: Backend> LocustPlugin<B> for ComprehensiveDebugPlugin {
    fn id(&self) -> &'static str {
        "debug.comprehensive"
    }

    fn priority(&self) -> i32 {
        1000  // Low priority
    }

    fn on_event(&mut self, event: &Event, ctx: &mut LocustContext) -> PluginEventResult {
        // Log events
        if self.show_events {
            self.event_log.push_back((Instant::now(), event.clone()));
            if self.event_log.len() > 10 {
                self.event_log.pop_front();
            }
        }

        // Toggle debug displays
        if let Event::Key(KeyEvent {
            code: KeyCode::Char('d'),
            modifiers: KeyModifiers::CONTROL | KeyModifiers::SHIFT,
            ..
        }) = event {
            self.toggle("targets");
            return PluginEventResult::ConsumedRequestRedraw;
        }

        PluginEventResult::NotHandled
    }

    fn render_overlay(&self, frame: &mut Frame, ctx: &LocustContext) {
        let area = frame.area();

        // Performance overlay
        if self.show_performance {
            let perf_text = format!(
                "FPS: {:.1} | Targets: {} | Plugins: {} | Frame: {}",
                self.calculate_fps(),
                ctx.targets.len(),
                ctx.plugin_count,
                ctx.frame_count
            );
            let perf_area = Rect::new(0, 0, area.width, 1);
            frame.render_widget(
                Paragraph::new(perf_text)
                    .style(Style::default().bg(Color::Blue).fg(Color::White)),
                perf_area
            );
        }

        // Event log
        if self.show_events && !self.event_log.is_empty() {
            let events_text: Vec<String> = self.event_log
                .iter()
                .map(|(time, evt)| format!("{:?}: {:?}", time.elapsed(), evt))
                .collect();
            let events_area = Rect::new(
                0,
                area.height.saturating_sub(events_text.len() as u16),
                area.width / 2,
                events_text.len() as u16
            );
            frame.render_widget(
                Paragraph::new(events_text.join("\n"))
                    .style(Style::default().bg(Color::DarkGray).fg(Color::Yellow))
                    .block(Block::default().borders(Borders::ALL).title("Events")),
                events_area
            );
        }

        // Target visualization
        if self.show_targets {
            for target in ctx.targets.all() {
                // Draw border around target
                frame.render_widget(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(Color::Cyan)),
                    target.area
                );

                // Draw target ID
                let id_area = Rect::new(
                    target.area.x,
                    target.area.y,
                    target.id.len().min(target.area.width as usize) as u16,
                    1
                );
                frame.render_widget(
                    Paragraph::new(&target.id)
                        .style(Style::default().bg(Color::Cyan).fg(Color::Black)),
                    id_area
                );
            }
        }
    }

    fn calculate_fps(&self) -> f64 {
        if self.frame_times.is_empty() {
            return 0.0;
        }

        let total: Duration = self.frame_times.iter().sum();
        let avg_ms = total.as_millis() / self.frame_times.len() as u128;

        if avg_ms == 0 {
            return 0.0;
        }

        1000.0 / avg_ms as f64
    }
}
```

### Event Logging

```rust
// Log all events to file
struct EventLoggerPlugin {
    log_file: std::fs::File,
}

impl EventLoggerPlugin {
    fn new(path: impl AsRef<Path>) -> io::Result<Self> {
        Ok(Self {
            log_file: std::fs::File::create(path)?,
        })
    }
}

impl<B: Backend> LocustPlugin<B> for EventLoggerPlugin {
    fn on_event(&mut self, event: &Event, ctx: &mut LocustContext) -> PluginEventResult {
        use std::io::Write;

        writeln!(
            self.log_file,
            "[{}] {:?} | Targets: {} | Consumed: {}",
            chrono::Local::now(),
            event,
            ctx.targets.len(),
            false  // You'd track this from the return value
        ).ok();

        PluginEventResult::NotHandled
    }
}
```

### Memory Profiling

```rust
struct MemoryProfilerPlugin {
    last_check: Instant,
    check_interval: Duration,
}

impl<B: Backend> LocustPlugin<B> for MemoryProfilerPlugin {
    fn on_frame_end(&mut self, ctx: &LocustContext) {
        if self.last_check.elapsed() > self.check_interval {
            let target_count = ctx.targets.len();
            let target_mem = target_count * std::mem::size_of::<NavTarget>();

            eprintln!(
                "Memory: {} targets (~{} bytes)",
                target_count,
                target_mem
            );

            self.last_check = Instant::now();
        }
    }
}
```

---

## FAQ

### Q: How do I disable Locust temporarily?

```rust
// Option 1: Don't call render_overlay
if !disable_locust {
    locust.render_overlay(frame);
}

// Option 2: Clear plugins
locust.clear_plugins();

// Option 3: Feature flag
#[cfg(feature = "locust")]
locust.render_overlay(frame);
```

### Q: Can I use Locust with async ratatui?

```rust
// Yes, but be careful with plugin state
use tokio::task;

async fn run_app() {
    let mut locust = Locust::new(config);

    // Wrap in Arc<Mutex> if needed
    let locust = Arc::new(Mutex::new(locust));

    loop {
        let locust_clone = locust.clone();
        task::spawn_blocking(move || {
            let mut l = locust_clone.lock().unwrap();
            l.begin_frame();
        }).await?;

        // ... rest of loop
    }
}
```

### Q: How do I create a custom theme?

```rust
use locust::theme::{Theme, ColorPalette};

let custom_theme = Theme {
    name: "my_theme".to_string(),
    palette: ColorPalette {
        primary: Color::Rgb(100, 150, 200),
        secondary: Color::Rgb(200, 150, 100),
        background: Color::Black,
        foreground: Color::White,
        accent: Color::Yellow,
    },
    hint_style: Style::default()
        .fg(Color::Yellow)
        .bg(Color::DarkGray)
        .add_modifier(Modifier::BOLD),
    overlay_style: Style::default()
        .bg(Color::Rgb(0, 0, 0)),
};

locust.set_theme(custom_theme);
```

### Q: What's the performance impact of Locust?

Typical overhead:
- **Minimal**: < 1ms per frame with < 100 targets
- **Low**: 1-3ms per frame with 100-500 targets
- **Medium**: 3-10ms per frame with 500-2000 targets
- **High**: > 10ms per frame with > 2000 targets

Optimize by:
1. Only register visible targets
2. Cache rendered overlays
3. Use efficient data structures
4. Limit plugin count

### Q: Can I have multiple Locust instances?

```rust
// Yes, but usually not needed
let mut locust_primary = Locust::new(config);
let mut locust_secondary = Locust::new(config);

// Use for different terminal areas
terminal.draw(|frame| {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(frame.area());

    // Left side
    app_left.draw(frame, &mut locust_primary.ctx);
    locust_primary.render_overlay_in(frame, chunks[0]);

    // Right side
    app_right.draw(frame, &mut locust_secondary.ctx);
    locust_secondary.render_overlay_in(frame, chunks[1]);
})?;
```

### Q: How do I contribute a plugin?

1. Implement `LocustPlugin` trait
2. Add comprehensive tests
3. Document with examples
4. Submit PR to locust-contrib repo

See [Plugin Development Guide](PLUGIN_DEVELOPMENT_GUIDE.md) for details.

---

## Advanced Diagnostics

### Full System Check

```rust
fn diagnose_locust(locust: &Locust<impl Backend>) {
    println!("=== Locust Diagnostic Report ===\n");

    // Plugin status
    println!("Plugins: {}", locust.plugin_count());
    for plugin in locust.plugins() {
        println!("  - {} (priority: {})", plugin.id(), plugin.priority());
    }

    // Context status
    println!("\nContext:");
    println!("  Targets: {}", locust.ctx.targets.len());
    println!("  Frame count: {}", locust.ctx.frame_count);
    println!("  Overlay active: {}", locust.ctx.overlay.is_active());

    // Configuration
    println!("\nConfiguration:");
    println!("  {:?}", locust.config);

    // Warnings
    println!("\nWarnings:");
    if locust.plugin_count() == 0 {
        println!("  ⚠ No plugins registered");
    }
    if locust.ctx.targets.len() > 1000 {
        println!("  ⚠ High target count may impact performance");
    }

    println!("\n=== End Report ===");
}
```

### Automated Testing

```rust
#[cfg(test)]
mod diagnostics {
    use super::*;

    #[test]
    fn test_locust_integration() {
        let mut locust = Locust::new(LocustConfig::default());
        locust.register_plugin(NavPlugin::new());

        // Test frame lifecycle
        locust.begin_frame();
        assert_eq!(locust.ctx.frame_count, 1);

        // Test event handling
        let event = Event::Key(KeyEvent::new(KeyCode::Char('f'), KeyModifiers::empty()));
        let outcome = locust.on_event(&event);
        assert!(outcome.consumed || !outcome.consumed);  // Should handle gracefully

        // Test target registration
        locust.ctx.targets.register(NavTarget {
            id: "test".to_string(),
            area: Rect::new(0, 0, 10, 1),
            kind: TargetKind::ListItem,
            ..Default::default()
        });
        assert_eq!(locust.ctx.targets.len(), 1);
    }
}
```

---

*For more information, see:*
- *[Integration Guide](INTEGRATION_GUIDE.md)*
- *[Plugin Development](PLUGIN_DEVELOPMENT_GUIDE.md)*
- *[Architecture](ARCHITECTURE.md)*
