//! Tooltip Plugin Demo
//!
//! This example demonstrates the tooltip plugin with various tooltip styles,
//! positioning, and content types.
//!
//! # Controls
//!
//! - Press 'h' to show tooltip for focused element (default activation key)
//! - Press 'f' to activate navigation hints
//! - Press ESC to hide tooltip or exit hint mode
//! - Press 'q' to quit
//!
//! # Features Demonstrated
//!
//! - Info, Warning, Error, and Success tooltip styles
//! - Multi-line tooltip content
//! - Tooltips with and without titles
//! - Smart positioning around screen edges
//! - Integration with navigation targets

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use locust::core::context::LocustContext;
use locust::core::plugin::LocustPlugin;
use locust::core::targets::{NavTarget, TargetBuilder, TargetPriority};
use locust::plugins::nav::NavPlugin;
use locust::plugins::tooltip::{TooltipConfig, TooltipContent, TooltipPlugin, TooltipStyle};
use ratatui::backend::{Backend, CrosstermBackend};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph, Wrap};
use ratatui::{Frame, Terminal};
use std::io;

struct App {
    context: LocustContext,
    nav_plugin: NavPlugin,
    tooltip_plugin: TooltipPlugin,
    should_quit: bool,
}

impl App {
    fn new() -> Self {
        let mut context = LocustContext::default();
        let mut nav_plugin = NavPlugin::new();
        let mut tooltip_plugin = TooltipPlugin::with_config(
            TooltipConfig::new()
                .with_activation_key('h')
                .with_hover_delay_ms(0) // Instant for demo
                .with_max_width(50)
                .with_border(true)
                .with_arrow(true),
        );

        // Initialize plugins
        nav_plugin.init(&mut context);
        tooltip_plugin.init(&mut context);

        Self {
            context,
            nav_plugin,
            tooltip_plugin,
            should_quit: false,
        }
    }

    fn register_demo_targets(&mut self) {
        let mut builder = TargetBuilder::new();

        // Header buttons with Info tooltips
        let btn1 = builder.button(Rect::new(2, 1, 18, 3), "Save Project");
        self.context.targets.register(btn1.clone());
        self.context.tooltips.register(
            btn1.id,
            TooltipContent::new(
                "Saves the current project to disk.\nAll changes will be persisted.",
            )
            .with_title("Save Project")
            .with_style(TooltipStyle::Info),
        );

        let btn2 = builder.button(Rect::new(22, 1, 18, 3), "Load Project");
        self.context.targets.register(btn2.clone());
        self.context.tooltips.register(
            btn2.id,
            TooltipContent::new("Opens a file dialog to load\nan existing project.")
                .with_title("Load Project")
                .with_style(TooltipStyle::Info),
        );

        let btn3 = builder.button(Rect::new(42, 1, 18, 3), "Settings");
        self.context.targets.register(btn3.clone());
        self.context.tooltips.register(
            btn3.id,
            TooltipContent::new("Configure application preferences,\nkeybindings, and appearance.")
                .with_title("Settings")
                .with_style(TooltipStyle::Info),
        );

        // List items with various tooltip styles
        let list_items = vec![
            (
                "Build Project",
                "Compiles the project with cargo build",
                TooltipStyle::Success,
            ),
            (
                "Run Tests",
                "Executes all unit and integration tests",
                TooltipStyle::Success,
            ),
            (
                "Debug Mode",
                "WARNING: Debug symbols increase binary size",
                TooltipStyle::Warning,
            ),
            (
                "Deploy",
                "CAUTION: This will deploy to production!\nMake sure all tests pass first.",
                TooltipStyle::Warning,
            ),
            (
                "Delete Project",
                "ERROR: This action cannot be undone!\nAll project files will be permanently deleted.",
                TooltipStyle::Error,
            ),
            (
                "Force Push",
                "DANGER: Force pushing rewrites history.\nOnly use if you know what you're doing!",
                TooltipStyle::Error,
            ),
        ];

        let mut y = 6;
        for (i, (label, tooltip_text, style)) in list_items.iter().enumerate() {
            let item = builder.list_item(Rect::new(2, y, 58, 2), *label);
            self.context.targets.register(item.clone());
            self.context.tooltips.register(
                item.id,
                TooltipContent::new(*tooltip_text)
                    .with_title(*label)
                    .with_style(*style),
            );
            y += 3;
        }

        // Footer with helpful info
        let help = builder.custom(
            Rect::new(2, 26, 58, 2),
            "Help",
            locust::core::targets::TargetAction::Activate,
            TargetPriority::Low,
        );
        self.context.targets.register(help.clone());
        self.context.tooltips.register(
            help.id,
            TooltipContent::new(
                "Press 'f' for navigation hints\nPress 'h' to show tooltips\nPress 'q' to quit",
            )
            .with_title("Keyboard Shortcuts")
            .with_style(TooltipStyle::Info),
        );
    }

