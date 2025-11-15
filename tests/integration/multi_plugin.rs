use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use locust::prelude::*;
use ratatui::backend::TestBackend;
use ratatui::Terminal;

#[test]
fn test_all_plugins_together() {
    let mut locust = Locust::<TestBackend>::new(LocustConfig::default());
    locust.register_plugin(NavPlugin::new());
    locust.register_plugin(OmnibarPlugin::new());
    locust.register_plugin(TooltipPlugin::new());
    locust.register_plugin(HighlightPlugin::new());
    assert_eq!(locust.plugin_count(), 4);
}

#[test]
fn test_plugin_event_consumption() {
    let mut locust = Locust::<TestBackend>::new(LocustConfig::default());
    locust.register_plugin(NavPlugin::new());
    let event = Event::Key(KeyEvent::new(KeyCode::Char('f'), KeyModifiers::NONE));
    let outcome = locust.on_event(&event);
    assert!(outcome.consumed);
}

#[test]
fn test_overlay_rendering() {
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut locust = Locust::new(LocustConfig::default());
    locust.register_plugin(NavPlugin::new());
    terminal.draw(|f| locust.render_overlay(f)).unwrap();
}

#[test]
fn test_plugin_independence() {
    let mut locust = Locust::<TestBackend>::new(LocustConfig::default());
    locust.register_plugin(NavPlugin::new());
    assert_eq!(locust.plugin_count(), 1);
    locust.register_plugin(OmnibarPlugin::new());
    assert_eq!(locust.plugin_count(), 2);
}
