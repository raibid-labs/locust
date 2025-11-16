/// Terminal Multiplexer - A tmux-like TUI demonstrating Locust integration
///
/// Features:
/// - Pane management (split horizontal/vertical)
/// - Pane navigation with hints
/// - Pane resizing
/// - Command palette for operations
/// - Session management
/// - Pane tooltips showing process info
/// - Guided tour for first-time users
///
/// Layout:
/// ```
/// ┌─────────────────────────────────────────────────┐
/// │  Pane 1: bash          │  Pane 2: htop          │
/// │  $ ls                  │  CPU:  45% [▇▇▇▇▇    ] │
/// │  file1.txt             │  Mem:  2.1G/8.0G       │
/// │  file2.txt             │  Tasks: 234            │
/// │  $ _                   │                        │
/// ├────────────────────────┴────────────────────────┤
/// │  Pane 3: logs                                   │
/// │  [2025-01-14 10:30:42] INFO: Server started    │
/// │  [2025-01-14 10:30:45] DEBUG: Connection from  │
/// │  [2025-01-14 10:30:47] WARN: High memory      │
/// └─────────────────────────────────────────────────┘
/// Press 'f' for hints | Ctrl+P for commands
/// ```
mod common;

use chrono::Local;
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use locust::{HighlightConfig, HighlightPlugin, Locust, NavPlugin, OmnibarPlugin, TooltipPlugin};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame, Terminal,
};
use std::{
    collections::VecDeque,
    io::{self, Stdout},
    time::{Duration, Instant},
};

use common::mock::LogEntry;

/// Main terminal multiplexer application
struct TerminalMultiplexer {
    /// All panes in the multiplexer
    panes: Vec<Pane>,
    /// Current layout tree
    layout: LayoutTree,
    /// Index of active pane
    active_pane: usize,
    /// Available sessions
    sessions: Vec<Session>,
    /// Current session index
    current_session: usize,
    /// Locust integration
    locust: Locust<CrosstermBackend<Stdout>>,
    /// Whether we're in pane selection mode
    selecting_pane: bool,
    /// Command history
    command_history: VecDeque<String>,
    /// Whether tour is active
    tour_active: bool,
    /// Current tour step
    tour_step: usize,
    /// FPS counter
    fps_counter: common::FpsCounter,
}

/// Represents a single pane in the multiplexer
#[derive(Clone)]
struct Pane {
    /// Unique pane ID
    id: usize,
    /// Pane title
    title: String,
    /// Pane type (shell, logs, etc)
    pane_type: PaneType,
    /// Content lines
    content: VecDeque<String>,
    /// Current scroll offset
    scroll_offset: usize,
    /// Process ID (mock)
    pid: u32,
    /// Memory usage in bytes
    mem_usage: u64,
    /// CPU usage percentage
    cpu_percent: f32,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum PaneType {
    Shell,
    Logs,
    Monitor,
    Editor,
}

impl PaneType {
    fn icon(&self) -> &'static str {
        match self {
            PaneType::Shell => "💻",
            PaneType::Logs => "📋",
            PaneType::Monitor => "📊",
            PaneType::Editor => "📝",
        }
    }
}

/// Layout tree for recursive pane splitting
#[derive(Clone)]
enum LayoutTree {
    /// Single pane (leaf node)
    Leaf(usize),
    /// Horizontal split (left, right, ratio)
    Horizontal(Box<LayoutTree>, Box<LayoutTree>, f32),
    /// Vertical split (top, bottom, ratio)
    Vertical(Box<LayoutTree>, Box<LayoutTree>, f32),
}

