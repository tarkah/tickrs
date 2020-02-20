use tui::style::{Color, Style};
use tui::widgets::{Block, Borders};

pub fn new(title: &str) -> Block {
    Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Blue))
        .title(title)
        .title_style(Style::default())
}
