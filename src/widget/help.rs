use tui::buffer::Buffer;
use tui::layout::{Constraint, Direction, Layout, Rect};
use tui::text::{Span, Spans};
use tui::widgets::{Paragraph, Widget};

use super::block;
use crate::draw::{add_padding, PaddingDirection};
use crate::theme::style;
use crate::THEME;

const LEFT_TEXT: &str = r#"
Quit: q or <Ctrl+c>
Add Stock:
  - /: open prompt
  - (while adding):
    - <Enter>: accept
    - <Escape>: quit
Remove Stock:
  - k: remove stock
Change Tab:
  - <Tab>: next stock
  - <Shift+Tab>: previous stock
Reorder Current Tab:
  - <Ctrl+Left>: move 1 tab left
  - <Ctrl+Right>: move 1 tab right
Change Time Frame:
  - <Right>: next time frame
  - <Left>: previous time frame
"#;

const RIGHT_TEXT: &str = r#"
Graphing Display:
  - c: toggle candlestick chart
  - p: toggle pre / post market
  - v: toggle volumes graph
  - x: toggle labels
Toggle Options Pane:
  - o: toggle pane
  - <Escape>: close pane
  - <Tab>: toggle calls / puts
  - Navigate with arrow keys
  - Cryptocurrency not supported
Toggle Summary Pane:
  - s: toggle pane
  - <Up / Down>: scroll pane
"#;

const LEFT_WIDTH: usize = 34;
const RIGHT_WIDTH: usize = 32;
pub const HELP_WIDTH: usize = 2 + LEFT_WIDTH + 2 + RIGHT_WIDTH + 2;
pub const HELP_HEIGHT: usize = 2 + 17 + 1;

#[derive(Copy, Clone)]
pub struct HelpWidget {}

impl HelpWidget {
    pub fn get_rect(self, area: Rect) -> Rect {
        Rect {
            x: (area.width - HELP_WIDTH as u16) / 2,
            y: (area.height - HELP_HEIGHT as u16) / 2,
            width: HELP_WIDTH as u16,
            height: HELP_HEIGHT as u16,
        }
    }
}

impl Widget for HelpWidget {
    fn render(self, mut area: Rect, buf: &mut Buffer) {
        block::new(" Help - <ESC> to go back ").render(area, buf);
        area = add_padding(area, 1, PaddingDirection::All);
        area = add_padding(area, 1, PaddingDirection::Left);

        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(LEFT_WIDTH as u16),
                Constraint::Length(2),
                Constraint::Length(RIGHT_WIDTH as u16),
            ])
            .split(area);

        let left_text: Vec<_> = LEFT_TEXT
            .lines()
            .map(|line| {
                Spans::from(Span::styled(
                    format!("{}\n", line),
                    style().fg(THEME.text_normal()),
                ))
            })
            .collect();

        let right_text: Vec<_> = RIGHT_TEXT
            .lines()
            .map(|line| {
                Spans::from(Span::styled(
                    format!("{}\n", line),
                    style().fg(THEME.text_normal()),
                ))
            })
            .collect();

        Paragraph::new(left_text).render(layout[0], buf);
        Paragraph::new(right_text).render(layout[2], buf);
    }
}
