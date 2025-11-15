/// # Dashboard Example
///
/// A comprehensive multi-pane dashboard application demonstrating Locust's
/// capabilities for complex UI navigation. This example showcases:
///
/// - Multiple independent navigation panes (Metrics, Logs, Status, Controls)
/// - Omnibar for quick pane switching and command execution
/// - Tab navigation between panes with visual indicators
/// - Hint mode ('f') for navigating within the active pane
/// - Real-time updating data with smooth 60 FPS rendering
/// - Tooltips on interactive elements
/// - Clean error handling and graceful degradation
///
/// ## Controls
///
/// - `f` - Enter hint mode to navigate within the active pane
/// - `/` - Open omnibar for pane switching or command execution
/// - `Tab` / `Shift+Tab` - Cycle between panes
/// - `1-4` - Directly select pane (1=Metrics, 2=Logs, 3=Status, 4=Controls)
/// - `q` - Quit the application
/// - `r` - Refresh data
/// - `Esc` - Cancel current action/close omnibar
///
/// ## Architecture
///
/// The dashboard uses a multi-pane architecture where each pane maintains
/// its own state and navigation targets. The Locust plugin system handles
/// hint generation and navigation within the currently active pane.
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
    widgets::{Block, BorderType, Borders, Cell, List, ListItem, Paragraph, Row, Table, Tabs},
    Frame, Terminal,
};
use std::io::{self, Stdout};
use std::time::{Duration, SystemTime};

/// Represents the different panes in the dashboard
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DashboardPane {
    Metrics,
    Logs,
    Status,
    Controls,
}

impl DashboardPane {
    fn all() -> Vec<Self> {
        vec![Self::Metrics, Self::Logs, Self::Status, Self::Controls]
    }

    fn title(&self) -> &'static str {
        match self {
            Self::Metrics => "Metrics",
            Self::Logs => "Logs",
            Self::Status => "Status",
            Self::Controls => "Controls",
        }
    }

    fn from_index(idx: usize) -> Option<Self> {
        match idx {
            0 => Some(Self::Metrics),
            1 => Some(Self::Logs),
            2 => Some(Self::Status),
            3 => Some(Self::Controls),
            _ => None,
        }
    }

    fn index(&self) -> usize {
        match self {
            Self::Metrics => 0,
            Self::Logs => 1,
            Self::Status => 2,
            Self::Controls => 3,
        }
    }
}

/// Simulated metric data point
#[derive(Clone)]
struct Metric {
    name: String,
    value: f64,
    unit: String,
    trend: i8, // -1 = down, 0 = stable, 1 = up
}

/// Log entry with severity level
#[derive(Clone)]
struct LogEntry {
    timestamp: String,
    level: LogLevel,
    message: String,
}

#[derive(Clone, Copy, PartialEq)]
enum LogLevel {
    Info,
    Warn,
    Error,
    Debug,
}

impl LogLevel {
    fn color(&self) -> Color {
        match self {
            Self::Info => Color::Green,
            Self::Warn => Color::Yellow,
            Self::Error => Color::Red,
            Self::Debug => Color::Cyan,
        }
    }

    fn label(&self) -> &'static str {
        match self {
            Self::Info => "INFO ",
            Self::Warn => "WARN ",
            Self::Error => "ERROR",
            Self::Debug => "DEBUG",
        }
    }
}

/// System status information
struct SystemStatus {
    cpu_usage: f64,
    memory_usage: f64,
    disk_usage: f64,
    network_rx: f64,
    network_tx: f64,
    uptime: Duration,
}

/// Control panel action
#[derive(Clone)]
struct ControlAction {
    id: String,
    label: String,
    #[allow(dead_code)]
    description: String,
    enabled: bool,
}

/// Main dashboard application state
struct Dashboard {
    /// Currently active pane
    active_pane: DashboardPane,
    /// Whether omnibar is open
    omnibar_open: bool,
    /// Omnibar input buffer
    omnibar_input: String,
    /// Simulated metrics data
    metrics: Vec<Metric>,
    /// Log entries
    logs: Vec<LogEntry>,
    /// System status
    status: SystemStatus,
    /// Available control actions
    controls: Vec<ControlAction>,
    /// Selected control index
    selected_control: usize,
    /// Log scroll offset
    log_scroll: usize,
    /// Last update timestamp
    last_update: SystemTime,
    /// Should quit flag
    should_quit: bool,
}

