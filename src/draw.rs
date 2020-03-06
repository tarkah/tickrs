use crate::app::{App, Mode};
use crate::widget::{AddStockWidget, StockWidget, HELP_HEIGHT, HELP_WIDTH};

use tui::backend::Backend;
use tui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use tui::style::{Color, Style};
use tui::widgets::{Paragraph, Tabs, Text};
use tui::Terminal;

pub fn draw<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) {
    terminal
        .draw(|mut frame| {
            // chunks[0] - Header
            // chunks[1] - Stock widget
            // chunks[2] - (Optional) Add Stock widget
            let chunks = match app.mode {
                Mode::AddStock => Layout::default()
                    .constraints(
                        [
                            Constraint::Length(3),
                            Constraint::Min(0),
                            Constraint::Length(3),
                        ]
                        .as_ref(),
                    )
                    .split(frame.size()),
                Mode::DisplayStock => Layout::default()
                    .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
                    .split(frame.size()),
                _ => vec![],
            };

            if !app.stocks.is_empty() {
                frame.render_widget(crate::widget::block::new(" Tabs "), chunks[0]);

                // header[0] - Stock symbol tabs
                // header[1] - (Optional) help icon
                let mut header = if app.hide_help {
                    vec![chunks[0]]
                } else {
                    Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints([Constraint::Min(0), Constraint::Length(10)].as_ref())
                        .split(chunks[0])
                };

                // Draw tabs
                {
                    header[0] = add_padding(header[0], 1, PaddingDirection::Top);
                    header[0] = add_padding(header[0], 1, PaddingDirection::Left);

                    let tabs: Vec<_> = app.stocks.iter().map(|w| w.symbol()).collect();

                    frame.render_widget(
                        Tabs::default()
                            .titles(&tabs)
                            .select(app.current_tab)
                            .style(Style::default().fg(Color::Cyan))
                            .highlight_style(Style::default().fg(Color::Yellow)),
                        header[0],
                    );
                }

                // Draw help icon
                if !app.hide_help {
                    header[1] = add_padding(header[1], 1, PaddingDirection::Top);
                    header[1] = add_padding(header[1], 2, PaddingDirection::Right);

                    frame.render_widget(
                        Paragraph::new([Text::raw("Help '?'")].iter())
                            .style(Style::default().fg(Color::White).bg(Color::Black))
                            .alignment(Alignment::Center)
                            .wrap(false),
                        header[1],
                    );
                }
            }

            // Draw stock widget
            if let Some(stock) = app.stocks.get_mut(app.current_tab) {
                frame.render_stateful_widget(StockWidget {}, chunks[1], stock);
            }

            // Draw add stock widget
            if app.mode == Mode::AddStock {
                frame.render_stateful_widget(AddStockWidget {}, chunks[2], &mut app.add_stock);
            }
        })
        .unwrap();
}

pub fn draw_help<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) {
    terminal
        .draw(|mut frame| {
            let mut rect = frame.size();

            if rect.width < HELP_WIDTH || rect.height < HELP_HEIGHT {
                frame.render_widget(
                    Paragraph::new([Text::raw("Increase screen size to display help")].iter()),
                    rect,
                );
            } else {
                rect = app.help.get_rect(frame.size());

                frame.render_widget(app.help, rect);
            }
        })
        .unwrap();
}

pub fn add_padding(mut rect: Rect, n: u16, direction: PaddingDirection) -> Rect {
    match direction {
        PaddingDirection::Top => {
            rect.y += n;
            rect.height -= n;
            rect
        }
        PaddingDirection::Bottom => {
            rect.height -= n;
            rect
        }
        PaddingDirection::Left => {
            rect.x += n;
            rect.width -= n;
            rect
        }
        PaddingDirection::Right => {
            rect.width -= n;
            rect
        }
    }
}

#[allow(dead_code)]
pub enum PaddingDirection {
    Top,
    Bottom,
    Left,
    Right,
}
