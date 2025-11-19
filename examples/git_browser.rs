/// Git Repository Browser - TUI for browsing git repositories
///
/// Features:
/// - Commit history view with full details
/// - File tree at specific commits
/// - Interactive diff viewer
/// - Branch and tag navigation
/// - Commit search functionality
/// - Command palette for git operations
/// - Tooltips for commit metadata
/// - Guided tour for git workflow
///
/// Layout:
/// ```
/// ┌─────────────────────────────────────────────────┐
/// │ Commits               │ Files      │ Diff       │
/// ├───────────────────────┼────────────┼────────────┤
/// │ [a1b2c3d] Fix bug     │ src/       │ + added    │
/// │ [e4f5g6h] Add feature │   main.rs  │ - removed  │
/// │ [i7j8k9l] Refactor    │   lib.rs   │ ~ modified │
/// │ [m0n1o2p] Initial     │ tests/     │            │
/// │                       │   test.rs  │            │
/// └───────────────────────┴────────────┴────────────┘
/// Branch: main | Commits: 4 | Press 'f' for hints
/// ```
mod common;

use chrono::{DateTime, Local};
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use locust::prelude::*;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
    Frame, Terminal,
};
use std::{
    collections::VecDeque,
    io::{self, Stdout},
    path::{PathBuf, Path},
    time::{Duration, Instant},
    fs::File,
};

use log::{debug, error, info, LevelFilter};
use simplelog::{CombinedLogger, Config, WriteLogger};
use locust::ratatui_ext::LogTailer;

use common::mock::{generate_commits, Commit};

/// Main git browser application
struct GitBrowser {
    /// Path to repository
    repo_path: PathBuf,
    /// List of commits
    commits: Vec<Commit>,
    /// Selected commit index
    commit_state: ListState,
    /// File tree for selected commit
    file_tree: FileTree,
    /// Selected file index
    file_state: ListState,
    /// Current diff view
    diff_view: Option<DiffView>,
    /// Available branches
    branches: Vec<String>,
    /// Current branch
    current_branch: String,
    /// Tags in repository
    tags: Vec<String>,
    /// Locust integration
    locust: Locust<CrosstermBackend<Stdout>>,
    /// Current view mode
    view_mode: ViewMode,
    /// Tour active
    tour_active: bool,
    /// Tour step
    tour_step: usize,
    /// FPS counter
    fps_counter: common::FpsCounter,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ViewMode {
    Commits,
    Files,
    Diff,
}

/// File tree structure
#[derive(Clone)]
struct FileTree {
    entries: Vec<FileEntry>,
}

#[derive(Clone)]
struct FileEntry {
    path: String,
    is_dir: bool,
    children: Vec<FileEntry>,
    expanded: bool,
}

impl FileTree {
    fn new() -> Self {
        let entries = vec![
            FileEntry {
                path: "src".to_string(),
                is_dir: true,
                expanded: true,
                children: vec![
                    FileEntry {
                        path: "main.rs".to_string(),
                        is_dir: false,
                        children: vec![],
                        expanded: false,
                    },
                    FileEntry {
                        path: "lib.rs".to_string(),
                        is_dir: false,
                        children: vec![],
                        expanded: false,
                    },
                    FileEntry {
                        path: "utils.rs".to_string(),
                        is_dir: false,
                        children: vec![],
                        expanded: false,
                    },
                ],
            },
            FileEntry {
                path: "tests".to_string(),
                is_dir: true,
                expanded: false,
                children: vec![FileEntry {
                    path: "integration.rs".to_string(),
                    is_dir: false,
                    children: vec![],
                    expanded: false,
                }],
            },
            FileEntry {
                path: "Cargo.toml".to_string(),
                is_dir: false,
                children: vec![],
                expanded: false,
            },
            FileEntry {
                path: "README.md".to_string(),
                is_dir: false,
                children: vec![],
                expanded: false,
            },
        ];

        Self { entries }
    }

