use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Widget};

use super::block;
use crate::draw::{add_padding, PaddingDirection};
use crate::theme::style;
use crate::THEME;

const LEFT_TEXT: &str = r#"
GLOBAL
──────
Ctrl+C             Quit app
Esc / Q            Close window


MAIN SCREEN
───────────
E                  Open chart configuration
O                  Open options pane
/                  Open stock search
?                  Open help menu

C                  Toggle chart type
X                  Toggle date labels
P                  Toggle pre/post price
S                  Toggle summary view
V                  Toggle volume chart

Home / End         Select first/last stock
↑↓                 Select prev/next stock
Ctrl+↑↓            Move stock
R / Del            Remove stock

< / >              Scroll horizontally
Tab                Change time frame

Q / Esc            Quit app
"#;

const RIGHT_TEXT: &str = r#"
STOCK SEARCH
────────────
Enter              Submit
/                  Close window


CHART CONFIGURATION
───────────────────
↑↓                 Select
Enter              Submit
E                  Close window


OPTIONS PANE
────────────
↑↓                 Select
Tab                Toggle calls/puts
O                  Close window


HELP MENU
─────────
?                  Close
"#;

const LEFT_WIDTH: usize = 46;
const RIGHT_WIDTH: usize = 35;
pub const HELP_WIDTH: usize = 2 + LEFT_WIDTH + 2 + RIGHT_WIDTH + 2;
pub const HELP_HEIGHT: usize = 28;

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
        block::new(" Help ").render(area, buf);
        area = add_padding(area, 2, PaddingDirection::Left);
        area = add_padding(area, 1, PaddingDirection::Top);

        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(LEFT_WIDTH as u16),
                Constraint::Length(2),
                Constraint::Length(RIGHT_WIDTH as u16),
            ])
            .split(area);

        let left_text: Vec<_> = LEFT_TEXT[1..]
            .lines()
            .map(|line| {
                Line::from(Span::styled(
                    format!("{}\n", line),
                    style().fg(THEME.text_normal()),
                ))
            })
            .collect();

        let right_text: Vec<_> = RIGHT_TEXT[1..]
            .lines()
            .map(|line| {
                Line::from(Span::styled(
                    format!("{}\n", line),
                    style().fg(THEME.text_normal()),
                ))
            })
            .collect();

        Paragraph::new(left_text).render(layout[0], buf);
        Paragraph::new(right_text).render(layout[2], buf);
    }
}
