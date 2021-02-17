use tui::style::Style;
use tui::text::Span;
use tui::widgets::{Block, Borders};

use crate::THEME;

pub fn new(title: &str) -> Block {
    Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(THEME.border_primary))
        .title(Span::styled(title, Style::default().fg(THEME.text_normal)))
}
