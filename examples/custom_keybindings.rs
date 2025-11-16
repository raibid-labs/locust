use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use locust::core::keybindings::{detect_conflicts, KeyBinding, KeyCodeDef, KeyMap};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame, Terminal,
};
use std::io;
use std::path::Path;

struct App {
    keymap: KeyMap,
    mode: AppMode,
    status_message: String,
}

#[derive(PartialEq)]
enum AppMode {
    Normal,
    Conflicts,
}

impl App {
    fn new() -> Self {
        let mut keymap = KeyMap::default();

        // Try to load keymap from keymaps/ directory
        if Path::new("keymaps/default.toml").exists() {
            if let Ok(loaded) = KeyMap::from_file(Path::new("keymaps/default.toml")) {
                keymap = loaded;
            }
        }

        Self {
            keymap,
            mode: AppMode::Normal,
            status_message:
                "Press 'c' to check for conflicts, 'a' to add binding, 'r' to remove binding"
                    .to_string(),
        }
    }

    fn check_conflicts(&mut self) {
        let conflicts = detect_conflicts(&self.keymap);
        if conflicts.is_empty() {
            self.status_message = "✓ No conflicts detected!".to_string();
            self.mode = AppMode::Normal;
        } else {
            self.status_message = format!("⚠ {} conflicts detected!", conflicts.len());
            self.mode = AppMode::Conflicts;
        }
    }

    fn add_sample_binding(&mut self) {
        let binding = KeyBinding {
            key: KeyCodeDef::Char('x'),
            modifiers: KeyModifiers::CONTROL,
        };
        if self.keymap.bind("custom.action", binding).is_ok() {
            self.status_message = "Added custom binding: Ctrl+X → custom.action".to_string();
        }
    }

    fn remove_sample_binding(&mut self) {
        self.keymap.unbind("custom.action");
        self.status_message = "Removed custom.action binding".to_string();
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app
    let mut app = App::new();

    // Run app
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
        println!("Error: {:?}", err);
    }

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, app))?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => return Ok(()),
                KeyCode::Char('c') => app.check_conflicts(),
                KeyCode::Char('a') => app.add_sample_binding(),
                KeyCode::Char('r') => app.remove_sample_binding(),
                _ => {}
            }
        }
    }
}

fn ui<B: Backend>(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints(
            [
                Constraint::Length(3),
                Constraint::Min(10),
                Constraint::Length(5),
                Constraint::Length(3),
            ]
            .as_ref(),
        )
        .split(f.area());

    // Title
    let title = Paragraph::new("Custom Keybindings Demo")
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow)),
        );
    f.render_widget(title, chunks[0]);

    // Keybindings list
    let mut bindings_list = Vec::new();

    // Global bindings
    bindings_list.push(ListItem::new(Line::from(vec![Span::styled(
        "Global Bindings",
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    )])));
    for (action, binding) in &app.keymap.global {
        let key_str = format!("{:?}", binding.key);
        bindings_list.push(ListItem::new(format!("  {} → {}", key_str, action)));
    }

    // Plugin bindings
    for (plugin, bindings) in &app.keymap.plugins {
        bindings_list.push(ListItem::new(""));
        bindings_list.push(ListItem::new(Line::from(vec![Span::styled(
            format!("{} Plugin", plugin),
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )])));
        for (action, binding) in bindings {
            let key_str = format!("{:?}", binding.key);
            bindings_list.push(ListItem::new(format!(
                "  {} → {}.{}",
                key_str, plugin, action
            )));
        }
    }

    let bindings_widget = List::new(bindings_list).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Current Keybindings")
            .border_style(Style::default().fg(Color::Blue)),
    );
    f.render_widget(bindings_widget, chunks[1]);

    // Conflicts
    let conflicts = detect_conflicts(&app.keymap);
    let conflict_lines: Vec<Line> = if conflicts.is_empty() {
        vec![Line::from(Span::styled(
            "✓ No conflicts detected",
            Style::default().fg(Color::Green),
        ))]
    } else {
        conflicts
            .iter()
            .flat_map(|conflict| {
                vec![
                    Line::from(vec![
                        Span::styled("⚠ Conflict: ", Style::default().fg(Color::Red)),
                        Span::raw(format!("{:?}", conflict.binding.key)),
                    ]),
                    Line::from(format!("  Actions: {}", conflict.actions.join(", "))),
                ]
            })
            .collect()
    };

    let conflicts_widget = Paragraph::new(conflict_lines).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Conflict Detection")
            .border_style(if app.mode == AppMode::Conflicts {
                Style::default().fg(Color::Red)
            } else {
                Style::default().fg(Color::Green)
            }),
    );
    f.render_widget(conflicts_widget, chunks[2]);

    // Status
    let status = Paragraph::new(app.status_message.clone())
        .style(Style::default().fg(Color::Yellow))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(status, chunks[3]);
}
