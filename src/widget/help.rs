use tui::buffer::Buffer;
use tui::layout::Rect;
use tui::style::Style;
use tui::text::{Span, Spans};
use tui::widgets::{Paragraph, Widget};

use super::block;
use crate::draw::{add_padding, PaddingDirection};
use crate::THEME;

const TEXT: &str = r#"
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
Toggle Options Pane:
  - o: toggle pane
  - <Escape>: close pane
  - <Tab>: toggle calls / puts
  - Navigate with arrow keys
  - Cryptocurrency not supported
Toggle Summary Pane:
  - s: toggle pane
  - <Up / Down>: scroll pane
Graphing Display:
  - p: toggle pre / post market
  - v: toggle volumes graph
  - x: toggle labels
"#;

pub const HELP_WIDTH: u16 = 37;
pub const HELP_HEIGHT: u16 = 33;

#[derive(Copy, Clone)]
pub struct HelpWidget {}

impl HelpWidget {
    pub fn get_rect(self, area: Rect) -> Rect {
        Rect {
            x: (area.width - HELP_WIDTH) / 2,
            y: (area.height - HELP_HEIGHT) / 2,
            width: HELP_WIDTH,
            height: HELP_HEIGHT,
        }
    }
}

impl Widget for HelpWidget {
    fn render(self, mut area: Rect, buf: &mut Buffer) {
        block::new(" Help - <ESC> to go back ").render(area, buf);
        area = add_padding(area, 1, PaddingDirection::All);
        area = add_padding(area, 1, PaddingDirection::Left);

        let text: Vec<_> = TEXT
            .lines()
            .map(|line| {
                Spans::from(Span::styled(
                    format!("{}\n", line),
                    Style::default().fg(THEME.text_normal()),
                ))
            })
            .collect();

        Paragraph::new(text).render(area, buf);
    }
}
