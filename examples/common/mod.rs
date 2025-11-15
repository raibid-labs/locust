/// Common utilities for Locust examples
///
/// Provides shared helpers for layout, timing, mock data, and event handling.

use chrono::{DateTime, Local, TimeZone};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use std::collections::VecDeque;
use std::io;
use std::time::Instant;

/// Create a centered rectangle within the given area
///
/// # Arguments
/// * `percent_x` - Percentage of width (0-100)
/// * `percent_y` - Percentage of height (0-100)
/// * `r` - Parent rectangle
///
/// # Returns
/// Centered rectangle with specified dimensions
pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

/// FPS counter for performance monitoring
pub struct FpsCounter {
    frame_times: VecDeque<Instant>,
    max_samples: usize,
}

impl FpsCounter {
    /// Create a new FPS counter
    pub fn new() -> Self {
        Self {
            frame_times: VecDeque::new(),
            max_samples: 60,
        }
    }

    /// Record a frame
    pub fn tick(&mut self) {
        let now = Instant::now();
        self.frame_times.push_back(now);

        while self.frame_times.len() > self.max_samples {
            self.frame_times.pop_front();
        }
    }

    /// Get current FPS
    pub fn fps(&self) -> f32 {
        if self.frame_times.len() < 2 {
            return 0.0;
        }

        let elapsed = self.frame_times.back().unwrap().duration_since(*self.frame_times.front().unwrap());
        let secs = elapsed.as_secs_f32();

        if secs > 0.0 {
            (self.frame_times.len() - 1) as f32 / secs
        } else {
            0.0
        }
    }

    /// Get average frame time in milliseconds
    pub fn avg_frame_time_ms(&self) -> f32 {
        let fps = self.fps();
        if fps > 0.0 {
            1000.0 / fps
        } else {
            0.0
        }
    }
}

impl Default for FpsCounter {
    fn default() -> Self {
        Self::new()
    }
}

/// Mock data generators for examples
pub mod mock {
    use super::*;
    use rand::Rng;

    /// Log entry for log viewer examples
    #[derive(Debug, Clone)]
    pub struct LogEntry {
        pub timestamp: DateTime<Local>,
        pub level: LogLevel,
        pub message: String,
        pub source: String,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum LogLevel {
        Trace,
        Debug,
        Info,
        Warn,
        Error,
    }

    impl LogLevel {
        pub fn as_str(&self) -> &'static str {
            match self {
                LogLevel::Trace => "TRACE",
                LogLevel::Debug => "DEBUG",
                LogLevel::Info => "INFO",
                LogLevel::Warn => "WARN",
                LogLevel::Error => "ERROR",
            }
        }
    }

    /// Generate mock log entries
    pub fn generate_logs(count: usize) -> Vec<LogEntry> {
        let mut rng = rand::rng();
        let messages = [
            "Server started successfully",
            "Connection established",
            "Request processed",
            "Cache miss, fetching from database",
            "High memory usage detected",
            "Retrying failed operation",
            "Authentication successful",
            "Rate limit exceeded",
            "Background job completed",
            "Configuration reloaded",
        ];
        let sources = ["server", "database", "cache", "auth", "worker"];

        (0..count)
            .map(|i| {
                let level = match rng.random_range(0..10) {
                    0..=5 => LogLevel::Info,
                    6..=7 => LogLevel::Debug,
                    8 => LogLevel::Warn,
                    _ => LogLevel::Error,
                };

                LogEntry {
                    timestamp: Local
                        .timestamp_opt(1705228800 + (i * 60) as i64, 0)
                        .single()
                        .unwrap(),
                    level,
                    message: messages[rng.random_range(0..messages.len())].to_string(),
                    source: sources[rng.random_range(0..sources.len())].to_string(),
                }
            })
            .collect()
    }

    /// Git commit for git browser example
    #[derive(Debug, Clone)]
    pub struct Commit {
        pub hash: String,
        pub author: String,
        pub date: DateTime<Local>,
        pub message: String,
        pub files_changed: Vec<String>,
    }

    /// Generate mock git commits
    pub fn generate_commits(count: usize) -> Vec<Commit> {
        let mut rng = rand::rng();
        let authors = ["Alice", "Bob", "Charlie", "Diana"];
        let messages = [
            "Fix critical bug in authentication",
            "Add new feature for file upload",
            "Refactor database layer",
            "Update dependencies",
            "Improve error handling",
            "Add unit tests",
            "Optimize query performance",
            "Fix memory leak",
            "Update documentation",
            "Initial commit",
        ];
        let files = [
            "src/main.rs",
            "src/lib.rs",
            "src/auth.rs",
            "src/db.rs",
            "tests/integration.rs",
            "Cargo.toml",
            "README.md",
        ];

        (0..count)
            .map(|i| {
                let hash = format!("{:07x}", rng.random_range(0x1000000..0xfffffff));
                let num_files = rng.random_range(1..4);
                let mut changed_files: Vec<String> = (0..num_files)
                    .map(|_| files[rng.random_range(0..files.len())].to_string())
                    .collect();
                changed_files.sort();
                changed_files.dedup();

                Commit {
                    hash,
                    author: authors[rng.random_range(0..authors.len())].to_string(),
                    date: Local
                        .timestamp_opt(1705228800 - (i * 3600) as i64, 0)
                        .single()
                        .unwrap(),
                    message: messages[rng.random_range(0..messages.len())].to_string(),
                    files_changed: changed_files,
                }
            })
            .collect()
    }

