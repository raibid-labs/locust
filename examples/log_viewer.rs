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
    /// Input mode
    input_mode: InputMode,
    /// Search query
    search_query: String,
    /// Search results (line indices)
    search_results: Vec<usize>,
    /// Current search result index
    search_result_idx: usize,
    /// Input buffer for goto line
    goto_input: String,
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
            input_mode: InputMode::Normal,
            search_query: String::new(),
            search_results: Vec::new(),
            search_result_idx: 0,
            goto_input: String::new(),
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

    /// Perform search in logs
    fn perform_search(&mut self) {
        self.search_results.clear();
        self.search_result_idx = 0;

        if self.search_query.is_empty() {
            return;
        }

        let query = self.search_query.to_lowercase();
        self.search_results = self
            .filtered_indices
            .iter()
            .filter(|&&idx| {
                let log = &self.logs[idx];
                log.message.to_lowercase().contains(&query)
                    || log.source.to_lowercase().contains(&query)
            })
            .copied()
            .collect();
    }

    /// Jump to next search result
    fn next_search_result(&mut self) {
        if self.search_results.is_empty() {
            return;
        }

        self.search_result_idx = (self.search_result_idx + 1) % self.search_results.len();
        let line_idx = self.search_results[self.search_result_idx];

        // Find position in filtered list
        if let Some(pos) = self
            .filtered_indices
            .iter()
            .position(|&idx| idx == line_idx)
        {
            self.scroll_pos = pos;
        }
    }

    /// Jump to previous search result
    fn prev_search_result(&mut self) {
        if self.search_results.is_empty() {
            return;
        }

        self.search_result_idx = if self.search_result_idx == 0 {
            self.search_results.len() - 1
        } else {
            self.search_result_idx - 1
        };

        let line_idx = self.search_results[self.search_result_idx];

        // Find position in filtered list
        if let Some(pos) = self
            .filtered_indices
            .iter()
            .position(|&idx| idx == line_idx)
        {
            self.scroll_pos = pos;
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
        match self.input_mode {
            InputMode::Normal => self.handle_normal_key(key, modifiers),
            InputMode::Search => self.handle_search_key(key),
            InputMode::GotoLine => self.handle_goto_key(key),
        }
    }

    fn handle_normal_key(&mut self, key: KeyCode, modifiers: KeyModifiers) {
        match key {
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Char('/') => {
                self.input_mode = InputMode::Search;
                self.search_query.clear();
            }
            KeyCode::Char('g') => {
                if modifiers.contains(KeyModifiers::SHIFT) {
                    // Shift+G = go to end
                    self.scroll_pos = self.filtered_indices.len().saturating_sub(1);
                } else {
                    self.input_mode = InputMode::GotoLine;
                    self.goto_input.clear();
                }
            }
            KeyCode::Char('m') => self.toggle_bookmark(),
            KeyCode::Char('n') => self.next_search_result(),
            KeyCode::Char('N') => self.prev_search_result(),
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

    fn handle_search_key(&mut self, key: KeyCode) {
        match key {
            KeyCode::Esc => {
                self.input_mode = InputMode::Normal;
                self.search_query.clear();
                self.search_results.clear();
            }
            KeyCode::Enter => {
                self.input_mode = InputMode::Normal;
                self.perform_search();
                if !self.search_results.is_empty() {
                    self.next_search_result();
                }
            }
            KeyCode::Char(c) => {
                self.search_query.push(c);
            }
            KeyCode::Backspace => {
                self.search_query.pop();
            }
            _ => {}
        }
    }

    fn handle_goto_key(&mut self, key: KeyCode) {
        match key {
            KeyCode::Esc => {
                self.input_mode = InputMode::Normal;
                self.goto_input.clear();
            }
            KeyCode::Enter => {
                self.input_mode = InputMode::Normal;
                if let Ok(line_num) = self.goto_input.parse::<usize>() {
                    if line_num > 0 && line_num <= self.logs.len() {
                        // Find position in filtered list
                        if let Some(pos) = self
                            .filtered_indices
                            .iter()
                            .position(|&idx| self.logs[idx].line_number == line_num)
                        {
                            self.scroll_pos = pos;
                        }
                    }
                }
                self.goto_input.clear();
            }
            KeyCode::Char(c @ '0'..='9') => {
                self.goto_input.push(c);
            }
            KeyCode::Backspace => {
                self.goto_input.pop();
            }
            _ => {}
        }
    }

    /// Render the log viewer UI
    fn draw(&self, f: &mut Frame, locust: &mut Locust<CrosstermBackend<Stdout>>) {
        let size = f.area();

        // Main layout: filter bar + content + status bar
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(3),
            ])
            .split(size);

        // Render components
        self.draw_filter_bar(f, chunks[0]);
        self.draw_logs(f, chunks[1]);
        self.draw_status_bar(f, chunks[2]);

        // Render input dialogs
        match self.input_mode {
            InputMode::Search => self.draw_search_dialog(f, size),
            InputMode::GotoLine => self.draw_goto_dialog(f, size),
            InputMode::Normal => {}
        }

        // Let Locust render overlays
        locust.render_overlay(f);
    }

    fn draw_filter_bar(&self, f: &mut Frame, area: Rect) {
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
                if !self.search_query.is_empty() {
                    format!("Search: {} ", self.search_query)
                } else {
                    "No search".to_string()
                },
                Style::default().fg(Color::Cyan),
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

        f.render_widget(paragraph, area);
    }

    fn draw_logs(&self, f: &mut Frame, area: Rect) {
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
    }

    fn draw_status_bar(&self, f: &mut Frame, area: Rect) {
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
                .border_type(BorderType::Rounded),
        );

        f.render_widget(paragraph, area);
    }

    fn draw_search_dialog(&self, f: &mut Frame, area: Rect) {
        let dialog_area = Rect {
            x: area.width / 4,
            y: area.height / 2 - 2,
            width: area.width / 2,
            height: 3,
        };

        let text = vec![Line::from(vec![
            Span::styled("Search: ", Style::default().fg(Color::Yellow)),
            Span::raw(&self.search_query),
            Span::styled("█", Style::default().fg(Color::White)),
        ])];

        let paragraph = Paragraph::new(text).block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Yellow))
                .title(" Search Logs "),
        );

        f.render_widget(paragraph, dialog_area);
    }

    fn draw_goto_dialog(&self, f: &mut Frame, area: Rect) {
        let dialog_area = Rect {
            x: area.width / 4,
            y: area.height / 2 - 2,
            width: area.width / 2,
            height: 3,
        };

        let text = vec![Line::from(vec![
            Span::styled("Go to line: ", Style::default().fg(Color::Yellow)),
            Span::raw(&self.goto_input),
            Span::styled("█", Style::default().fg(Color::White)),
        ])];

        let paragraph = Paragraph::new(text).block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Yellow))
                .title(" Jump to Line "),
        );

        f.render_widget(paragraph, dialog_area);
    }
}

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
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create Locust instance with navigation plugin
    let mut locust = Locust::new(LocustConfig::default());
    locust.register_plugin(NavPlugin::new());

    // Create log viewer
    let mut viewer = LogViewer::new();
    let mut last_tick = SystemTime::now();
    let tick_rate = Duration::from_millis(1000); // 1 second for tail mode

    // Main event loop
    loop {
        locust.begin_frame();

        // Draw UI
        terminal.draw(|f| {
            viewer.draw(f, &mut locust);
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
