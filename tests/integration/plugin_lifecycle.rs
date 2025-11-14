//! Integration tests for plugin lifecycle.

use locust::prelude::*;
use ratatui::backend::TestBackend;
use ratatui::Frame;
use std::cell::RefCell;
use std::rc::Rc;

/// Mock plugin for testing lifecycle hooks.
#[derive(Clone)]
struct MockPlugin {
    id: &'static str,
    priority: i32,
    lifecycle: Rc<RefCell<Vec<String>>>,
}

impl MockPlugin {
    fn new(id: &'static str, priority: i32) -> Self {
        Self {
            id,
            priority,
            lifecycle: Rc::new(RefCell::new(Vec::new())),
        }
    }

    fn lifecycle(&self) -> Vec<String> {
        self.lifecycle.borrow().clone()
    }
}

impl<B: Backend> LocustPlugin<B> for MockPlugin {
    fn id(&self) -> &'static str {
        self.id
    }

    fn priority(&self) -> i32 {
        self.priority
    }

    fn init(&mut self, _ctx: &mut LocustContext) {
        self.lifecycle
            .borrow_mut()
            .push(format!("{}:init", self.id));
    }

    fn on_event(&mut self, event: &Event, _ctx: &mut LocustContext) -> PluginEventResult {
        self.lifecycle
            .borrow_mut()
            .push(format!("{}:on_event", self.id));

        // High priority plugin consumes 'x' events
        if self.priority < 100 {
            if let Event::Key(KeyEvent {
                code: KeyCode::Char('x'),
                ..
            }) = event
            {
                return PluginEventResult::ConsumedRequestRedraw;
            }
        }

        PluginEventResult::NotHandled
    }

    fn render_overlay(&self, _frame: &mut Frame, _ctx: &LocustContext) {
        self.lifecycle
            .borrow_mut()
            .push(format!("{}:render", self.id));
    }

    fn cleanup(&mut self, _ctx: &mut LocustContext) {
        self.lifecycle
            .borrow_mut()
            .push(format!("{}:cleanup", self.id));
    }
}

#[test]
fn test_plugin_registration_and_init() {
    let mut locust = Locust::<TestBackend>::new(LocustConfig::default());

    let plugin1 = MockPlugin::new("plugin1", 100);
    let lifecycle1 = plugin1.lifecycle.clone();

    locust.register_plugin(plugin1);

    // Init should have been called
    assert_eq!(lifecycle1.borrow().len(), 1);
    assert_eq!(lifecycle1.borrow()[0], "plugin1:init");
}

#[test]
fn test_plugin_priority_ordering() {
    let mut locust = Locust::<TestBackend>::new(LocustConfig::default());

    let plugin_low = MockPlugin::new("low_priority", 200);
    let plugin_high = MockPlugin::new("high_priority", 50);
    let plugin_mid = MockPlugin::new("mid_priority", 100);

    let lifecycle_low = plugin_low.lifecycle.clone();
    let lifecycle_high = plugin_high.lifecycle.clone();
    let lifecycle_mid = plugin_mid.lifecycle.clone();

    // Register in non-priority order
    locust.register_plugin(plugin_low);
    locust.register_plugin(plugin_high);
    locust.register_plugin(plugin_mid);

    // Plugins should be sorted by priority after registration
    assert_eq!(locust.plugin_count(), 3);

    // Send an event that doesn't get consumed
    use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
    let event = Event::Key(KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE));
    let _ = locust.on_event(&event);

    // All plugins should have received the event in priority order
    assert_eq!(lifecycle_high.borrow()[1], "high_priority:on_event");
    assert_eq!(lifecycle_mid.borrow()[1], "mid_priority:on_event");
    assert_eq!(lifecycle_low.borrow()[1], "low_priority:on_event");
}

#[test]
fn test_plugin_event_consumption() {
    let mut locust = Locust::<TestBackend>::new(LocustConfig::default());

    let plugin_high = MockPlugin::new("high_priority", 50);
    let plugin_low = MockPlugin::new("low_priority", 200);

    let lifecycle_high = plugin_high.lifecycle.clone();
    let lifecycle_low = plugin_low.lifecycle.clone();

    locust.register_plugin(plugin_high);
    locust.register_plugin(plugin_low);

    // Clear init events
    lifecycle_high.borrow_mut().clear();
    lifecycle_low.borrow_mut().clear();

    // Send 'x' event that high priority plugin will consume
    use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
    let event = Event::Key(KeyEvent::new(KeyCode::Char('x'), KeyModifiers::NONE));
    let outcome = locust.on_event(&event);

    assert!(outcome.consumed);
    assert!(outcome.request_redraw);

    // High priority plugin should have handled it
    assert_eq!(lifecycle_high.borrow().len(), 1);
    assert_eq!(lifecycle_high.borrow()[0], "high_priority:on_event");

    // Low priority plugin should NOT have received it
    assert_eq!(lifecycle_low.borrow().len(), 0);
}

#[test]
fn test_plugin_render_order() {
    let mut locust = Locust::<TestBackend>::new(LocustConfig::default());

    let plugin1 = MockPlugin::new("plugin1", 100);
    let plugin2 = MockPlugin::new("plugin2", 50);

    let lifecycle1 = plugin1.lifecycle.clone();
    let lifecycle2 = plugin2.lifecycle.clone();

    locust.register_plugin(plugin1);
    locust.register_plugin(plugin2);

    // Clear init events
    lifecycle1.borrow_mut().clear();
    lifecycle2.borrow_mut().clear();

    // Create a test backend and render
    let backend = TestBackend::new(80, 24);
    let mut terminal = ratatui::Terminal::new(backend).unwrap();

    terminal
        .draw(|frame| {
            locust.render_overlay(frame);
        })
        .unwrap();

    // Both plugins should have rendered in priority order
    assert_eq!(lifecycle2.borrow()[0], "plugin2:render");
    assert_eq!(lifecycle1.borrow()[0], "plugin1:render");
}

#[test]
fn test_plugin_cleanup_on_drop() {
    let plugin = MockPlugin::new("cleanup_test", 100);
    let lifecycle = plugin.lifecycle.clone();

    {
        let mut locust = Locust::<TestBackend>::new(LocustConfig::default());
        locust.register_plugin(plugin);

        assert_eq!(lifecycle.borrow().len(), 1);
        assert_eq!(lifecycle.borrow()[0], "cleanup_test:init");
    } // locust dropped here

    // Cleanup should have been called
    assert_eq!(lifecycle.borrow().len(), 2);
    assert_eq!(lifecycle.borrow()[1], "cleanup_test:cleanup");
}

#[test]
fn test_frame_lifecycle() {
    let mut locust = Locust::<TestBackend>::new(LocustConfig::default());

    assert_eq!(locust.ctx.frame_count, 0);

    locust.begin_frame();
    assert_eq!(locust.ctx.frame_count, 1);
    assert!(locust.ctx.targets.is_empty());
    assert!(!locust.ctx.overlay.has_overlay);

    locust.begin_frame();
    assert_eq!(locust.ctx.frame_count, 2);
}

#[test]
fn test_has_plugin() {
    let mut locust = Locust::<TestBackend>::new(LocustConfig::default());

    assert!(!locust.has_plugin("test_plugin"));

    let plugin = MockPlugin::new("test_plugin", 100);
    locust.register_plugin(plugin);

    assert!(locust.has_plugin("test_plugin"));
    assert!(!locust.has_plugin("other_plugin"));
}