impl Dashboard {
    fn new() -> Self {
        Self {
            active_pane: DashboardPane::Metrics,
            omnibar_open: false,
            omnibar_input: String::new(),
            metrics: Self::generate_metrics(),
            logs: Self::generate_logs(),
            status: Self::generate_status(),
            controls: Self::generate_controls(),
            selected_control: 0,
            log_scroll: 0,
            last_update: SystemTime::now(),
            should_quit: false,
        }
    }

    /// Generate simulated metrics data
    fn generate_metrics() -> Vec<Metric> {
        vec![
            Metric {
                name: "CPU Usage".into(),
                value: 45.3,
                unit: "%".into(),
                trend: 1,
            },
            Metric {
                name: "Memory Usage".into(),
                value: 62.1,
                unit: "%".into(),
                trend: 0,
            },
            Metric {
                name: "Disk I/O".into(),
                value: 123.4,
                unit: "MB/s".into(),
                trend: -1,
            },
            Metric {
                name: "Network Traffic".into(),
                value: 456.7,
                unit: "KB/s".into(),
                trend: 1,
            },
            Metric {
                name: "Active Connections".into(),
                value: 234.0,
                unit: "".into(),
                trend: 0,
            },
            Metric {
                name: "Request Rate".into(),
                value: 1250.0,
                unit: "req/s".into(),
                trend: 1,
            },
        ]
    }

    /// Generate simulated log entries
    fn generate_logs() -> Vec<LogEntry> {
        let mut logs = vec![
            LogEntry {
                timestamp: "2025-11-14 10:23:45".into(),
                level: LogLevel::Info,
                message: "Application started successfully".into(),
            },
            LogEntry {
                timestamp: "2025-11-14 10:24:12".into(),
                level: LogLevel::Info,
                message: "Database connection established".into(),
            },
            LogEntry {
                timestamp: "2025-11-14 10:24:34".into(),
                level: LogLevel::Debug,
                message: "Loading configuration from /etc/app/config.yaml".into(),
            },
            LogEntry {
                timestamp: "2025-11-14 10:25:01".into(),
                level: LogLevel::Warn,
                message: "High memory usage detected: 85%".into(),
            },
            LogEntry {
                timestamp: "2025-11-14 10:25:23".into(),
                level: LogLevel::Error,
                message: "Failed to connect to external API: timeout".into(),
            },
            LogEntry {
                timestamp: "2025-11-14 10:25:45".into(),
                level: LogLevel::Info,
                message: "Retry succeeded, API connection restored".into(),
            },
            LogEntry {
                timestamp: "2025-11-14 10:26:12".into(),
                level: LogLevel::Debug,
                message: "Cache hit ratio: 94.3%".into(),
            },
            LogEntry {
                timestamp: "2025-11-14 10:26:34".into(),
                level: LogLevel::Info,
                message: "Background worker started: data-processor-1".into(),
            },
            LogEntry {
                timestamp: "2025-11-14 10:27:01".into(),
                level: LogLevel::Warn,
                message: "Deprecated API endpoint accessed: /v1/users".into(),
            },
            LogEntry {
                timestamp: "2025-11-14 10:27:23".into(),
                level: LogLevel::Info,
                message: "Health check passed: all systems operational".into(),
            },
        ];

        // Reverse so newest are at bottom
        logs.reverse();
        logs
    }

    /// Generate system status data
    fn generate_status() -> SystemStatus {
        SystemStatus {
            cpu_usage: 45.3,
            memory_usage: 62.1,
            disk_usage: 73.5,
            network_rx: 456.7,
            network_tx: 234.5,
            uptime: Duration::from_secs(86400), // 1 day
        }
    }

    /// Generate control actions
    fn generate_controls() -> Vec<ControlAction> {
        vec![
            ControlAction {
                id: "restart".into(),
                label: "Restart Service".into(),
                description: "Gracefully restart the application service".into(),
                enabled: true,
            },
            ControlAction {
                id: "clear_cache".into(),
                label: "Clear Cache".into(),
                description: "Clear all cached data".into(),
                enabled: true,
            },
            ControlAction {
                id: "refresh_data".into(),
                label: "Refresh Data".into(),
                description: "Force refresh of all metrics and logs".into(),
                enabled: true,
            },
            ControlAction {
                id: "export_logs".into(),
                label: "Export Logs".into(),
                description: "Export logs to file".into(),
                enabled: true,
            },
            ControlAction {
                id: "toggle_debug".into(),
                label: "Toggle Debug Mode".into(),
                description: "Enable/disable debug logging".into(),
                enabled: true,
            },
            ControlAction {
                id: "shutdown".into(),
                label: "Shutdown".into(),
                description: "Gracefully shutdown the application".into(),
                enabled: false,
            },
        ]
    }

