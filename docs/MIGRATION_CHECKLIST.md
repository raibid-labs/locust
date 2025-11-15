# Locust Migration Checklist

## Table of Contents

1. [Pre-Migration Checklist](#pre-migration-checklist)
2. [Migration Checklist](#migration-checklist)
3. [Post-Migration Checklist](#post-migration-checklist)
4. [Phase-by-Phase Integration](#phase-by-phase-integration)
5. [Testing Checklist](#testing-checklist)
6. [Rollback Plan](#rollback-plan)

---

## Pre-Migration Checklist

### Code Audit

- [ ] **Review current event handling**
  - Document all key bindings in use
  - Identify potential conflicts with Locust default keys (f, Ctrl+P, etc.)
  - List custom event handlers that might consume events

- [ ] **Identify navigation points**
  - List all lists, tables, tabs, and other navigable UI elements
  - Document current navigation patterns (arrow keys, vim keys, etc.)
  - Identify pain points in current navigation

- [ ] **Document desired features**
  - Which Locust plugins do you need? (Nav, Omnibar, Tooltip, Tour)
  - Custom commands to add to Omnibar
  - Tooltips needed for UI elements
  - Onboarding tour steps

- [ ] **Review state management**
  - How is app state currently managed?
  - Will Locust context need access to app state?
  - Plan integration between app state and Locust context

- [ ] **Audit rendering pipeline**
  - Identify where UI rendering happens
  - Confirm draw functions can accept LocustContext parameter
  - Check for Z-order issues (overlays rendering under app UI)

### Technical Preparation

- [ ] **Check compatibility**
  - ratatui version >= 0.26
  - crossterm version >= 0.27
  - Backend type consistency (all CrosstermBackend, TestBackend, etc.)

- [ ] **Set up feature flags** (optional)
  ```toml
  [features]
  default = []
  locust = ["dep:locust"]
  ```

- [ ] **Create integration branch**
  ```bash
  git checkout -b feature/locust-integration
  ```

- [ ] **Backup current state**
  - Tag current version
  - Document current metrics (if available)
  - Create rollback plan

### Documentation

- [ ] **Document current user flows**
  - How do users navigate currently?
  - What are common operations?
  - Where do users get stuck?

- [ ] **Plan user communication**
  - Changelog notes
  - Migration guide for users
  - New features announcement

---

## Migration Checklist

### Phase 1: Basic Setup (Est. 1-2 hours)

#### Add Dependencies

- [ ] Add locust to Cargo.toml
  ```toml
  [dependencies]
  locust = "0.1"
  ratatui = "0.28"
  crossterm = "0.28"
  ```

- [ ] Run `cargo check` to verify compilation

#### Initialize Locust

- [ ] Create Locust instance in main
  ```rust
  let mut locust = Locust::new(LocustConfig::default());
  ```

- [ ] Register NavPlugin
  ```rust
  locust.register_plugin(NavPlugin::new());
  ```

- [ ] Add `begin_frame()` to event loop
  ```rust
  loop {
      locust.begin_frame();
      // ... rest of loop
  }
  ```

- [ ] Add `render_overlay()` to draw call
  ```rust
  terminal.draw(|frame| {
      app.draw(frame);
      locust.render_overlay(frame);  // After app rendering
  })?;
  ```

- [ ] Update event handling
  ```rust
  let outcome = locust.on_event(&event);
  if !outcome.consumed {
      app.handle_event(event);
  }
  ```

- [ ] Test basic compilation and runtime

#### Verify Integration

- [ ] App still runs without errors
- [ ] App still renders correctly
- [ ] App still responds to input
- [ ] No visible changes yet (nav not configured)

### Phase 2: Navigation Targets (Est. 2-4 hours)

#### Update Draw Signatures

- [ ] Add `LocustContext` parameter to draw functions
  ```rust
  // Before
  fn draw(&self, frame: &mut Frame, area: Rect)

  // After
  fn draw(&self, frame: &mut Frame, area: Rect, ctx: &mut LocustContext)
  ```

- [ ] Update all call sites

#### Register Targets

- [ ] Identify first component to make navigable (start with simplest)

- [ ] Add target registration in draw function
  ```rust
  ctx.targets.register(NavTarget {
      id: format!("item_{}", i),
      area: item_area,
      kind: TargetKind::ListItem,
      priority: 0,
      metadata: hashmap! {
          "index".to_string() => i.to_string(),
      },
      actions: vec![TargetAction::Select],
  });
  ```

- [ ] Test navigation hints appear (press 'f')

- [ ] Repeat for all major components

#### Handle Activations

- [ ] Create integration plugin
  ```rust
  struct MyAppIntegration {
      app_tx: mpsc::Sender<AppCommand>,
  }
  ```

- [ ] Implement `LocustPlugin` trait

- [ ] Handle target activations
  ```rust
  if let Some(target_id) = ctx.get_data::<String>("nav_activated") {
      // Handle the activation
  }
  ```

- [ ] Test navigation actually works (hints → selection)

### Phase 3: Command Palette (Est. 1-2 hours)

#### Register Omnibar Plugin

- [ ] Create OmnibarPlugin
  ```rust
  let mut omnibar = OmnibarPlugin::new();
  ```

- [ ] Register common commands
  ```rust
  omnibar.register_command(Command {
      id: "app.save".to_string(),
      name: "Save".to_string(),
      description: Some("Save current file".to_string()),
      aliases: vec!["save".to_string(), "s".to_string()],
      category: Some("File".to_string()),
  });
  ```

- [ ] Register plugin with Locust
  ```rust
  locust.register_plugin(omnibar);
  ```

#### Handle Command Execution

- [ ] Check for executed commands in integration plugin
  ```rust
  if let Some(cmd_id) = ctx.get_data::<String>("omnibar_executed") {
      match cmd_id.as_str() {
          "app.save" => // Handle save
          _ => {}
      }
  }
  ```

- [ ] Test command palette (Ctrl+P)

- [ ] Test command search and execution

### Phase 4: Tooltips (Est. 1 hour)

#### Register Tooltip Plugin

- [ ] Create TooltipPlugin
  ```rust
  let tooltip = TooltipPlugin::new();
  locust.register_plugin(tooltip);
  ```

#### Add Tooltips

- [ ] Register tooltips for targets
  ```rust
  ctx.tooltips.register(
      target_id.clone(),
      "Tooltip content here",
      TooltipPosition::Auto,
  );
  ```

- [ ] Build helpful tooltip content
  ```rust
  let tooltip = format!(
      "Name: {}\nType: {}\n\n💡 Press Enter to select",
      item.name, item.type
  );
  ```

- [ ] Test tooltips appear on navigation

### Phase 5: Configuration (Est. 1 hour)

#### Create Configuration File

- [ ] Create `locust.toml` in config directory
  ```toml
  [theme]
  primary_color = "#00FFFF"
  secondary_color = "#FFFF00"

  [keybindings]
  navigate = "f"
  omnibar = "ctrl+p"

  [plugins]
  nav_enabled = true
  omnibar_enabled = true
  tooltip_enabled = true
  ```

- [ ] Load configuration
  ```rust
  let config = LocustConfig::load_from_file("locust.toml")?;
  let mut locust = Locust::new(config);
  ```

#### Create Custom Theme

- [ ] Define theme
  ```rust
  let theme = Theme {
      name: "my_app".to_string(),
      palette: ColorPalette { /* ... */ },
      hint_style: Style::default().fg(Color::Yellow),
      overlay_style: Style::default().bg(Color::DarkGray),
  };
  ```

- [ ] Apply theme
  ```rust
  locust.set_theme(theme);
  ```

### Phase 6: Onboarding Tour (Est. 1-2 hours)

#### Create Tour

- [ ] Create TourPlugin with steps
  ```rust
  let mut tour = TourPlugin::new();

  tour.add_step(TourStep {
      id: "welcome".to_string(),
      title: "Welcome!".to_string(),
      description: "Let's explore the new features.".to_string(),
      target_id: None,
      position: TooltipPosition::Center,
      highlight: false,
  });

  // Add more steps...
  ```

- [ ] Register tour plugin
  ```rust
  locust.register_plugin(tour);
  ```

#### First-Run Detection

- [ ] Implement first-run detection
  ```rust
  fn is_first_run() -> bool {
      !Path::new(".config/myapp/initialized").exists()
  }
  ```

- [ ] Start tour on first run
  ```rust
  if is_first_run()? {
      if let Some(tour) = locust.get_plugin_mut::<TourPlugin>("highlight.tour") {
          tour.start();
      }
      mark_initialized()?;
  }
  ```

---

## Post-Migration Checklist

### Testing

- [ ] **Functionality tests**
  - All features still work
  - Navigation works as expected
  - Command palette accessible
  - Tooltips appear
  - Tour runs on first launch

- [ ] **Performance tests**
  - Measure frame times
  - Check for regressions
  - Profile hot paths
  - Memory usage acceptable

- [ ] **Integration tests**
  - Events not being eaten inappropriately
  - State synchronization works
  - No race conditions

- [ ] **User acceptance testing**
  - Internal team testing
  - Beta user testing
  - Gather feedback

### Documentation

- [ ] **Update README**
  - Mention new navigation features
  - Document new keybindings
  - Link to configuration guide

- [ ] **Create LOCUST_INTEGRATION.md**
  - Document Locust features enabled
  - Explain configuration options
  - Troubleshooting section

- [ ] **Update CHANGELOG**
  - List new features
  - Document breaking changes (if any)
  - Migration notes for users

- [ ] **Update user documentation**
  - Screenshots of new features
  - Tutorial for navigation
  - Command palette guide

### Cleanup

- [ ] **Remove temporary code**
  - Remove debug logging
  - Remove feature flags (if not needed)
  - Clean up commented code

- [ ] **Code review**
  - Review all changes
  - Check for potential issues
  - Ensure code quality

- [ ] **Performance optimization**
  - Address any performance issues found
  - Optimize target registration
  - Cache where appropriate

### Deployment Preparation

- [ ] **Version bump**
  - Update version number
  - Follow semantic versioning

- [ ] **Create release notes**
  - Highlight new features
  - Include migration guide
  - Known issues section

- [ ] **Tag release**
  ```bash
  git tag -a v1.0.0-locust -m "Locust integration release"
  git push origin v1.0.0-locust
  ```

---

## Phase-by-Phase Integration

### Gradual Migration (Recommended)

#### Week 1: Foundation
- [ ] Day 1: Add dependencies, basic setup
- [ ] Day 2: First component navigable
- [ ] Day 3: All major components navigable
- [ ] Day 4-5: Bug fixes, polish

#### Week 2: Enhancement
- [ ] Day 1: Add command palette
- [ ] Day 2: Register all commands
- [ ] Day 3: Add tooltips
- [ ] Day 4-5: Create onboarding tour

#### Week 3: Polish & Launch
- [ ] Day 1-2: Testing, bug fixes
- [ ] Day 3: Documentation
- [ ] Day 4: Beta release
- [ ] Day 5: Gather feedback, iterate

### Fast-Track Migration (Experienced team)

#### Day 1: Setup
- [ ] Morning: Dependencies, basic integration
- [ ] Afternoon: First component navigable

#### Day 2: Core Features
- [ ] Morning: All components navigable
- [ ] Afternoon: Command palette, tooltips

#### Day 3: Polish & Launch
- [ ] Morning: Tour, configuration
- [ ] Afternoon: Testing, documentation, release

---

## Testing Checklist

### Manual Testing

#### Navigation Tests

- [ ] Press 'f', hints appear
- [ ] Type hint, navigates to item
- [ ] Hints don't overlap
- [ ] Hints are readable
- [ ] Navigation works in all components
- [ ] ESC cancels navigation mode

#### Command Palette Tests

- [ ] Ctrl+P opens palette
- [ ] Search finds commands
- [ ] Aliases work
- [ ] Categories displayed
- [ ] Enter executes command
- [ ] ESC closes palette
- [ ] Commands actually execute

#### Tooltip Tests

- [ ] Tooltips appear on navigation
- [ ] Tooltips positioned correctly
- [ ] Tooltip content is helpful
- [ ] Tooltips don't block UI
- [ ] Tooltips dismiss correctly

#### Tour Tests

- [ ] Tour starts on first run
- [ ] Tour steps display correctly
- [ ] Highlights work
- [ ] Next/Previous navigation
- [ ] Skip functionality
- [ ] Tour completes properly

### Automated Testing

#### Integration Tests

```rust
#[test]
fn test_locust_integration() {
    let mut locust = Locust::new(LocustConfig::default());
    locust.register_plugin(NavPlugin::new());

    // Test event handling
    let event = Event::Key(KeyEvent::new(KeyCode::Char('f'), KeyModifiers::empty()));
    let outcome = locust.on_event(&event);
    assert!(outcome.consumed || !outcome.consumed);

    // Test target registration
    locust.begin_frame();
    locust.ctx.targets.register(NavTarget { /* ... */ });
    assert_eq!(locust.ctx.targets.len(), 1);
}
```

- [ ] Event handling tests
- [ ] Target registration tests
- [ ] Plugin lifecycle tests
- [ ] State synchronization tests

#### Performance Tests

```rust
#[test]
fn test_performance() {
    let mut locust = Locust::new(LocustConfig::default());
    locust.register_plugin(NavPlugin::new());

    // Register many targets
    locust.begin_frame();
    for i in 0..1000 {
        locust.ctx.targets.register(NavTarget { /* ... */ });
    }

    // Measure frame time
    let start = Instant::now();
    locust.render_overlay(&mut frame);
    let elapsed = start.elapsed();

    assert!(elapsed < Duration::from_millis(16));  // 60 FPS
}
```

- [ ] Frame time tests
- [ ] Memory usage tests
- [ ] Target registration performance
- [ ] Event processing performance

### Regression Testing

- [ ] All existing functionality works
- [ ] No new bugs introduced
- [ ] Performance not degraded
- [ ] User workflows unaffected

---

## Rollback Plan

### If Integration Fails

#### Immediate Rollback

- [ ] Revert to previous git tag
  ```bash
  git checkout <previous-tag>
  ```

- [ ] Remove Locust dependency
  ```bash
  cargo remove locust
  ```

- [ ] Remove feature flag (if used)
  ```toml
  # Remove from Cargo.toml
  [features]
  locust = [...]
  ```

#### Partial Rollback

- [ ] Disable specific plugins
  ```rust
  // Don't register problematic plugin
  // locust.register_plugin(ProblematicPlugin::new());
  ```

- [ ] Feature flag specific features
  ```rust
  #[cfg(feature = "locust-nav")]
  locust.register_plugin(NavPlugin::new());
  ```

- [ ] Disable Locust entirely at runtime
  ```rust
  let enable_locust = env::var("ENABLE_LOCUST").is_ok();
  if enable_locust {
      locust.render_overlay(frame);
  }
  ```

### Communicate Issues

- [ ] Document what went wrong
- [ ] File issues on GitHub
- [ ] Communicate to users
- [ ] Plan next steps

---

## Additional Resources

### Templates

#### Integration Plugin Template

```rust
use locust::prelude::*;
use std::sync::mpsc::Sender;

pub struct MyAppIntegration {
    command_tx: Sender<AppCommand>,
}

impl MyAppIntegration {
    pub fn new(command_tx: Sender<AppCommand>) -> Self {
        Self { command_tx }
    }
}

impl<B: Backend> LocustPlugin<B> for MyAppIntegration {
    fn id(&self) -> &'static str {
        "myapp.integration"
    }

    fn priority(&self) -> i32 {
        50  // Between NavPlugin (0) and app (100)
    }

    fn on_event(&mut self, event: &Event, ctx: &mut LocustContext) -> PluginEventResult {
        // Handle nav activations
        if let Some(target_id) = ctx.get_data::<String>("nav_activated") {
            // Handle activation
            ctx.remove_data("nav_activated");
            return PluginEventResult::ConsumedRequestRedraw;
        }

        // Handle command executions
        if let Some(cmd_id) = ctx.get_data::<String>("omnibar_executed") {
            // Handle command
            ctx.remove_data("omnibar_executed");
            return PluginEventResult::ConsumedRequestRedraw;
        }

        PluginEventResult::NotHandled
    }

    fn render_overlay(&self, frame: &mut Frame, ctx: &LocustContext) {
        // Optional: render app-specific overlays
    }
}
```

#### Configuration Template

```toml
# locust.toml

[theme]
name = "my_app_theme"
primary_color = "#00FFFF"
secondary_color = "#FFFF00"
background_color = "#000000"
foreground_color = "#FFFFFF"
accent_color = "#FF00FF"

[keybindings]
navigate = "f"
omnibar = "ctrl+p"
tooltip_toggle = "?"

[nav]
enabled = true
hint_style = "alphabetic"  # alphabetic, numeric, or custom
show_borders = true
highlight_selected = true

[omnibar]
enabled = true
max_results = 10
fuzzy_search = true

[tooltip]
enabled = true
auto_show = true
delay_ms = 500

[tour]
enabled = true
auto_start_first_run = true
```

---

## Appendix: Common Issues & Solutions

### Issue: Events Not Captured

**Symptoms**: Pressing 'f' doesn't show hints

**Solutions**:
1. Check plugin registration
2. Verify event handling order
3. Check event consumption

### Issue: Overlays Not Visible

**Symptoms**: Hints/tooltips don't appear

**Solutions**:
1. Verify `render_overlay()` called after app rendering
2. Check `begin_frame()` called each frame
3. Verify targets registered during draw

### Issue: Performance Degradation

**Symptoms**: App becomes laggy

**Solutions**:
1. Limit target registration to visible items
2. Profile frame times
3. Optimize hot paths
4. Consider lazy rendering

### Issue: Keybinding Conflicts

**Symptoms**: App features stop working

**Solutions**:
1. Change Locust keybindings in config
2. Use Ctrl modifiers for Locust keys
3. Check event consumption order

---

## Related Documentation

This migration checklist connects with other Locust documentation to guide your migration:

### Migration Resources
- **[INTEGRATION_GUIDE.md](INTEGRATION_GUIDE.md)** - Complete integration instructions
- **[EXAMPLES.md](EXAMPLES.md)** - Migration examples
- **[CASE_STUDIES.md](CASE_STUDIES.md)** - Real-world migration stories

### Implementation Guides
- **[PLUGINS.md](PLUGINS.md)** - Available plugins for migration
- **[WIDGET_ADAPTERS.md](WIDGET_ADAPTERS.md)** - Widget adapter migration
- **[PLUGIN_DEVELOPMENT_GUIDE.md](PLUGIN_DEVELOPMENT_GUIDE.md)** - Custom plugin migration

### Configuration
- **[CONFIGURATION.md](CONFIGURATION.md)** - Configuration migration
- **[THEMING.md](THEMING.md)** - Theme migration
- **[KEYBINDINGS.md](KEYBINDINGS.md)** - Keybinding migration

### System Understanding
- **[ARCHITECTURE.md](ARCHITECTURE.md)** - Architecture changes
- **[API_PATTERNS.md](API_PATTERNS.md)** - Migration patterns

### Troubleshooting
- **[TROUBLESHOOTING.md](TROUBLESHOOTING.md)** - Migration issues and solutions

### Project Documentation
- **[README.md](../README.md#quick-start)** - Quick migration overview
- **[ROADMAP.md](ROADMAP.md)** - Future migration features
- **[CONTRIBUTING.md](../CONTRIBUTING.md)** - Contributing migration improvements

*For more information, see:*
- *[Integration Guide](INTEGRATION_GUIDE.md)*
- *[Troubleshooting](TROUBLESHOOTING.md)*
- *[Case Studies](CASE_STUDIES.md)*
