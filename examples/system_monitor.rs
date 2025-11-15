/// System Monitor - Comprehensive system monitoring TUI
///
/// Features:
/// - Real-time CPU usage graphs (per-core)
/// - Memory usage tracking
/// - Disk I/O statistics
/// - Network traffic monitoring
/// - Process list with sorting
/// - Kill/nice process operations
/// - Resource alerts and notifications
/// - Command palette for system operations
/// - Tooltips for detailed process info
/// - Guided tour for monitoring features
///
/// Layout:
/// ```
/// ┌─────────────────────────────────────────────────┐
/// │ CPU Usage (4 cores)           Memory: 4.2G/8.0G │
/// │ Core 0: [▇▇▇▇▇▇     ] 65%    Used:   [▇▇▇▇▇ ]   │
/// │ Core 1: [▇▇▇▇▇      ] 55%    Buffers: [▇    ]   │
/// │ Core 2: [▇▇▇▇▇▇▇▇   ] 82%    Cache:  [▇▇   ]   │
/// │ Core 3: [▇▇▇▇       ] 43%                       │
/// ├─────────────────────────────────────────────────┤
/// │ Processes (234 running, 12 sleeping)            │
/// │ PID    Name          CPU%   Memory    Status    │
/// │ 1234   chrome        45.2   1.2G      Running   │
/// │ 5678   firefox       32.1   890M      Running   │
/// │ 9012   vscode        12.4   450M      Running   │
/// └─────────────────────────────────────────────────┘
/// Press 'f' for hints | Ctrl+P for commands | Ctrl+K to kill
/// ```

mod common;

use chrono::Local;
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use locust::{
    HighlightConfig, HighlightPlugin, Locust, NavPlugin, OmnibarPlugin, TooltipPlugin,
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Axis, Block, Borders, Chart, Dataset, GraphType, List, ListItem, ListState, Paragraph,
        Wrap,
    },
    Frame, Terminal,
};
use std::{
    collections::VecDeque,
    io::{self, Stdout},
    time::{Duration, Instant},
};

use common::mock::{generate_processes, Process, ProcessStatus};

/// Main system monitor application
struct SystemMonitor {
    /// CPU usage history (per core)
    cpu_history: VecDeque<Vec<f32>>,
    /// Memory usage history (percentage)
    mem_history: VecDeque<f32>,
    /// Disk I/O statistics
    disk_io: DiskStats,
    /// Network I/O statistics
    network_io: NetworkStats,
    /// Process list
    processes: Vec<Process>,
    /// Process list state
    process_state: ListState,
    /// Process sort key
    sort_by: ProcessSortKey,
    /// Sort ascending
    sort_ascending: bool,
    /// Resource alerts
    alerts: Vec<Alert>,
    /// Locust integration
    locust: Locust<CrosstermBackend<Stdout>>,
    /// Current view mode
    view_mode: ViewMode,
    /// Search/filter query
    filter_query: String,
    /// Number of CPU cores
    num_cores: usize,
    /// Total memory in bytes
    total_memory: u64,
    /// Tour active
    tour_active: bool,
    /// Tour step
    tour_step: usize,
    /// FPS counter
    fps_counter: common::FpsCounter,
    /// Update counter for simulating changes
    update_counter: u64,
}

#[derive(Clone)]
struct DiskStats {
    read_bytes_sec: u64,
    write_bytes_sec: u64,
    read_ops_sec: u64,
    write_ops_sec: u64,
}

#[derive(Clone)]
struct NetworkStats {
    rx_bytes_sec: u64,
    tx_bytes_sec: u64,
    rx_packets_sec: u64,
    tx_packets_sec: u64,
}

#[derive(Clone)]
struct Alert {
    alert_type: AlertType,
    message: String,
    timestamp: chrono::DateTime<Local>,
    threshold: f32,
    current_value: f32,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum AlertType {
    CpuHigh,
    MemoryHigh,
    DiskHigh,
    ProcessCrash,
}

impl AlertType {
    fn color(&self) -> Color {
        match self {
            AlertType::CpuHigh => Color::Yellow,
            AlertType::MemoryHigh => Color::Red,
            AlertType::DiskHigh => Color::Magenta,
            AlertType::ProcessCrash => Color::Red,
        }
    }

