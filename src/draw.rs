use crate::app::{App, Mode};

use tui::backend::Backend;
use tui::layout::{Constraint, Layout};
use tui::style::{Color, Style};
use tui::widgets::{Tabs, Widget};
use tui::Terminal;

pub fn draw<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) {
    terminal
        .draw(|mut frame| {
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
            };

            if !app.stocks.is_empty() {
                let tabs: Vec<_> = app.stocks.iter().map(|w| w.symbol()).collect();

                Tabs::default()
                    .block(crate::widget::block::new(" Tabs "))
                    .titles(&tabs)
                    .select(app.current_tab)
                    .style(Style::default().fg(Color::Cyan))
                    .highlight_style(Style::default().fg(Color::Yellow))
                    .render(&mut frame, chunks[0]);
            }

            if let Some(stock) = app.stocks.get_mut(app.current_tab) {
                stock.render(&mut frame, chunks[1]);
            }

            if app.mode == Mode::AddStock {
                app.add_stock.render(&mut frame, chunks[2])
            }
        })
        .unwrap();
}
