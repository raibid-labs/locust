//! Custom Plugin Development Example
//!
//! This example demonstrates how to create your own custom Locust plugins
//! for application-specific functionality.
//!
//! Covers:
//! - Creating a custom plugin from scratch
//! - Handling custom events
//! - Rendering custom overlays
//! - Managing plugin state
//! - Integrating with app state
//!
//! Run with: cargo run --example custom_plugin

use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::Rect,
    style::{Color, Style},
    text::Line,
    widgets::{Block, Borders, Paragraph},
    Frame, Terminal,
};
use std::{collections::VecDeque, io, time::Instant};

// ============================================================================
// Custom Plugin #1: Event Logger
// ============================================================================
//
// Logs all events and displays recent history

struct EventLoggerPlugin {
    events: VecDeque<(Instant, String)>,
    max_events: usize,
    enabled: bool,
}

impl EventLoggerPlugin {
    pub fn new(max_events: usize) -> Self {
        Self {
            events: VecDeque::with_capacity(max_events),
            max_events,
            enabled: true,
        }
    }

    pub fn toggle(&mut self) {
        self.enabled = !self.enabled;
    }

    pub fn add_event(&mut self, event: &Event) {
        if self.enabled {
            let event_str = format!("{:?}", event);
            self.events.push_back((Instant::now(), event_str));

            if self.events.len() > self.max_events {
                self.events.pop_front();
            }
        }
    }

    pub fn recent_events(&self, count: usize) -> Vec<String> {
        self.events
            .iter()
            .rev()
            .take(count)
            .map(|(time, event)| format!("{:>4}ms: {}", time.elapsed().as_millis(), event))
            .collect()
    }
}

// In a real implementation, this would implement LocustPlugin<B>
// impl<B: Backend> LocustPlugin<B> for EventLoggerPlugin {
//     fn id(&self) -> &'static str {
//         "app.event_logger"
//     }
//
//     fn priority(&self) -> i32 {
//         -100  // High priority to see events first
//     }
//
//     fn on_event(&mut self, event: &Event, ctx: &mut LocustContext) -> PluginEventResult {
//         self.add_event(event);
//
//         // Check for toggle key
//         if matches!(event, Event::Key(KeyEvent {
//             code: KeyCode::Char('l'),
//             modifiers: KeyModifiers::CONTROL,
//             ..
//         })) {
//             self.toggle();
//             return PluginEventResult::ConsumedRequestRedraw;
//         }
//
//         PluginEventResult::NotHandled
//     }
//
//     fn render_overlay(&self, frame: &mut Frame, ctx: &LocustContext) {
//         if !self.enabled {
//             return;
//         }
//
//         let events = self.recent_events(10);
//         let height = events.len().min(10) as u16 + 2;
//         let width = 60;
//
//         let area = Rect::new(
//             frame.area().width.saturating_sub(width),
//             0,
//             width,
//             height,
//         );
//
//         let block = Block::default()
//             .title("Event Log (Ctrl+L to toggle)")
//             .borders(Borders::ALL)
//             .border_style(Style::default().fg(Color::Cyan));
//
//         let text = events.join("\n");
//         let paragraph = Paragraph::new(text)
//             .block(block)
//             .style(Style::default().fg(Color::White));
//
//         frame.render_widget(paragraph, area);
//     }
// }

// ============================================================================
// Custom Plugin #2: Keystroke Recorder
// ============================================================================
//
// Records and replays keystroke sequences

#[derive(Clone, Debug)]
struct Recording {
    name: String,
    keystrokes: Vec<(std::time::Duration, KeyCode)>,
}

struct KeystrokeRecorderPlugin {
    recording: bool,
    replaying: bool,
    current_recording: Option<Recording>,
    recordings: Vec<Recording>,
    record_start: Option<Instant>,
    replay_index: usize,
}

impl KeystrokeRecorderPlugin {
    pub fn new() -> Self {
        Self {
            recording: false,
            replaying: false,
            current_recording: None,
            recordings: Vec::new(),
            record_start: None,
            replay_index: 0,
        }
    }

    pub fn start_recording(&mut self, name: String) {
        self.recording = true;
        self.record_start = Some(Instant::now());
        self.current_recording = Some(Recording {
            name,
            keystrokes: Vec::new(),
        });
    }