    fn icon(&self) -> &'static str {
        match self {
            AlertType::CpuHigh => "⚠️",
            AlertType::MemoryHigh => "🔴",
            AlertType::DiskHigh => "💾",
            AlertType::ProcessCrash => "❌",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ProcessSortKey {
    Pid,
    Name,
    Cpu,
    Memory,
    Status,
}

impl ProcessSortKey {
    fn as_str(&self) -> &'static str {
        match self {
            ProcessSortKey::Pid => "PID",
            ProcessSortKey::Name => "Name",
            ProcessSortKey::Cpu => "CPU%",
            ProcessSortKey::Memory => "Memory",
            ProcessSortKey::Status => "Status",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ViewMode {
    Overview,
    Processes,
    Alerts,
}

impl SystemMonitor {
    fn new(terminal: Terminal<CrosstermBackend<Stdout>>) -> io::Result<Self> {
        let mut locust = Locust::new(terminal);

        // Initialize plugins
        locust.add_plugin(NavPlugin::default());
        locust.add_plugin(OmnibarPlugin::default());
        locust.add_plugin(TooltipPlugin::default());

        let highlight_config = HighlightConfig {
            steps: vec![
                "Welcome to System Monitor! Press 'n' to continue.".to_string(),
                "View real-time CPU and memory graphs in the overview".to_string(),
                "Press Tab to switch to process list view".to_string(),
                "Sort processes by clicking column headers or using Ctrl+P".to_string(),
                "Kill processes with Ctrl+K (select with 'f' for hints)".to_string(),
                "Set alerts with 'set alert cpu 80' in command palette".to_string(),
            ],
            highlight_color: Color::Yellow,
            text_color: Color::White,
        };
        locust.add_plugin(HighlightPlugin::new(highlight_config));

        let num_cores = 4;
        let mut cpu_history = VecDeque::new();
        let mut mem_history = VecDeque::new();

        // Initialize with some history
        for _ in 0..60 {
            cpu_history.push_back(vec![0.0; num_cores]);
            mem_history.push_back(0.0);
        }

        let processes = generate_processes(50);
        let mut process_state = ListState::default();
        process_state.select(Some(0));

        Ok(Self {
            cpu_history,
            mem_history,
            disk_io: DiskStats {
                read_bytes_sec: 1024 * 1024 * 10,  // 10 MB/s
                write_bytes_sec: 1024 * 1024 * 5,  // 5 MB/s
                read_ops_sec: 100,
                write_ops_sec: 50,
            },
            network_io: NetworkStats {
                rx_bytes_sec: 1024 * 1024 * 2,  // 2 MB/s
                tx_bytes_sec: 1024 * 512,       // 512 KB/s
                rx_packets_sec: 1000,
                tx_packets_sec: 500,
            },
            processes,
            process_state,
            sort_by: ProcessSortKey::Cpu,
            sort_ascending: false,
            alerts: Vec::new(),
            locust,
            view_mode: ViewMode::Overview,
            filter_query: String::new(),
            num_cores,
            total_memory: 8 * 1024 * 1024 * 1024, // 8 GB
            tour_active: false,
            tour_step: 0,
            fps_counter: common::FpsCounter::new(),
            update_counter: 0,
        })
    }

    fn run(&mut self) -> io::Result<()> {
        let tick_rate = Duration::from_millis(16); // ~60 FPS
        let update_rate = Duration::from_millis(1000); // Update stats every second
        let mut last_tick = Instant::now();
        let mut last_update = Instant::now();

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
                last_tick = Instant::now();
            }

            if last_update.elapsed() >= update_rate {
                self.update_stats();
                last_update = Instant::now();
            }
        }

        Ok(())
    }

    fn handle_input(&mut self, key: KeyEvent) -> io::Result<bool> {
        // Tour handling
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
                self.locust.omnibar_mut().toggle();
                return Ok(false);
            }
            (KeyModifiers::CONTROL, KeyCode::Char('k')) => {
                self.kill_selected_process();
                return Ok(false);
            }
            (_, KeyCode::Char('t')) => {
                self.tour_active = !self.tour_active;
                if self.tour_active {
                    self.tour_step = 0;
                }
                return Ok(false);
            }
            (_, KeyCode::Tab) => {
                self.view_mode = match self.view_mode {
                    ViewMode::Overview => ViewMode::Processes,
                    ViewMode::Processes => ViewMode::Alerts,
                    ViewMode::Alerts => ViewMode::Overview,
                };
                return Ok(false);
            }
            _ => {}
        }

        // Command palette mode
        if self.locust.omnibar().is_active() {
            return self.handle_omnibar_input(key);
        }

        // View-specific controls
        match self.view_mode {
            ViewMode::Overview => {}
            ViewMode::Processes => self.handle_processes_input(key),
            ViewMode::Alerts => {}
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

    fn handle_processes_input(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                let i = self.process_state.selected().unwrap_or(0);
                if i > 0 {
                    self.process_state.select(Some(i - 1));
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                let i = self.process_state.selected().unwrap_or(0);
                if i < self.processes.len().saturating_sub(1) {
                    self.process_state.select(Some(i + 1));
                }
            }
            KeyCode::Char('c') => {
                self.sort_by = ProcessSortKey::Cpu;
                self.sort_processes();
            }
            KeyCode::Char('m') => {
                self.sort_by = ProcessSortKey::Memory;
                self.sort_processes();
            }
            KeyCode::Char('n') => {
                self.sort_by = ProcessSortKey::Name;
                self.sort_processes();
            }
            _ => {}
        }
    }

    fn execute_command(&mut self, command: &str) {
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.is_empty() {
            return;
        }

        match parts[0].to_lowercase().as_str() {
            "kill" => {
                if parts.len() > 1 {
                    if let Ok(pid) = parts[1].parse::<u32>() {
                        self.kill_process(pid);
                    }
                }
            }
            "nice" => {
                if parts.len() > 2 {
                    if let Ok(pid) = parts[1].parse::<u32>() {
                        if let Ok(_priority) = parts[2].parse::<i32>() {
                            // Mock implementation
                        }
                    }
                }
            }
            "sort" => {
                if parts.len() > 2 && parts[1] == "by" {
                    self.sort_by = match parts[2] {
                        "cpu" => ProcessSortKey::Cpu,
                        "memory" | "mem" => ProcessSortKey::Memory,
                        "name" => ProcessSortKey::Name,
                        "pid" => ProcessSortKey::Pid,
                        "status" => ProcessSortKey::Status,
                        _ => self.sort_by,
                    };
                    self.sort_processes();
                }
            }
            "filter" => {
                if parts.len() > 1 {
                    self.filter_query = parts[1..].join(" ");
                }
            }
            "set" => {
                if parts.len() > 3 && parts[1] == "alert" {
                    let metric = parts[2];
                    if let Ok(threshold) = parts[3].parse::<f32>() {
                        let alert_type = match metric {
                            "cpu" => AlertType::CpuHigh,
                            "memory" | "mem" => AlertType::MemoryHigh,
                            "disk" => AlertType::DiskHigh,
                            _ => return,
                        };

                        self.alerts.push(Alert {
                            alert_type,
                            message: format!("{} threshold set to {}%", metric, threshold),
                            timestamp: Local::now(),
                            threshold,
                            current_value: 0.0,
                        });
                    }
                }
            }
            _ => {}
        }
    }

    fn kill_selected_process(&mut self) {
        if let Some(i) = self.process_state.selected() {
            if let Some(process) = self.processes.get(i) {
                self.kill_process(process.pid);
            }
        }
    }

    fn kill_process(&mut self, pid: u32) {
        self.processes.retain(|p| p.pid != pid);

        self.alerts.push(Alert {
            alert_type: AlertType::ProcessCrash,
            message: format!("Process {} terminated", pid),
            timestamp: Local::now(),
            threshold: 0.0,
            current_value: 0.0,
        });
    }

    fn sort_processes(&mut self) {
        let ascending = self.sort_ascending;
        match self.sort_by {
            ProcessSortKey::Pid => {
                self.processes.sort_by_key(|p| p.pid);
            }
            ProcessSortKey::Name => {
                self.processes.sort_by(|a, b| a.name.cmp(&b.name));
            }
            ProcessSortKey::Cpu => {
                self.processes
                    .sort_by(|a, b| a.cpu_percent.partial_cmp(&b.cpu_percent).unwrap());
            }
            ProcessSortKey::Memory => {
                self.processes.sort_by_key(|p| p.mem_bytes);
            }
            ProcessSortKey::Status => {
                self.processes.sort_by_key(|p| p.status as u8);
            }
        }

        if !ascending {
            self.processes.reverse();
        }
    }

    fn update_stats(&mut self) {
        use rand::Rng;
        let mut rng = rand::rng();

        self.update_counter += 1;

        // Update CPU history
        let cpu_values: Vec<f32> = (0..self.num_cores)
            .map(|i| {
                let base = 40.0 + (i as f32 * 10.0);
                let variation = rng.random_range(-10.0..10.0);
                (base + variation).clamp(0.0, 100.0)
            })
            .collect();

        self.cpu_history.push_back(cpu_values);
        if self.cpu_history.len() > 60 {
            self.cpu_history.pop_front();
        }

        // Update memory history
        let mem_percent = 50.0 + rng.random_range(-5.0..5.0);
        self.mem_history.push_back(mem_percent.clamp(0.0, 100.0));
        if self.mem_history.len() > 60 {
            self.mem_history.pop_front();
        }

        // Update disk I/O
        self.disk_io.read_bytes_sec =
            (1024 * 1024 * 10) + rng.random_range(-1024 * 1024..1024 * 1024 * 5);
        self.disk_io.write_bytes_sec =
            (1024 * 1024 * 5) + rng.random_range(-1024 * 1024..1024 * 1024 * 2);

        // Update network I/O
        self.network_io.rx_bytes_sec =
            (1024 * 1024 * 2) + rng.random_range(-1024 * 512..1024 * 1024);
        self.network_io.tx_bytes_sec =
            (1024 * 512) + rng.random_range(-1024 * 256..1024 * 512);

        // Randomly update process stats
        for process in &mut self.processes {
            process.cpu_percent = (process.cpu_percent + rng.random_range(-5.0..5.0)).clamp(0.0, 100.0);
            let mem_change = rng.random_range(-10_000_000..10_000_000);
            process.mem_bytes = (process.mem_bytes as i64 + mem_change).max(10_000_000) as u64;
        }

        // Check alerts
        let avg_cpu: f32 = self.cpu_history.back().map(|v| v.iter().sum::<f32>() / v.len() as f32).unwrap_or(0.0);
        for alert in &mut self.alerts {
            match alert.alert_type {
                AlertType::CpuHigh => {
                    alert.current_value = avg_cpu;
                    if avg_cpu > alert.threshold {
                        alert.message = format!("CPU usage {:.1}% exceeds threshold {:.1}%", avg_cpu, alert.threshold);
                    }
                }
                AlertType::MemoryHigh => {
                    let mem_percent = self.mem_history.back().copied().unwrap_or(0.0);
                    alert.current_value = mem_percent;
                    if mem_percent > alert.threshold {
                        alert.message = format!("Memory usage {:.1}% exceeds threshold {:.1}%", mem_percent, alert.threshold);
                    }
                }
                _ => {}
            }
        }
    }

    fn draw(&mut self) -> io::Result<()> {
        self.locust.terminal_mut().draw(|f| {
            let area = f.area();

            match self.view_mode {
                ViewMode::Overview => self.draw_overview(f, area),
                ViewMode::Processes => self.draw_processes(f, area),
                ViewMode::Alerts => self.draw_alerts(f, area),
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

    fn draw_overview(&self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(40),
                Constraint::Percentage(30),
                Constraint::Percentage(30),
            ])
            .split(area);

        // CPU and Memory graphs
        let top_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
            .split(chunks[0]);

        self.draw_cpu_graph(f, top_chunks[0]);
        self.draw_memory_info(f, top_chunks[1]);

        // Disk and Network I/O
        let middle_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(chunks[1]);

        self.draw_disk_io(f, middle_chunks[0]);
        self.draw_network_io(f, middle_chunks[1]);

        // Recent alerts
        self.draw_recent_alerts(f, chunks[2]);
    }

    fn draw_cpu_graph(&self, f: &mut Frame, area: Rect) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title(format!(" CPU Usage ({} cores) ", self.num_cores));

        let inner = block.inner(area);
        f.render_widget(block, area);

        if inner.height < 3 {
            return;
        }

        // Create datasets for each core
        let colors = [Color::Cyan, Color::Green, Color::Yellow, Color::Magenta];
        let datasets: Vec<Dataset> = (0..self.num_cores)
            .map(|core_idx| {
                let data: Vec<(f64, f64)> = self
                    .cpu_history
                    .iter()
                    .enumerate()
                    .map(|(i, cores)| (i as f64, cores[core_idx] as f64))
                    .collect();

                Dataset::default()
                    .name(format!("Core {}", core_idx))
                    .marker(ratatui::symbols::Marker::Braille)
                    .graph_type(GraphType::Line)
                    .style(Style::default().fg(colors[core_idx % colors.len()]))
                    .data(&data)
            })
            .collect();

        let chart = Chart::new(datasets)
            .x_axis(
                Axis::default()
                    .bounds([0.0, 60.0])
                    .labels(vec!["0s".into(), "30s".into(), "60s".into()]),
            )
            .y_axis(
                Axis::default()
                    .bounds([0.0, 100.0])
                    .labels(vec!["0%".into(), "50%".into(), "100%".into()]),
            );

        f.render_widget(chart, inner);
    }

    fn draw_memory_info(&self, f: &mut Frame, area: Rect) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title(" Memory ");

        let inner = block.inner(area);
        f.render_widget(block, area);

        let mem_percent = self.mem_history.back().copied().unwrap_or(0.0);
        let used_mem = (self.total_memory as f64 * mem_percent as f64 / 100.0) as u64;

        let bar_width = 20;
        let filled = ((mem_percent / 100.0) * bar_width as f32) as usize;
        let bar = format!(
            "[{}{}]",
            "▇".repeat(filled),
            " ".repeat(bar_width - filled)
        );

        let lines = vec![
            Line::from(""),
            Line::from(vec![
                Span::raw("Total:  "),
                Span::styled(
                    format!("{:.1} GB", self.total_memory as f64 / 1024.0 / 1024.0 / 1024.0),
                    Style::default().fg(Color::White),
                ),
            ]),
            Line::from(vec![
                Span::raw("Used:   "),
                Span::styled(
                    format!("{:.1} GB", used_mem as f64 / 1024.0 / 1024.0 / 1024.0),
                    Style::default().fg(Color::Yellow),
                ),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled(bar, Style::default().fg(Color::Cyan)),
                Span::raw(format!(" {:.1}%", mem_percent)),
            ]),
        ];

        let paragraph = Paragraph::new(lines).alignment(Alignment::Left);
        f.render_widget(paragraph, inner);
    }

    fn draw_disk_io(&self, f: &mut Frame, area: Rect) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title(" Disk I/O ");

        let inner = block.inner(area);
        f.render_widget(block, area);

        let lines = vec![
            Line::from(""),
            Line::from(vec![
                Span::raw("Read:  "),
                Span::styled(
                    format!("{:.1} MB/s", self.disk_io.read_bytes_sec as f64 / 1024.0 / 1024.0),
                    Style::default().fg(Color::Green),
                ),
            ]),
            Line::from(vec![
                Span::raw("Write: "),
                Span::styled(
                    format!("{:.1} MB/s", self.disk_io.write_bytes_sec as f64 / 1024.0 / 1024.0),
                    Style::default().fg(Color::Red),
                ),
            ]),
            Line::from(""),
            Line::from(format!("Ops/s: {} read, {} write", self.disk_io.read_ops_sec, self.disk_io.write_ops_sec)),
        ];

        let paragraph = Paragraph::new(lines);
        f.render_widget(paragraph, inner);
    }

