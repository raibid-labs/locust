//! Omnibar plugin demonstration.
//!
//! This example shows the Omnibar plugin in action:
//! - Press '/' to activate the omnibar
//! - Type commands (they'll be logged to stderr)
//! - Press Enter to submit, Esc to cancel
//! - Use arrow keys to navigate command history
//! - Press 'q' to quit
//!
//! Run with: cargo run --example omnibar_demo

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use locust::core::context::LocustContext;
use locust::core::plugin::LocustPlugin;
use locust::plugins::omnibar::{BorderType, OmnibarConfig, OmnibarPlugin};
use ratatui::backend::{Backend, CrosstermBackend};
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::{Frame, Terminal};
use std::io;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app state
    let mut app = App::new();

    // Run the app
    let res = run_app(&mut terminal, &mut app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("Error: {:?}", err);
    }

    Ok(())
}

struct App {
    /// Locust context
    ctx: LocustContext,
    /// Omnibar plugin
    omnibar: OmnibarPlugin,
    /// Should quit
    quit: bool,
}

impl App {
    fn new() -> Self {
        let mut ctx = LocustContext::default();

        // Create omnibar with custom styling
        let config = OmnibarConfig::new()
            .with_activation_key('/')
            .with_max_width(70)
            .with_max_height(3)
            .with_placeholder("Type a command... (try: help, quit, hello)")
            .with_border_type(BorderType::Rounded)
            .with_border_style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
            .with_title_style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            );

        let mut omnibar = OmnibarPlugin::with_config(config);
        LocustPlugin::<CrosstermBackend<io::Stdout>>::init(&mut omnibar, &mut ctx);

        Self {
            ctx,
            omnibar,
            quit: false,
        }
    }

    fn handle_event(&mut self, event: Event) {
        // Check for quit key when omnibar is not active
        if let Event::Key(key) = &event {
            if key.code == KeyCode::Char('q') && !self.omnibar.state().is_active() {
                self.quit = true;
                return;
            }
        }

        // Let omnibar handle the event
        let result = <OmnibarPlugin as LocustPlugin<CrosstermBackend<io::Stdout>>>::on_event(
            &mut self.omnibar,
            &event,
            &mut self.ctx,
        );

        // Check if we need to redraw
        match result {
            locust::core::input::PluginEventResult::ConsumedRequestRedraw => {
                // Event was handled, redraw will happen automatically
            }
            _ => {
                // Event not handled by omnibar
            }
        }
    }
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui::<B>(f, app))?;

        if app.quit {
            break;
        }

        if event::poll(std::time::Duration::from_millis(100))? {
            app.handle_event(event::read()?);
        }
    }

    Ok(())
}

fn ui<B: Backend>(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(3),
        ])
        .split(f.area());

    // Title
    let title = Paragraph::new("Locust Omnibar Demo")
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        );
    f.render_widget(title, chunks[0]);

    // Main content area
    let mut lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            "Welcome to the Omnibar Demo!",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::raw("Press '/' to activate the omnibar")),
        Line::from(Span::raw("Press 'q' to quit")),
        Line::from(""),
        Line::from(Span::styled(
            "Omnibar Features:",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from("  • Type any command and press Enter"),
        Line::from("  • Press Esc to cancel"),
        Line::from("  • Use ← → Home End to move cursor"),
        Line::from("  • Use ↑ ↓ to navigate command history"),
        Line::from("  • Commands are logged to stderr"),
        Line::from(""),
    ];

    // Show command history
    if !app.omnibar.state().history().is_empty() {
        lines.push(Line::from(Span::styled(
            "Command History:",
            Style::default().add_modifier(Modifier::BOLD),
        )));

        for (i, cmd) in app.omnibar.state().history().iter().enumerate() {
            lines.push(Line::from(Span::styled(
                format!("  {}. {}", i + 1, cmd),
                Style::default().fg(Color::Green),
            )));
        }
    }

    let content = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::White))
                .title(" Instructions "),
        )
        .alignment(Alignment::Left);
    f.render_widget(content, chunks[1]);

    // Status bar
    let status_text = if app.omnibar.state().is_active() {
        "Omnibar: ACTIVE"
    } else {
        "Omnibar: inactive"
    };

    let status_style = if app.omnibar.state().is_active() {
        Style::default()
            .fg(Color::Black)
            .bg(Color::Green)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let status = Paragraph::new(status_text)
        .style(status_style)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(status, chunks[2]);

    // Render omnibar overlay (if active)
    <OmnibarPlugin as LocustPlugin<B>>::render_overlay(&app.omnibar, f, &app.ctx);
}