impl LayoutTree {
    /// Calculate layout rects for all panes
    fn calculate_rects(&self, area: Rect, panes: &mut Vec<(usize, Rect)>) {
        match self {
            LayoutTree::Leaf(id) => {
                panes.push((*id, area));
            }
            LayoutTree::Horizontal(left, right, ratio) => {
                let width = (area.width as f32 * ratio) as u16;
                let chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Length(width), Constraint::Min(0)])
                    .split(area);

                left.calculate_rects(chunks[0], panes);
                right.calculate_rects(chunks[1], panes);
            }
            LayoutTree::Vertical(top, bottom, ratio) => {
                let height = (area.height as f32 * ratio) as u16;
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Length(height), Constraint::Min(0)])
                    .split(area);

                top.calculate_rects(chunks[0], panes);
                bottom.calculate_rects(chunks[1], panes);
            }
        }
    }

    /// Find a pane in the tree
    fn find_pane(&self, id: usize) -> bool {
        match self {
            LayoutTree::Leaf(pane_id) => *pane_id == id,
            LayoutTree::Horizontal(left, right, _) => left.find_pane(id) || right.find_pane(id),
            LayoutTree::Vertical(top, bottom, _) => top.find_pane(id) || bottom.find_pane(id),
        }
    }

    /// Replace a pane with a new layout
    fn replace_pane(&mut self, id: usize, new_tree: LayoutTree) -> bool {
        match self {
            LayoutTree::Leaf(pane_id) => {
                if *pane_id == id {
                    *self = new_tree;
                    true
                } else {
                    false
                }
            }
            LayoutTree::Horizontal(left, right, _) => {
                left.replace_pane(id, new_tree.clone()) || right.replace_pane(id, new_tree)
            }
            LayoutTree::Vertical(top, bottom, _) => {
                top.replace_pane(id, new_tree.clone()) || bottom.replace_pane(id, new_tree)
            }
        }
    }
}

/// Session containing multiple panes
struct Session {
    name: String,
    layout: LayoutTree,
    active_pane: usize,
}

impl Pane {
    fn new(id: usize, title: String, pane_type: PaneType) -> Self {
        let content = match pane_type {
            PaneType::Shell => {
                let mut lines = VecDeque::new();
                lines.push_back("$ ls -la".to_string());
                lines.push_back("total 48".to_string());
                lines.push_back("drwxr-xr-x  6 user  staff   192 Jan 14 10:30 .".to_string());
                lines.push_back("drwxr-xr-x  3 user  staff    96 Jan 13 09:15 ..".to_string());
                lines.push_back(
                    "-rw-r--r--  1 user  staff  1234 Jan 14 10:29 file1.txt".to_string(),
                );
                lines.push_back(
                    "-rw-r--r--  1 user  staff  5678 Jan 14 10:30 file2.txt".to_string(),
                );
                lines.push_back("drwxr-xr-x  4 user  staff   128 Jan 14 09:00 src".to_string());
                lines.push_back("$ _".to_string());
                lines
            }
            PaneType::Logs => {
                let logs = common::mock::generate_logs(10);
                logs.iter()
                    .map(|log| {
                        format!(
                            "[{}] {}: {}",
                            log.timestamp.format("%Y-%m-%d %H:%M:%S"),
                            log.level.as_str(),
                            log.message
                        )
                    })
                    .collect()
            }
            PaneType::Monitor => {
                let mut lines = VecDeque::new();
                lines.push_back("System Monitor".to_string());
                lines.push_back("".to_string());
                lines.push_back("CPU:  45% [▇▇▇▇▇▇▇▇▇          ]".to_string());
                lines.push_back("Mem:  2.1G/8.0G [▇▇▇▇▇          ]".to_string());
                lines.push_back("Disk: 45G/120G [▇▇▇▇▇▇▇         ]".to_string());
                lines.push_back("".to_string());
                lines.push_back("Tasks: 234 total".to_string());
                lines.push_back("  Running: 12".to_string());
                lines.push_back("  Sleeping: 220".to_string());
                lines.push_back("  Stopped: 2".to_string());
                lines
            }
            PaneType::Editor => {
                let mut lines = VecDeque::new();
                lines.push_back("// main.rs".to_string());
                lines.push_back("".to_string());
                lines.push_back("fn main() {".to_string());
                lines.push_back("    println!(\"Hello, world!\");".to_string());
                lines.push_back("}".to_string());
                lines
            }
        };

        Self {
            id,
            title,
            pane_type,
            content,
            scroll_offset: 0,
            pid: 1000 + id as u32,
            mem_usage: 50_000_000 + (id as u64 * 10_000_000),
            cpu_percent: 5.0 + (id as f32 * 3.0),
        }
    }

    fn add_line(&mut self, line: String) {
        self.content.push_back(line);
        if self.content.len() > 1000 {
            self.content.pop_front();
        }
    }

    fn scroll_up(&mut self) {
        if self.scroll_offset > 0 {
            self.scroll_offset -= 1;
        }
    }

    fn scroll_down(&mut self) {
        if self.scroll_offset < self.content.len().saturating_sub(1) {
            self.scroll_offset += 1;
        }
    }
}

