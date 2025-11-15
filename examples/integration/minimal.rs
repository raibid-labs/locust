//! Minimal Locust Integration Example
//!
//! This example demonstrates the absolute minimum code needed to add
//! Locust navigation to an existing ratatui application.
//!
//! Total changes required: ~15 lines of code
//!
//! Run with: cargo run --example minimal

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem},
    Frame, Terminal,
};
use std::{error::Error, io};

// This would be your existing app
struct SimpleApp {
    items: Vec<String>,
    selected: usize,
}

impl SimpleApp {
    fn new() -> Self {
        Self {
            items: vec![
                "Item 1".to_string(),
                "Item 2".to_string(),
                "Item 3".to_string(),
                "Item 4".to_string(),
                "Item 5".to_string(),
            ],
            selected: 0,
        }
    }

    // Your existing draw function - NO CHANGES NEEDED
    fn draw(&self, frame: &mut Frame) {
        let chunks = Layout::vertical([Constraint::Min(0)]).split(frame.area());

        let items: Vec<ListItem> = self
            .items
            .iter()
            .enumerate()
            .map(|(i, item)| {
                let style = if i == self.selected {
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };
                ListItem::new(item.as_str()).style(style)
            })
            .collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .title("Press 'f' for hints, 'q' to quit")
                    .borders(Borders::ALL),
            )
            .highlight_style(Style::default().add_modifier(Modifier::BOLD));

        frame.render_widget(list, chunks[0]);
    }

    // Your existing event handler - NO CHANGES NEEDED
    fn handle_key(&mut self, key: KeyCode) -> bool {
        match key {
            KeyCode::Char('q') => return false,
            KeyCode::Up | KeyCode::Char('k') => {
                if self.selected > 0 {
                    self.selected -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.selected < self.items.len() - 1 {
                    self.selected += 1;
                }
            }
            _ => {}
        }
        true
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    // Setup terminal (existing code)
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Your existing app
    let mut app = SimpleApp::new();

    // === LOCUST INTEGRATION: ADD THESE 2 LINES ===
    // Note: In a real integration, you'd use actual Locust types
    // let mut locust = Locust::new(LocustConfig::default());
    // locust.register_plugin(NavPlugin::new());

    // Main loop (modified)
    let result = run_app(&mut terminal, &mut app);

    // Restore terminal (existing code)
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = result {
        println!("{:?}", err);
    }

    Ok(())
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut SimpleApp,
) -> io::Result<()> {
    loop {
        // === LOCUST INTEGRATION: ADD THIS LINE ===
        // locust.begin_frame();

        terminal.draw(|frame| {
            app.draw(frame);

            // === LOCUST INTEGRATION: ADD THIS LINE ===
            // locust.render_overlay(frame);
        })?;

        if let Event::Key(KeyEvent { code, .. }) = event::read()? {
            // === LOCUST INTEGRATION: MODIFY EVENT HANDLING ===
            // let outcome = locust.on_event(&Event::Key(KeyEvent::new(code, KeyModifiers::empty())));
            // if !outcome.consumed {
            //     if !app.handle_key(code) {
            //         return Ok(());
            //     }
            // }

            // Original event handling
            if !app.handle_key(code) {
                return Ok(());
            }
        }
    }
}

// === SUMMARY OF CHANGES ===
//
// To add Locust navigation to this app:
//
// 1. Add to Cargo.toml:
//    locust = "0.1"
//
// 2. Add imports:
//    use locust::prelude::*;
//
// 3. Initialize Locust (in main):
//    let mut locust = Locust::new(LocustConfig::default());
//    locust.register_plugin(NavPlugin::new());
//
// 4. Add to event loop (in run_app):
//    - locust.begin_frame() at start of loop
//    - locust.render_overlay(frame) in draw closure
//    - locust.on_event() before app event handling
//
// That's it! Your app now has Vimium-style navigation.
// Press 'f' to see hints, type the hint to select an item.
//
// Next steps:
// - Register actual navigation targets in draw() function
// - Handle target activations
// - Add more plugins (Omnibar, Tooltip, etc.)
