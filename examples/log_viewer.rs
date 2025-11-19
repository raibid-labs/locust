/// # Log Viewer Example
///
/// A comprehensive log analysis tool demonstrating advanced Locust features
/// for large-scale data navigation. This example showcases:
///
/// - Large scrollable log display with efficient rendering
/// - Multi-level log filtering (ERROR, WARN, INFO, DEBUG)
/// - Full-text search with highlighting
/// - Jump to line number functionality
/// - Bookmarks and markers for important log entries
/// - Tail mode for following real-time log updates
/// - Statistics panel showing log distribution
/// - Hint mode ('f') for quick navigation to specific lines
///
/// ## Controls
///
/// - `f` - Enter hint mode to jump to specific log lines
/// - `/` - Open search dialog
/// - `g` - Go to line number
/// - `m` - Toggle bookmark on current line
/// - `n` / `N` - Next/previous search result
/// - `F` - Toggle tail mode (follow new logs)
/// - `1-4` - Filter by level (1=ERROR, 2=WARN, 3=INFO, 4=DEBUG)
/// - `0` - Clear filter (show all)
/// - `Up/Down` - Scroll log view
/// - `Page Up/Down` - Scroll by page
/// - `Home/End` - Jump to start/end
/// - `q` - Quit the application
/// - `Esc` - Cancel current action
///
/// ## Architecture
///
/// The log viewer efficiently handles large log files through:
/// - Virtual scrolling for smooth performance
/// - Incremental search with result caching
/// - Level-based filtering with statistics tracking
/// - Bookmark management for quick navigation
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use locust::prelude::*;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, ListItem, Paragraph},
    Frame, Terminal,
};
use std::io::{self, Stdout};
use std::time::{Duration, SystemTime};
use std::fs::File;
use std::path::PathBuf;

use log::{debug, LevelFilter};
use simplelog::{CombinedLogger, Config, WriteLogger};
use locust::ratatui_ext::LogTailer;

/// Log level enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
}

impl LogLevel {
    fn color(&self) -> Color {
        match self {
            Self::Error => Color::Red,
            Self::Warn => Color::Yellow,
            Self::Info => Color::Green,
            Self::Debug => Color::Cyan,
        }
    }

    fn label(&self) -> &'static str {
        match self {
            Self::Error => "ERROR",
            Self::Warn => "WARN ",
            Self::Info => "INFO ",
            Self::Debug => "DEBUG",
        }
    }

    fn from_number(n: u8) -> Option<Self> {
        match n {
            1 => Some(Self::Error),
            2 => Some(Self::Warn),
            3 => Some(Self::Info),
            4 => Some(Self::Debug),
            _ => None,
        }
    }
}

/// A single log entry
#[derive(Clone)]
struct LogEntry {
    line_number: usize,
    timestamp: String,
    level: LogLevel,
    source: String,
    message: String,
    bookmarked: bool,
}

/// Filter state for log display
#[derive(Clone, Copy, PartialEq)]
enum FilterMode {
    All,
    Level(LogLevel),
}

impl FilterMode {
    fn matches(&self, entry: &LogEntry) -> bool {
        match self {
            Self::All => true,
            Self::Level(level) => entry.level == *level,
        }
    }

    fn label(&self) -> String {
        match self {
            Self::All => "ALL".to_string(),
            Self::Level(level) => level.label().to_string(),
        }
    }
}

/// Input mode for the log viewer
#[derive(Clone, Copy, PartialEq)]
enum InputMode {
    Normal,
    Search,
    GotoLine,
}

/// Log statistics
#[derive(Default)]
struct LogStats {
    total: usize,
    errors: usize,
    warnings: usize,
    info: usize,
    debug: usize,
    bookmarked: usize,
}

/// Main log viewer application state
struct LogViewer {
    /// All log entries
    logs: Vec<LogEntry>,
    /// Filtered log indices
    filtered_indices: Vec<usize>,
    /// Current scroll position
    scroll_pos: usize,
    /// Current filter mode
    filter_mode: FilterMode,
    /// Tail mode enabled
    tail_mode: bool,
    /// Bookmarked line numbers
    #[allow(dead_code)]
    bookmarks: Vec<usize>,
    /// Log statistics
    stats: LogStats,
    /// Last update time
    #[allow(dead_code)]
    last_update: SystemTime,
    /// Should quit flag
    should_quit: bool,
}

