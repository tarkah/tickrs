use ratatui::text::Span;
use ratatui::widgets::{Block, Borders};

use crate::theme::style;
use crate::THEME;

pub fn new(title: &str) -> Block {
    Block::default()
        .borders(Borders::ALL)
        .border_style(style().fg(THEME.border_primary()))
        .title(Span::styled(title, style().fg(THEME.text_normal())))
}
