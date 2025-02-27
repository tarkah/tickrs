use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Rect};
use ratatui::style::Modifier;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, StatefulWidget, Widget, Wrap};

use super::block;
use crate::common::ChartType;
use crate::theme::style;
use crate::THEME;

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

    pub fn enter(&mut self, chart_type: ChartType) -> super::StockState {
        super::StockState::new(self.search_string.clone().to_ascii_uppercase(), chart_type)
    }
}

pub struct AddStockWidget {}

impl StatefulWidget for AddStockWidget {
    type State = AddStockState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let spans = if !state.has_user_input && state.error_msg.is_some() {
            Line::from(vec![
                Span::styled("> ", style().fg(THEME.text_normal())),
                Span::styled(
                    state.error_msg.as_ref().unwrap(),
                    style().add_modifier(Modifier::BOLD).fg(THEME.loss()),
                ),
            ])
        } else {
            Line::from(vec![
                Span::styled("> ", style().fg(THEME.text_normal())),
                Span::styled(
                    &state.search_string,
                    style()
                        .add_modifier(Modifier::BOLD)
                        .fg(THEME.text_secondary()),
                ),
            ])
        };

        Paragraph::new(spans)
            .block(block::new(" Add Ticker "))
            .style(style())
            .alignment(Alignment::Left)
            .wrap(Wrap { trim: true })
            .render(area, buf);
    }
}
