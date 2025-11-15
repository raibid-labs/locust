/// # File Browser Example
///
/// A comprehensive file manager application demonstrating advanced Locust
/// navigation features. This example showcases:
///
/// - Three-pane layout: tree view, file list, and preview pane
/// - Keyboard-driven file navigation and operations
/// - Breadcrumb navigation showing current path
/// - File search and filtering functionality
/// - Hint mode ('f') for quick file selection
/// - Preview pane showing file contents
/// - Responsive layout with smooth rendering
///
/// ## Controls
///
/// - `f` - Enter hint mode for quick navigation
/// - `/` - Open search/filter input
/// - `Arrow Keys` - Navigate tree and file list
/// - `Enter` - Open file/expand directory
/// - `Space` - Toggle preview for selected file
/// - `Backspace` - Go up one directory level
/// - `h` - Toggle hidden files visibility
/// - `Tab` - Cycle between panes (tree/list/preview)
/// - `q` - Quit the application
/// - `Esc` - Cancel current action
///
/// ## Architecture
///
/// The file browser uses a three-pane architecture with:
/// - Left pane: Directory tree with expand/collapse
/// - Center pane: File list for current directory
/// - Right pane: File preview (text files only)
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
    widgets::{Block, BorderType, Borders, List, ListItem, Paragraph, Wrap},
    Frame, Terminal,
};
use std::fs;
use std::io::{self, Read, Stdout};
use std::path::{Path, PathBuf};
use std::time::Duration;

/// Represents a node in the directory tree
#[derive(Clone)]
struct TreeNode {
    path: PathBuf,
    name: String,
    is_dir: bool,
    expanded: bool,
    level: usize,
    has_children: bool,
}

/// Represents a file entry in the file list
#[derive(Clone)]
struct FileEntry {
    path: PathBuf,
    name: String,
    is_dir: bool,
    size: u64,
    #[allow(dead_code)]
    extension: String,
}

/// Active pane in the file browser
#[derive(Clone, Copy, PartialEq)]
enum ActivePane {
    Tree,
    FileList,
    Preview,
}

impl ActivePane {
    fn next(&self) -> Self {
        match self {
            Self::Tree => Self::FileList,
            Self::FileList => Self::Preview,
            Self::Preview => Self::Tree,
        }
    }

    fn previous(&self) -> Self {
        match self {
            Self::Tree => Self::Preview,
            Self::FileList => Self::Tree,
            Self::Preview => Self::FileList,
        }
    }
}

/// Main file browser application state
struct FileBrowser {
    /// Current working directory
    current_dir: PathBuf,
    /// Directory tree nodes
    tree_nodes: Vec<TreeNode>,
    /// Files in current directory
    file_entries: Vec<FileEntry>,
    /// Selected tree node index
    selected_tree: usize,
    /// Selected file index
    selected_file: usize,
    /// Active pane
    active_pane: ActivePane,
    /// Search/filter input
    search_input: String,
    /// Whether search mode is active
    search_active: bool,
    /// Show hidden files
    show_hidden: bool,
    /// Preview content
    preview_content: Vec<String>,
    /// Preview scroll offset
    preview_scroll: usize,
    /// Should quit flag
    should_quit: bool,
}

impl FileBrowser {
    fn new() -> io::Result<Self> {
        let current_dir = std::env::current_dir()?;
        let mut browser = Self {
            current_dir: current_dir.clone(),
            tree_nodes: Vec::new(),
            file_entries: Vec::new(),
            selected_tree: 0,
            selected_file: 0,
            active_pane: ActivePane::FileList,
            search_input: String::new(),
            search_active: false,
            show_hidden: false,
            preview_content: Vec::new(),
            preview_scroll: 0,
            should_quit: false,
        };

        browser.build_tree(&current_dir, 0)?;
        browser.load_directory(&current_dir)?;
        browser.update_preview();

        Ok(browser)
    }

