//! Comprehensive demo of the highlight plugin with multi-step tours.
//!
//! This example demonstrates:
//! - Creating a multi-step guided tour
//! - Different message positions
//! - Highlighting specific areas
//! - Tour navigation
//! - Multiple tours
//!
//! Run with: cargo run --example tour_demo

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use locust::{
    core::{
        context::{Locust, LocustConfig},
        plugin::LocustPlugin,
        targets::NavTarget,
    },
    plugins::highlight::{HighlightPlugin, MessagePosition, Tour, TourStep},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame, Terminal,
};
use std::io;

/// Application state
struct App {
    should_quit: bool,
    current_screen: usize,
    screens: Vec<&'static str>,
}

impl App {
    fn new() -> Self {
        Self {
            should_quit: false,
            current_screen: 0,
            screens: vec!["Welcome", "Features", "Settings", "Help"],
        }
    }

    fn next_screen(&mut self) {
        self.current_screen = (self.current_screen + 1) % self.screens.len();
    }

    fn previous_screen(&mut self) {
        if self.current_screen == 0 {
            self.current_screen = self.screens.len() - 1;
        } else {
            self.current_screen -= 1;
        }
    }
}

fn main() -> Result<(), io::Error> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app state
    let mut app = App::new();

    // Create Locust with highlight plugin
    let config = LocustConfig::default();
    let mut locust = Locust::new(config);

    let mut highlight_plugin = HighlightPlugin::new();

    // Create onboarding tour
    let onboarding_tour = Tour::new("onboarding")
        .add_step(
            TourStep::new(
                "Welcome to the Tour Demo",
                "This is a guided tour of the application.\nPress → or Enter to continue, ← to go back, Esc to skip."
            )
            .with_position(MessagePosition::Center)
        )
        .add_step(
            TourStep::new(
                "Navigation Tabs",
                "Use Tab and Shift+Tab to navigate between screens.\nEach screen shows different content."
            )
            .with_area(Rect::new(0, 0, 80, 3))
            .with_position(MessagePosition::Bottom)
        )
        .add_step(
            TourStep::new(
                "Main Content Area",
                "This is where the main content is displayed.\nDifferent screens show different information."
            )
            .with_area(Rect::new(2, 4, 76, 15))
            .with_position(MessagePosition::Top)
        )
        .add_step(
            TourStep::new(
                "Status Bar",
                "The status bar shows helpful information\nand keyboard shortcuts."
            )
            .with_area(Rect::new(0, 21, 80, 3))
            .with_position(MessagePosition::Top)
        )
        .add_step(
            TourStep::new(
                "Tour Complete!",
                "You've completed the tour!\nPress ? to restart the tour anytime.\nPress q to quit the application."
            )
            .with_position(MessagePosition::Center)
        )
        .with_description("Application onboarding tour")
        .with_skippable(true);

    highlight_plugin.register_tour(onboarding_tour);

    // Create feature tour
    let feature_tour = Tour::new("features")
        .add_step(
            TourStep::new(
                "Feature Highlights",
                "This tour highlights key features.\nLet's explore what this app can do!",
            )
            .with_position(MessagePosition::Center),
        )
        .add_step(
            TourStep::new(
                "Fast Navigation",
                "Quickly navigate between sections\nusing keyboard shortcuts.",
            )
            .with_area(Rect::new(10, 8, 60, 3))
            .with_position(MessagePosition::Right),
        )
        .add_step(
            TourStep::new(
                "Interactive Tours",
                "Learn with interactive guided tours\nthat highlight important areas.",
            )
            .with_area(Rect::new(10, 12, 60, 3))
            .with_position(MessagePosition::Left),
        )
        .with_description("Feature tour")
        .with_skippable(true);

    highlight_plugin.register_tour(feature_tour);

    locust.register_plugin(highlight_plugin);

    // Start the onboarding tour
    if let Some(plugin) = locust.plugins.iter_mut().next() {
        // This is a bit hacky for the demo - in real code you'd get the plugin properly
        // For now, the tour will start when user presses '?'
    }

    // Main loop
    let result = run_app(&mut terminal, &mut app, &mut locust);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    result
}

fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
    locust: &mut Locust<B>,
) -> io::Result<()> {
    loop {
        terminal.draw(|f| {
            locust.begin_frame();
            ui(f, app, locust);
            locust.render_overlay(f);
        })?;

        if let Event::Key(key) = event::read()? {
            let event = Event::Key(key);
            let outcome = locust.on_event(&event);

            if !outcome.consumed {
                // App-level key handling
                match key.code {
                    KeyCode::Char('q') => app.should_quit = true,
                    KeyCode::Tab => app.next_screen(),
                    KeyCode::BackTab => app.previous_screen(),
                    _ => {}
                }
            }
        }

        if app.should_quit {
            break;
        }
    }

    Ok(())
}

