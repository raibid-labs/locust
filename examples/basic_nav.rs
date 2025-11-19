use crossterm::event::{self, Event, KeyCode};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use ratatui::backend::CrosstermBackend;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};
use std::io::{self, Stdout};
use std::fs::File;
use std::path::PathBuf;

use log::{debug, info, LevelFilter};
use simplelog::{CombinedLogger, Config, WriteLogger};

use locust::prelude::*;
use locust::core::targets::{TargetBuilder, TargetAction, TargetPriority};
use locust::ratatui_ext::LogTailer;

#[derive(Debug, PartialEq, Eq)]
enum FocusedPane {
    Left,
    Right,
}

fn main() -> io::Result<()> {
    // Initialize logger
    let log_file_path = PathBuf::from("locust.log");
    CombinedLogger::init(
        vec![
            WriteLogger::new(
                LevelFilter::Debug,
                Config::default(),
                File::create(&log_file_path).unwrap(),
            ),
        ]
    ).unwrap();
    debug!("Logger initialized.");

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    crossterm::execute!(
        stdout,
        crossterm::terminal::EnterAlternateScreen,
        crossterm::event::EnableMouseCapture
    )?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut locust = Locust::<CrosstermBackend<Stdout>>::new(LocustConfig::default());
    locust.register_plugin(NavPlugin::new());

    let mut left_text = String::from("Hello from left pane!");
    let mut left_cursor: usize = left_text.len();
    let mut right_text = String::from("Hello from right pane!");
    let mut right_cursor: usize = right_text.len();
    let mut focused_pane = FocusedPane::Left;

    let mut log_tailer = LogTailer::new(log_file_path, 10); // Display last 10 log lines

    'outer: loop {
        log_tailer.read_tail()?; // Update log tail at the beginning of each frame
        locust.begin_frame();
        terminal.draw(|f| {
            let size = f.area();
            let block = Block::default().title("Locust demo").borders(Borders::ALL);
            f.render_widget(block, size);

            let main_layout_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(0), Constraint::Length(12)].as_ref()) // Added space for log pane
                .split(size);

            let app_area = main_layout_chunks[0];
            let log_area = main_layout_chunks[1];

            let pane_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
                .split(app_area);

            let mut target_builder = TargetBuilder::new();

            // Left Pane: Text Input
            let left_block = Block::default()
                .title("Left Pane")
                .borders(Borders::ALL)
                .border_style(if focused_pane == FocusedPane::Left { Style::default().fg(Color::Green) } else { Style::default().fg(Color::White) });
            let left_inner_area = left_block.inner(pane_chunks[0]);
            let left_paragraph = Paragraph::new(left_text.as_str())
                .block(left_block);
            f.render_widget(left_paragraph, pane_chunks[0]);

            // Set cursor for left pane
            if focused_pane == FocusedPane::Left {
                f.set_cursor_position((
                    left_inner_area.x + left_cursor as u16,
                    left_inner_area.y,
                ));
            }

            // Register NavTarget for left text input
            let target = target_builder.custom(
                left_inner_area,
                "Left Text Input",
                TargetAction::Activate,
                TargetPriority::Normal,
            );
            locust.ctx.targets.register(target);

            // Right Pane: Text Input
            let right_block = Block::default()
                .title("Right Pane")
                .borders(Borders::ALL)
                .border_style(if focused_pane == FocusedPane::Right { Style::default().fg(Color::Green) } else { Style::default().fg(Color::White) });
            let right_inner_area = right_block.inner(pane_chunks[1]);
            let right_paragraph = Paragraph::new(right_text.as_str())
                .block(right_block);
            f.render_widget(right_paragraph, pane_chunks[1]);

            // Set cursor for right pane
            if focused_pane == FocusedPane::Right {
                f.set_cursor_position((
                    right_inner_area.x + right_cursor as u16,
                    right_inner_area.y,
                ));
            }

            // Register NavTarget for right text input
            let target = target_builder.custom(
                right_inner_area,
                "Right Text Input",
                TargetAction::Activate,
                TargetPriority::Normal,
            );
            locust.ctx.targets.register(target);

            // Render Log Tailer
            f.render_widget(&mut log_tailer, log_area);

            // Finally, let Locust render any overlays.
            locust.render_overlay(f);
        })?;

        if event::poll(std::time::Duration::from_millis(250))? {
            let ev = event::read()?;
            let outcome = locust.on_event(&ev);
            if !outcome.consumed {
                if let Event::Key(key) = ev {
                    match key.code {
                        KeyCode::Char('q') => break 'outer,
                        KeyCode::Tab => {
                            focused_pane = match focused_pane {
                                FocusedPane::Left => FocusedPane::Right,
                                FocusedPane::Right => FocusedPane::Left,
                            };
                            info!("Focused: {:?}", focused_pane);
                        }
                        KeyCode::Backspace => {
                            match focused_pane {
                                FocusedPane::Left => {
                                    if left_cursor > 0 {
                                        left_text.remove(left_cursor - 1);
                                        left_cursor -= 1;
                                    }
                                }
                                FocusedPane::Right => {
                                    if right_cursor > 0 {
                                        right_text.remove(right_cursor - 1);
                                        right_cursor -= 1;
                                    }
                                }
                            }
                            debug!("Backspace pressed.");
                        }
                        KeyCode::Left => {
                            match focused_pane {
                                FocusedPane::Left => {
                                    if left_cursor > 0 {
                                        left_cursor -= 1;
                                    }
                                }
                                FocusedPane::Right => {
                                    if right_cursor > 0 {
                                        right_cursor -= 1;
                                    }
                                }
                            }
                            debug!("Left arrow pressed.");
                        }
                        KeyCode::Right => {
                            match focused_pane {
                                FocusedPane::Left => {
                                    if left_cursor < left_text.len() {
                                        left_cursor += 1;
                                    }
                                }
                                FocusedPane::Right => {
                                    if right_cursor < right_text.len() {
                                        right_cursor += 1;
                                    }
                                }
                            }
                            debug!("Right arrow pressed.");
                        }
                        KeyCode::Char(c) => {
                            match focused_pane {
                                FocusedPane::Left => {
                                    left_text.insert(left_cursor, c);
                                    left_cursor += 1;
                                }
                                FocusedPane::Right => {
                                    right_text.insert(right_cursor, c);
                                    right_cursor += 1;
                                }
                            }
                            info!("Typed: {}", c);
                        }
                        _ => {
                            debug!("Unhandled key: {:?}", key.code);
                        }
                    }
                }
            }
        }
    }

    disable_raw_mode()?;
    crossterm::execute!(
        terminal.backend_mut(),
        crossterm::event::DisableMouseCapture,
        crossterm::terminal::LeaveAlternateScreen
    )?;
    terminal.show_cursor()?;
    Ok(())
}
