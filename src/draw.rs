use tui::backend::Backend;
use tui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use tui::style::{Color, Style};
use tui::text::{Span, Spans, Text};
use tui::widgets::{Block, Borders, Paragraph, Tabs, Wrap};
use tui::{Frame, Terminal};

use crate::app::{App, Mode, ScrollDirection};
use crate::common::TimeFrame;
use crate::widget::{
    block, AddStockWidget, OptionsWidget, StockSummaryWidget, StockWidget, HELP_HEIGHT, HELP_WIDTH,
};
use crate::SHOW_VOLUMES;

pub fn draw<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) {
    let current_size = terminal.size().unwrap_or_default();

    if current_size.width <= 10 || current_size.height <= 10 {
        return;
    }

    if app.debug.enabled {
        app.debug.dimensions = (current_size.width, current_size.height);
    }

    terminal
        .draw(|mut frame| {
            if app.debug.enabled && app.mode == Mode::AddStock {
                // layout[0] - Main window
                // layout[1] - Add Stock window
                // layout[2] - Debug window
                let layout = Layout::default()
                    .constraints(
                        [
                            Constraint::Min(0),
                            Constraint::Length(3),
                            Constraint::Length(5),
                        ]
                        .as_ref(),
                    )
                    .split(frame.size());

                if !app.stocks.is_empty() {
                    match app.previous_mode {
                        Mode::DisplaySummary => draw_summary(&mut frame, app, layout[0]),
                        _ => draw_main(&mut frame, app, layout[0]),
                    }
                }

                draw_add_stock(&mut frame, app, layout[1]);
                draw_debug(&mut frame, app, layout[2]);
            } else if app.debug.enabled {
                // layout[0] - Main window
                // layout[1] - Debug window
                let layout = Layout::default()
                    .constraints([Constraint::Min(0), Constraint::Length(5)].as_ref())
                    .split(frame.size());

                match app.mode {
                    Mode::DisplaySummary => draw_summary(&mut frame, app, layout[0]),
                    Mode::Help => draw_help(&mut frame, app, layout[0]),
                    _ => draw_main(&mut frame, app, layout[0]),
                }

                draw_debug(&mut frame, app, layout[1]);
            } else if app.mode == Mode::AddStock {
                // layout[0] - Main window
                // layout[1] - Add Stock window
                let layout = Layout::default()
                    .constraints([Constraint::Min(0), Constraint::Length(3)].as_ref())
                    .split(frame.size());

                if !app.stocks.is_empty() {
                    match app.previous_mode {
                        Mode::DisplaySummary => draw_summary(&mut frame, app, layout[0]),
                        _ => draw_main(&mut frame, app, layout[0]),
                    }
                }

                draw_add_stock(&mut frame, app, layout[1]);
            } else {
                // layout - Main window
                let layout = frame.size();

                match app.mode {
                    Mode::DisplaySummary => draw_summary(&mut frame, app, layout),
                    Mode::Help => draw_help(&mut frame, app, layout),
                    _ => draw_main(&mut frame, app, layout),
                }
            };
        })
        .unwrap();
}

fn draw_main<B: Backend>(frame: &mut Frame<B>, app: &mut App, area: Rect) {
    // layout[0] - Header
    // layout[1] - Main widget
    let mut layout = Layout::default()
        .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
        .split(area);

    if !app.stocks.is_empty() {
        frame.render_widget(crate::widget::block::new(" Tabs ", None), layout[0]);
        layout[0] = add_padding(layout[0], 1, PaddingDirection::All);

        // header[0] - Stock symbol tabs
        // header[1] - (Optional) help icon
        let header = if app.hide_help {
            vec![layout[0]]
        } else {
            Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Min(0), Constraint::Length(10)].as_ref())
                .split(layout[0])
        };

        // Draw tabs
        {
            let tabs: Vec<_> = app.stocks.iter().map(|w| Spans::from(w.symbol())).collect();

            frame.render_widget(
                Tabs::new(tabs)
                    .select(app.current_tab)
                    .style(Style::default().fg(Color::Cyan))
                    .highlight_style(Style::default().fg(Color::Yellow)),
                header[0],
            );
        }

        // Draw help icon
        if !app.hide_help {
            frame.render_widget(
                Paragraph::new(Text::styled("Help '?'", Style::default()))
                    .style(Style::reset())
                    .alignment(Alignment::Center),
                header[1],
            );
        }
    }

    // Draw main widget
    if let Some(stock) = app.stocks.get_mut(app.current_tab) {
        // main_chunks[0] - Stock widget
        // main_chunks[1] - Options widget (optional)
        let mut main_chunks = if app.mode == Mode::DisplayOptions {
            Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Min(0), Constraint::Length(44)].as_ref())
                .split(layout[1])
        } else {
            vec![layout[1]]
        };

        // If width is too small, don't render stock widget and use entire space
        // for options widget
        if main_chunks[0].width >= 19 {
            frame.render_stateful_widget(StockWidget {}, main_chunks[0], stock);
        } else {
            main_chunks[1] = layout[1];
        }

        if let Some(options) = stock.options.as_mut() {
            if main_chunks[1].width >= 44 && main_chunks[1].height >= 14 {
                frame.render_stateful_widget(OptionsWidget {}, main_chunks[1], options);
            } else {
                main_chunks[1] = add_padding(main_chunks[1], 1, PaddingDirection::Left);
                main_chunks[1] = add_padding(main_chunks[1], 1, PaddingDirection::Top);

                frame.render_widget(
                    Paragraph::new(Text::styled(
                        "Increase screen size to display options",
                        Style::default(),
                    )),
                    main_chunks[1],
                );
            }
        }
    }
}

