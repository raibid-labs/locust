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
use locust::plugins::omnibar::{BorderType, Command, CommandResult, OmnibarConfig, OmnibarPlugin};
use ratatui::backend::{Backend, CrosstermBackend};
use locust::{Locust, LocustConfig};
use locust::prelude::{NavPlugin, TargetAction, TargetBuilder, TargetPriority};
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::{Frame, Terminal};
use std::io;
use std::sync::Arc;
use std::fs::File;
use std::path::PathBuf;

use log::{debug, error, info, LevelFilter};
use simplelog::{CombinedLogger, Config, WriteLogger};
use locust::ratatui_ext::LogTailer;

// Custom demo command
struct CustomDemoCommand {
    name: &'static str,
    desc: &'static str,
}

impl Command for CustomDemoCommand {
    fn name(&self) -> &str {
        self.name
    }

    fn description(&self) -> &str {
        self.desc
    }

    fn category(&self) -> Option<&str> {
        Some("demo")
    }

    fn execute(&self, _ctx: &mut LocustContext) -> CommandResult {
        eprintln!("Custom demo command '{}' executed!", self.name);
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logger
    let log_file_path = PathBuf::from("locust.log");
    CombinedLogger::init(
        vec![
            WriteLogger::new(
                LevelFilter::Debug,
                Config::default(),
                File::create(&log_file_path).unwrap(),
            ),
        ]
    ).unwrap();
    debug!("Logger initialized for Omnibar Demo.");

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app state
    let mut app = App::new();
    let mut log_tailer = LogTailer::new(log_file_path, 10); // Display last 10 log lines

    // Run the app
    let res = run_app(&mut terminal, &mut app, &mut log_tailer);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        error!("Error: {:?}", err);
    }

    Ok(())
}

struct App {
    /// Locust context
    ctx: LocustContext,
    /// Locust instance
    locust: Locust<CrosstermBackend<io::Stdout>>,
    /// Should quit
    quit: bool,
}

impl App {
    fn new() -> Self {
        let ctx = LocustContext::default();

        // Create omnibar with custom styling
        let config = OmnibarConfig::new()
            .with_activation_key('O')
            .with_max_width(70)
            .with_max_height(3)
            .with_placeholder("Type a command... (try: help, quit, hello, demo)")
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

        let mut omnibar_plugin = OmnibarPlugin::with_config(config);

        // Register built-in commands (quit, help, version, etc.)
        omnibar_plugin.register_builtin_commands();

        // Register custom demo commands
        omnibar_plugin.register_command(Arc::new(CustomDemoCommand {
            name: "demo",
            desc: "Run a custom demo command",
        }));
        omnibar_plugin.register_command(Arc::new(CustomDemoCommand {
            name: "test",
            desc: "Test command for demonstration",
        }));

        // Create Locust instance and register plugins
        let mut locust = Locust::new(LocustConfig::default());
        locust.register_plugin(omnibar_plugin);
        locust.register_plugin(NavPlugin::default()); // Register NavPlugin here

        Self {
            ctx,
            locust,
            quit: false,
        }
    }
    fn handle_event(&mut self, event: Event) {
        // Check for quit key when omnibar is not active
        if let Event::Key(key) = &event {
            if let Some(omnibar_plugin) = self.locust.get_plugin::<OmnibarPlugin>() {
                if key.code == KeyCode::Char('q') && !omnibar_plugin.state().is_active() {
                    self.quit = true;
                    return;
                }
            }
        }

        // Let Locust handle the event
        let result = self.locust.on_event(&event);
        log::debug!("Locust on_event result: {:?}", result);

        // Check if quit command was executed
        if let Some(omnibar_plugin) = self.locust.get_plugin::<OmnibarPlugin>() {
            if omnibar_plugin.should_quit() {
                self.quit = true;
            }
        }

        // Check if we need to redraw
        if result.request_redraw {
            // Event was handled, redraw will happen automatically
        }
    }
}

