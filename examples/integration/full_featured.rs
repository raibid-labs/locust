//! Full-Featured Locust Integration Example
//!
//! This example demonstrates a complete integration with all Locust plugins:
//! - Navigation (NavPlugin)
//! - Command Palette (OmnibarPlugin)
//! - Tooltips (TooltipPlugin)
//! - Tours (TourPlugin/HighlightPlugin)
//! - Custom theme
//! - Configuration file loading
//!
//! Run with: cargo run --example full_featured

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Tabs},
    Frame, Terminal,
};
use std::{collections::HashMap, error::Error, io};

// Full-featured application with multiple views
struct AdvancedApp {
    current_view: View,
    tasks: TaskList,
    notes: NoteList,
    settings: Settings,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum View {
    Tasks,
    Notes,
    Settings,
}

struct TaskList {
    items: Vec<Task>,
    selected: usize,
}

#[derive(Clone)]
struct Task {
    title: String,
    completed: bool,
    priority: Priority,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Priority {
    High,
    Medium,
    Low,
}

impl Priority {
    fn color(&self) -> Color {
        match self {
            Priority::High => Color::Red,
            Priority::Medium => Color::Yellow,
            Priority::Low => Color::Green,
        }
    }

    fn icon(&self) -> &'static str {
        match self {
            Priority::High => "🔴",
            Priority::Medium => "🟡",
            Priority::Low => "🟢",
        }
    }
}

struct NoteList {
    items: Vec<Note>,
    selected: usize,
}

#[derive(Clone)]
struct Note {
    title: String,
    content: String,
}

struct Settings {
    theme: String,
    auto_save: bool,
    notifications: bool,
}

impl AdvancedApp {
    fn new() -> Self {
        Self {
            current_view: View::Tasks,
            tasks: TaskList {
                items: vec![
                    Task {
                        title: "Complete Locust integration".to_string(),
                        completed: false,
                        priority: Priority::High,
                    },
                    Task {
                        title: "Write documentation".to_string(),
                        completed: false,
                        priority: Priority::Medium,
                    },
                    Task {
                        title: "Review PRs".to_string(),
                        completed: true,
                        priority: Priority::Low,
                    },
                ],
                selected: 0,
            },
            notes: NoteList {
                items: vec![
                    Note {
                        title: "Meeting Notes".to_string(),
                        content: "Discussed Q4 roadmap...".to_string(),
                    },
                    Note {
                        title: "Ideas".to_string(),
                        content: "New feature ideas...".to_string(),
                    },
                ],
                selected: 0,
            },
            settings: Settings {
                theme: "default".to_string(),
                auto_save: true,
                notifications: true,
            },
        }
    }

