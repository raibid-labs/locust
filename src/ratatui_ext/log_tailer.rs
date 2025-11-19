use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget, Wrap},
};
use std::{
    fs::File,
    io::{self, Read},
    path::PathBuf,
};

/// A widget that displays the tail of a log file.
pub struct LogTailer {
    log_file_path: PathBuf,
    lines: Vec<String>,
    max_lines: usize,
}

impl LogTailer {
    /// Creates a new `LogTailer` instance.
    ///
    /// `log_file_path`: The path to the log file to tail.
    /// `max_lines`: The maximum number of lines to display from the tail.
    pub fn new(log_file_path: PathBuf, max_lines: usize) -> Self {
        Self {
            log_file_path,
            lines: Vec::new(),
            max_lines,
        }
    }

    /// Reads the tail of the log file and updates the internal buffer.
    pub fn read_tail(&mut self) -> io::Result<()> {
        let mut file = File::open(&self.log_file_path)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;

        self.lines = content
            .lines()
            .map(String::from)
            .collect::<Vec<String>>()
            .into_iter()
            .rev()
            .take(self.max_lines)
            .rev()
            .collect();

        Ok(())
    }
}

impl Widget for &mut LogTailer {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title(" Log ")
            .border_style(Style::default().fg(Color::DarkGray));

        let inner_area = block.inner(area);
        block.render(area, buf);

        let lines: Vec<Line> = self
            .lines
            .iter()
            .map(|s| Line::from(Span::raw(s)))
            .collect();

        let paragraph = Paragraph::new(lines).wrap(Wrap { trim: true });
        paragraph.render(inner_area, buf);
    }
}
