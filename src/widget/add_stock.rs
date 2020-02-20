use super::block;

use tui::buffer::Buffer;
use tui::layout::{Alignment, Rect};
use tui::style::{Color, Modifier, Style};
use tui::widgets::{Paragraph, Text, Widget};

pub struct AddStockWidget {
    search_string: String,
    has_user_input: bool,
    error_msg: Option<String>,
}

impl AddStockWidget {
    pub fn new() -> AddStockWidget {
        AddStockWidget {
            search_string: String::new(),
            has_user_input: false,
            error_msg: Some(String::new()),
        }
    }

    pub fn add_char(&mut self, c: char) {
        self.search_string.push(c);
        self.has_user_input = true;
    }

    pub fn del_char(&mut self) {
        self.search_string.pop();
    }

    pub fn reset(&mut self) {
        self.search_string.drain(..);
        self.has_user_input = false;
        self.error_msg = None;
    }

    pub fn enter(&mut self) -> super::StockWidget {
        super::StockWidget::new(self.search_string.clone())
    }
}

impl Widget for AddStockWidget {
    fn draw(&mut self, area: Rect, buf: &mut Buffer) {
        let text = if !self.has_user_input && self.error_msg.is_some() {
            [
                Text::raw("> "),
                Text::styled(
                    self.error_msg.as_ref().unwrap(),
                    Style::default().modifier(Modifier::BOLD).fg(Color::Red),
                ),
            ]
        } else {
            [
                Text::raw("> "),
                Text::styled(
                    &self.search_string,
                    Style::default().modifier(Modifier::BOLD).fg(Color::Cyan),
                ),
            ]
        };

        Paragraph::new(text.iter())
            .block(block::new(" Add Ticker "))
            .style(Style::default())
            .alignment(Alignment::Left)
            .wrap(true)
            .draw(area, buf);
    }
}