    /// Process information for system monitor
    #[derive(Debug, Clone)]
    pub struct Process {
        pub pid: u32,
        pub name: String,
        pub cpu_percent: f32,
        pub mem_bytes: u64,
        pub status: ProcessStatus,
        pub user: String,
        pub command: String,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum ProcessStatus {
        Running,
        Sleeping,
        Stopped,
        Zombie,
    }

    impl ProcessStatus {
        pub fn as_str(&self) -> &'static str {
            match self {
                ProcessStatus::Running => "Running",
                ProcessStatus::Sleeping => "Sleeping",
                ProcessStatus::Stopped => "Stopped",
                ProcessStatus::Zombie => "Zombie",
            }
        }
    }

    /// Generate mock processes
    pub fn generate_processes(count: usize) -> Vec<Process> {
        let mut rng = rand::rng();
        let names = [
            "chrome", "firefox", "vscode", "terminal", "docker", "node", "python", "postgres",
            "redis", "nginx",
        ];
        let users = ["root", "user", "www-data", "postgres"];

        (0..count)
            .map(|i| {
                let name = names[rng.random_range(0..names.len())].to_string();
                let status = match rng.random_range(0..10) {
                    0..=6 => ProcessStatus::Running,
                    7..=8 => ProcessStatus::Sleeping,
                    _ => ProcessStatus::Stopped,
                };

                Process {
                    pid: 1000 + i as u32,
                    name: name.clone(),
                    cpu_percent: rng.random_range(0.0..100.0),
                    mem_bytes: rng.random_range(100_000_000..2_000_000_000),
                    status,
                    user: users[rng.random_range(0..users.len())].to_string(),
                    command: format!("/usr/bin/{} --config /etc/{}.conf", name, name),
                }
            })
            .collect()
    }

    /// Database table for database tool example
    #[derive(Debug, Clone)]
    pub struct Table {
        pub name: String,
        pub columns: Vec<Column>,
        pub row_count: usize,
    }

    #[derive(Debug, Clone)]
    pub struct Column {
        pub name: String,
        pub data_type: String,
        pub nullable: bool,
        pub primary_key: bool,
    }

    /// Generate mock database schema
    pub fn generate_schema() -> Vec<Table> {
        vec![
            Table {
                name: "users".to_string(),
                columns: vec![
                    Column {
                        name: "id".to_string(),
                        data_type: "INTEGER".to_string(),
                        nullable: false,
                        primary_key: true,
                    },
                    Column {
                        name: "name".to_string(),
                        data_type: "TEXT".to_string(),
                        nullable: false,
                        primary_key: false,
                    },
                    Column {
                        name: "email".to_string(),
                        data_type: "TEXT".to_string(),
                        nullable: false,
                        primary_key: false,
                    },
                    Column {
                        name: "created_at".to_string(),
                        data_type: "TIMESTAMP".to_string(),
                        nullable: false,
                        primary_key: false,
                    },
                ],
                row_count: 124,
            },
            Table {
                name: "posts".to_string(),
                columns: vec![
                    Column {
                        name: "id".to_string(),
                        data_type: "INTEGER".to_string(),
                        nullable: false,
                        primary_key: true,
                    },
                    Column {
                        name: "user_id".to_string(),
                        data_type: "INTEGER".to_string(),
                        nullable: false,
                        primary_key: false,
                    },
                    Column {
                        name: "title".to_string(),
                        data_type: "TEXT".to_string(),
                        nullable: false,
                        primary_key: false,
                    },
                    Column {
                        name: "content".to_string(),
                        data_type: "TEXT".to_string(),
                        nullable: true,
                        primary_key: false,
                    },
                ],
                row_count: 456,
            },
            Table {
                name: "comments".to_string(),
                columns: vec![
                    Column {
                        name: "id".to_string(),
                        data_type: "INTEGER".to_string(),
                        nullable: false,
                        primary_key: true,
                    },
                    Column {
                        name: "post_id".to_string(),
                        data_type: "INTEGER".to_string(),
                        nullable: false,
                        primary_key: false,
                    },
                    Column {
                        name: "user_id".to_string(),
                        data_type: "INTEGER".to_string(),
                        nullable: false,
                        primary_key: false,
                    },
                    Column {
                        name: "text".to_string(),
                        data_type: "TEXT".to_string(),
                        nullable: false,
                        primary_key: false,
                    },
                ],
                row_count: 789,
            },
        ]
    }
}
