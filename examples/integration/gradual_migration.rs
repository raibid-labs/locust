//! Gradual Migration Example
//!
//! This example shows how to incrementally add Locust to an existing app,
//! one feature at a time. Each phase can be tested independently before
//! moving to the next.
//!
//! Run with: cargo run --example gradual_migration --features <phase>
//! where <phase> is: phase1, phase2, phase3, phase4

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame, Terminal,
};
use std::{error::Error, io};

// ========================================================================
// ORIGINAL APP (Before Locust)
// ========================================================================

struct TodoApp {
    items: Vec<String>,
    selected: usize,
    input: String,
    input_mode: bool,
}

impl TodoApp {
    fn new() -> Self {
        Self {
            items: vec![
                "Buy groceries".to_string(),
                "Write documentation".to_string(),
                "Review pull requests".to_string(),
                "Fix bug in authentication".to_string(),
                "Deploy to production".to_string(),
            ],
            selected: 0,
            input: String::new(),
            input_mode: false,
        }
    }

    #[cfg(not(feature = "phase1"))]
    fn draw(&self, frame: &mut Frame) {
        self.draw_impl(frame);
    }

    #[cfg(feature = "phase1")]
    fn draw(&self, frame: &mut Frame, ctx: &mut LocustContext) {
        self.draw_impl(frame);
        self.register_targets_phase1(frame, ctx);
    }

    fn draw_impl(&self, frame: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // Header
                Constraint::Min(1),     // List
                Constraint::Length(3),  // Input
            ])
            .split(frame.area());

        // Header
        let header = Paragraph::new(self.get_header_text())
            .style(Style::default().fg(Color::Cyan))
            .block(Block::default().borders(Borders::ALL));
        frame.render_widget(header, chunks[0]);

        // List
        let items: Vec<ListItem> = self
            .items
            .iter()
            .enumerate()
            .map(|(i, item)| {
                let style = if i == self.selected {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };
                ListItem::new(Line::from(format!("{}. {}", i + 1, item))).style(style)
            })
            .collect();

        let list = List::new(items).block(Block::default().borders(Borders::ALL).title("Tasks"));
        frame.render_widget(list, chunks[1]);

        // Input
        let input_title = if self.input_mode {
            "New Task (Press Esc to cancel)"
        } else {
            "Press 'a' to add, 'd' to delete, 'q' to quit"
        };

        let input = Paragraph::new(self.input.as_str())
            .style(if self.input_mode {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default()
            })
            .block(Block::default().borders(Borders::ALL).title(input_title));
        frame.render_widget(input, chunks[2]);
    }

    fn get_header_text(&self) -> String {
        #[cfg(not(any(feature = "phase1", feature = "phase2", feature = "phase3", feature = "phase4")))]
        return "Todo List - Original (No Locust)".to_string();

        #[cfg(feature = "phase1")]
        return "Todo List - Phase 1: Navigation".to_string();

        #[cfg(feature = "phase2")]
        return "Todo List - Phase 2: Command Palette".to_string();

        #[cfg(feature = "phase3")]
        return "Todo List - Phase 3: Tooltips".to_string();

        #[cfg(feature = "phase4")]
        return "Todo List - Phase 4: Tour".to_string();
    }

    fn handle_key(&mut self, code: KeyCode) -> bool {
        if self.input_mode {
            match code {
                KeyCode::Esc => {
                    self.input_mode = false;
                    self.input.clear();
                }
                KeyCode::Enter => {
                    if !self.input.is_empty() {
                        self.items.push(self.input.clone());
                        self.input.clear();
                        self.input_mode = false;
                    }
                }
                KeyCode::Backspace => {
                    self.input.pop();
                }
                KeyCode::Char(c) => {
                    self.input.push(c);
                }
                _ => {}
            }
        } else {
            match code {
                KeyCode::Char('q') => return false,
                KeyCode::Char('a') => self.input_mode = true,
                KeyCode::Char('d') => {
                    if !self.items.is_empty() {
                        self.items.remove(self.selected);
                        if self.selected >= self.items.len() && self.selected > 0 {
                            self.selected -= 1;
                        }
                    }
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    if self.selected > 0 {
                        self.selected -= 1;
                    }
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    if self.selected < self.items.len().saturating_sub(1) {
                        self.selected += 1;
                    }
                }
                _ => {}
            }
        }
        true
    }
}

// ========================================================================
// PHASE 1: Add Navigation
// ========================================================================

#[cfg(feature = "phase1")]
mod phase1 {
    use super::*;
    use locust::prelude::*;