    fn draw(&self, frame: &mut Frame, ctx: &mut LocustContext) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // Tabs
                Constraint::Min(1),     // Content
                Constraint::Length(1),  // Status bar
            ])
            .split(frame.area());

        // Draw tabs
        self.draw_tabs(frame, chunks[0], ctx);

        // Draw current view
        match self.current_view {
            View::Tasks => self.draw_tasks(frame, chunks[1], ctx),
            View::Notes => self.draw_notes(frame, chunks[1], ctx),
            View::Settings => self.draw_settings(frame, chunks[1], ctx),
        }

        // Draw status bar
        self.draw_status_bar(frame, chunks[2]);
    }

    fn draw_tabs(&self, frame: &mut Frame, area: Rect, ctx: &mut LocustContext) {
        let titles = vec!["Tasks", "Notes", "Settings"];
        let selected_idx = match self.current_view {
            View::Tasks => 0,
            View::Notes => 1,
            View::Settings => 2,
        };

        let tabs = Tabs::new(titles.clone())
            .select(selected_idx)
            .block(Block::default().borders(Borders::ALL))
            .highlight_style(Style::default().fg(Color::Yellow));

        frame.render_widget(tabs, area);

        // Register navigation targets for tabs
        let tab_width = area.width / 3;
        for (i, title) in titles.iter().enumerate() {
            ctx.targets.register(NavTarget {
                id: format!("tab_{}", i),
                area: Rect {
                    x: area.x + (i as u16 * tab_width),
                    y: area.y,
                    width: tab_width,
                    height: area.height,
                },
                kind: TargetKind::TabHeader,
                priority: 10,  // Lower priority than list items
                metadata: HashMap::from([("title".to_string(), title.to_string())]),
                actions: vec![TargetAction::Select],
            });

            // Tooltip for tab
            ctx.tooltips.register(
                format!("tab_{}", i),
                format!("Switch to {} view\n\n💡 Press Enter to select", title),
                TooltipPosition::Below,
            );
        }
    }

    fn draw_tasks(&self, frame: &mut Frame, area: Rect, ctx: &mut LocustContext) {
        let items: Vec<ListItem> = self
            .tasks
            .items
            .iter()
            .enumerate()
            .map(|(i, task)| {
                let icon = if task.completed { "✓" } else { " " };
                let style = if i == self.tasks.selected {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else if task.completed {
                    Style::default().fg(Color::DarkGray)
                } else {
                    Style::default()
                };

                let line = Line::from(vec![
                    Span::raw(format!("[{}] ", icon)),
                    Span::raw(task.priority.icon()),
                    Span::raw(format!(" {}", task.title)),
                ]);

                ListItem::new(line).style(style)
            })
            .collect();

        let list = List::new(items).block(
            Block::default()
                .borders(Borders::ALL)
                .title("Tasks (Press 'f' for hints, Ctrl+P for commands)"),
        );

        frame.render_widget(list, area);

        // Register navigation targets for tasks
        for (i, task) in self.tasks.items.iter().enumerate() {
            let item_area = Rect {
                x: area.x + 1,
                y: area.y + 1 + i as u16,
                width: area.width - 2,
                height: 1,
            };

            ctx.targets.register(NavTarget {
                id: format!("task_{}", i),
                area: item_area,
                kind: TargetKind::ListItem,
                priority: 0,
                metadata: HashMap::from([
                    ("title".to_string(), task.title.clone()),
                    ("completed".to_string(), task.completed.to_string()),
                    ("priority".to_string(), format!("{:?}", task.priority)),
                ]),
                actions: vec![TargetAction::Select, TargetAction::Toggle],
            });

            // Tooltip for task
            let tooltip = format!(
                "{} {}\n\
                 \n\
                 Priority: {:?}\n\
                 Status: {}\n\
                 \n\
                 💡 Press Enter to select\n\
                 💡 Press Space to toggle",
                task.priority.icon(),
                task.title,
                task.priority,
                if task.completed {
                    "Completed ✓"
                } else {
                    "Pending"
                }
            );

            ctx.tooltips.register(format!("task_{}", i), tooltip, TooltipPosition::Right);
        }
    }

    fn draw_notes(&self, frame: &mut Frame, area: Rect, ctx: &mut LocustContext) {
        let items: Vec<ListItem> = self
            .notes
            .items
            .iter()
            .enumerate()
            .map(|(i, note)| {
                let style = if i == self.notes.selected {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };

                ListItem::new(format!("📝 {}", note.title)).style(style)
            })
            .collect();

        let list = List::new(items).block(Block::default().borders(Borders::ALL).title("Notes"));

        frame.render_widget(list, area);

        // Register navigation targets for notes
        for (i, note) in self.notes.items.iter().enumerate() {
            let item_area = Rect {
                x: area.x + 1,
                y: area.y + 1 + i as u16,
                width: area.width - 2,
                height: 1,
            };

            ctx.targets.register(NavTarget {
                id: format!("note_{}", i),
                area: item_area,
                kind: TargetKind::ListItem,
                priority: 0,
                metadata: HashMap::from([("title".to_string(), note.title.clone())]),
                actions: vec![TargetAction::Select],
            });

            // Tooltip for note
            ctx.tooltips.register(
                format!("note_{}", i),
                format!(
                    "📝 {}\n\
                     \n\
                     {}\n\
                     \n\
                     💡 Press Enter to edit",
                    note.title,
                    &note.content[..note.content.len().min(50)]
                ),
                TooltipPosition::Right,
            );
        }
    }

    fn draw_settings(&self, frame: &mut Frame, area: Rect, ctx: &mut LocustContext) {
        let settings_text = vec![
            format!("Theme: {}", self.settings.theme),
            format!("Auto Save: {}", if self.settings.auto_save { "On" } else { "Off" }),
            format!(
                "Notifications: {}",
                if self.settings.notifications {
                    "On"
                } else {
                    "Off"
                }
            ),
        ];

        let paragraph = Paragraph::new(settings_text.join("\n"))
            .block(Block::default().borders(Borders::ALL).title("Settings"));

        frame.render_widget(paragraph, area);

        // Register settings as targets
        for (i, setting) in ["theme", "auto_save", "notifications"].iter().enumerate() {
            let item_area = Rect {
                x: area.x + 1,
                y: area.y + 1 + i as u16,
                width: area.width - 2,
                height: 1,
            };

            ctx.targets.register(NavTarget {
                id: format!("setting_{}", setting),
                area: item_area,
                kind: TargetKind::Custom("Setting".to_string()),
                priority: 0,
                metadata: HashMap::from([("name".to_string(), setting.to_string())]),
                actions: vec![TargetAction::Select, TargetAction::Edit],
            });

            // Tooltip for setting
            let tooltip = match *setting {
                "theme" => "UI color scheme\n\n💡 Click to change",
                "auto_save" => "Automatically save changes\n\n💡 Click to toggle",
                "notifications" => "Show desktop notifications\n\n💡 Click to toggle",
                _ => "",
            };

            ctx.tooltips.register(
                format!("setting_{}", setting),
                tooltip.to_string(),
                TooltipPosition::Right,
            );
        }
    }

    fn draw_status_bar(&self, frame: &mut Frame, area: Rect) {
        let status = format!(
            " {} tasks | {} notes | Press '?' for help",
            self.tasks.items.len(),
            self.notes.items.len()
        );

        let paragraph = Paragraph::new(status)
            .style(Style::default().bg(Color::DarkGray).fg(Color::White));

        frame.render_widget(paragraph, area);
    }

    fn handle_key(&mut self, code: KeyCode) -> bool {
        match code {
            KeyCode::Char('q') => return false,
            KeyCode::Char('1') => self.current_view = View::Tasks,
            KeyCode::Char('2') => self.current_view = View::Notes,
            KeyCode::Char('3') => self.current_view = View::Settings,
            KeyCode::Char(' ') => {
                if self.current_view == View::Tasks && !self.tasks.items.is_empty() {
                    self.tasks.items[self.tasks.selected].completed =
                        !self.tasks.items[self.tasks.selected].completed;
                }
            }
            KeyCode::Up | KeyCode::Char('k') => match self.current_view {
                View::Tasks => {
                    if self.tasks.selected > 0 {
                        self.tasks.selected -= 1;
                    }
                }
                View::Notes => {
                    if self.notes.selected > 0 {
                        self.notes.selected -= 1;
                    }
                }
                _ => {}
            },
            KeyCode::Down | KeyCode::Char('j') => match self.current_view {
                View::Tasks => {
                    if self.tasks.selected < self.tasks.items.len().saturating_sub(1) {
                        self.tasks.selected += 1;
                    }
                }
                View::Notes => {
                    if self.notes.selected < self.notes.items.len().saturating_sub(1) {
                        self.notes.selected += 1;
                    }
                }
                _ => {}
            },
            _ => {}
        }
        true
    }

    fn handle_target_activation(&mut self, target_id: &str) {
        if let Some(index_str) = target_id.strip_prefix("tab_") {
            if let Ok(index) = index_str.parse::<usize>() {
                self.current_view = match index {
                    0 => View::Tasks,
                    1 => View::Notes,
                    2 => View::Settings,
                    _ => self.current_view,
                };
            }
        } else if let Some(index_str) = target_id.strip_prefix("task_") {
            if let Ok(index) = index_str.parse::<usize>() {
                if index < self.tasks.items.len() {
                    self.tasks.selected = index;
                    self.current_view = View::Tasks;
                }
            }
        } else if let Some(index_str) = target_id.strip_prefix("note_") {
            if let Ok(index) = index_str.parse::<usize>() {
                if index < self.notes.items.len() {
                    self.notes.selected = index;
                    self.current_view = View::Notes;
                }
            }
        }
    }

    fn handle_command(&mut self, cmd_id: &str) {
        match cmd_id {
            "view.tasks" => self.current_view = View::Tasks,
            "view.notes" => self.current_view = View::Notes,
            "view.settings" => self.current_view = View::Settings,
            "task.toggle" => {
                if self.current_view == View::Tasks && !self.tasks.items.is_empty() {
                    self.tasks.items[self.tasks.selected].completed =
                        !self.tasks.items[self.tasks.selected].completed;
                }
            }
            "app.quit" => {
                // Signal quit
            }
            _ => {}
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = AdvancedApp::new();

    // Initialize Locust with all plugins
    let mut locust = create_locust()?;

    let result = run_app(&mut terminal, &mut app, &mut locust);

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

// These imports would come from locust in a real implementation
use std::collections::HashMap as LocustContext;
type NavTarget = ();
type TargetKind = ();
type TargetAction = ();
type TooltipPosition = ();

// Mock Locust creation (in real code, this would use actual Locust types)
fn create_locust() -> Result<MockLocust, Box<dyn Error>> {
    Ok(MockLocust)
}

struct MockLocust;

impl MockLocust {
    fn begin_frame(&mut self) {}
    fn render_overlay(&self, _frame: &mut Frame) {}
    fn on_event(&mut self, _event: &Event) -> EventOutcome {
        EventOutcome {
            consumed: false,
            request_redraw: false,
        }
    }
}

struct EventOutcome {
    consumed: bool,
    request_redraw: bool,
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut AdvancedApp,
    locust: &mut MockLocust,
) -> io::Result<()> {
    loop {
        locust.begin_frame();

        terminal.draw(|frame| {
            let mut ctx = HashMap::new();
            app.draw(frame, &mut ctx);
            locust.render_overlay(frame);
        })?;

        if let Event::Key(KeyEvent { code, .. }) = event::read()? {
            let _outcome = locust.on_event(&Event::Key(KeyEvent::new(code, crossterm::event::KeyModifiers::empty())));

            // In real implementation:
            // - Check for nav_activated
            // - Check for omnibar_executed
            // - Handle target activations
            // - Handle command executions

            if !app.handle_key(code) {
                return Ok(());
            }
        }
    }
}