impl LogViewer {
    fn new() -> Self {
        let logs = Self::generate_sample_logs();
        let stats = Self::calculate_stats(&logs);
        let filtered_indices: Vec<usize> = (0..logs.len()).collect();

        Self {
            logs,
            filtered_indices,
            scroll_pos: 0,
            filter_mode: FilterMode::All,
            tail_mode: false,
            bookmarks: Vec::new(),
            stats,
            last_update: SystemTime::now(),
            should_quit: false,
        }
    }

    /// Generate sample log entries for demonstration
    fn generate_sample_logs() -> Vec<LogEntry> {
        let sources = ["auth", "db", "api", "worker", "cache", "scheduler"];
        let messages = vec![
            ("Starting service...", LogLevel::Info),
            ("Configuration loaded successfully", LogLevel::Info),
            ("Database connection established", LogLevel::Info),
            ("High memory usage detected", LogLevel::Warn),
            ("Connection timeout occurred", LogLevel::Error),
            ("Retry attempt successful", LogLevel::Info),
            ("Deprecated API endpoint used", LogLevel::Warn),
            ("Cache miss for key", LogLevel::Debug),
            ("Request processed in 45ms", LogLevel::Debug),
            ("Failed to authenticate user", LogLevel::Error),
            ("Session expired for user", LogLevel::Warn),
            ("Background task completed", LogLevel::Info),
            ("Rate limit exceeded", LogLevel::Warn),
            ("Database query slow", LogLevel::Warn),
            ("Critical error in worker", LogLevel::Error),
            ("Health check passed", LogLevel::Info),
            ("Metrics collected", LogLevel::Debug),
            ("Queue depth: 1234", LogLevel::Debug),
        ];

        let mut logs = Vec::new();
        let mut timestamp_base = 1731571200; // 2024-11-14 10:00:00

        for i in 0..500 {
            let (message, level) = &messages[i % messages.len()];
            let source = sources[i % sources.len()];

            // Generate realistic timestamp
            timestamp_base += (i % 60) as u64;
            let datetime = format_timestamp(timestamp_base);

            logs.push(LogEntry {
                line_number: i + 1,
                timestamp: datetime,
                level: *level,
                source: source.to_string(),
                message: format!("{} [id:{}]", message, i),
                bookmarked: false,
            });
        }

        logs
    }

    /// Calculate statistics from logs
    fn calculate_stats(logs: &[LogEntry]) -> LogStats {
        let mut stats = LogStats::default();
        stats.total = logs.len();

        for log in logs {
            match log.level {
                LogLevel::Error => stats.errors += 1,
                LogLevel::Warn => stats.warnings += 1,
                LogLevel::Info => stats.info += 1,
                LogLevel::Debug => stats.debug += 1,
            }
            if log.bookmarked {
                stats.bookmarked += 1;
            }
        }

        stats
    }

    /// Apply current filter to logs
    fn apply_filter(&mut self) {
        self.filtered_indices = self
            .logs
            .iter()
            .enumerate()
            .filter(|(_, log)| self.filter_mode.matches(log))
            .map(|(idx, _)| idx)
            .collect();

        // Adjust scroll position if needed
        if self.scroll_pos >= self.filtered_indices.len() {
            self.scroll_pos = self.filtered_indices.len().saturating_sub(1);
        }
    }

    /// Toggle bookmark on current line
    fn toggle_bookmark(&mut self) {
        if let Some(&idx) = self.filtered_indices.get(self.scroll_pos) {
            if let Some(log) = self.logs.get_mut(idx) {
                log.bookmarked = !log.bookmarked;
                self.stats = Self::calculate_stats(&self.logs);
            }
        }
    }