    impl TodoApp {
        pub fn register_targets_phase1(&self, frame: &Frame, ctx: &mut LocustContext) {
            // Calculate list area (same as in draw_impl)
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Min(1),
                    Constraint::Length(3),
                ])
                .split(frame.area());

            let list_area = chunks[1];

            // Register each todo item as a navigation target
            for (i, item) in self.items.iter().enumerate() {
                let item_area = Rect {
                    x: list_area.x + 1,
                    y: list_area.y + 1 + i as u16,
                    width: list_area.width - 2,
                    height: 1,
                };

                ctx.targets.register(NavTarget {
                    id: format!("todo_{}", i),
                    area: item_area,
                    kind: TargetKind::ListItem,
                    priority: 0,
                    metadata: std::collections::HashMap::new(),
                    actions: vec![TargetAction::Select],
                });
            }
        }

        pub fn handle_target_activation(&mut self, target_id: &str) {
            if let Some(index_str) = target_id.strip_prefix("todo_") {
                if let Ok(index) = index_str.parse::<usize>() {
                    if index < self.items.len() {
                        self.selected = index;
                    }
                }
            }
        }
    }
}

// ========================================================================
// PHASE 2: Add Command Palette
// ========================================================================

#[cfg(feature = "phase2")]
mod phase2 {
    use super::*;
    use locust::plugins::omnibar::{Command, OmnibarPlugin};

    pub fn create_omnibar() -> OmnibarPlugin {
        let mut omnibar = OmnibarPlugin::new();

        omnibar.register_command(Command {
            id: "todo.add".to_string(),
            name: "Add New Task".to_string(),
            description: Some("Add a new todo item".to_string()),
            aliases: vec!["add".to_string(), "new".to_string()],
            category: Some("Tasks".to_string()),
        });

        omnibar.register_command(Command {
            id: "todo.delete".to_string(),
            name: "Delete Selected Task".to_string(),
            description: Some("Delete the currently selected task".to_string()),
            aliases: vec!["delete".to_string(), "del".to_string(), "rm".to_string()],
            category: Some("Tasks".to_string()),
        });

        omnibar.register_command(Command {
            id: "todo.clear_all".to_string(),
            name: "Clear All Tasks".to_string(),
            description: Some("Remove all tasks".to_string()),
            aliases: vec!["clear".to_string(), "reset".to_string()],
            category: Some("Tasks".to_string()),
        });

        omnibar
    }

    impl TodoApp {
        pub fn handle_command(&mut self, cmd_id: &str) {
            match cmd_id {
                "todo.add" => {
                    self.input_mode = true;
                }
                "todo.delete" => {
                    if !self.items.is_empty() {
                        self.items.remove(self.selected);
                        if self.selected >= self.items.len() && self.selected > 0 {
                            self.selected -= 1;
                        }
                    }
                }
                "todo.clear_all" => {
                    self.items.clear();
                    self.selected = 0;
                }
                _ => {}
            }
        }
    }
}

// ========================================================================
// PHASE 3: Add Tooltips
// ========================================================================

#[cfg(feature = "phase3")]
mod phase3 {
    use super::*;
    use locust::plugins::tooltip::TooltipPosition;

    impl TodoApp {
        pub fn register_tooltips(&self, ctx: &mut LocustContext) {
            for (i, item) in self.items.iter().enumerate() {
                let tooltip_content = format!(
                    "📝 Task #{}\n\
                     \n\
                     {}\n\
                     \n\
                     💡 Press Enter to select\n\
                     💡 Press 'd' to delete",
                    i + 1,
                    item
                );

                ctx.tooltips.register(
                    format!("todo_{}", i),
                    tooltip_content,
                    TooltipPosition::Right,
                );
            }
        }
    }
}

// ========================================================================
// PHASE 4: Add Onboarding Tour
// ========================================================================

#[cfg(feature = "phase4")]
mod phase4 {
    use super::*;
    use locust::plugins::highlight::{TourPlugin, TourStep};
    use locust::plugins::tooltip::TooltipPosition;

