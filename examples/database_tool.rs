/// Database Query Tool - Interactive TUI for database interaction
///
/// Features:
/// - Connection management for multiple databases
/// - Interactive schema browser
/// - SQL query editor with syntax highlighting
/// - Result table navigation
/// - Query history tracking
/// - Export results to CSV
/// - Visual query builder (basic)
/// - Command palette for database operations
/// - Tooltips for schema metadata
/// - Guided tour for database workflow
///
/// Layout:
/// ```
/// ┌─────────────────────────────────────────────────┐
/// │ Schema         │ Query Editor                   │
/// ├────────────────┼────────────────────────────────┤
/// │ Tables:        │ SELECT name, email, created_at │
/// │  users         │ FROM users                     │
/// │  posts         │ WHERE status = 'active'        │
/// │  comments      │ ORDER BY created_at DESC;      │
/// │                │                                │
/// │ Views:         │ [Execute: Ctrl+E]              │
/// │  active_users  │                                │
/// ├────────────────┴────────────────────────────────┤
/// │ Results (124 rows)                              │
/// │ Name          │ Email           │ Created       │
/// │ John Doe      │ john@example    │ 2025-01-10   │
/// │ Jane Smith    │ jane@example    │ 2025-01-11   │
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
    widgets::{Block, Borders, Cell, List, ListItem, ListState, Paragraph, Row, Table, Wrap},
    Frame, Terminal,
};
use std::{
    collections::VecDeque,
    io::{self, Stdout},
    time::{Duration, Instant},
};

use common::mock::{generate_schema, Column, Table as SchemaTable};

/// Main database tool application
struct DatabaseTool {
    /// Available database connections
    databases: Vec<Database>,
    /// Current database index
    current_db: Option<usize>,
    /// Schema information
    schema: Schema,
    /// Query editor
    query_editor: QueryEditor,
    /// Query results
    results: QueryResults,
    /// Query history
    history: VecDeque<Query>,
    /// Locust integration
    locust: Locust<CrosstermBackend<Stdout>>,
    /// Current view mode
    view_mode: ViewMode,
    /// Schema list state
    schema_state: ListState,
    /// Whether schema is focused
    schema_focused: bool,
    /// Tour active
    tour_active: bool,
    /// Tour step
    tour_step: usize,
    /// FPS counter
    fps_counter: common::FpsCounter,
}

#[derive(Clone)]
struct Database {
    name: String,
    db_type: DatabaseType,
    connection_string: String,
    connected: bool,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum DatabaseType {
    SQLite,
    PostgreSQL,
    MySQL,
}

impl DatabaseType {
    fn icon(&self) -> &'static str {
        match self {
            DatabaseType::SQLite => "📋",
            DatabaseType::PostgreSQL => "🐘",
            DatabaseType::MySQL => "🐬",
        }
    }

    fn as_str(&self) -> &'static str {
        match self {
            DatabaseType::SQLite => "SQLite",
            DatabaseType::PostgreSQL => "PostgreSQL",
            DatabaseType::MySQL => "MySQL",
        }
    }
}

#[derive(Clone)]
struct Schema {
    tables: Vec<SchemaTable>,
    views: Vec<View>,
    indexes: Vec<Index>,
}

#[derive(Clone)]
struct View {
    name: String,
    definition: String,
}

#[derive(Clone)]
struct Index {
    name: String,
    table: String,
    columns: Vec<String>,
    unique: bool,
}

impl Schema {
    fn new() -> Self {
        Self {
            tables: generate_schema(),
            views: vec![
                View {
                    name: "active_users".to_string(),
                    definition: "SELECT * FROM users WHERE status = 'active'".to_string(),
                },
                View {
                    name: "recent_posts".to_string(),
                    definition: "SELECT * FROM posts ORDER BY created_at DESC LIMIT 100"
                        .to_string(),
                },
            ],
            indexes: vec![
                Index {
                    name: "idx_users_email".to_string(),
                    table: "users".to_string(),
                    columns: vec!["email".to_string()],
                    unique: true,
                },
                Index {
                    name: "idx_posts_user_id".to_string(),
                    table: "posts".to_string(),
                    columns: vec!["user_id".to_string()],
                    unique: false,
                },
            ],
        }
    }

    fn all_items(&self) -> Vec<SchemaItem> {
        let mut items = Vec::new();

        items.push(SchemaItem::Section("Tables".to_string()));
        for table in &self.tables {
            items.push(SchemaItem::Table(table.clone()));
        }

        items.push(SchemaItem::Section("Views".to_string()));
        for view in &self.views {
            items.push(SchemaItem::View(view.clone()));
        }

        items.push(SchemaItem::Section("Indexes".to_string()));
        for index in &self.indexes {
            items.push(SchemaItem::Index(index.clone()));
        }

        items
    }
}