    fn flatten(&self) -> Vec<(String, bool, usize)> {
        fn flatten_entry(
            entry: &FileEntry,
            prefix: &str,
            depth: usize,
            result: &mut Vec<(String, bool, usize)>,
        ) {
            let path = if prefix.is_empty() {
                entry.path.clone()
            } else {
                format!("{}/{}", prefix, entry.path)
            };

            result.push((path.clone(), entry.is_dir, depth));

            if entry.is_dir && entry.expanded {
                for child in &entry.children {
                    flatten_entry(child, &path, depth + 1, result);
                }
            }
        }

        let mut result = Vec::new();
        for entry in &self.entries {
            flatten_entry(entry, "", 0, &mut result);
        }
        result
    }
}

/// Diff view for file changes
#[derive(Clone)]
struct DiffView {
    file_path: String,
    hunks: Vec<DiffHunk>,
    scroll_offset: usize,
}

#[derive(Clone)]
struct DiffHunk {
    header: String,
    lines: Vec<DiffLine>,
}

#[derive(Clone)]
struct DiffLine {
    line_type: DiffLineType,
    content: String,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum DiffLineType {
    Added,
    Removed,
    Context,
}

impl DiffView {
    fn new(file_path: String) -> Self {
        // Mock diff data
        let hunks = vec![
            DiffHunk {
                header: "@@ -1,5 +1,7 @@".to_string(),
                lines: vec![
                    DiffLine {
                        line_type: DiffLineType::Context,
                        content: " fn main() {".to_string(),
                    },
                    DiffLine {
                        line_type: DiffLineType::Removed,
                        content: "-     println!(\"Old version\");".to_string(),
                    },
                    DiffLine {
                        line_type: DiffLineType::Added,
                        content: "+     println!(\"New version\");".to_string(),
                    },
                    DiffLine {
                        line_type: DiffLineType::Added,
                        content: "+     println!(\"Additional feature\");".to_string(),
                    },
                    DiffLine {
                        line_type: DiffLineType::Context,
                        content: " }".to_string(),
                    },
                ],
            },
            DiffHunk {
                header: "@@ -10,3 +12,6 @@".to_string(),
                lines: vec![
                    DiffLine {
                        line_type: DiffLineType::Context,
                        content: " fn helper() {".to_string(),
                    },
                    DiffLine {
                        line_type: DiffLineType::Added,
                        content: "+     // New comment".to_string(),
                    },
                    DiffLine {
                        line_type: DiffLineType::Context,
                        content: "      return 42;".to_string(),
                    },
                    DiffLine {
                        line_type: DiffLineType::Context,
                        content: " }".to_string(),
                    },
                ],
            },
        ];

        Self {
            file_path,
            hunks,
            scroll_offset: 0,
        }
    }

    fn scroll_up(&mut self) {
        if self.scroll_offset > 0 {
            self.scroll_offset -= 1;
        }
    }

    fn scroll_down(&mut self) {
        let total_lines: usize = self.hunks.iter().map(|h| h.lines.len() + 1).sum();
        if self.scroll_offset < total_lines.saturating_sub(1) {
            self.scroll_offset += 1;
        }
    }
}

impl GitBrowser {
    fn new() -> io::Result<Self> {
        let mut locust = Locust::new(LocustConfig::default());

        // Initialize plugins
        locust.register_plugin(NavPlugin::default());
        locust.register_plugin(OmnibarPlugin::default());
        locust.register_plugin(TooltipPlugin::default());

        let mut highlight_plugin = HighlightPlugin::new();
        let tour = Tour::new("git_workflow_tour")
            .add_step(TourStep::new("Welcome", "Welcome to Git Browser! Press 'n' to continue."))
            .add_step(TourStep::new("Navigate Commits", "Navigate commits with arrow keys and 'f' for hints"))
            .add_step(TourStep::new("View Files", "View files by pressing Tab to switch to files panel"))
            .add_step(TourStep::new("View Diff", "Press Enter on a file to see the diff"))
            .add_step(TourStep::new("Command Palette", "Use Ctrl+P for command palette (checkout, search, etc.)"))
            .add_step(TourStep::new("Branches & Tags", "Press 'b' to switch branches. Press 'q' to exit tour."));
        highlight_plugin.register_tour(tour);
        locust.register_plugin(highlight_plugin);

        // Generate mock commits
        let commits = generate_commits(50);

        let mut commit_state = ListState::default();
        commit_state.select(Some(0));

        let mut file_state = ListState::default();
        file_state.select(Some(0));

        Ok(Self {
            repo_path: PathBuf::from("."),
            commits,
            commit_state,
            file_tree: FileTree::new(),
            file_state,
            diff_view: None,
            branches: vec![
                "main".to_string(),
                "develop".to_string(),
                "feature/new-ui".to_string(),
                "bugfix/memory-leak".to_string(),
            ],
            current_branch: "main".to_string(),
            tags: vec![
                "v1.0.0".to_string(),
                "v1.1.0".to_string(),
                "v2.0.0-beta".to_string(),
            ],
            locust,
            view_mode: ViewMode::Commits,
            tour_active: false,
            tour_step: 0,
            fps_counter: common::FpsCounter::new(),
        })
    }