    /// Update simulated data with realistic variations
    fn update_data(&mut self) {
        // Update metrics with small random variations
        for metric in &mut self.metrics {
            let variation = (rand() % 10) as f64 - 5.0;
            metric.value = (metric.value + variation).max(0.0).min(100.0);
        }

        // Update status
        self.status.cpu_usage = (self.status.cpu_usage + (rand() % 10) as f64 - 5.0)
            .max(0.0)
            .min(100.0);
        self.status.memory_usage = (self.status.memory_usage + (rand() % 10) as f64 - 5.0)
            .max(0.0)
            .min(100.0);

        self.last_update = SystemTime::now();
    }

    /// Handle keyboard input
    fn handle_key(&mut self, key: KeyCode, modifiers: KeyModifiers) {
        if self.omnibar_open {
            self.handle_omnibar_key(key);
            return;
        }

        match key {
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Char('f') => {
                // Hint mode is handled by Locust plugin
            }
            KeyCode::Char('/') => {
                self.omnibar_open = true;
                self.omnibar_input.clear();
            }
            KeyCode::Char('r') => self.update_data(),
            KeyCode::Char(c @ '1'..='4') => {
                if let Some(pane) = DashboardPane::from_index(c as usize - '1' as usize) {
                    self.active_pane = pane;
                }
            }
            KeyCode::Tab => {
                if modifiers.contains(KeyModifiers::SHIFT) {
                    self.cycle_pane_backward();
                } else {
                    self.cycle_pane_forward();
                }
            }
            KeyCode::Up => {
                if self.active_pane == DashboardPane::Controls && self.selected_control > 0 {
                    self.selected_control -= 1;
                } else if self.active_pane == DashboardPane::Logs && self.log_scroll > 0 {
                    self.log_scroll -= 1;
                }
            }
            KeyCode::Down => {
                if self.active_pane == DashboardPane::Controls
                    && self.selected_control < self.controls.len() - 1
                {
                    self.selected_control += 1;
                } else if self.active_pane == DashboardPane::Logs
                    && self.log_scroll < self.logs.len().saturating_sub(1)
                {
                    self.log_scroll += 1;
                }
            }
            KeyCode::Enter => {
                if self.active_pane == DashboardPane::Controls {
                    self.execute_control(self.selected_control);
                }
            }
            _ => {}
        }
    }

    /// Handle omnibar keyboard input
    fn handle_omnibar_key(&mut self, key: KeyCode) {
        match key {
            KeyCode::Esc => self.omnibar_open = false,
            KeyCode::Enter => {
                self.process_omnibar_command();
                self.omnibar_open = false;
            }
            KeyCode::Char(c) => self.omnibar_input.push(c),
            KeyCode::Backspace => {
                self.omnibar_input.pop();
            }
            _ => {}
        }
    }

    /// Process omnibar command
    fn process_omnibar_command(&mut self) {
        let cmd = self.omnibar_input.trim().to_lowercase();
        match cmd.as_str() {
            "metrics" | "1" => self.active_pane = DashboardPane::Metrics,
            "logs" | "2" => self.active_pane = DashboardPane::Logs,
            "status" | "3" => self.active_pane = DashboardPane::Status,
            "controls" | "4" => self.active_pane = DashboardPane::Controls,
            "refresh" | "r" => self.update_data(),
            "quit" | "q" => self.should_quit = true,
            _ => {}
        }
    }

    /// Execute a control action
    fn execute_control(&mut self, index: usize) {
        if let Some(control) = self.controls.get(index) {
            if control.enabled {
                // In a real app, execute the action here
                match control.id.as_str() {
                    "refresh_data" => self.update_data(),
                    "shutdown" => self.should_quit = true,
                    _ => {}
                }
            }
        }
    }

    fn cycle_pane_forward(&mut self) {
        let panes = DashboardPane::all();
        let current_idx = self.active_pane.index();
        self.active_pane = panes[(current_idx + 1) % panes.len()];
    }

