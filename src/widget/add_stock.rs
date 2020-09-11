use super::block;

use tui::buffer::Buffer;
use tui::layout::{Alignment, Rect};
use tui::style::{Color, Modifier, Style};
use tui::widgets::{Paragraph, StatefulWidget, Text, Widget};

pub struct AddStockState {
    search_string: String,
    has_user_input: bool,
    error_msg: Option<String>,
}

impl AddStockState {
    pub fn new() -> AddStockState {
        AddStockState {
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

    pub fn enter(&mut self) -> super::StockState {
        super::StockState::new(self.search_string.clone())
    }
}

pub struct AddStockWidget {}

impl StatefulWidget for AddStockWidget {
    type State = AddStockState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let text = if !state.has_user_input && state.error_msg.is_some() {
            [
                Text::styled("> ", Style::default()),
                Text::styled(
                    state.error_msg.as_ref().unwrap(),
                    Style::default().modifier(Modifier::BOLD).fg(Color::Red),
                ),
            ]
        } else {
            [
                Text::styled("> ", Style::default()),
                Text::styled(
                    &state.search_string,
                    Style::default().modifier(Modifier::BOLD).fg(Color::Cyan),
                ),
            ]
        };

        Paragraph::new(text.iter())
            .block(block::new(" Add Ticker ", None))
            .style(Style::default())
            .alignment(Alignment::Left)
            .wrap(true)
            .render(area, buf);
    }
}