    pub fn stop_recording(&mut self) {
        if let Some(recording) = self.current_recording.take() {
            if !recording.keystrokes.is_empty() {
                self.recordings.push(recording);
            }
        }
        self.recording = false;
        self.record_start = None;
    }

    pub fn record_keystroke(&mut self, code: KeyCode) {
        if self.recording {
            if let Some((start, recording)) = self.record_start.as_ref().zip(self.current_recording.as_mut()) {
                let elapsed = start.elapsed();
                recording.keystrokes.push((elapsed, code));
            }
        }
    }

    pub fn start_replay(&mut self, index: usize) {
        if index < self.recordings.len() {
            self.replaying = true;
            self.replay_index = index;
            // In real implementation, would start async playback
        }
    }
}

// In a real implementation:
// impl<B: Backend> LocustPlugin<B> for KeystrokeRecorderPlugin {
//     fn id(&self) -> &'static str {
//         "app.keystroke_recorder"
//     }
//
//     fn on_event(&mut self, event: &Event, ctx: &mut LocustContext) -> PluginEventResult {
//         if let Event::Key(KeyEvent { code, modifiers, .. }) = event {
//             // Ctrl+R: Start/stop recording
//             if *code == KeyCode::Char('r') && modifiers.contains(KeyModifiers::CONTROL) {
//                 if self.recording {
//                     self.stop_recording();
//                 } else {
//                     self.start_recording("Recording".to_string());
//                 }
//                 return PluginEventResult::ConsumedRequestRedraw;
//             }
//
//             // Ctrl+P: Replay last recording
//             if *code == KeyCode::Char('p') && modifiers.contains(KeyModifiers::CONTROL) {
//                 if !self.recordings.is_empty() && !self.replaying {
//                     self.start_replay(self.recordings.len() - 1);
//                 }
//                 return PluginEventResult::ConsumedRequestRedraw;
//             }
//
//             // Record keystroke if recording
//             if self.recording {
//                 self.record_keystroke(*code);
//             }
//         }
//
//         PluginEventResult::NotHandled
//     }
//
//     fn render_overlay(&self, frame: &mut Frame, ctx: &LocustContext) {
//         if self.recording {
//             let status = format!(
//                 "🔴 Recording... ({} keystrokes)",
//                 self.current_recording.as_ref().map_or(0, |r| r.keystrokes.len())
//             );
//
//             let area = Rect::new(0, 0, status.len() as u16 + 2, 1);
//             frame.render_widget(
//                 Paragraph::new(status).style(Style::default().bg(Color::Red).fg(Color::White)),
//                 area,
//             );
//         } else if self.replaying {
//             let status = "▶️  Replaying...".to_string();
//             let area = Rect::new(0, 0, status.len() as u16 + 2, 1);
//             frame.render_widget(
//                 Paragraph::new(status).style(Style::default().bg(Color::Green).fg(Color::White)),
//                 area,
//             );
//         }
//
//         // Show available recordings
//         if !self.recordings.is_empty() && !self.recording && !self.replaying {
//             let recordings_text: Vec<String> = self
//                 .recordings
//                 .iter()
//                 .enumerate()
//                 .map(|(i, r)| format!("{}: {} ({} keys)", i + 1, r.name, r.keystrokes.len()))
//                 .collect();
//
//             let height = recordings_text.len().min(5) as u16 + 2;
//             let width = 40;
//
//             let area = Rect::new(0, frame.area().height.saturating_sub(height), width, height);
//
//             let block = Block::default()
//                 .title("Recordings (Ctrl+P to replay)")
//                 .borders(Borders::ALL)
//                 .border_style(Style::default().fg(Color::Yellow));
//
//             let paragraph = Paragraph::new(recordings_text.join("\n"))
//                 .block(block)
//                 .style(Style::default().fg(Color::Yellow));
//
//             frame.render_widget(paragraph, area);
//         }
//     }
// }

// ============================================================================
// Custom Plugin #3: Performance Monitor
// ============================================================================
//
// Tracks frame times and displays FPS

struct PerformanceMonitorPlugin {
    frame_times: VecDeque<std::time::Duration>,
    last_frame: Option<Instant>,
    show_details: bool,
    max_samples: usize,
}