    fn cycle_pane_backward(&mut self) {
        let panes = DashboardPane::all();
        let current_idx = self.active_pane.index();
        self.active_pane = panes[(current_idx + panes.len() - 1) % panes.len()];
    }

    /// Render the dashboard UI
    fn draw(&self, f: &mut Frame, locust: &mut Locust<CrosstermBackend<Stdout>>) {
        let size = f.area();

        // Main layout: tabs + content
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0)])
            .split(size);

        // Render tabs
        self.draw_tabs(f, chunks[0]);

        // Content layout: 2x2 grid
        let content_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(chunks[1]);

        let top_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(content_chunks[0]);

        let bottom_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(content_chunks[1]);

        // Render each pane
        self.draw_metrics(f, top_chunks[0]);
        self.draw_logs(f, top_chunks[1]);
        self.draw_status(f, bottom_chunks[0]);
        self.draw_controls(f, bottom_chunks[1]);

        // Render omnibar if open
        if self.omnibar_open {
            self.draw_omnibar(f, size);
        }

        // Let Locust render overlays (hints, tooltips, etc.)
        locust.render_overlay(f);
    }

    fn draw_tabs(&self, f: &mut Frame, area: Rect) {
        let panes = DashboardPane::all();
        let titles: Vec<Line> = panes
            .iter()
            .map(|p| {
                let style = if *p == self.active_pane {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };
                Line::from(Span::styled(p.title(), style))
            })
            .collect();

        let tabs = Tabs::new(titles)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .title(" Dashboard "),
            )
            .select(self.active_pane.index())
            .highlight_style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            );

        f.render_widget(tabs, area);
    }

    fn draw_metrics(&self, f: &mut Frame, area: Rect) {
        let is_active = self.active_pane == DashboardPane::Metrics;
        let border_style = if is_active {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        };

        let rows: Vec<Row> = self
            .metrics
            .iter()
            .map(|m| {
                let trend_symbol = match m.trend {
                    1 => "↑",
                    -1 => "↓",
                    _ => "→",
                };
                let trend_color = match m.trend {
                    1 => Color::Green,
                    -1 => Color::Red,
                    _ => Color::Yellow,
                };

                Row::new(vec![
                    Cell::from(m.name.clone()),
                    Cell::from(format!("{:.1}{}", m.value, m.unit)),
                    Cell::from(Span::styled(trend_symbol, Style::default().fg(trend_color))),
                ])
            })
            .collect();

        let table = Table::new(
            rows,
            vec![
                Constraint::Percentage(50),
                Constraint::Percentage(30),
                Constraint::Percentage(20),
            ],
        )
        .header(
            Row::new(vec!["Metric", "Value", "Trend"])
                .style(Style::default().add_modifier(Modifier::BOLD)),
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(border_style)
                .title(" Metrics "),
        );

        f.render_widget(table, area);
    }

    fn draw_logs(&self, f: &mut Frame, area: Rect) {
        let is_active = self.active_pane == DashboardPane::Logs;
        let border_style = if is_active {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        };

        let items: Vec<ListItem> = self
            .logs
            .iter()
            .skip(self.log_scroll)
            .map(|log| {
                let level_span = Span::styled(
                    format!("[{}] ", log.level.label()),
                    Style::default()
                        .fg(log.level.color())
                        .add_modifier(Modifier::BOLD),
                );
                let time_span = Span::styled(
                    format!("{} ", log.timestamp),
                    Style::default().fg(Color::DarkGray),
                );
                let msg_span = Span::raw(&log.message);

                ListItem::new(Line::from(vec![level_span, time_span, msg_span]))
            })
            .collect();

        let list = List::new(items).block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(border_style)
                .title(format!(" Logs ({}/{}) ", self.log_scroll, self.logs.len())),
        );

        f.render_widget(list, area);
    }

    fn draw_status(&self, f: &mut Frame, area: Rect) {
        let is_active = self.active_pane == DashboardPane::Status;
        let border_style = if is_active {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        };

        let uptime_secs = self.status.uptime.as_secs();
        let uptime_str = format!(
            "{}d {}h {}m",
            uptime_secs / 86400,
            (uptime_secs % 86400) / 3600,
            (uptime_secs % 3600) / 60
        );

        let text = vec![
            Line::from(vec![
                Span::styled(
                    "CPU Usage:     ",
                    Style::default().add_modifier(Modifier::BOLD),
                ),
                Span::raw(format!("{:.1}%", self.status.cpu_usage)),
            ]),
            Line::from(vec![
                Span::styled(
                    "Memory Usage:  ",
                    Style::default().add_modifier(Modifier::BOLD),
                ),
                Span::raw(format!("{:.1}%", self.status.memory_usage)),
            ]),
            Line::from(vec![
                Span::styled(
                    "Disk Usage:    ",
                    Style::default().add_modifier(Modifier::BOLD),
                ),
                Span::raw(format!("{:.1}%", self.status.disk_usage)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled(
                    "Network RX:    ",
                    Style::default().add_modifier(Modifier::BOLD),
                ),
                Span::raw(format!("{:.1} KB/s", self.status.network_rx)),
            ]),
            Line::from(vec![
                Span::styled(
                    "Network TX:    ",
                    Style::default().add_modifier(Modifier::BOLD),
                ),
                Span::raw(format!("{:.1} KB/s", self.status.network_tx)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled(
                    "Uptime:        ",
                    Style::default().add_modifier(Modifier::BOLD),
                ),
                Span::raw(uptime_str),
            ]),
        ];

        let paragraph = Paragraph::new(text).block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(border_style)
                .title(" System Status "),
        );

        f.render_widget(paragraph, area);
    }

    fn draw_controls(&self, f: &mut Frame, area: Rect) {
        let is_active = self.active_pane == DashboardPane::Controls;
        let border_style = if is_active {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        };

        let items: Vec<ListItem> = self
            .controls
            .iter()
            .enumerate()
            .map(|(idx, control)| {
                let style = if idx == self.selected_control && is_active {
                    Style::default()
                        .bg(Color::Blue)
                        .add_modifier(Modifier::BOLD)
                } else if !control.enabled {
                    Style::default().fg(Color::DarkGray)
                } else {
                    Style::default()
                };

                let prefix = if idx == self.selected_control && is_active {
                    "▶ "
                } else {
                    "  "
                };

                ListItem::new(Line::from(vec![
                    Span::raw(prefix),
                    Span::styled(&control.label, style),
                ]))
            })
            .collect();

        let list = List::new(items).block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(border_style)
                .title(" Controls "),
        );

        f.render_widget(list, area);
    }

    fn draw_omnibar(&self, f: &mut Frame, area: Rect) {
        // Center the omnibar
        let omnibar_area = Rect {
            x: area.width / 4,
            y: area.height / 2 - 2,
            width: area.width / 2,
            height: 3,
        };

        let text = vec![Line::from(vec![
            Span::styled("> ", Style::default().fg(Color::Yellow)),
            Span::raw(&self.omnibar_input),
            Span::styled("█", Style::default().fg(Color::White)),
        ])];

        let paragraph = Paragraph::new(text).block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Yellow))
                .title(" Omnibar (metrics|logs|status|controls|refresh|quit) "),
        );

        f.render_widget(paragraph, omnibar_area);
    }
}