impl TerminalMultiplexer {
    fn new(terminal: Terminal<CrosstermBackend<Stdout>>) -> io::Result<Self> {
        let mut locust = Locust::new(terminal);

        // Initialize plugins
        locust.add_plugin(NavPlugin::default());
        locust.add_plugin(OmnibarPlugin::default());
        locust.add_plugin(TooltipPlugin::default());

        let highlight_config = HighlightConfig {
            steps: vec![
                "Welcome to Terminal Multiplexer! Press 'n' to continue.".to_string(),
                "Split panes with Ctrl+B, H (horizontal) or V (vertical)".to_string(),
                "Navigate panes by pressing 'f' to show hints".to_string(),
                "Resize panes with Ctrl+B, then arrow keys".to_string(),
                "Open command palette with Ctrl+P".to_string(),
                "Close panes with Ctrl+B, X".to_string(),
            ],
            highlight_color: Color::Yellow,
            text_color: Color::White,
        };
        locust.add_plugin(HighlightPlugin::new(highlight_config));

        // Create initial panes
        let panes = vec![
            Pane::new(0, "bash".to_string(), PaneType::Shell),
            Pane::new(1, "htop".to_string(), PaneType::Monitor),
            Pane::new(2, "logs".to_string(), PaneType::Logs),
        ];

        // Create initial layout (horizontal split with bottom pane)
        let layout = LayoutTree::Vertical(
            Box::new(LayoutTree::Horizontal(
                Box::new(LayoutTree::Leaf(0)),
                Box::new(LayoutTree::Leaf(1)),
                0.5,
            )),
            Box::new(LayoutTree::Leaf(2)),
            0.6,
        );

        // Create sessions
        let sessions = vec![Session {
            name: "main".to_string(),
            layout: layout.clone(),
            active_pane: 0,
        }];

        Ok(Self {
            panes,
            layout,
            active_pane: 0,
            sessions,
            current_session: 0,
            locust,
            selecting_pane: false,
            command_history: VecDeque::new(),
            tour_active: false,
            tour_step: 0,
            fps_counter: common::FpsCounter::new(),
        })
    }

    fn run(&mut self) -> io::Result<()> {
        let tick_rate = Duration::from_millis(16); // ~60 FPS
        let mut last_tick = Instant::now();

        loop {
            self.fps_counter.tick();
            self.draw()?;

            let timeout = tick_rate.saturating_sub(last_tick.elapsed());
            if event::poll(timeout)? {
                if let Event::Key(key) = event::read()? {
                    if self.handle_input(key)? {
                        break;
                    }
                }
            }

            if last_tick.elapsed() >= tick_rate {
                self.update();
                last_tick = Instant::now();
            }
        }

        Ok(())
    }

    fn handle_input(&mut self, key: KeyEvent) -> io::Result<bool> {
        // Check if tour is handling input
        if self.tour_active {
            if key.code == KeyCode::Char('n') {
                self.tour_step += 1;
                if self.tour_step >= 6 {
                    self.tour_active = false;
                }
                return Ok(false);
            } else if key.code == KeyCode::Char('q') {
                self.tour_active = false;
                return Ok(false);
            }
        }

        // Global commands
        match (key.modifiers, key.code) {
            (KeyModifiers::CONTROL, KeyCode::Char('c')) => return Ok(true),
            (KeyModifiers::CONTROL, KeyCode::Char('p')) => {
                // Open command palette
                self.locust.omnibar_mut().toggle();
                return Ok(false);
            }
            (_, KeyCode::Char('f')) => {
                // Toggle pane selection mode
                self.selecting_pane = !self.selecting_pane;
                return Ok(false);
            }
            (_, KeyCode::Char('t')) => {
                // Toggle tour
                self.tour_active = !self.tour_active;
                if self.tour_active {
                    self.tour_step = 0;
                }
                return Ok(false);
            }
            _ => {}
        }

        // Pane selection mode
        if self.selecting_pane {
            if let KeyCode::Char(c) = key.code {
                if let Some(digit) = c.to_digit(10) {
                    let idx = digit as usize;
                    if idx < self.panes.len() {
                        self.active_pane = idx;
                        self.selecting_pane = false;
                    }
                }
            }
            return Ok(false);
        }

        // Command palette mode
        if self.locust.omnibar().is_active() {
            return self.handle_omnibar_input(key);
        }

        // Ctrl+B prefix commands
        static mut CTRL_B_MODE: bool = false;
        unsafe {
            if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('b') {
                CTRL_B_MODE = true;
                return Ok(false);
            }

            if CTRL_B_MODE {
                CTRL_B_MODE = false;
                match key.code {
                    KeyCode::Char('h') => self.split_horizontal(),
                    KeyCode::Char('v') => self.split_vertical(),
                    KeyCode::Char('x') => self.close_pane(),
                    KeyCode::Left => self.resize_pane(-5, 0),
                    KeyCode::Right => self.resize_pane(5, 0),
                    KeyCode::Up => self.resize_pane(0, -5),
                    KeyCode::Down => self.resize_pane(0, 5),
                    _ => {}
                }
                return Ok(false);
            }
        }

        // Pane-specific controls
        match key.code {
            KeyCode::Up => self.panes[self.active_pane].scroll_up(),
            KeyCode::Down => self.panes[self.active_pane].scroll_down(),
            KeyCode::PageUp => {
                for _ in 0..10 {
                    self.panes[self.active_pane].scroll_up();
                }
            }
            KeyCode::PageDown => {
                for _ in 0..10 {
                    self.panes[self.active_pane].scroll_down();
                }
            }
            _ => {}
        }

        Ok(false)
    }

