use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use tui::backend::Backend;
use tui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use tui::style::{Color, Style};
use tui::widgets::{Block, Borders, Paragraph, Tabs, Text};
use tui::{Frame, Terminal};

use crate::app::{App, Mode};
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
    let layout = Layout::default()
        .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
        .split(area);

    if !app.stocks.is_empty() {
        frame.render_widget(crate::widget::block::new(" Tabs ", None), layout[0]);

        // header[0] - Stock symbol tabs
        // header[1] - (Optional) help icon
        let mut header = if app.hide_help {
            vec![layout[0]]
        } else {
            Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Min(0), Constraint::Length(10)].as_ref())
                .split(layout[0])
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
                Paragraph::new([Text::styled("Help '?'", Style::default())].iter())
                    .style(Style::default().fg(Color::White))
                    .alignment(Alignment::Center)
                    .wrap(false),
                header[1],
            );
        }
    }

    // Draw main widget
    if let Some(stock) = app.stocks.get_mut(app.current_tab) {
        // main_chunks[0] - Stock widget
        // main_chunks[1] - Options widget (optional)
        let main_chunks = if app.mode == Mode::DisplayOptions {
            Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Min(0), Constraint::Length(44)].as_ref())
                .split(layout[1])
        } else {
            vec![layout[1]]
        };

        let mut hasher = DefaultHasher::default();
        stock.hash(&mut hasher);
        let hash = hasher.finish();

        let cached_area = stock.cached_area;

        if hash == stock.prev_hash
            && cached_area.width == main_chunks[0].width
            && cached_area.height == main_chunks[0].height
        {
            stock.use_cache = true;
        } else if hash != stock.prev_hash {
            stock.prev_hash = hash;
        }

        frame.render_stateful_widget(StockWidget {}, main_chunks[0], stock);

        if let Some(options) = stock.options.as_mut() {
            let mut hasher = DefaultHasher::default();
            options.hash(&mut hasher);
            let hash = hasher.finish();

            let cached_area = options.cached_area;

            if hash == options.prev_hash
                && cached_area.width == main_chunks[1].width
                && cached_area.height == main_chunks[1].height
            {
                options.use_cache = true;
            } else if hash != options.prev_hash {
                options.prev_hash = hash;
            }

            frame.render_stateful_widget(OptionsWidget {}, main_chunks[1], options);
        }
    }
}

fn draw_add_stock<B: Backend>(frame: &mut Frame<B>, app: &mut App, area: Rect) {
    frame.render_stateful_widget(AddStockWidget {}, area, &mut app.add_stock);
}

fn draw_summary<B: Backend>(frame: &mut Frame<B>, app: &mut App, area: Rect) {
    let border = block::new(" Summary ", None);
    frame.render_widget(border, area);

    let show_volumes = *SHOW_VOLUMES.read().unwrap();
    let stock_widget_height = if show_volumes { 7 } else { 6 };

    let height = area.height;
    let num_to_render = (((height - 5) / stock_widget_height) as usize).min(app.stocks.len());

    // layouy[0] - Header
    // layouy[1] - Summary window
    // layouy[2] - Empty
    let mut layout = Layout::default()
        .constraints(
            [
                Constraint::Length(2),
                Constraint::Length((num_to_render * stock_widget_height as usize) as u16),
                Constraint::Min(0),
            ]
            .as_ref(),
        )
        .split(area);

    // header[0]
    // header[1] - (Optional) help icon
    let mut header = if app.hide_help {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(0)].as_ref())
            .split(layout[0])
    } else {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(0), Constraint::Length(10)].as_ref())
            .split(layout[0])
    };

    // Draw help icon
    if !app.hide_help {
        header[1] = add_padding(header[1], 1, PaddingDirection::Top);
        header[1] = add_padding(header[1], 2, PaddingDirection::Right);

        frame.render_widget(
            Paragraph::new([Text::styled("Help '?'", Style::default())].iter())
                .style(Style::default().fg(Color::White))
                .alignment(Alignment::Center),
            header[1],
        );
    }

    layout[1] = add_padding(layout[1], 1, PaddingDirection::Left);
    layout[1] = add_padding(layout[1], 2, PaddingDirection::Right);

    let contraints = app.stocks[..num_to_render]
        .iter()
        .map(|_| Constraint::Length(stock_widget_height))
        .collect::<Vec<_>>();

    let stock_layout = Layout::default().constraints(contraints).split(layout[1]);

    for (idx, stock) in app.stocks[..num_to_render].iter_mut().enumerate() {
        let mut hasher = DefaultHasher::default();
        stock.hash(&mut hasher);
        let hash = hasher.finish();

        let cached_area = stock.cached_area;

        if hash == stock.prev_hash
            && cached_area.width == stock_layout[idx].width
            && cached_area.height == stock_layout[idx].height
        {
            stock.use_cache = true;
        } else if hash != stock.prev_hash {
            stock.prev_hash = hash;
        }

        frame.render_stateful_widget(StockSummaryWidget {}, stock_layout[idx], stock);
    }

    // Draw time frame
    {
        layout[2] = add_padding(layout[2], 2, PaddingDirection::Left);
        layout[2] = add_padding(layout[2], 2, PaddingDirection::Right);

        // Clear out empty area
        frame.render_widget(
            Block::default()
                .borders(Borders::TOP)
                .border_style(Style::default().bg(Color::Black).fg(Color::Black)),
            layout[2],
        );

        let offset = layout[2].height - 3;
        layout[2] = add_padding(layout[2], offset, PaddingDirection::Top);

        frame.render_widget(Block::default().borders(Borders::TOP), layout[2]);

        layout[2] = add_padding(layout[2], 1, PaddingDirection::Top);

        let time_frames = TimeFrame::tab_names();

        let tabs = Tabs::default()
            //.block(Block::default().borders(Borders::NONE))
            .titles(&time_frames)
            .select(app.summary_time_frame.idx())
            .style(Style::default().fg(Color::Cyan))
            .highlight_style(Style::default().fg(Color::Yellow));

        frame.render_widget(tabs, layout[2]);
    }
}

fn draw_help<B: Backend>(frame: &mut Frame<B>, app: &mut App, area: Rect) {
    let mut layout = area;

    if layout.width < HELP_WIDTH || layout.height < HELP_HEIGHT {
        frame.render_widget(
            Paragraph::new(
                [Text::styled(
                    "Increase screen size to display help",
                    Style::default(),
                )]
                .iter(),
            ),
            layout,
        );
    } else {
        layout = app.help.get_rect(layout);

        frame.render_widget(app.help, layout);
    }
}

fn draw_debug<B: Backend>(frame: &mut Frame<B>, app: &mut App, area: Rect) {
    app.debug.mode = app.mode;

    let debug_text = [Text::styled(format!("{:?}", app.debug), Style::default())];
    let debug_paragraph = Paragraph::new(debug_text.iter()).wrap(true);

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
    }
}

#[allow(dead_code)]
pub enum PaddingDirection {
    Top,
    Bottom,
    Left,
    Right,
}