    fn handle_event(&mut self, event: Event) -> io::Result<()> {
        // Let plugins handle event first
        let nav_result = self.nav_plugin.on_event(&event, &mut self.context);
        if nav_result.should_redraw() {
            return Ok(());
        }

        let tooltip_result = self.tooltip_plugin.on_event(&event, &mut self.context);
        if tooltip_result.should_redraw() {
            return Ok(());
        }

        // Handle app-level events
        if let Event::Key(key) = event {
            match key.code {
                KeyCode::Char('q') => self.should_quit = true,
                _ => {}
            }
        }

        Ok(())
    }

    fn draw<B: Backend>(&mut self, f: &mut Frame, terminal: &mut Terminal<B>) {
        // Clear targets from previous frame
        self.context.targets.clear();

        // Re-register targets each frame
        self.register_demo_targets();

        // Main UI
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(0)
            .constraints([Constraint::Length(5), Constraint::Min(0)])
            .split(f.area());

        // Header
        self.draw_header(f, chunks[0]);

        // Main content
        self.draw_content(f, chunks[1]);

        // Plugin overlays
        self.nav_plugin.render_overlay(f, &self.context);
        self.tooltip_plugin.render_overlay(f, &self.context);
    }

    fn draw_header(&self, f: &mut Frame, area: Rect) {
        let block = Block::default()
            .title("Tooltip Plugin Demo")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));

        f.render_widget(block, area);

        // Buttons are registered as targets
        let button_areas = [
            Rect::new(2, 1, 18, 3),
            Rect::new(22, 1, 18, 3),
            Rect::new(42, 1, 18, 3),
        ];

        let button_labels = ["Save Project", "Load Project", "Settings"];

        for (area, label) in button_areas.iter().zip(button_labels.iter()) {
            let button = Paragraph::new(*label)
                .style(
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::LightBlue)
                        .add_modifier(Modifier::BOLD),
                )
                .block(Block::default().borders(Borders::ALL));

            f.render_widget(button, *area);
        }
    }

    fn draw_content(&self, f: &mut Frame, area: Rect) {
        let block = Block::default()
            .title("Project Actions (hover over items)")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::White));

        f.render_widget(block, area);

        // List items
        let items = vec![
            "  Build Project",
            "  Run Tests",
            "  Debug Mode",
            "  Deploy",
            "  Delete Project",
            "  Force Push",
        ];

        let list_items: Vec<ListItem> = items
            .iter()
            .map(|&text| ListItem::new(text).style(Style::default().fg(Color::White)))
            .collect();

        let list = List::new(list_items).block(
            Block::default()
                .borders(Borders::NONE)
                .style(Style::default()),
        );

        let list_area = Rect {
            x: area.x + 1,
            y: area.y + 1,
            width: area.width.saturating_sub(2),
            height: area.height.saturating_sub(2),
        };

        f.render_widget(list, list_area);

        // Help footer
        let help_area = Rect::new(2, 26, 58, 2);
        let help =
            Paragraph::new("Press 'h' for help • Press 'f' for navigation • Press 'q' to quit")
                .style(Style::default().fg(Color::DarkGray))
                .wrap(Wrap { trim: false });

        f.render_widget(help, help_area);
    }
}

fn main() -> io::Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app
    let mut app = App::new();

    // Main loop
    loop {
        terminal.draw(|f| app.draw(f, &mut terminal))?;

        if event::poll(std::time::Duration::from_millis(100))? {
            app.handle_event(event::read()?)?;
        }

        if app.should_quit {
            break;
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}