    fn handle_omnibar_input(&mut self, key: KeyEvent) -> io::Result<bool> {
        match key.code {
            KeyCode::Esc => {
                self.locust.omnibar_mut().toggle();
            }
            KeyCode::Enter => {
                let query = self.locust.omnibar().query().to_string();
                self.execute_command(&query);
                self.locust.omnibar_mut().toggle();
            }
            KeyCode::Char(c) => {
                self.locust.omnibar_mut().push_char(c);
            }
            KeyCode::Backspace => {
                self.locust.omnibar_mut().pop_char();
            }
            _ => {}
        }
        Ok(false)
    }

    fn execute_command(&mut self, command: &str) {
        self.command_history.push_back(command.to_string());
        if self.command_history.len() > 50 {
            self.command_history.pop_front();
        }

        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.is_empty() {
            return;
        }

        match parts[0] {
            "split" => {
                if parts.len() > 1 {
                    match parts[1] {
                        "horizontal" | "h" => self.split_horizontal(),
                        "vertical" | "v" => self.split_vertical(),
                        _ => {}
                    }
                }
            }
            "close" => self.close_pane(),
            "session" => {
                if parts.len() > 1 {
                    match parts[1] {
                        "new" => self.new_session(),
                        "switch" => {
                            if parts.len() > 2 {
                                if let Ok(idx) = parts[2].parse::<usize>() {
                                    self.switch_session(idx);
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }

    fn split_horizontal(&mut self) {
        let new_id = self.panes.len();
        let new_pane = Pane::new(new_id, format!("pane-{}", new_id), PaneType::Shell);
        self.panes.push(new_pane);

        let new_tree = LayoutTree::Horizontal(
            Box::new(LayoutTree::Leaf(self.active_pane)),
            Box::new(LayoutTree::Leaf(new_id)),
            0.5,
        );

        self.layout.replace_pane(self.active_pane, new_tree);
        self.active_pane = new_id;
    }

    fn split_vertical(&mut self) {
        let new_id = self.panes.len();
        let new_pane = Pane::new(new_id, format!("pane-{}", new_id), PaneType::Shell);
        self.panes.push(new_pane);

        let new_tree = LayoutTree::Vertical(
            Box::new(LayoutTree::Leaf(self.active_pane)),
            Box::new(LayoutTree::Leaf(new_id)),
            0.5,
        );

        self.layout.replace_pane(self.active_pane, new_tree);
        self.active_pane = new_id;
    }

    fn close_pane(&mut self) {
        if self.panes.len() <= 1 {
            return; // Don't close the last pane
        }
        // Simplified: just hide the pane (full implementation would rebuild tree)
    }

    fn resize_pane(&mut self, _dx: i32, _dy: i32) {
        // Simplified: would adjust split ratios in tree
    }

    fn new_session(&mut self) {
        let session = Session {
            name: format!("session-{}", self.sessions.len()),
            layout: LayoutTree::Leaf(0),
            active_pane: 0,
        };
        self.sessions.push(session);
    }

    fn switch_session(&mut self, idx: usize) {
        if idx < self.sessions.len() {
            self.current_session = idx;
            self.layout = self.sessions[idx].layout.clone();
            self.active_pane = self.sessions[idx].active_pane;
        }
    }

    fn update(&mut self) {
        // Update pane content (simulate activity)
        let now = Local::now();
        if now.timestamp() % 2 == 0 {
            for pane in &mut self.panes {
                if pane.pane_type == PaneType::Logs {
                    let log = format!(
                        "[{}] INFO: Activity detected",
                        now.format("%Y-%m-%d %H:%M:%S")
                    );
                    pane.add_line(log);
                }
            }
        }
    }

    fn draw(&mut self) -> io::Result<()> {
        self.locust.terminal_mut().draw(|f| {
            let area = f.area();

            // Calculate pane rects
            let mut pane_rects = Vec::new();
            self.layout.calculate_rects(area, &mut pane_rects);

            // Draw each pane
            for (pane_id, rect) in pane_rects {
                if let Some(pane) = self.panes.get(pane_id) {
                    self.draw_pane(f, pane, rect, pane_id == self.active_pane);
                }
            }

            // Draw hints if selecting
            if self.selecting_pane {
                self.draw_hints(f, &pane_rects);
            }

            // Draw status bar
            self.draw_status_bar(f, area);

            // Draw tour if active
            if self.tour_active {
                self.draw_tour(f, area);
            }
        })?;

        Ok(())
    }

    fn draw_pane(&self, f: &mut Frame, pane: &Pane, area: Rect, is_active: bool) {
        let border_style = if is_active {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().fg(Color::Gray)
        };

        let title = format!(" {} {} ", pane.pane_type.icon(), pane.title);
        let block = Block::default()
            .borders(Borders::ALL)
            .title(title)
            .border_style(border_style);

        let inner = block.inner(area);
        f.render_widget(block, area);

        // Render content
        let visible_lines: Vec<Line> = pane
            .content
            .iter()
            .skip(pane.scroll_offset)
            .take(inner.height as usize)
            .map(|s| Line::from(s.as_str()))
            .collect();

        let paragraph = Paragraph::new(visible_lines).wrap(Wrap { trim: false });
        f.render_widget(paragraph, inner);
    }

    fn draw_hints(&self, f: &mut Frame, pane_rects: &[(usize, Rect)]) {
        for (idx, (_, rect)) in pane_rects.iter().enumerate() {
            let hint = format!(" {} ", idx);
            let hint_widget = Paragraph::new(hint)
                .style(
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )
                .alignment(Alignment::Center);

            let hint_area = Rect {
                x: rect.x + rect.width / 2 - 2,
                y: rect.y + rect.height / 2,
                width: 5,
                height: 1,
            };
            f.render_widget(hint_widget, hint_area);
        }
    }

    fn draw_status_bar(&self, f: &mut Frame, area: Rect) {
        let status_area = Rect {
            x: area.x,
            y: area.y + area.height - 1,
            width: area.width,
            height: 1,
        };

        let fps = format!("{:.1} FPS", self.fps_counter.fps());
        let session_info = format!(
            "Session: {} | Panes: {} | {}",
            self.sessions[self.current_session].name,
            self.panes.len(),
            fps
        );
        let help = "Press 'f' for hints | Ctrl+P for commands | 't' for tour | Ctrl+C to quit";

        let status_text = format!("{} | {}", session_info, help);
        let status = Paragraph::new(status_text)
            .style(Style::default().bg(Color::DarkGray).fg(Color::White));

        f.render_widget(status, status_area);
    }

    fn draw_tour(&self, f: &mut Frame, area: Rect) {
        let messages = [
            "Welcome to Terminal Multiplexer! Press 'n' to continue.",
            "Split panes with Ctrl+B, H (horizontal) or V (vertical)",
            "Navigate panes by pressing 'f' to show hints",
            "Resize panes with Ctrl+B, then arrow keys",
            "Open command palette with Ctrl+P",
            "Close panes with Ctrl+B, X. Press 'q' to exit tour.",
        ];

        let popup_area = common::centered_rect(60, 30, area);
        let message = messages.get(self.tour_step).unwrap_or(&messages[0]);

        let tour_widget = Paragraph::new(*message)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Tutorial ")
                    .border_style(Style::default().fg(Color::Yellow)),
            )
            .style(Style::default().fg(Color::White))
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });

        f.render_widget(tour_widget, popup_area);
    }
}

fn main() -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;

    let mut app = TerminalMultiplexer::new(terminal)?;
    let result = app.run();

    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen)?;

    result
}