    fn run(&mut self, terminal: &mut Terminal<CrosstermBackend<Stdout>>, log_tailer: &mut LogTailer) -> io::Result<()> {
        let tick_rate = Duration::from_millis(16); // ~60 FPS
        let mut last_tick = Instant::now();
        let mut target_builder = TargetBuilder::new();

        loop {
            self.fps_counter.tick();
            log_tailer.read_tail()?; // Update log tail at the beginning of each frame
            self.draw(terminal, log_tailer, &mut target_builder)?;

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

        // Let Locust handle the event first
        let event_result = self.locust.on_event(&Event::Key(key));
        if event_result.consumed {
            return Ok(false);
        }

        // Global commands
        match (key.modifiers, key.code) {
            (KeyModifiers::CONTROL, KeyCode::Char('c')) => return Ok(true),
            (_, KeyCode::Char('t')) => {
                self.tour_active = !self.tour_active;
                if self.tour_active {
                    self.tour_step = 0;
                }
                return Ok(false);
            }
            (_, KeyCode::Tab) => {
                self.view_mode = match self.view_mode {
                    ViewMode::Commits => ViewMode::Files,
                    ViewMode::Files => ViewMode::Diff,
                    ViewMode::Diff => ViewMode::Commits,
                };
                return Ok(false);
            }
            _ => {}
        }

        // View-specific controls
        match self.view_mode {
            ViewMode::Commits => self.handle_commits_input(key),
            ViewMode::Files => self.handle_files_input(key),
            ViewMode::Diff => self.handle_diff_input(key),
        }

        Ok(false)
    }

    fn handle_commits_input(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                let i = self.commit_state.selected().unwrap_or(0);
                if i > 0 {
                    self.commit_state.select(Some(i - 1));
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                let i = self.commit_state.selected().unwrap_or(0);
                if i < self.commits.len().saturating_sub(1) {
                    self.commit_state.select(Some(i + 1));
                }
            }
            KeyCode::Enter => {
                self.view_mode = ViewMode::Files;
            }
            _ => {}
        }
    }

    fn handle_files_input(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                let i = self.file_state.selected().unwrap_or(0);
                if i > 0 {
                    self.file_state.select(Some(i - 1));
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                let files = self.file_tree.flatten();
                let i = self.file_state.selected().unwrap_or(0);
                if i < files.len().saturating_sub(1) {
                    self.file_state.select(Some(i + 1));
                }
            }
            KeyCode::Enter => {
                let files = self.file_tree.flatten();
                if let Some(i) = self.file_state.selected() {
                    if let Some((path, is_dir, _)) = files.get(i) {
                        if !is_dir {
                            self.diff_view = Some(DiffView::new(path.clone()));
                            self.view_mode = ViewMode::Diff;
                        }
                    }
                }
            }
            _ => {}
        }
    }