fn ui<B: ratatui::backend::Backend>(f: &mut Frame, app: &App, locust: &mut Locust<B>) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(f.area());

    // Header with tabs
    let tabs: Vec<Line> = app
        .screens
        .iter()
        .enumerate()
        .map(|(i, name)| {
            let style = if i == app.current_screen {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Gray)
            };
            Line::from(vec![
                Span::raw(" "),
                Span::styled(format!("[{}] {}", i + 1, name), style),
                Span::raw(" "),
            ])
        })
        .collect();

    let header_text: Vec<Line> = tabs;
    let header = Paragraph::new(header_text)
        .block(Block::default().borders(Borders::ALL).title("Navigation"))
        .alignment(Alignment::Left);
    f.render_widget(header, chunks[0]);

    // Register nav target for header
    locust
        .ctx
        .targets
        .register(NavTarget::new(1, chunks[0]).with_label("Navigation Tabs"));

    // Main content
    let content = match app.current_screen {
        0 => render_welcome(chunks[1], locust),
        1 => render_features(chunks[1], locust),
        2 => render_settings(chunks[1], locust),
        3 => render_help(chunks[1], locust),
        _ => unreachable!(),
    };
    f.render_widget(content, chunks[1]);

    // Status bar
    let status_text = vec![Line::from(vec![
        Span::styled("Tab", Style::default().fg(Color::Cyan)),
        Span::raw("/"),
        Span::styled("Shift+Tab", Style::default().fg(Color::Cyan)),
        Span::raw(": Navigate | "),
        Span::styled("?", Style::default().fg(Color::Cyan)),
        Span::raw(": Tour | "),
        Span::styled("q", Style::default().fg(Color::Cyan)),
        Span::raw(": Quit"),
    ])];

    let status = Paragraph::new(status_text)
        .block(Block::default().borders(Borders::ALL).title("Status"))
        .alignment(Alignment::Left);
    f.render_widget(status, chunks[2]);

    // Register nav target for status bar
    locust
        .ctx
        .targets
        .register(NavTarget::new(100, chunks[2]).with_label("Status Bar"));
}

fn render_welcome<B: ratatui::backend::Backend>(
    area: Rect,
    locust: &mut Locust<B>,
) -> Paragraph<'static> {
    let text = vec![
        Line::from(Span::styled(
            "Welcome to the Tour Demo!",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from("This example demonstrates the Locust highlight plugin"),
        Line::from("with multi-step guided tours."),
        Line::from(""),
        Line::from(vec![
            Span::raw("Press "),
            Span::styled(
                "?",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" to start the guided tour!"),
        ]),
        Line::from(""),
        Line::from("Features:"),
        Line::from("  • Multi-step guided tours"),
        Line::from("  • Visual highlights with dim overlay"),
        Line::from("  • Flexible message positioning"),
        Line::from("  • Tour progress tracking"),
        Line::from("  • Navigation controls"),
    ];

    locust.ctx.targets.register(
        NavTarget::new(10, Rect::new(area.x + 2, area.y + 2, area.width - 4, 12))
            .with_label("Welcome Content"),
    );

    Paragraph::new(text)
        .block(Block::default().borders(Borders::ALL).title("Welcome"))
        .alignment(Alignment::Left)
}

fn render_features<B: ratatui::backend::Backend>(
    area: Rect,
    locust: &mut Locust<B>,
) -> List<'static> {
    let items = vec![
        ListItem::new("Fast keyboard navigation"),
        ListItem::new("Interactive guided tours"),
        ListItem::new("Contextual help and tooltips"),
        ListItem::new("Customizable themes"),
        ListItem::new("Plugin system"),
    ];

    for (i, _) in items.iter().enumerate() {
        locust.ctx.targets.register(
            NavTarget::new(
                20 + i as u64,
                Rect::new(area.x + 2, area.y + 2 + i as u16, area.width - 4, 1),
            )
            .with_label(&format!("Feature {}", i + 1)),
        );
    }

    List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Features"))
        .style(Style::default().fg(Color::White))
}

fn render_settings<B: ratatui::backend::Backend>(
    area: Rect,
    _locust: &mut Locust<B>,
) -> Paragraph<'static> {
    let text = vec![
        Line::from("Settings"),
        Line::from(""),
        Line::from("• Theme: Default"),
        Line::from("• Animation: Enabled"),
        Line::from("• Tours: Enabled"),
        Line::from("• Dim Opacity: 70%"),
    ];

    Paragraph::new(text)
        .block(Block::default().borders(Borders::ALL).title("Settings"))
        .alignment(Alignment::Left)
}

fn render_help<B: ratatui::backend::Backend>(
    area: Rect,
    _locust: &mut Locust<B>,
) -> Paragraph<'static> {
    let text = vec![
        Line::from(Span::styled(
            "Keyboard Shortcuts",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from("Navigation:"),
        Line::from("  Tab / Shift+Tab  - Switch screens"),
        Line::from("  ? - Start tour"),
        Line::from(""),
        Line::from("During Tour:"),
        Line::from("  → / n / Enter - Next step"),
        Line::from("  ← / p - Previous step"),
        Line::from("  Esc - Skip tour"),
        Line::from(""),
        Line::from("General:"),
        Line::from("  q - Quit application"),
    ];

    Paragraph::new(text)
        .block(Block::default().borders(Borders::ALL).title("Help"))
        .alignment(Alignment::Left)
}