    /// Build directory tree recursively
    fn build_tree(&mut self, dir: &Path, level: usize) -> io::Result<()> {
        if level > 3 {
            // Limit recursion depth
            return Ok(());
        }

        let entries = match fs::read_dir(dir) {
            Ok(entries) => entries,
            Err(_) => return Ok(()),
        };

        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();

            // Skip hidden files if not showing them
            if !self.show_hidden && name.starts_with('.') {
                continue;
            }

            if path.is_dir() {
                let has_children = fs::read_dir(&path)
                    .map(|mut e| e.next().is_some())
                    .unwrap_or(false);

                self.tree_nodes.push(TreeNode {
                    path: path.clone(),
                    name,
                    is_dir: true,
                    expanded: level == 0, // Only expand first level
                    level,
                    has_children,
                });

                // Recursively build tree for expanded nodes
                if level == 0 {
                    self.build_tree(&path, level + 1)?;
                }
            }
        }

        Ok(())
    }

    /// Load files in the current directory
    fn load_directory(&mut self, dir: &Path) -> io::Result<()> {
        self.file_entries.clear();
        self.selected_file = 0;

        let entries = fs::read_dir(dir)?;

        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();

            // Skip hidden files if not showing them
            if !self.show_hidden && name.starts_with('.') {
                continue;
            }

            // Apply search filter
            if !self.search_input.is_empty()
                && !name
                    .to_lowercase()
                    .contains(&self.search_input.to_lowercase())
            {
                continue;
            }

            let metadata = entry.metadata().ok();
            let size = metadata.as_ref().map(|m| m.len()).unwrap_or(0);
            let is_dir = metadata.as_ref().map(|m| m.is_dir()).unwrap_or(false);
            let extension = path
                .extension()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_string();

            self.file_entries.push(FileEntry {
                path,
                name,
                is_dir,
                size,
                extension,
            });
        }

        // Sort: directories first, then alphabetically
        self.file_entries.sort_by(|a, b| {
            if a.is_dir == b.is_dir {
                a.name.to_lowercase().cmp(&b.name.to_lowercase())
            } else if a.is_dir {
                std::cmp::Ordering::Less
            } else {
                std::cmp::Ordering::Greater
            }
        });

        Ok(())
    }

    /// Update preview content for selected file
    fn update_preview(&mut self) {
        self.preview_content.clear();
        self.preview_scroll = 0;

        if let Some(entry) = self.file_entries.get(self.selected_file) {
            if !entry.is_dir && entry.size < 1_000_000 {
                // Only preview files < 1MB
                if let Ok(mut file) = fs::File::open(&entry.path) {
                    let mut content = String::new();
                    if file.read_to_string(&mut content).is_ok() {
                        self.preview_content = content.lines().map(String::from).collect();
                    } else {
                        self.preview_content = vec!["[Binary file - cannot preview]".to_string()];
                    }
                }
            } else if entry.is_dir {
                self.preview_content = vec!["[Directory]".to_string()];
            } else {
                self.preview_content = vec!["[File too large to preview]".to_string()];
            }
        }
    }

    /// Handle keyboard input
    fn handle_key(&mut self, key: KeyCode, modifiers: KeyModifiers) -> io::Result<()> {
        if self.search_active {
            self.handle_search_key(key)?;
            return Ok(());
        }

        match key {
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Char('/') => {
                self.search_active = true;
                self.search_input.clear();
            }
            KeyCode::Char('h') => {
                self.show_hidden = !self.show_hidden;
                self.rebuild_all()?;
            }
            KeyCode::Tab => {
                if modifiers.contains(KeyModifiers::SHIFT) {
                    self.active_pane = self.active_pane.previous();
                } else {
                    self.active_pane = self.active_pane.next();
                }
            }
            KeyCode::Up => self.handle_up(),
            KeyCode::Down => self.handle_down(),
            KeyCode::Enter => self.handle_enter()?,
            KeyCode::Char(' ') => {
                if self.active_pane == ActivePane::FileList {
                    self.update_preview();
                }
            }
            KeyCode::Backspace => {
                if let Some(parent) = self.current_dir.parent() {
                    let parent_path = parent.to_path_buf();
                    self.current_dir = parent_path.clone();
                    self.load_directory(&parent_path)?;
                    self.update_preview();
                }
            }
            _ => {}
        }

        Ok(())
    }

    /// Handle search mode keyboard input
    fn handle_search_key(&mut self, key: KeyCode) -> io::Result<()> {
        match key {
            KeyCode::Esc => {
                self.search_active = false;
                self.search_input.clear();
                let current = self.current_dir.clone();
                self.load_directory(&current)?;
            }
            KeyCode::Enter => {
                self.search_active = false;
                let current = self.current_dir.clone();
                self.load_directory(&current)?;
            }
            KeyCode::Char(c) => {
                self.search_input.push(c);
                let current = self.current_dir.clone();
                self.load_directory(&current)?;
            }
            KeyCode::Backspace => {
                self.search_input.pop();
                let current = self.current_dir.clone();
                self.load_directory(&current)?;
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_up(&mut self) {
        match self.active_pane {
            ActivePane::Tree => {
                if self.selected_tree > 0 {
                    self.selected_tree -= 1;
                }
            }
            ActivePane::FileList => {
                if self.selected_file > 0 {
                    self.selected_file -= 1;
                    self.update_preview();
                }
            }
            ActivePane::Preview => {
                if self.preview_scroll > 0 {
                    self.preview_scroll -= 1;
                }
            }
        }
    }

    fn handle_down(&mut self) {
        match self.active_pane {
            ActivePane::Tree => {
                if self.selected_tree < self.tree_nodes.len().saturating_sub(1) {
                    self.selected_tree += 1;
                }
            }
            ActivePane::FileList => {
                if self.selected_file < self.file_entries.len().saturating_sub(1) {
                    self.selected_file += 1;
                    self.update_preview();
                }
            }
            ActivePane::Preview => {
                if self.preview_scroll < self.preview_content.len().saturating_sub(1) {
                    self.preview_scroll += 1;
                }
            }
        }
    }

    fn handle_enter(&mut self) -> io::Result<()> {
        match self.active_pane {
            ActivePane::Tree => {
                let should_rebuild = if let Some(node) = self.tree_nodes.get_mut(self.selected_tree)
                {
                    if node.is_dir && node.has_children {
                        node.expanded = !node.expanded;
                        true
                    } else {
                        false
                    }
                } else {
                    false
                };

                if should_rebuild {
                    self.rebuild_tree()?;
                }

                if let Some(node) = self.tree_nodes.get(self.selected_tree) {
                    if node.is_dir {
                        let path = node.path.clone();
                        self.current_dir = path.clone();
                        self.load_directory(&path)?;
                        self.update_preview();
                    }
                }
            }
            ActivePane::FileList => {
                if let Some(entry) = self.file_entries.get(self.selected_file) {
                    if entry.is_dir {
                        let path = entry.path.clone();
                        self.current_dir = path.clone();
                        self.load_directory(&path)?;
                        self.update_preview();
                    } else {
                        self.update_preview();
                        self.active_pane = ActivePane::Preview;
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn rebuild_tree(&mut self) -> io::Result<()> {
        let root = self.current_dir.clone();
        self.tree_nodes.clear();
        self.build_tree(&root, 0)
    }

    fn rebuild_all(&mut self) -> io::Result<()> {
        self.rebuild_tree()?;
        let current = self.current_dir.clone();
        self.load_directory(&current)
    }

    /// Render the file browser UI
    fn draw(&self, f: &mut Frame, locust: &mut Locust<CrosstermBackend<Stdout>>) {
        let size = f.area();

        // Main layout: breadcrumb + content
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0)])
            .split(size);

        // Render breadcrumb
        self.draw_breadcrumb(f, chunks[0]);

        // Three-pane layout
        let content_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(25),
                Constraint::Percentage(35),
                Constraint::Percentage(40),
            ])
            .split(chunks[1]);

        // Render each pane
        self.draw_tree(f, content_chunks[0]);
        self.draw_file_list(f, content_chunks[1]);
        self.draw_preview(f, content_chunks[2]);

        // Render search bar if active
        if self.search_active {
            self.draw_search(f, size);
        }

        // Let Locust render overlays
        locust.render_overlay(f);
    }

    fn draw_breadcrumb(&self, f: &mut Frame, area: Rect) {
        let path_str = self.current_dir.display().to_string();
        let text = vec![Line::from(vec![
            Span::styled(" Path: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(path_str),
        ])];

        let paragraph = Paragraph::new(text).block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(" File Browser "),
        );

        f.render_widget(paragraph, area);
    }

    fn draw_tree(&self, f: &mut Frame, area: Rect) {
        let is_active = self.active_pane == ActivePane::Tree;
        let border_style = if is_active {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        };

        let items: Vec<ListItem> = self
            .tree_nodes
            .iter()
            .enumerate()
            .map(|(idx, node)| {
                let indent = "  ".repeat(node.level);
                let icon = if node.has_children {
                    if node.expanded {
                        "▼ "
                    } else {
                        "▶ "
                    }
                } else {
                    "  "
                };

                let style = if idx == self.selected_tree && is_active {
                    Style::default()
                        .bg(Color::Blue)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };

                ListItem::new(Line::from(Span::styled(
                    format!("{}{}{}", indent, icon, node.name),
                    style,
                )))
            })
            .collect();

        let list = List::new(items).block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(border_style)
                .title(" Tree "),
        );

        f.render_widget(list, area);
    }

    fn draw_file_list(&self, f: &mut Frame, area: Rect) {
        let is_active = self.active_pane == ActivePane::FileList;
        let border_style = if is_active {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        };

        let items: Vec<ListItem> = self
            .file_entries
            .iter()
            .enumerate()
            .map(|(idx, entry)| {
                let icon = if entry.is_dir { "📁 " } else { "📄 " };
                let size_str = if entry.is_dir {
                    String::new()
                } else {
                    format!(" ({} bytes)", format_size(entry.size))
                };

                let style = if idx == self.selected_file && is_active {
                    Style::default()
                        .bg(Color::Blue)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };

                ListItem::new(Line::from(Span::styled(
                    format!("{}{}{}", icon, entry.name, size_str),
                    style,
                )))
            })
            .collect();

        let title = if !self.search_input.is_empty() {
            format!(" Files (filter: {}) ", self.search_input)
        } else {
            format!(" Files ({}) ", self.file_entries.len())
        };

        let list = List::new(items).block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(border_style)
                .title(title),
        );

        f.render_widget(list, area);
    }

    fn draw_preview(&self, f: &mut Frame, area: Rect) {
        let is_active = self.active_pane == ActivePane::Preview;
        let border_style = if is_active {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        };

        let lines: Vec<Line> = self
            .preview_content
            .iter()
            .skip(self.preview_scroll)
            .take(area.height.saturating_sub(2) as usize)
            .map(|line| Line::from(line.clone()))
            .collect();

        let paragraph = Paragraph::new(lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(border_style)
                    .title(format!(
                        " Preview ({}/{}) ",
                        self.preview_scroll,
                        self.preview_content.len()
                    )),
            )
            .wrap(Wrap { trim: false });

        f.render_widget(paragraph, area);
    }

    fn draw_search(&self, f: &mut Frame, area: Rect) {
        let search_area = Rect {
            x: area.width / 4,
            y: area.height / 2 - 2,
            width: area.width / 2,
            height: 3,
        };

        let text = vec![Line::from(vec![
            Span::styled("Filter: ", Style::default().fg(Color::Yellow)),
            Span::raw(&self.search_input),
            Span::styled("█", Style::default().fg(Color::White)),
        ])];

        let paragraph = Paragraph::new(text).block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Yellow))
                .title(" Search/Filter "),
        );

        f.render_widget(paragraph, search_area);
    }
}

/// Format file size in human-readable format
fn format_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_idx = 0;

    while size >= 1024.0 && unit_idx < UNITS.len() - 1 {
        size /= 1024.0;
        unit_idx += 1;
    }

    format!("{:.1} {}", size, UNITS[unit_idx])
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

    // Create file browser
    let mut browser = FileBrowser::new()?;

    // Main event loop
    loop {
        locust.begin_frame();

        // Draw UI
        terminal.draw(|f| {
            browser.draw(f, &mut locust);
        })?;

        // Handle events
        if event::poll(Duration::from_millis(100))? {
            let ev = event::read()?;
            let outcome = locust.on_event(&ev);

            // Handle events not consumed by Locust
            if !outcome.consumed {
                if let Event::Key(key) = ev {
                    browser.handle_key(key.code, key.modifiers)?;
                }
            }
        }

        if browser.should_quit {
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