impl PerformanceMonitorPlugin {
    pub fn new(max_samples: usize) -> Self {
        Self {
            frame_times: VecDeque::with_capacity(max_samples),
            last_frame: None,
            show_details: true,
            max_samples,
        }
    }

    pub fn toggle_details(&mut self) {
        self.show_details = !self.show_details;
    }

    pub fn record_frame(&mut self) {
        if let Some(last) = self.last_frame {
            let frame_time = last.elapsed();
            self.frame_times.push_back(frame_time);

            if self.frame_times.len() > self.max_samples {
                self.frame_times.pop_front();
            }
        }
        self.last_frame = Some(Instant::now());
    }

    pub fn average_frame_time(&self) -> std::time::Duration {
        if self.frame_times.is_empty() {
            return std::time::Duration::ZERO;
        }

        let total: std::time::Duration = self.frame_times.iter().sum();
        total / self.frame_times.len() as u32
    }

    pub fn fps(&self) -> f64 {
        let avg = self.average_frame_time();
        if avg.as_millis() == 0 {
            return 0.0;
        }
        1000.0 / avg.as_millis() as f64
    }

    pub fn max_frame_time(&self) -> std::time::Duration {
        self.frame_times.iter().max().copied().unwrap_or(std::time::Duration::ZERO)
    }

    pub fn min_frame_time(&self) -> std::time::Duration {
        self.frame_times.iter().min().copied().unwrap_or(std::time::Duration::ZERO)
    }
}

// In a real implementation:
// impl<B: Backend> LocustPlugin<B> for PerformanceMonitorPlugin {
//     fn id(&self) -> &'static str {
//         "app.performance_monitor"
//     }
//
//     fn priority(&self) -> i32 {
//         1000  // Low priority (runs last)
//     }
//
//     fn on_frame_begin(&mut self, ctx: &mut LocustContext) {
//         self.record_frame();
//     }
//
//     fn on_event(&mut self, event: &Event, ctx: &mut LocustContext) -> PluginEventResult {
//         if matches!(event, Event::Key(KeyEvent {
//             code: KeyCode::Char('m'),
//             modifiers: KeyModifiers::CONTROL,
//             ..
//         })) {
//             self.toggle_details();
//             return PluginEventResult::ConsumedRequestRedraw;
//         }
//
//         PluginEventResult::NotHandled
//     }
//
//     fn render_overlay(&self, frame: &mut Frame, ctx: &LocustContext) {
//         let fps = self.fps();
//         let fps_color = if fps >= 60.0 {
//             Color::Green
//         } else if fps >= 30.0 {
//             Color::Yellow
//         } else {
//             Color::Red
//         };
//
//         if self.show_details {
//             let stats = vec![
//                 format!("FPS: {:.1}", fps),
//                 format!("Avg: {:.2}ms", self.average_frame_time().as_secs_f64() * 1000.0),
//                 format!("Min: {:.2}ms", self.min_frame_time().as_secs_f64() * 1000.0),
//                 format!("Max: {:.2}ms", self.max_frame_time().as_secs_f64() * 1000.0),
//                 format!("Samples: {}", self.frame_times.len()),
//             ];
//
//             let width = 20;
//             let height = stats.len() as u16 + 2;
//
//             let area = Rect::new(
//                 frame.area().width.saturating_sub(width),
//                 frame.area().height.saturating_sub(height),
//                 width,
//                 height,
//             );
//
//             let block = Block::default()
//                 .title("Perf (Ctrl+M)")
//                 .borders(Borders::ALL)
//                 .border_style(Style::default().fg(fps_color));
//
//             let paragraph = Paragraph::new(stats.join("\n"))
//                 .block(block)
//                 .style(Style::default().fg(Color::White));
//
//             frame.render_widget(paragraph, area);
//         } else {
//             // Compact mode: just FPS
//             let text = format!("FPS: {:.1}", fps);
//             let area = Rect::new(
//                 frame.area().width.saturating_sub(12),
//                 0,
//                 12,
//                 1,
//             );
//
//             frame.render_widget(
//                 Paragraph::new(text).style(Style::default().bg(Color::Black).fg(fps_color)),
//                 area,
//             );
//         }
//     }
// }

