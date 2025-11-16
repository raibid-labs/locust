use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use locust::core::theme_manager::ThemeManager;
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame, Terminal,
};
use std::io;
use std::path::Path;

struct App {
    theme_manager: ThemeManager,
    selected_index: usize,
}

impl App {
    fn new() -> Self {
        let mut theme_manager = ThemeManager::new();

        // Try to load themes from themes/ directory
        if Path::new("themes").exists() {
            if let Ok(manager) = ThemeManager::load_themes_from_dir(Path::new("themes")) {
                theme_manager = manager;
            }
        }

        Self {
            theme_manager,
            selected_index: 0,
        }
    }

    fn next_theme(&mut self) {
        let themes = self.theme_manager.list_themes();
        if !themes.is_empty() {
            self.selected_index = (self.selected_index + 1) % themes.len();
            let theme_name = themes[self.selected_index].to_string();
            let _ = self.theme_manager.set_theme(&theme_name);
        }
    }

    fn previous_theme(&mut self) {
        let themes = self.theme_manager.list_themes();
        if !themes.is_empty() {
            if self.selected_index == 0 {
                self.selected_index = themes.len() - 1;
            } else {
                self.selected_index -= 1;
            }
            let theme_name = themes[self.selected_index].to_string();
            let _ = self.theme_manager.set_theme(&theme_name);
        }
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

    // Run app
    let res = run_app(&mut terminal, &mut app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("Error: {:?}", err);
    }

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, app))?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => return Ok(()),
                KeyCode::Right | KeyCode::Char('n') => app.next_theme(),
                KeyCode::Left | KeyCode::Char('p') => app.previous_theme(),
                _ => {}
            }
        }
    }
}

fn ui<B: Backend>(f: &mut Frame, app: &App) {
    let theme = app.theme_manager.get_current_theme();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints(
            [
                Constraint::Length(3),
                Constraint::Min(10),
                Constraint::Length(3),
            ]
            .as_ref(),
        )
        .split(f.area());

    // Title
    let title = Paragraph::new("Theme Switcher Demo")
        .style(theme.styles.focused.to_style())
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(theme.styles.highlight_border.to_style()),
        );
    f.render_widget(title, chunks[0]);

    // Theme list
    let themes: Vec<ListItem> = app
        .theme_manager
        .list_themes()
        .iter()
        .enumerate()
        .map(|(i, name)| {
            let style = if i == app.selected_index {
                theme.styles.selected.to_style()
            } else {
                theme.styles.normal.to_style()
            };
            ListItem::new(name.to_string()).style(style)
        })
        .collect();

    let themes_list = List::new(themes).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Available Themes")
            .border_style(theme.styles.focused.to_style()),
    );
    f.render_widget(themes_list, chunks[1]);

    // Color preview
    let current_theme = app.theme_manager.get_current_theme();
    let color_lines = vec![
        Line::from(vec![
            Span::styled("Background: ", theme.styles.normal.to_style()),
            Span::styled(
                "████",
                Style::default().bg(current_theme.colors.background.to_color()),
            ),
        ]),
        Line::from(vec![
            Span::styled("Foreground: ", theme.styles.normal.to_style()),
            Span::styled(
                "████",
                Style::default().bg(current_theme.colors.foreground.to_color()),
            ),
        ]),
        Line::from(vec![
            Span::styled("Primary:    ", theme.styles.normal.to_style()),
            Span::styled(
                "████",
                Style::default().bg(current_theme.colors.primary.to_color()),
            ),
        ]),
        Line::from(vec![
            Span::styled("Secondary:  ", theme.styles.normal.to_style()),
            Span::styled(
                "████",
                Style::default().bg(current_theme.colors.secondary.to_color()),
            ),
        ]),
        Line::from(vec![
            Span::styled("Accent:     ", theme.styles.normal.to_style()),
            Span::styled(
                "████",
                Style::default().bg(current_theme.colors.accent.to_color()),
            ),
        ]),
        Line::from(vec![
            Span::styled("Success:    ", theme.styles.normal.to_style()),
            Span::styled(
                "████",
                Style::default().bg(current_theme.colors.success.to_color()),
            ),
        ]),
        Line::from(vec![
            Span::styled("Warning:    ", theme.styles.normal.to_style()),
            Span::styled(
                "████",
                Style::default().bg(current_theme.colors.warning.to_color()),
            ),
        ]),
        Line::from(vec![
            Span::styled("Error:      ", theme.styles.normal.to_style()),
            Span::styled(
                "████",
                Style::default().bg(current_theme.colors.error.to_color()),
            ),
        ]),
    ];

    let color_preview = Paragraph::new(color_lines).block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!("Color Preview: {}", current_theme.name))
            .border_style(theme.styles.focused.to_style()),
    );

    // Split the middle section
    let middle_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(chunks[1]);

    f.render_widget(themes_list, middle_chunks[0]);
    f.render_widget(color_preview, middle_chunks[1]);

    // Help
    let help = Paragraph::new("← / p: Previous  |  → / n: Next  |  q: Quit")
        .style(theme.styles.hint.to_style())
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(help, chunks[2]);
}