    fn handle_diff_input(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                if let Some(diff) = &mut self.diff_view {
                    diff.scroll_up();
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if let Some(diff) = &mut self.diff_view {
                    diff.scroll_down();
                }
            }
            KeyCode::PageUp => {
                if let Some(diff) = &mut self.diff_view {
                    for _ in 0..10 {
                        diff.scroll_up();
                    }
                }
            }
            KeyCode::PageDown => {
                if let Some(diff) = &mut self.diff_view {
                    for _ in 0..10 {
                        diff.scroll_down();
                    }
                }
            }
            _ => {}
        }
    }

    fn draw(&mut self, terminal: &mut Terminal<CrosstermBackend<Stdout>>, log_tailer: &mut LogTailer, target_builder: &mut TargetBuilder) -> io::Result<()> {
        terminal.draw(|f| {
            let size = f.area();

            // Main layout: three columns + status bar + log tailer
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Min(0), // Main content
                    Constraint::Length(1), // Status bar
                    Constraint::Length(10), // Log tailer
                ])
                .split(size);

            let main_content_area = chunks[0];
            let status_bar_area = chunks[1];
            let log_tailer_area = chunks[2];

            // Three-column layout for main content
            let main_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(40),
                    Constraint::Percentage(30),
                    Constraint::Percentage(30),
                ])
                .split(main_content_area);

            // Draw commits panel
            self.draw_commits(f, main_chunks[0], target_builder);

            // Draw files panel
            self.draw_files(f, main_chunks[1], target_builder);

            // Draw diff panel
            self.draw_diff(f, main_chunks[2], target_builder);

            // Draw status bar
            self.draw_status_bar(f, status_bar_area);

            // Draw Log Tailer
            f.render_widget(log_tailer, log_tailer_area);

            // Draw tour if active
            if self.tour_active {
                self.draw_tour(f, size);
            }
        })?;

        Ok(())
    }

    fn draw_commits(&mut self, f: &mut Frame, area: Rect, target_builder: &mut TargetBuilder) {
        let border_style = if self.view_mode == ViewMode::Commits {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().fg(Color::Gray)
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .title(" Commits ")
            .border_style(border_style);

        let items: Vec<ListItem> = self
            .commits
            .iter()
            .map(|commit| {
                let line = Line::from(vec![
                    Span::styled(
                        format!("[{}] ", &commit.hash[..7]),
                        Style::default().fg(Color::Yellow),
                    ),
                    Span::raw(&commit.message),
                ]);
                ListItem::new(line)
            })
            .collect();

        let list = List::new(items)
            .block(block.clone())
            .highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(">> ");

        f.render_stateful_widget(list, area, &mut self.commit_state);

        // Register NavTargets for commits
        let list_items_area = block.inner(area);
        let row_height = 1;
        for (idx, commit) in self.commits.iter().enumerate() {
            let item_rect = Rect::new(
                list_items_area.x,
                list_items_area.y + idx as u16 * row_height,
                list_items_area.width,
                row_height,
            );
            self.locust.ctx.targets.register(
                target_builder.list_item(item_rect, format!("Commit: {}", commit.message))
            );
        }
    }

    fn draw_files(&mut self, f: &mut Frame, area: Rect, target_builder: &mut TargetBuilder) {
        let border_style = if self.view_mode == ViewMode::Files {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().fg(Color::Gray)
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .title(" Files ")
            .border_style(border_style);

        let files = self.file_tree.flatten();
        let items: Vec<ListItem> = files
            .iter()
            .map(|(path, is_dir, depth)| {
                let indent = "  ".repeat(*depth);
                let icon = if *is_dir { "📁" } else { "📄" };
                let name = path.split('/').last().unwrap_or(path);
                let line = format!("{}{} {}", indent, icon, name);
                ListItem::new(line)
            })
            .collect();

        let list = List::new(items)
            .block(block.clone())
            .highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(">> ");

        f.render_stateful_widget(list, area, &mut self.file_state);

        // Register NavTargets for files
        let list_items_area = block.inner(area);
        let row_height = 1;
        for (idx, (path, _, _)) in files.iter().enumerate() {
            let item_rect = Rect::new(
                list_items_area.x,
                list_items_area.y + idx as u16 * row_height,
                list_items_area.width,
                row_height,
            );
            self.locust.ctx.targets.register(
                target_builder.list_item(item_rect, format!("File: {}", path))
            );
        }
    }

    fn draw_diff(&mut self, f: &mut Frame, area: Rect, target_builder: &mut TargetBuilder) {
        let border_style = if self.view_mode == ViewMode::Diff {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().fg(Color::Gray)
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .title(" Diff ")
            .border_style(border_style);

        if let Some(diff) = &self.diff_view {
            let mut lines = Vec::new();

            for hunk in &diff.hunks {
                lines.push(Line::from(Span::styled(
                    &hunk.header,
                    Style::default().fg(Color::Cyan),
                )));

                for line in &hunk.lines {
                    let (prefix, color) = match line.line_type {
                        DiffLineType::Added => ("+", Color::Green),
                        DiffLineType::Removed => ("-", Color::Red),
                        DiffLineType::Context => (" ", Color::White),
                    };

                    lines.push(Line::from(Span::styled(
                        format!("{}{}", prefix, line.content),
                        Style::default().fg(color),
                    )));
                }
            }

            let visible_lines: Vec<Line> = lines
                .into_iter()
                .skip(diff.scroll_offset)
                .take(area.height.saturating_sub(2) as usize)
                .collect();

            let paragraph = Paragraph::new(visible_lines)
                .block(block)
                .wrap(Wrap { trim: false });

            f.render_widget(paragraph, area);

        // Register NavTarget for the diff pane
        self.locust.ctx.targets.register(
            target_builder.custom(area, "Diff Pane", TargetAction::Activate, TargetPriority::Low)
        );
        } else {
            let text = Paragraph::new("Select a file to view diff")
                .block(block)
                .alignment(Alignment::Center);
            f.render_widget(text, area);
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
        let status = format!(
            "Branch: {} | Commits: {} | {}",
            self.current_branch,
            self.commits.len(),
            fps
        );
        let help =
            "Tab: switch panel | Enter: select | Ctrl+P: commands | 't': tour | Ctrl+C: quit";

        let status_text = format!("{} | {}", status, help);
        let status_widget = Paragraph::new(status_text)
            .style(Style::default().bg(Color::DarkGray).fg(Color::White));

        f.render_widget(status_widget, status_area);
    }

    fn draw_tour(&self, f: &mut Frame, area: Rect) {
        let messages = [
            "Welcome to Git Browser! Press 'n' to continue.",
            "Navigate commits with arrow keys and 'f' for hints",
            "View files by pressing Tab to switch to files panel",
            "Press Enter on a file to see the diff",
            "Use Ctrl+P for command palette (checkout, search, etc.)",
            "Press 'b' to switch branches. Press 'q' to exit tour.",
        ];

        let popup_area = common::centered_rect(60, 30, area);
        let message = messages.get(self.tour_step).unwrap_or(&messages[0]);

        let tour_widget = Paragraph::new(*message)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Git Workflow Tutorial ")
                    .border_style(Style::default().fg(Color::Yellow)),
            )
            .style(Style::default().fg(Color::White))
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });

        f.render_widget(tour_widget, popup_area);
    }
}

fn main() -> io::Result<()> {
    // Initialize logger
    let log_file_path = PathBuf::from("locust-git-browser.log");
    CombinedLogger::init(
        vec![
            WriteLogger::new(
                LevelFilter::Debug,
                Config::default(),
                File::create(&log_file_path).unwrap(),
            ),
        ]
    ).unwrap();
    debug!("Logger initialized for Git Browser.");

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = GitBrowser::new()?;
    let mut log_tailer = LogTailer::new(log_file_path, 10); // Display last 10 log lines
    let result = app.run(&mut terminal, &mut log_tailer);

    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen)?;

    result
}