// ============================================================================
// Custom Plugin #4: App State Bridge
// ============================================================================
//
// Bridges between app state and Locust context

use std::sync::{Arc, RwLock};

#[derive(Clone, Debug)]
struct AppState {
    current_file: String,
    unsaved_changes: bool,
    selection_count: usize,
}

struct StateBridgePlugin {
    state: Arc<RwLock<AppState>>,
}

impl StateBridgePlugin {
    pub fn new(state: Arc<RwLock<AppState>>) -> Self {
        Self { state }
    }
}

// In a real implementation:
// impl<B: Backend> LocustPlugin<B> for StateBridgePlugin {
//     fn id(&self) -> &'static str {
//         "app.state_bridge"
//     }
//
//     fn on_frame_begin(&mut self, ctx: &mut LocustContext) {
//         // Sync app state to Locust context
//         if let Ok(state) = self.state.read() {
//             ctx.store_data("app_state", Box::new(state.clone()));
//         }
//     }
//
//     fn on_event(&mut self, event: &Event, ctx: &mut LocustContext) -> PluginEventResult {
//         // Check for state changes from other plugins
//         if let Some(new_state) = ctx.get_data::<AppState>("updated_app_state") {
//             if let Ok(mut state) = self.state.write() {
//                 *state = new_state.clone();
//             }
//             ctx.remove_data("updated_app_state");
//         }
//
//         PluginEventResult::NotHandled
//     }
//
//     fn render_overlay(&self, frame: &mut Frame, ctx: &LocustContext) {
//         if let Ok(state) = self.state.read() {
//             let status_line = format!(
//                 " {} {} | {} selected",
//                 state.current_file,
//                 if state.unsaved_changes { "*" } else { "" },
//                 state.selection_count
//             );
//
//             let area = Rect::new(0, frame.area().height - 1, frame.area().width, 1);
//
//             frame.render_widget(
//                 Paragraph::new(status_line)
//                     .style(Style::default().bg(Color::DarkGray).fg(Color::White)),
//                 area,
//             );
//         }
//     }
// }

// ============================================================================
// Main Application
// ============================================================================

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Custom Plugin Example");
    println!("\nThis example demonstrates custom plugin development patterns.");
    println!("\nKey Plugins:");
    println!("  1. EventLoggerPlugin - Logs all events (Ctrl+L to toggle)");
    println!("  2. KeystrokeRecorderPlugin - Record/replay (Ctrl+R to record, Ctrl+P to replay)");
    println!("  3. PerformanceMonitorPlugin - FPS tracking (Ctrl+M to toggle)");
    println!("  4. StateBridgePlugin - App state synchronization");
    println!("\nTo actually run these plugins, integrate them into a Locust instance.");
    println!("\nPress Enter to see example plugin initialization code...");

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    println!("\n=== Example Plugin Registration ===\n");
    println!("let mut locust = Locust::new(LocustConfig::default());");
    println!();
    println!("// Register built-in plugins");
    println!("locust.register_plugin(NavPlugin::new());");
    println!("locust.register_plugin(OmnibarPlugin::new());");
    println!();
    println!("// Register custom plugins");
    println!("locust.register_plugin(EventLoggerPlugin::new(100));");
    println!("locust.register_plugin(KeystrokeRecorderPlugin::new());");
    println!("locust.register_plugin(PerformanceMonitorPlugin::new(60));");
    println!();
    println!("// Register state bridge");
    println!("let app_state = Arc::new(RwLock::new(AppState {{ .. }}));");
    println!("locust.register_plugin(StateBridgePlugin::new(app_state.clone()));");
    println!();
    println!("=== Plugin Priority Guidelines ===\n");
    println!("  -1000 to -100: Critical monitoring plugins (event logger)");
    println!("     -99 to   0: Built-in Locust plugins");
    println!("       1 to  99: App-specific plugins");
    println!("     100 to 999: Integration plugins");
    println!("    1000+      : Performance monitors (run last)");

    Ok(())
}

// ============================================================================
// See Also
// ============================================================================
//
// - API_PATTERNS.md for more plugin patterns
// - PLUGIN_DEVELOPMENT_GUIDE.md for detailed plugin development
// - examples/dashboard.rs for real-world plugin usage