    pub fn create_tour() -> TourPlugin {
        let mut tour = TourPlugin::new();

        tour.add_step(TourStep {
            id: "welcome".to_string(),
            title: "Welcome to Todo List!".to_string(),
            description: "Let's take a quick tour of the new features.".to_string(),
            target_id: None,
            position: TooltipPosition::Center,
            highlight: false,
        });

        tour.add_step(TourStep {
            id: "navigation".to_string(),
            title: "Quick Navigation".to_string(),
            description: "Press 'f' to show navigation hints. Type the hint to jump to any task!".to_string(),
            target_id: Some("todo_0".to_string()),
            position: TooltipPosition::Right,
            highlight: true,
        });

        tour.add_step(TourStep {
            id: "commands".to_string(),
            title: "Command Palette".to_string(),
            description: "Press Ctrl+P to open the command palette.\n\nSearch for operations like 'add', 'delete', or 'clear'.".to_string(),
            target_id: None,
            position: TooltipPosition::Center,
            highlight: false,
        });

        tour.add_step(TourStep {
            id: "tooltips".to_string(),
            title: "Helpful Tooltips".to_string(),
            description: "Hover over tasks (or navigate to them) to see helpful information.".to_string(),
            target_id: Some("todo_1".to_string()),
            position: TooltipPosition::Right,
            highlight: true,
        });

        tour
    }
}

// ========================================================================
// Main Function (Conditional Compilation)
// ========================================================================

fn main() -> Result<(), Box<dyn Error>> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = TodoApp::new();

    #[cfg(not(any(feature = "phase1", feature = "phase2", feature = "phase3", feature = "phase4")))]
    let result = run_app_original(&mut terminal, &mut app);

    #[cfg(any(feature = "phase1", feature = "phase2", feature = "phase3", feature = "phase4"))]
    let result = run_app_locust(&mut terminal, &mut app);

    // Restore terminal
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

// Original app without Locust
#[cfg(not(any(feature = "phase1", feature = "phase2", feature = "phase3", feature = "phase4")))]
fn run_app_original(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut TodoApp,
) -> io::Result<()> {
    loop {
        terminal.draw(|frame| {
            app.draw(frame);
        })?;

        if let Event::Key(KeyEvent { code, .. }) = event::read()? {
            if !app.handle_key(code) {
                return Ok(());
            }
        }
    }
}

// App with Locust (phases 1-4)
#[cfg(any(feature = "phase1", feature = "phase2", feature = "phase3", feature = "phase4"))]
fn run_app_locust(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut TodoApp,
) -> io::Result<()> {
    use locust::prelude::*;

    let mut locust = Locust::new(LocustConfig::default());

    // Phase 1: Add navigation
    #[cfg(feature = "phase1")]
    locust.register_plugin(NavPlugin::new());

    // Phase 2: Add command palette
    #[cfg(feature = "phase2")]
    locust.register_plugin(phase2::create_omnibar());

    // Phase 3: Add tooltips
    #[cfg(feature = "phase3")]
    locust.register_plugin(TooltipPlugin::new());

    // Phase 4: Add tour
    #[cfg(feature = "phase4")]
    {
        locust.register_plugin(phase4::create_tour());
        // Start tour automatically
        // In real app, check if first run
    }

    loop {
        locust.begin_frame();

        terminal.draw(|frame| {
            #[cfg(feature = "phase1")]
            app.draw(frame, &mut locust.ctx);

            #[cfg(not(feature = "phase1"))]
            app.draw(frame);

            #[cfg(feature = "phase3")]
            app.register_tooltips(&mut locust.ctx);

            locust.render_overlay(frame);
        })?;

        if let Event::Key(KeyEvent { code, modifiers, .. }) = event::read()? {
            let outcome = locust.on_event(&Event::Key(KeyEvent::new(code, modifiers)));

            // Handle Locust actions
            #[cfg(feature = "phase1")]
            if let Some(target_id) = locust.ctx.get_data::<String>("nav_activated") {
                app.handle_target_activation(&target_id);
                locust.ctx.remove_data("nav_activated");
            }

            #[cfg(feature = "phase2")]
            if let Some(cmd_id) = locust.ctx.get_data::<String>("omnibar_executed") {
                app.handle_command(&cmd_id);
                locust.ctx.remove_data("omnibar_executed");
            }

            if !outcome.consumed {
                if !app.handle_key(code) {
                    return Ok(());
                }
            }
        }
    }
}

// ========================================================================
// BUILD INSTRUCTIONS
// ========================================================================
//
// Run the original app (no Locust):
//   cargo run --example gradual_migration
//
// Run Phase 1 (Navigation only):
//   cargo run --example gradual_migration --features phase1
//
// Run Phase 2 (Navigation + Command Palette):
//   cargo run --example gradual_migration --features phase2
//
// Run Phase 3 (Navigation + Commands + Tooltips):
//   cargo run --example gradual_migration --features phase3
//
// Run Phase 4 (All features + Tour):
//   cargo run --example gradual_migration --features phase4
//
// ========================================================================
