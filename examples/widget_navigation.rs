use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use locust::core::targets::TargetRegistry;
use locust::ratatui_ext::adapters::{
    NavigableList, NavigableTable, NavigableTabs, NavigableTree, TableNavMode, TreeNode,
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Row, Table, TableState, Tabs},
    Frame, Terminal,
};
use std::io;

/// Demo application showcasing all widget adapters
struct App {
    selected_tab: usize,
    list_state: ListState,
    table_state: TableState,
    tree_expanded: Vec<bool>,
    should_quit: bool,
}

impl App {
    fn new() -> Self {
        Self {
            selected_tab: 0,
            list_state: ListState::default().with_selected(Some(0)),
            table_state: TableState::default().with_selected(0),
            tree_expanded: vec![true, false, true, false],
            should_quit: false,
        }
    }

    fn on_key(&mut self, key: KeyCode) {
        match key {
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Left => {
                if self.selected_tab > 0 {
                    self.selected_tab -= 1;
                }
            }
            KeyCode::Right => {
                if self.selected_tab < 3 {
                    self.selected_tab += 1;
                }
            }
            KeyCode::Up => {
                if let Some(selected) = self.list_state.selected() {
                    if selected > 0 {
                        self.list_state.select(Some(selected - 1));
                    }
                }
                if let Some(selected) = self.table_state.selected() {
                    if selected > 0 {
                        self.table_state.select(Some(selected - 1));
                    }
                }
            }
            KeyCode::Down => {
                if let Some(selected) = self.list_state.selected() {
                    if selected < 4 {
                        self.list_state.select(Some(selected + 1));
                    }
                }
                if let Some(selected) = self.table_state.selected() {
                    if selected < 2 {
                        self.table_state.select(Some(selected + 1));
                    }
                }
            }
            KeyCode::Char(' ') => {
                if let Some(selected) = self.list_state.selected() {
                    if selected < self.tree_expanded.len() {
                        self.tree_expanded[selected] = !self.tree_expanded[selected];
                    }
                }
            }
            _ => {}
        }
    }

    fn draw_list_demo(&self, f: &mut Frame, area: Rect, registry: &mut TargetRegistry) {
        let items = vec![
            ListItem::new("Home - Go to main page"),
            ListItem::new("Settings - Configure app"),
            ListItem::new("Profile - View user profile"),
            ListItem::new("Help - Get assistance"),
            ListItem::new("Exit - Close application"),
        ];

        let list = List::new(items.clone())
            .block(Block::default().borders(Borders::ALL).title("List Demo"))
            .highlight_style(
                Style::default()
                    .bg(Color::Blue)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(">> ");

        let labels = vec![
            "Home".into(),
            "Settings".into(),
            "Profile".into(),
            "Help".into(),
            "Exit".into(),
        ];

        let nav_list = NavigableList::new(list, items.len()).with_labels(labels);
        nav_list.register_targets(area, registry);

        f.render_stateful_widget(
            nav_list.widget().clone(),
            area,
            &mut self.list_state.clone(),
        );
    }

    fn draw_table_demo(&self, f: &mut Frame, area: Rect, registry: &mut TargetRegistry) {
        let header = Row::new(vec!["Name", "Role", "Status"])
            .style(Style::default().fg(Color::Yellow))
            .bottom_margin(1);

        let rows = vec![
            Row::new(vec!["Alice", "Admin", "Active"]),
            Row::new(vec!["Bob", "User", "Active"]),
            Row::new(vec!["Carol", "Manager", "Away"]),
        ];

        let table = Table::new(
            rows,
            vec![
                Constraint::Percentage(33),
                Constraint::Percentage(33),
                Constraint::Percentage(34),
            ],
        )
        .header(header)
        .block(Block::default().borders(Borders::ALL).title("Table Demo"))
        .highlight_style(
            Style::default()
                .bg(Color::Blue)
                .add_modifier(Modifier::BOLD),
        );

        let nav_table = NavigableTable::new(table, 3, vec![15, 15, 15]).with_header();
        nav_table.register_targets(area, registry, TableNavMode::Row);

        f.render_stateful_widget(
            nav_table.widget().clone(),
            area,
            &mut self.table_state.clone(),
        );
    }

    fn draw_tabs_demo(&self, f: &mut Frame, area: Rect, registry: &mut TargetRegistry) {
        let titles = vec![
            "Overview".into(),
            "Details".into(),
            "History".into(),
            "Settings".into(),
        ];

        let tabs = Tabs::new(titles.clone())
            .block(Block::default().borders(Borders::ALL).title("Tabs Demo"))
            .select(self.selected_tab)
            .style(Style::default().fg(Color::White))
            .highlight_style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            );

        let nav_tabs = NavigableTabs::new(tabs, titles, self.selected_tab);
        nav_tabs.register_targets(area, registry);

        f.render_widget(nav_tabs.widget().clone(), area);
    }

