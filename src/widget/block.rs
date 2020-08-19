use tui::style::{Color, Style};
use tui::widgets::{Block, Borders};

pub fn new(title: &str, border_color: Option<Color>) -> Block {
    Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color.unwrap_or(Color::Blue)))
        .title(title)
        .title_style(Style::default())
}