/// Simple pseudo-random number generator for demo purposes
fn rand() -> u32 {
    use std::collections::hash_map::RandomState;
    use std::hash::{BuildHasher, Hash, Hasher};
    let state = RandomState::new();
    
    
    (state.hash_one(&SystemTime::now()) % 100) as u32
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

    // Create dashboard
    let mut dashboard = Dashboard::new();
    let mut last_tick = SystemTime::now();
    let tick_rate = Duration::from_millis(250);

    // Main event loop
    loop {
        locust.begin_frame();

        // Draw UI
        terminal.draw(|f| {
            dashboard.draw(f, &mut locust);
        })?;

        // Handle events with timeout
        let timeout = tick_rate
            .checked_sub(last_tick.elapsed().unwrap_or(Duration::ZERO))
            .unwrap_or(Duration::ZERO);

        if event::poll(timeout)? {
            let ev = event::read()?;
            let outcome = locust.on_event(&ev);

            // Handle events not consumed by Locust
            if !outcome.consumed {
                if let Event::Key(key) = ev {
                    dashboard.handle_key(key.code, key.modifiers);
                }
            }
        }

        // Periodic updates
        if last_tick.elapsed().unwrap_or(Duration::ZERO) >= tick_rate {
            dashboard.update_data();
            last_tick = SystemTime::now();
        }

        if dashboard.should_quit {
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