#[derive(Clone)]
enum SchemaItem {
    Section(String),
    Table(SchemaTable),
    View(View),
    Index(Index),
}

struct QueryEditor {
    lines: Vec<String>,
    cursor_line: usize,
    cursor_col: usize,
    scroll_offset: usize,
}

impl QueryEditor {
    fn new() -> Self {
        let lines = vec![
            "SELECT name, email, created_at".to_string(),
            "FROM users".to_string(),
            "WHERE status = 'active'".to_string(),
            "ORDER BY created_at DESC;".to_string(),
        ];

        Self {
            lines,
            cursor_line: 0,
            cursor_col: 0,
            scroll_offset: 0,
        }
    }

    fn insert_char(&mut self, c: char) {
        if self.cursor_line >= self.lines.len() {
            self.lines.push(String::new());
        }

        self.lines[self.cursor_line].insert(self.cursor_col, c);
        self.cursor_col += 1;
    }

    fn delete_char(&mut self) {
        if self.cursor_line < self.lines.len() && self.cursor_col > 0 {
            self.lines[self.cursor_line].remove(self.cursor_col - 1);
            self.cursor_col -= 1;
        }
    }

    fn move_cursor_up(&mut self) {
        if self.cursor_line > 0 {
            self.cursor_line -= 1;
            self.cursor_col = self.cursor_col.min(self.lines[self.cursor_line].len());
        }
    }

    fn move_cursor_down(&mut self) {
        if self.cursor_line < self.lines.len().saturating_sub(1) {
            self.cursor_line += 1;
            self.cursor_col = self.cursor_col.min(self.lines[self.cursor_line].len());
        }
    }

    fn move_cursor_left(&mut self) {
        if self.cursor_col > 0 {
            self.cursor_col -= 1;
        }
    }

    fn move_cursor_right(&mut self) {
        if self.cursor_line < self.lines.len() {
            if self.cursor_col < self.lines[self.cursor_line].len() {
                self.cursor_col += 1;
            }
        }
    }

    fn get_query(&self) -> String {
        self.lines.join("\n")
    }

    fn set_query(&mut self, query: &str) {
        self.lines = query.lines().map(|s| s.to_string()).collect();
        if self.lines.is_empty() {
            self.lines.push(String::new());
        }
        self.cursor_line = 0;
        self.cursor_col = 0;
    }
}

struct QueryResults {
    columns: Vec<String>,
    rows: Vec<Vec<String>>,
    selected_row: usize,
    selected_col: usize,
    scroll_offset: usize,
    execution_time_ms: f64,
}

impl QueryResults {
    fn new() -> Self {
        Self {
            columns: vec![],
            rows: vec![],
            selected_row: 0,
            selected_col: 0,
            scroll_offset: 0,
            execution_time_ms: 0.0,
        }
    }

    fn set_data(&mut self, columns: Vec<String>, rows: Vec<Vec<String>>, exec_time: f64) {
        self.columns = columns;
        self.rows = rows;
        self.execution_time_ms = exec_time;
        self.selected_row = 0;
        self.selected_col = 0;
        self.scroll_offset = 0;
    }

    fn move_up(&mut self) {
        if self.selected_row > 0 {
            self.selected_row -= 1;
            if self.selected_row < self.scroll_offset {
                self.scroll_offset = self.selected_row;
            }
        }
    }

    fn move_down(&mut self, visible_rows: usize) {
        if self.selected_row < self.rows.len().saturating_sub(1) {
            self.selected_row += 1;
            if self.selected_row >= self.scroll_offset + visible_rows {
                self.scroll_offset = self.selected_row - visible_rows + 1;
            }
        }
    }

    fn move_left(&mut self) {
        if self.selected_col > 0 {
            self.selected_col -= 1;
        }
    }

    fn move_right(&mut self) {
        if self.selected_col < self.columns.len().saturating_sub(1) {
            self.selected_col += 1;
        }
    }
}