    /// Simulate new log entries in tail mode
    fn append_new_log(&mut self) {
        let new_line_number = self.logs.len() + 1;
        let timestamp = format_timestamp(
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        );

        let messages = [
            ("New request received", LogLevel::Info),
            ("Processing task", LogLevel::Debug),
            ("Warning: slow query", LogLevel::Warn),
        ];

        let (message, level) = &messages[new_line_number % messages.len()];

        self.logs.push(LogEntry {
            line_number: new_line_number,
            timestamp,
            level: *level,
            source: "live".to_string(),
            message: format!("{} [id:{}]", message, new_line_number),
            bookmarked: false,
        });

        self.stats = Self::calculate_stats(&self.logs);
        self.apply_filter();

        if self.tail_mode {
            self.scroll_pos = self.filtered_indices.len().saturating_sub(1);
        }
    }

    /// Handle keyboard input
    fn handle_key(&mut self, key: KeyCode, modifiers: KeyModifiers) {
        self.handle_normal_key(key, modifiers)
    }

    fn handle_normal_key(&mut self, key: KeyCode, modifiers: KeyModifiers) {
        match key {
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Char('m') => self.toggle_bookmark(),
            KeyCode::Char('F') => self.tail_mode = !self.tail_mode,
            KeyCode::Char(c @ '0'..='4') => {
                let num = c as u8 - b'0';
                if num == 0 {
                    self.filter_mode = FilterMode::All;
                } else if let Some(level) = LogLevel::from_number(num) {
                    self.filter_mode = FilterMode::Level(level);
                }
                self.apply_filter();
            }
            KeyCode::Up => {
                if self.scroll_pos > 0 {
                    self.scroll_pos -= 1;
                }
            }
            KeyCode::Down => {
                if self.scroll_pos < self.filtered_indices.len().saturating_sub(1) {
                    self.scroll_pos += 1;
                }
            }
            KeyCode::PageUp => {
                self.scroll_pos = self.scroll_pos.saturating_sub(10);
            }
            KeyCode::PageDown => {
                self.scroll_pos =
                    (self.scroll_pos + 10).min(self.filtered_indices.len().saturating_sub(1));
            }
            KeyCode::Home => {
                self.scroll_pos = 0;
            }
            KeyCode::End => {
                self.scroll_pos = self.filtered_indices.len().saturating_sub(1);
            }
            _ => {}
        }
    }