fn draw_add_stock<B: Backend>(frame: &mut Frame<B>, app: &mut App, area: Rect) {
    frame.render_stateful_widget(AddStockWidget {}, area, &mut app.add_stock);
}

fn draw_summary<B: Backend>(frame: &mut Frame<B>, app: &mut App, mut area: Rect) {
    let border = block::new(" Summary ", None);
    frame.render_widget(border, area);
    area = add_padding(area, 1, PaddingDirection::All);
    area = add_padding(area, 1, PaddingDirection::Right);

    let show_volumes = *SHOW_VOLUMES.read().unwrap();
    let stock_widget_height = if show_volumes { 7 } else { 6 };

    let height = area.height;
    let num_to_render = (((height - 3) / stock_widget_height) as usize).min(app.stocks.len());

    // If the user queued an up / down scroll, calculate the new offset, store it in
    // state and use it for this render. Otherwise use stored offset from state.
    let mut scroll_offset = if let Some(direction) = app.summary_scroll_state.queued_scroll.take() {
        let new_offset = match direction {
            ScrollDirection::Up => {
                if app.summary_scroll_state.offset == 0 {
                    0
                } else {
                    (app.summary_scroll_state.offset - 1).min(app.stocks.len())
                }
            }
            ScrollDirection::Down => {
                (app.summary_scroll_state.offset + 1).min(app.stocks.len() - num_to_render)
            }
        };

        app.summary_scroll_state.offset = new_offset;

        new_offset
    } else {
        app.summary_scroll_state.offset
    };

    // If we resize the app up, adj the offset
    if num_to_render + scroll_offset > app.stocks.len() {
        scroll_offset -= (num_to_render + scroll_offset) - app.stocks.len();
        app.summary_scroll_state.offset = scroll_offset;
    }

    // layouy[0] - Header
    // layouy[1] - Summary window
    // layouy[2] - Empty
    let mut layout = Layout::default()
        .constraints(
            [
                Constraint::Length(1),
                Constraint::Length((num_to_render * stock_widget_height as usize) as u16),
                Constraint::Min(0),
            ]
            .as_ref(),
        )
        .split(area);

    // header[0]
    // header[1] - (Optional) help icon
    let header = if app.hide_help {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(0)].as_ref())
            .split(layout[0])
    } else {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(0), Constraint::Length(8)].as_ref())
            .split(layout[0])
    };

    // Draw help icon
    if !app.hide_help {
        frame.render_widget(
            Paragraph::new(Text::styled("Help '?'", Style::default()))
                .style(Style::reset())
                .alignment(Alignment::Center),
            header[1],
        );
    }

    let contraints = app.stocks[scroll_offset..num_to_render + scroll_offset]
        .iter()
        .map(|_| Constraint::Length(stock_widget_height))
        .collect::<Vec<_>>();

    let stock_layout = Layout::default().constraints(contraints).split(layout[1]);

    for (idx, stock) in app.stocks[scroll_offset..num_to_render + scroll_offset]
        .iter_mut()
        .enumerate()
    {
        frame.render_stateful_widget(StockSummaryWidget {}, stock_layout[idx], stock);
    }

    // Draw time frame & paging
    {
        layout[2] = add_padding(layout[2], 1, PaddingDirection::Left);

        let offset = layout[2].height - 2;
        layout[2] = add_padding(layout[2], offset, PaddingDirection::Top);

        frame.render_widget(
            Block::default()
                .borders(Borders::TOP)
                .border_style(Style::reset()),
            layout[2],
        );

        layout[2] = add_padding(layout[2], 1, PaddingDirection::Top);

        let time_frames = TimeFrame::tab_names()
            .iter()
            .map(|s| Spans::from(*s))
            .collect::<Vec<_>>();

        // botton_layout[0] - time frame
        // botton_layout[1] - paging indicator
        let bottom_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(0), Constraint::Length(3)].as_ref())
            .split(layout[2]);

        let tabs = Tabs::new(time_frames)
            .select(app.summary_time_frame.idx())
            .style(Style::default().fg(Color::Cyan))
            .highlight_style(Style::default().fg(Color::Yellow));

        frame.render_widget(tabs, bottom_layout[0]);

        let more_up = scroll_offset > 0;
        let more_down = scroll_offset + num_to_render < app.stocks.len();

        let up_arrow = Span::styled(
            "ᐱ",
            Style::default().fg(if more_up {
                Color::Reset
            } else {
                Color::DarkGray
            }),
        );
        let down_arrow = Span::styled(
            "ᐯ",
            Style::default().fg(if more_down {
                Color::Reset
            } else {
                Color::DarkGray
            }),
        );

        frame.render_widget(
            Paragraph::new(Spans::from(vec![up_arrow, Span::raw(" "), down_arrow])),
            bottom_layout[1],
        );
    }
}

fn draw_help<B: Backend>(frame: &mut Frame<B>, app: &mut App, area: Rect) {
    let mut layout = area;

    if layout.width < HELP_WIDTH || layout.height < HELP_HEIGHT {
        frame.render_widget(
            Paragraph::new(Text::styled(
                "Increase screen size to display help",
                Style::default(),
            )),
            layout,
        );
    } else {
        layout = app.help.get_rect(layout);

        frame.render_widget(app.help, layout);
    }
}

fn draw_debug<B: Backend>(frame: &mut Frame<B>, app: &mut App, area: Rect) {
    app.debug.mode = app.mode;

    let debug_text = Text::styled(format!("{:?}", app.debug), Style::default());
    let debug_paragraph = Paragraph::new(debug_text).wrap(Wrap { trim: true });

    frame.render_widget(debug_paragraph, area);
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
        PaddingDirection::All => {
            rect.y += n;
            rect.height -= n * 2;

            rect.x += n;
            rect.width -= n * 2;

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
    All,
}