#[derive(Clone)]
struct Query {
    sql: String,
    timestamp: chrono::DateTime<Local>,
    execution_time_ms: f64,
    row_count: usize,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ViewMode {
    Schema,
    Editor,
    Results,
}

impl DatabaseTool {
    fn new(terminal: Terminal<CrosstermBackend<Stdout>>) -> io::Result<Self> {
        let mut locust = Locust::new(terminal);

        // Initialize plugins
        locust.add_plugin(NavPlugin::default());
        locust.add_plugin(OmnibarPlugin::default());
        locust.add_plugin(TooltipPlugin::default());

        let highlight_config = HighlightConfig {
            steps: vec![
                "Welcome to Database Tool! Press 'n' to continue.".to_string(),
                "Browse schema in the left panel (tables, views, indexes)".to_string(),
                "Write queries in the editor (middle panel)".to_string(),
                "Execute queries with Ctrl+E".to_string(),
                "Navigate results with arrow keys in the bottom panel".to_string(),
                "Use Ctrl+P for commands (EXPORT, DESCRIBE, etc.)".to_string(),
            ],
            highlight_color: Color::Yellow,
            text_color: Color::White,
        };
        locust.add_plugin(HighlightPlugin::new(highlight_config));

        let databases = vec![
            Database {
                name: "demo.db".to_string(),
                db_type: DatabaseType::SQLite,
                connection_string: "demo.db".to_string(),
                connected: true,
            },
            Database {
                name: "production".to_string(),
                db_type: DatabaseType::PostgreSQL,
                connection_string: "postgresql://localhost/prod".to_string(),
                connected: false,
            },
        ];

        let mut schema_state = ListState::default();
        schema_state.select(Some(0));

        // Generate mock results
        let mut results = QueryResults::new();
        results.set_data(
            vec![
                "id".to_string(),
                "name".to_string(),
                "email".to_string(),
                "created_at".to_string(),
            ],
            vec![
                vec![
                    "1".to_string(),
                    "John Doe".to_string(),
                    "john@example.com".to_string(),
                    "2025-01-10 10:30:00".to_string(),
                ],
                vec![
                    "2".to_string(),
                    "Jane Smith".to_string(),
                    "jane@example.com".to_string(),
                    "2025-01-11 14:22:00".to_string(),
                ],
                vec![
                    "3".to_string(),
                    "Bob Wilson".to_string(),
                    "bob@example.com".to_string(),
                    "2025-01-12 09:15:00".to_string(),
                ],
                vec![
                    "4".to_string(),
                    "Alice Brown".to_string(),
                    "alice@example.com".to_string(),
                    "2025-01-13 16:45:00".to_string(),
                ],
            ],
            12.5,
        );

        Ok(Self {
            databases,
            current_db: Some(0),
            schema: Schema::new(),
            query_editor: QueryEditor::new(),
            results,
            history: VecDeque::new(),
            locust,
            view_mode: ViewMode::Editor,
            schema_state,
            schema_focused: false,
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

        // Global commands
        match (key.modifiers, key.code) {
            (KeyModifiers::CONTROL, KeyCode::Char('c')) => return Ok(true),
            (KeyModifiers::CONTROL, KeyCode::Char('p')) => {
                self.locust.omnibar_mut().toggle();
                return Ok(false);
            }
            (KeyModifiers::CONTROL, KeyCode::Char('e')) => {
                self.execute_query();
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
                    ViewMode::Schema => ViewMode::Editor,
                    ViewMode::Editor => ViewMode::Results,
                    ViewMode::Results => ViewMode::Schema,
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
            ViewMode::Schema => self.handle_schema_input(key),
            ViewMode::Editor => self.handle_editor_input(key),
            ViewMode::Results => self.handle_results_input(key),
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

    fn handle_schema_input(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                let i = self.schema_state.selected().unwrap_or(0);
                if i > 0 {
                    self.schema_state.select(Some(i - 1));
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                let items = self.schema.all_items();
                let i = self.schema_state.selected().unwrap_or(0);
                if i < items.len().saturating_sub(1) {
                    self.schema_state.select(Some(i + 1));
                }
            }
            KeyCode::Enter => {
                // Insert selected table/view into query
                let items = self.schema.all_items();
                if let Some(i) = self.schema_state.selected() {
                    if let Some(item) = items.get(i) {
                        match item {
                            SchemaItem::Table(table) => {
                                let query = format!("SELECT * FROM {} LIMIT 10;", table.name);
                                self.query_editor.set_query(&query);
                            }
                            SchemaItem::View(view) => {
                                let query = format!("SELECT * FROM {} LIMIT 10;", view.name);
                                self.query_editor.set_query(&query);
                            }
                            _ => {}
                        }
                    }
                }
            }
            _ => {}
        }
    }

    fn handle_editor_input(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Up => self.query_editor.move_cursor_up(),
            KeyCode::Down => self.query_editor.move_cursor_down(),
            KeyCode::Left => self.query_editor.move_cursor_left(),
            KeyCode::Right => self.query_editor.move_cursor_right(),
            KeyCode::Char(c) => self.query_editor.insert_char(c),
            KeyCode::Backspace => self.query_editor.delete_char(),
            KeyCode::Enter => {
                self.query_editor
                    .lines
                    .insert(self.query_editor.cursor_line + 1, String::new());
                self.query_editor.cursor_line += 1;
                self.query_editor.cursor_col = 0;
            }
            _ => {}
        }
    }

    fn handle_results_input(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                self.results.move_up();
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.results.move_down(10);
            }
            KeyCode::Left | KeyCode::Char('h') => {
                self.results.move_left();
            }
            KeyCode::Right | KeyCode::Char('l') => {
                self.results.move_right();
            }
            _ => {}
        }
    }

    fn execute_command(&mut self, command: &str) {
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.is_empty() {
            return;
        }

        match parts[0].to_uppercase().as_str() {
            "CONNECT" => {
                if parts.len() > 1 {
                    // Mock connection
                    for (i, db) in self.databases.iter().enumerate() {
                        if db.name == parts[1] {
                            self.current_db = Some(i);
                            break;
                        }
                    }
                }
            }
            "DESCRIBE" | "DESC" => {
                if parts.len() > 1 {
                    let table_name = parts[1];
                    if let Some(table) = self.schema.tables.iter().find(|t| t.name == table_name) {
                        self.describe_table(table);
                    }
                }
            }
            "EXPORT" => {
                // Mock CSV export
                self.export_results();
            }
            "SELECT" => {
                // Execute SQL query
                let query = parts.join(" ");
                self.query_editor.set_query(&query);
                self.execute_query();
            }
            _ => {}
        }
    }

    fn execute_query(&mut self) {
        let sql = self.query_editor.get_query();
        let start = Instant::now();

        // Mock execution
        let execution_time = start.elapsed().as_secs_f64() * 1000.0 + 12.5; // Mock time

        // Add to history
        self.history.push_back(Query {
            sql: sql.clone(),
            timestamp: Local::now(),
            execution_time_ms: execution_time,
            row_count: self.results.rows.len(),
        });

        if self.history.len() > 50 {
            self.history.pop_front();
        }

        // Results already populated in mock data
        self.view_mode = ViewMode::Results;
    }

    fn describe_table(&mut self, table: &SchemaTable) {
        let columns = vec![
            "Column".to_string(),
            "Type".to_string(),
            "Nullable".to_string(),
        ];
        let rows: Vec<Vec<String>> = table
            .columns
            .iter()
            .map(|col| {
                vec![
                    col.name.clone(),
                    col.data_type.clone(),
                    if col.nullable { "YES" } else { "NO" }.to_string(),
                ]
            })
            .collect();

        self.results.set_data(columns, rows, 0.5);
        self.view_mode = ViewMode::Results;
    }

    fn export_results(&self) {
        // Mock implementation - would write CSV file
    }

    fn draw(&mut self) -> io::Result<()> {
        self.locust.terminal_mut().draw(|f| {
            let area = f.area();

            // Main layout: top (schema + editor), bottom (results)
            let main_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(area);

            // Top layout: schema + editor
            let top_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
                .split(main_chunks[0]);

            // Draw panels
            self.draw_schema(f, top_chunks[0]);
            self.draw_editor(f, top_chunks[1]);
            self.draw_results(f, main_chunks[1]);

            // Draw status bar
            self.draw_status_bar(f, area);

            // Draw tour if active
            if self.tour_active {
                self.draw_tour(f, area);
            }
        })?;

        Ok(())
    }

    fn draw_schema(&mut self, f: &mut Frame, area: Rect) {
        let border_style = if self.view_mode == ViewMode::Schema {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().fg(Color::Gray)
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .title(" Schema ")
            .border_style(border_style);

        let items: Vec<ListItem> = self
            .schema
            .all_items()
            .iter()
            .map(|item| match item {
                SchemaItem::Section(name) => ListItem::new(Line::from(Span::styled(
                    name,
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ))),
                SchemaItem::Table(table) => {
                    ListItem::new(format!("  📊 {} ({} rows)", table.name, table.row_count))
                }
                SchemaItem::View(view) => ListItem::new(format!("  👁  {}", view.name)),
                SchemaItem::Index(index) => {
                    let unique = if index.unique { "UNIQUE" } else { "" };
                    ListItem::new(format!("  🔑 {} {}", index.name, unique))
                }
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

        f.render_stateful_widget(list, area, &mut self.schema_state);
    }

    fn draw_editor(&self, f: &mut Frame, area: Rect) {
        let border_style = if self.view_mode == ViewMode::Editor {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().fg(Color::Gray)
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .title(" Query Editor [Ctrl+E to execute] ")
            .border_style(border_style);

        let inner = block.inner(area);
        f.render_widget(block, area);

        // Render query lines with syntax highlighting
        let lines: Vec<Line> = self
            .query_editor
            .lines
            .iter()
            .enumerate()
            .map(|(i, line)| {
                let prefix = format!("{:3} │ ", i + 1);
                let mut spans = vec![Span::styled(prefix, Style::default().fg(Color::DarkGray))];

                // Simple syntax highlighting
                let words: Vec<&str> = line.split_whitespace().collect();
                for (j, word) in words.iter().enumerate() {
                    let style = match word.to_uppercase().as_str() {
                        "SELECT" | "FROM" | "WHERE" | "ORDER" | "BY" | "LIMIT" | "JOIN" | "AND"
                        | "OR" => Style::default().fg(Color::Cyan),
                        _ => Style::default().fg(Color::White),
                    };

                    spans.push(Span::styled(*word, style));
                    if j < words.len() - 1 {
                        spans.push(Span::raw(" "));
                    }
                }

                Line::from(spans)
            })
            .collect();

        let paragraph = Paragraph::new(lines).wrap(Wrap { trim: false });
        f.render_widget(paragraph, inner);
    }

    fn draw_results(&mut self, f: &mut Frame, area: Rect) {
        let border_style = if self.view_mode == ViewMode::Results {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().fg(Color::Gray)
        };

        let title = format!(
            " Results ({} rows, {:.2}ms) ",
            self.results.rows.len(),
            self.results.execution_time_ms
        );

        let block = Block::default()
            .borders(Borders::ALL)
            .title(title)
            .border_style(border_style);

        if self.results.columns.is_empty() {
            let text = Paragraph::new("No results yet. Execute a query with Ctrl+E.")
                .block(block)
                .alignment(Alignment::Center);
            f.render_widget(text, area);
            return;
        }

        let header_cells = self
            .results
            .columns
            .iter()
            .map(|h| Cell::from(h.as_str()).style(Style::default().fg(Color::Yellow)));
        let header = Row::new(header_cells)
            .style(Style::default().add_modifier(Modifier::BOLD))
            .height(1);

        let rows = self
            .results
            .rows
            .iter()
            .skip(self.results.scroll_offset)
            .enumerate()
            .map(|(i, row)| {
                let cells = row.iter().map(|c| Cell::from(c.as_str()));
                let style = if i + self.results.scroll_offset == self.results.selected_row {
                    Style::default().bg(Color::DarkGray)
                } else {
                    Style::default()
                };
                Row::new(cells).style(style).height(1)
            });

        let widths: Vec<Constraint> = self
            .results
            .columns
            .iter()
            .map(|_| Constraint::Percentage(100 / self.results.columns.len() as u16))
            .collect();

        let table = Table::new(rows, widths)
            .header(header)
            .block(block)
            .column_spacing(1);

        f.render_widget(table, area);
    }

    fn draw_status_bar(&self, f: &mut Frame, area: Rect) {
        let status_area = Rect {
            x: area.x,
            y: area.y + area.height - 1,
            width: area.width,
            height: 1,
        };

        let db_info = if let Some(idx) = self.current_db {
            let db = &self.databases[idx];
            format!("{} {} {}", db.db_type.icon(), db.db_type.as_str(), db.name)
        } else {
            "No connection".to_string()
        };

        let fps = format!("{:.1} FPS", self.fps_counter.fps());
        let status = format!("{} | History: {} | {}", db_info, self.history.len(), fps);
        let help =
            "Tab: switch panel | Ctrl+E: execute | Ctrl+P: commands | 't': tour | Ctrl+C: quit";

        let status_text = format!("{} | {}", status, help);
        let status_widget = Paragraph::new(status_text)
            .style(Style::default().bg(Color::DarkGray).fg(Color::White));

        f.render_widget(status_widget, status_area);
    }

    fn draw_tour(&self, f: &mut Frame, area: Rect) {
        let messages = [
            "Welcome to Database Tool! Press 'n' to continue.",
            "Browse schema in the left panel (tables, views, indexes)",
            "Write queries in the editor (middle panel)",
            "Execute queries with Ctrl+E",
            "Navigate results with arrow keys in the bottom panel",
            "Use Ctrl+P for commands. Press 'q' to exit tour.",
        ];

        let popup_area = common::centered_rect(60, 30, area);
        let message = messages.get(self.tour_step).unwrap_or(&messages[0]);

        let tour_widget = Paragraph::new(*message)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Database Tutorial ")
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

    let mut app = DatabaseTool::new(terminal)?;
    let result = app.run();

    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen)?;

    result
}