    fn draw_network_io(&self, f: &mut Frame, area: Rect) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title(" Network I/O ");

        let inner = block.inner(area);
        f.render_widget(block, area);

        let lines = vec![
            Line::from(""),
            Line::from(vec![
                Span::raw("RX:    "),
                Span::styled(
                    format!("{:.1} MB/s", self.network_io.rx_bytes_sec as f64 / 1024.0 / 1024.0),
                    Style::default().fg(Color::Green),
                ),
            ]),
            Line::from(vec![
                Span::raw("TX:    "),
                Span::styled(
                    format!("{:.1} KB/s", self.network_io.tx_bytes_sec as f64 / 1024.0),
                    Style::default().fg(Color::Red),
                ),
            ]),
            Line::from(""),
            Line::from(format!("Packets: {} RX, {} TX", self.network_io.rx_packets_sec, self.network_io.tx_packets_sec)),
        ];

        let paragraph = Paragraph::new(lines);
        f.render_widget(paragraph, inner);
    }

    fn draw_recent_alerts(&self, f: &mut Frame, area: Rect) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title(" Recent Alerts ");

        let items: Vec<ListItem> = self
            .alerts
            .iter()
            .rev()
            .take(5)
            .map(|alert| {
                let line = Line::from(vec![
                    Span::raw(alert.alert_type.icon()),
                    Span::raw(" "),
                    Span::styled(
                        format!("[{}] ", alert.timestamp.format("%H:%M:%S")),
                        Style::default().fg(Color::DarkGray),
                    ),
                    Span::styled(&alert.message, Style::default().fg(alert.alert_type.color())),
                ]);
                ListItem::new(line)
            })
            .collect();