fn run_app<B: Backend + 'static>(terminal: &mut Terminal<B>, app: &mut App, log_tailer: &mut LogTailer) -> io::Result<()> {
    let mut target_builder = TargetBuilder::new();
    loop {
        log_tailer.read_tail()?; // Update log tail at the beginning of each frame
        terminal.draw(|f| ui::<B>(f, app, log_tailer, &mut target_builder))?;

        if app.quit {
            break;
        }

        if event::poll(std::time::Duration::from_millis(100))? {
            app.handle_event(event::read()?);
        }
    }

    Ok(())
}

fn ui<B: Backend + 'static>(f: &mut Frame, app: &mut App, log_tailer: &mut LogTailer, target_builder: &mut TargetBuilder) {
    let size = f.area();

    let main_layout_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Min(0),    // Content
            Constraint::Length(3), // Status bar
            Constraint::Length(12), // Log pane
        ])
        .split(size);

    let title_area = main_layout_chunks[0];
    let content_area = main_layout_chunks[1];
    let status_area = main_layout_chunks[2];
    let log_area = main_layout_chunks[3];

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
    f.render_widget(title, title_area);
    app.ctx.targets.register(
        target_builder.custom(title_area, "Omnibar Demo Title", TargetAction::Activate, TargetPriority::Low)
    );

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
        Line::from(Span::raw("Press 'Shift+O' to activate the omnibar")),
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
        Line::from("  • Fuzzy search with live suggestions"),
        Line::from(""),
        Line::from(Span::styled(
            "Available Commands:",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from("  quit (q, exit) - Exit the application"),
        Line::from("  help (h, ?)    - Show all commands"),
        Line::from("  version (v)    - Show version info"),
        Line::from("  hello (hi)     - Say hello"),
        Line::from("  demo           - Run demo command"),
        Line::from("  clear          - Clear command history"),
        Line::from(""),
    ];

    // Register NavTargets for "Available Commands"
    let available_commands_start_line = 15; // Approximate starting line index
    let commands = vec![
        "quit", "help", "version", "hello", "demo", "clear"
    ];
    for (i, cmd_name) in commands.iter().enumerate() {
        let line_idx = available_commands_start_line + i;
        let item_rect = Rect::new(
            content_area.x + 2, // Indent
            content_area.y + line_idx as u16,
            content_area.width.saturating_sub(4),
            1,
        );
        app.ctx.targets.register(
            target_builder.list_item(item_rect, format!("Command: {}", cmd_name))
        );
    }

    // Show command history
    if !app.locust.get_plugin::<OmnibarPlugin>().unwrap().state().history().is_empty() {
        lines.push(Line::from(Span::styled(
            "Command History:",
            Style::default().add_modifier(Modifier::BOLD),
        )));

        for (i, cmd) in app.locust.get_plugin::<OmnibarPlugin>().unwrap().state().history().iter().enumerate() {
            let line_idx = lines.len() - app.locust.get_plugin::<OmnibarPlugin>().unwrap().state().history().len() + i; // Calculate actual line index
            let item_rect = Rect::new(
                content_area.x + 2, // Indent
                content_area.y + line_idx as u16,
                content_area.width.saturating_sub(4),
                1,
            );
            app.ctx.targets.register(
                target_builder.list_item(item_rect, format!("History: {}", cmd))
            );
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
    f.render_widget(content, content_area);
    app.ctx.targets.register(
        target_builder.custom(content_area, "Instructions", TargetAction::Activate, TargetPriority::Low)
    );

    // Status bar
    let status_text = if app.locust.get_plugin::<OmnibarPlugin>().unwrap().state().is_active() {
        "Omnibar: ACTIVE"
    } else {
        "Omnibar: inactive"
    };

    let status_style = if app.locust.get_plugin::<OmnibarPlugin>().unwrap().state().is_active() {
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
    f.render_widget(status, status_area);
    app.ctx.targets.register(
        target_builder.custom(status_area, "Status Bar", TargetAction::Activate, TargetPriority::Low)
    );

    // Render Log Tailer
    f.render_widget(log_tailer, log_area);

    // Render omnibar overlay (if active)
    // The locust.render_overlay(f) call at the end of the ui function will handle this.
}