    /// Render the log viewer UI
    fn draw(&self, f: &mut Frame, locust: &mut Locust<CrosstermBackend<Stdout>>, log_tailer: &mut LogTailer, target_builder: &mut TargetBuilder) {
        let size = f.area();

        // Main layout: filter bar + content + status bar + log tailer
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // Filter bar
                Constraint::Min(0),     // Content
                Constraint::Length(3),  // Status bar
                Constraint::Length(10), // Log tailer
            ])
            .split(size);

        // Render components
        self.draw_filter_bar(f, chunks[0], locust, target_builder);
        self.draw_logs(f, chunks[1], locust, target_builder);
        self.draw_status_bar(f, chunks[2], locust, target_builder);

        // Render Log Tailer
        f.render_widget(log_tailer, chunks[3]);

        // Let Locust render overlays
        locust.render_overlay(f);
    }

    fn draw_filter_bar(&self, f: &mut Frame, area: Rect, locust: &mut Locust<CrosstermBackend<Stdout>>, target_builder: &mut TargetBuilder) {
        let text = vec![Line::from(vec![
            Span::styled(" Filter: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(
                format!("[{}] ", self.filter_mode.label()),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" | "),
            Span::styled(
                if self.tail_mode {
                    "TAIL ON"
                } else {
                    "TAIL OFF"
                },
                Style::default().fg(if self.tail_mode {
                    Color::Green
                } else {
                    Color::DarkGray
                }),
            ),
        ])];

        let paragraph = Paragraph::new(text).block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(" Log Viewer "),
        );

        // Register NavTargets for filter options
        let filter_area = Rect::new(area.x + 10, area.y + 1, 10, 1); // Approximate position
        locust.ctx.targets.register(
            target_builder.custom(filter_area, "Filter Mode", TargetAction::Activate, TargetPriority::Normal)
        );

        // Register NavTarget for tail mode toggle
        let tail_area = Rect::new(area.width - 10, area.y + 1, 8, 1); // Approximate position
        locust.ctx.targets.register(
            target_builder.custom(tail_area, "Tail Mode Toggle", TargetAction::Activate, TargetPriority::Normal)
        );

    }

    fn draw_logs(&self, f: &mut Frame, area: Rect, locust: &mut Locust<CrosstermBackend<Stdout>>, target_builder: &mut TargetBuilder) {
        let visible_lines = area.height.saturating_sub(2) as usize;
        let start = self.scroll_pos;
        let end = (start + visible_lines).min(self.filtered_indices.len());

        let items: Vec<ListItem> = self
            .filtered_indices
            .iter()
            .skip(start)
            .take(visible_lines)
            .map(|&idx| {
                let log = &self.logs[idx];
                let bookmark_icon = if log.bookmarked { "★ " } else { "  " };

                let line_span = Span::styled(
                    format!("{:04} ", log.line_number),
                    Style::default().fg(Color::DarkGray),
                );
                let level_span = Span::styled(
                    format!("[{}] ", log.level.label()),
                    Style::default()
                        .fg(log.level.color())
                        .add_modifier(Modifier::BOLD),
                );
                let time_span = Span::styled(
                    format!("{} ", log.timestamp),
                    Style::default().fg(Color::Blue),
                );
                let source_span = Span::styled(
                    format!("{:8} ", log.source),
                    Style::default().fg(Color::Magenta),
                );
                let bookmark_span = Span::styled(bookmark_icon, Style::default().fg(Color::Yellow));
                let msg_span = Span::raw(&log.message);

                ListItem::new(Line::from(vec![
                    bookmark_span,
                    line_span,
                    level_span,
                    time_span,
                    source_span,
                    msg_span,
                ]))
            })
            .collect();

        let list = List::new(items).block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(format!(
                    " Logs {}-{} of {} ",
                    start + 1,
                    end,
                    self.filtered_indices.len()
                )),
        );

        f.render_widget(list, area);

        // Register NavTargets for log entries
        let list_items_area = Block::default().borders(Borders::ALL).inner(area);
        let row_height = 1;
        for (i, &idx) in self.filtered_indices.iter().skip(start).take(visible_lines).enumerate() {
            let log = &self.logs[idx];
            let item_rect = Rect::new(
                list_items_area.x,
                list_items_area.y + i as u16 * row_height,
                list_items_area.width,
                row_height,
            );
            locust.ctx.targets.register(
                target_builder.list_item(item_rect, format!("Log Entry: {}", log.message))
            );
        }
    }

    fn draw_status_bar(&self, f: &mut Frame, area: Rect, locust: &mut Locust<CrosstermBackend<Stdout>>, target_builder: &mut TargetBuilder) {
        let text = vec![Line::from(vec![
            Span::styled(
                format!(" Total: {} ", self.stats.total),
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::raw("| "),
            Span::styled(
                format!("ERROR: {} ", self.stats.errors),
                Style::default().fg(Color::Red),
            ),
            Span::styled(
                format!("WARN: {} ", self.stats.warnings),
                Style::default().fg(Color::Yellow),
            ),
            Span::styled(
                format!("INFO: {} ", self.stats.info),
                Style::default().fg(Color::Green),
            ),
            Span::styled(
                format!("DEBUG: {} ", self.stats.debug),
                Style::default().fg(Color::Cyan),
            ),
            Span::raw("| "),
            Span::styled(
                format!("Bookmarks: {} ", self.stats.bookmarked),
                Style::default().fg(Color::Yellow),
            ),
            Span::raw("| "),
            Span::styled(
                format!("Filtered: {} ", self.filtered_indices.len()),
                Style::default().fg(Color::Magenta),
            ),
        ])];

        let paragraph = Paragraph::new(text).block(
            Block::default()
                .borders(Borders::ALL)
        f.render_widget(paragraph, area);

        // Register NavTargets for statistics
        let base_x = area.x + 1;
        let base_y = area.y + 1;
        let item_height = 1;

        let total_area = Rect::new(base_x, base_y, 10, item_height);
        locust.ctx.targets.register(
            target_builder.custom(total_area, format!("Total Logs: {}", self.stats.total), TargetAction::Activate, TargetPriority::Low)
        );

        let error_area = Rect::new(base_x + 12, base_y, 10, item_height);
        locust.ctx.targets.register(
            target_builder.custom(error_area, format!("Error Logs: {}", self.stats.errors), TargetAction::Activate, TargetPriority::Low)
        );

        let warn_area = Rect::new(base_x + 24, base_y, 10, item_height);
        locust.ctx.targets.register(
            target_builder.custom(warn_area, format!("Warning Logs: {}", self.stats.warnings), TargetAction::Activate, TargetPriority::Low)
        );

        let info_area = Rect::new(base_x + 36, base_y, 10, item_height);
        locust.ctx.targets.register(
            target_builder.custom(info_area, format!("Info Logs: {}", self.stats.info), TargetAction::Activate, TargetPriority::Low)
        );

        let debug_area = Rect::new(base_x + 48, base_y, 10, item_height);
        locust.ctx.targets.register(
            target_builder.custom(debug_area, format!("Debug Logs: {}", self.stats.debug), TargetAction::Activate, TargetPriority::Low)
        );

        let bookmark_area = Rect::new(base_x + 60, base_y, 15, item_height);
        locust.ctx.targets.register(
            target_builder.custom(bookmark_area, format!("Bookmarked Logs: {}", self.stats.bookmarked), TargetAction::Activate, TargetPriority::Low)
        );

        let filtered_area = Rect::new(base_x + 77, base_y, 15, item_height);
        locust.ctx.targets.register(
            target_builder.custom(filtered_area, format!("Filtered Logs: {}", self.filtered_indices.len()), TargetAction::Activate, TargetPriority::Low)
        );
    }
} // This closes the impl LogViewer block

/// Format Unix timestamp to readable datetime
fn format_timestamp(secs: u64) -> String {
    // Simple formatting - in production, use chrono or time crate
    let datetime = secs;
    let seconds = datetime % 60;
    let minutes = (datetime / 60) % 60;
    let hours = (datetime / 3600) % 24;

    format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logger
    let log_file_path = PathBuf::from("locust-log-viewer.log");
    CombinedLogger::init(
        vec![
            WriteLogger::new(
                LevelFilter::Debug,
                Config::default(),
                File::create(&log_file_path).unwrap(),
            ),
        ]
    ).unwrap();
    debug!("Logger initialized for Log Viewer.");

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create Locust instance with navigation plugin
    let mut locust = Locust::new(LocustConfig::default());
    locust.register_plugin(NavPlugin::new());
    locust.register_plugin(OmnibarPlugin::with_config(
        OmnibarConfig::new().with_activation_key('O'),
    ));

    // Create log viewer
    let mut viewer = LogViewer::new();
    let mut log_tailer = LogTailer::new(log_file_path, 10); // Display last 10 log lines
    let mut target_builder = TargetBuilder::new();
    let mut last_tick = SystemTime::now();
    let tick_rate = Duration::from_millis(1000); // 1 second for tail mode

    // Main event loop
    loop {
        locust.begin_frame();
        log_tailer.read_tail()?; // Update log tail at the beginning of each frame

        // Draw UI
        terminal.draw(|f| {
            viewer.draw(f, &mut locust, &mut log_tailer, &mut target_builder);
        })?;

        // Handle events
        let timeout = tick_rate
            .checked_sub(last_tick.elapsed().unwrap_or(Duration::ZERO))
            .unwrap_or(Duration::ZERO);

        if event::poll(timeout)? {
            let ev = event::read()?;
            let outcome = locust.on_event(&ev);

            // Handle events not consumed by Locust
            if !outcome.consumed {
                if let Event::Key(key) = ev {
                    viewer.handle_key(key.code, key.modifiers);
                }
            }
        }

        // Simulate new logs in tail mode
        if last_tick.elapsed().unwrap_or(Duration::ZERO) >= tick_rate {
            if viewer.tail_mode {
                viewer.append_new_log();
            }
            last_tick = SystemTime::now();
        }

        if viewer.should_quit {
            break;
        }
    }

    // Cleanup terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}