        let list = List::new(items).block(block);
        f.render_widget(list, area);
    }

    fn draw_processes(&mut self, f: &mut Frame, area: Rect) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title(format!(
                " Processes ({} total) - Sort by: {} ",
                self.processes.len(),
                self.sort_by.as_str()
            ));

        let running = self.processes.iter().filter(|p| p.status == ProcessStatus::Running).count();
        let sleeping = self.processes.iter().filter(|p| p.status == ProcessStatus::Sleeping).count();

        let header = format!("Running: {} | Sleeping: {} | ", running, sleeping);

        let items: Vec<ListItem> = self
            .processes
            .iter()
            .map(|process| {
                let mem_mb = process.mem_bytes as f64 / 1024.0 / 1024.0;
                let line = format!(
                    "{:6} {:<15} {:>5.1}% {:>7.1}M  {}",
                    process.pid,
                    if process.name.len() > 15 {
                        &process.name[..15]
                    } else {
                        &process.name
                    },
                    process.cpu_percent,
                    mem_mb,
                    process.status.as_str()
                );
                ListItem::new(line)
            })
            .collect();

        let list = List::new(items)
            .block(block)
            .highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(">> ");

        f.render_stateful_widget(list, area, &mut self.process_state);
    }

    fn draw_alerts(&self, f: &mut Frame, area: Rect) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title(format!(" Alerts ({}) ", self.alerts.len()));

        let items: Vec<ListItem> = self
            .alerts
            .iter()
            .rev()
            .map(|alert| {
                let lines = vec![
                    Line::from(vec![
                        Span::raw(alert.alert_type.icon()),
                        Span::raw(" "),
                        Span::styled(
                            format!("[{}]", alert.timestamp.format("%Y-%m-%d %H:%M:%S")),
                            Style::default().fg(Color::DarkGray),
                        ),
                    ]),
                    Line::from(Span::styled(
                        &alert.message,
                        Style::default().fg(alert.alert_type.color()),
                    )),
                    Line::from(""),
                ];
                ListItem::new(lines)
            })
            .collect();

        let list = List::new(items).block(block);
        f.render_widget(list, area);
    }

    fn draw_status_bar(&self, f: &mut Frame, area: Rect) {
        let status_area = Rect {
            x: area.x,
            y: area.y + area.height - 1,
            width: area.width,
            height: 1,
        };

        let fps = format!("{:.1} FPS", self.fps_counter.fps());
        let view = match self.view_mode {
            ViewMode::Overview => "Overview",
            ViewMode::Processes => "Processes",
            ViewMode::Alerts => "Alerts",
        };

        let status = format!("View: {} | Alerts: {} | {}", view, self.alerts.len(), fps);
        let help = "Tab: switch view | Ctrl+K: kill | Ctrl+P: commands | 't': tour | Ctrl+C: quit";

        let status_text = format!("{} | {}", status, help);
        let status_widget = Paragraph::new(status_text)
            .style(Style::default().bg(Color::DarkGray).fg(Color::White));

        f.render_widget(status_widget, status_area);
    }

    fn draw_tour(&self, f: &mut Frame, area: Rect) {
        let messages = [
            "Welcome to System Monitor! Press 'n' to continue.",
            "View real-time CPU and memory graphs in the overview",
            "Press Tab to switch to process list view",
            "Sort processes by CPU (c), Memory (m), or Name (n)",
            "Kill processes with Ctrl+K (select with arrow keys)",
            "Set alerts with 'set alert cpu 80'. Press 'q' to exit tour.",
        ];

        let popup_area = common::centered_rect(60, 30, area);
        let message = messages.get(self.tour_step).unwrap_or(&messages[0]);

        let tour_widget = Paragraph::new(*message)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" System Monitor Tutorial ")
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

    let mut app = SystemMonitor::new(terminal)?;
    let result = app.run();

    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen)?;

    result
}