    fn draw_tree_demo(&self, f: &mut Frame, area: Rect, registry: &mut TargetRegistry) {
        let nodes = vec![
            TreeNode {
                id: 1,
                label: "src/".into(),
                expanded: self.tree_expanded[0],
                level: 0,
                has_children: true,
            },
            TreeNode {
                id: 2,
                label: "core/".into(),
                expanded: self.tree_expanded[1],
                level: 1,
                has_children: true,
            },
            TreeNode {
                id: 3,
                label: "targets.rs".into(),
                expanded: false,
                level: 2,
                has_children: false,
            },
            TreeNode {
                id: 4,
                label: "ratatui_ext/".into(),
                expanded: self.tree_expanded[2],
                level: 1,
                has_children: true,
            },
            TreeNode {
                id: 5,
                label: "adapters.rs".into(),
                expanded: false,
                level: 2,
                has_children: false,
            },
            TreeNode {
                id: 6,
                label: "tests/".into(),
                expanded: self.tree_expanded[3],
                level: 0,
                has_children: true,
            },
        ];

        let tree = NavigableTree::new(nodes.clone());
        tree.register_targets(area, registry);

        // Manually render tree since there's no built-in ratatui tree widget
        let mut items = Vec::new();
        for node in &nodes {
            let indent = "  ".repeat(node.level);
            let icon = if node.has_children {
                if node.expanded {
                    "▼"
                } else {
                    "▶"
                }
            } else {
                " "
            };
            items.push(ListItem::new(format!("{}{} {}", indent, icon, node.label)));
        }

        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title("Tree Demo"))
            .highlight_style(
                Style::default()
                    .bg(Color::Blue)
                    .add_modifier(Modifier::BOLD),
            );

        f.render_stateful_widget(list, area, &mut self.list_state.clone());
    }

    fn draw_info(&self, f: &mut Frame, area: Rect) {
        let info = vec![
            Line::from(vec![
                Span::styled(
                    "Navigation: ",
                    Style::default().add_modifier(Modifier::BOLD),
                ),
                Span::raw("Arrow Keys"),
            ]),
            Line::from(vec![
                Span::styled(
                    "Toggle Tree: ",
                    Style::default().add_modifier(Modifier::BOLD),
                ),
                Span::raw("Space"),
            ]),
            Line::from(vec![
                Span::styled("Quit: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw("q"),
            ]),
            Line::from(""),
            Line::from(vec![Span::styled(
                "Target Registry Stats:",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )]),
        ];

        let paragraph = Paragraph::new(info).block(
            Block::default()
                .borders(Borders::ALL)
                .title("Widget Navigation Demo"),
        );

        f.render_widget(paragraph, area);
    }

    fn draw(&self, f: &mut Frame, registry: &mut TargetRegistry) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(6), // Info
                Constraint::Length(3), // Tabs
                Constraint::Min(0),    // Main content
            ])
            .split(f.area());

        // Draw info section
        self.draw_info(f, chunks[0]);

        // Draw tabs section
        self.draw_tabs_demo(f, chunks[1], registry);

        // Split main content area
        let main_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(25),
                Constraint::Percentage(50),
                Constraint::Percentage(25),
            ])
            .split(chunks[2]);

        // Draw based on selected tab
        match self.selected_tab {
            0 => {
                // Overview: List + Table + Tree
                self.draw_list_demo(f, main_chunks[0], registry);
                self.draw_table_demo(f, main_chunks[1], registry);
                self.draw_tree_demo(f, main_chunks[2], registry);
            }
            1 => {
                // Details: Focus on table
                let detail_chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                    .split(main_chunks[1]);

                self.draw_table_demo(f, detail_chunks[0], registry);
                self.draw_list_demo(f, main_chunks[0], registry);
                self.draw_tree_demo(f, main_chunks[2], registry);
            }
            2 => {
                // History: Focus on tree
                self.draw_tree_demo(f, main_chunks[1], registry);
                self.draw_list_demo(f, main_chunks[0], registry);
                self.draw_table_demo(f, main_chunks[2], registry);
            }
            3 => {
                // Settings: All three side by side
                self.draw_list_demo(f, main_chunks[0], registry);
                self.draw_table_demo(f, main_chunks[1], registry);
                self.draw_tree_demo(f, main_chunks[2], registry);
            }
            _ => {}
        }

        // Draw registry stats
        let stats_area = Rect::new(chunks[0].x + 2, chunks[0].y + 5, chunks[0].width - 4, 1);
        let stats = Paragraph::new(format!("Registered Targets: {}", registry.len()))
            .style(Style::default().fg(Color::Cyan));
        f.render_widget(stats, stats_area);
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

    // Main loop
    loop {
        // Create fresh registry for each frame
        let mut registry = TargetRegistry::new();

        // Draw UI
        terminal.draw(|f| {
            app.draw(f, &mut registry);
        })?;

        // Handle input
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                app.on_key(key.code);
            }
        }

        if app.should_quit {
            break;
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}
