use ratatui::backend::Backend;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Tabs, Wrap};
use ratatui::{Frame, Terminal};

use crate::app::{App, Mode, ScrollDirection};
use crate::common::{ChartType, TimeFrame};
use crate::service::Service;
use crate::theme::style;
use crate::widget::{
    block, AddStockWidget, ChartConfigurationWidget, OptionsWidget, StockSummaryWidget,
    StockWidget, HELP_HEIGHT, HELP_WIDTH,
};
use crate::{SHOW_VOLUMES, THEME};

pub fn draw(terminal: &mut Terminal<impl Backend>, app: &mut App) {
    let current_size = terminal.size().unwrap_or_default();

    if current_size.width <= 10 || current_size.height <= 10 {
        return;
    }

    if app.debug.enabled {
        app.debug.dimensions = (current_size.width, current_size.height);
    }

    terminal
        .draw(|frame| {
            // Set background color
            frame.render_widget(Block::default().style(style()), frame.size());

            if app.debug.enabled && app.mode == Mode::AddStock {
                // layout[0] - Main window
                // layout[1] - Add Stock window
                // layout[2] - Debug window
                let layout = Layout::default()
                    .constraints([
                        Constraint::Min(0),
                        Constraint::Length(3),
                        Constraint::Length(5),
                    ])
                    .split(frame.size());

                if !app.stocks.is_empty() {
                    match app.previous_mode {
                        Mode::DisplaySummary => draw_summary(frame, app, layout[0]),
                        _ => draw_main(frame, app, layout[0]),
                    }
                }

                draw_add_stock(frame, app, layout[1]);
                draw_debug(frame, app, layout[2]);
            } else if app.debug.enabled {
                // layout[0] - Main window
                // layout[1] - Debug window
                let layout = Layout::default()
                    .constraints([Constraint::Min(0), Constraint::Length(5)])
                    .split(frame.size());

                match app.mode {
                    Mode::DisplaySummary => draw_summary(frame, app, layout[0]),
                    Mode::Help => draw_help(frame, app, layout[0]),
                    _ => draw_main(frame, app, layout[0]),
                }

                draw_debug(frame, app, layout[1]);
            } else if app.mode == Mode::AddStock {
                // layout[0] - Main window
                // layout[1] - Add Stock window
                let layout = Layout::default()
                    .constraints([Constraint::Min(0), Constraint::Length(3)])
                    .split(frame.size());

                if !app.stocks.is_empty() {
                    match app.previous_mode {
                        Mode::DisplaySummary => draw_summary(frame, app, layout[0]),
                        _ => draw_main(frame, app, layout[0]),
                    }
                }

                draw_add_stock(frame, app, layout[1]);
            } else {
                // layout - Main window
                let layout = frame.size();

                match app.mode {
                    Mode::DisplaySummary => draw_summary(frame, app, layout),
                    Mode::Help => draw_help(frame, app, layout),
                    _ => draw_main(frame, app, layout),
                }
            };
        })
        .unwrap();
}

fn draw_main(frame: &mut Frame, app: &mut App, area: Rect) {
    // layout[0] - Header
    // layout[1] - Main widget
    let mut layout = Layout::default()
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(area)
        .to_vec();

    if !app.stocks.is_empty() {
        frame.render_widget(crate::widget::block::new(" Tabs "), layout[0]);
        let padded = add_padding(layout[0], 1, PaddingDirection::All);
        layout[0] = padded;

        // header[0] - Stock symbol tabs
        // header[1] - (Optional) help icon
        let header = if app.hide_help {
            vec![layout[0]]
        } else {
            let split = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Min(0), Constraint::Length(10)])
                .split(layout[0]);
            split.to_vec()
        };

        // Draw tabs
        {
            let tabs: Vec<_> = app.stocks.iter().map(|w| Line::from(w.symbol())).collect();

            frame.render_widget(
                Tabs::new(tabs)
                    .select(app.current_tab)
                    .style(style().fg(THEME.text_secondary()))
                    .highlight_style(style().fg(THEME.text_primary())),
                header[0],
            );
        }

        // Draw help icon
        if !app.hide_help {
            frame.render_widget(
                Paragraph::new(Line::from(Span::styled("Help '?'", style())))
                    .style(style().fg(THEME.text_normal()))
                    .alignment(Alignment::Center),
                header[1],
            );
        }
    }

    // Make sure only displayed stock has network activity
    app.stocks.iter().enumerate().for_each(|(idx, s)| {
        if idx == app.current_tab {
            s.stock_service.resume();
        } else {
            s.stock_service.pause();
        }
    });

    // Draw main widget
    if let Some(stock) = app.stocks.get_mut(app.current_tab) {
        // main_chunks[0] - Stock widget
        // main_chunks[1] - Options widget / Configuration widget (optional)
        let mut main_chunks =
            if app.mode == Mode::DisplayOptions || app.mode == Mode::ConfigureChart {
                Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Min(0), Constraint::Length(44)])
                    .split(layout[1])
                    .to_vec()
            } else {
                vec![layout[1]]
            };

        match app.mode {
            Mode::DisplayStock | Mode::AddStock => {
                frame.render_stateful_widget(StockWidget {}, main_chunks[0], stock);
            }
            // If width is too small, don't render stock widget and use entire space
            // for options / configure widget
            Mode::DisplayOptions | Mode::ConfigureChart => {
                if main_chunks[0].width >= 19 {
                    frame.render_stateful_widget(StockWidget {}, main_chunks[0], stock);
                } else {
                    main_chunks[1] = layout[1];
                }
            }
            _ => {}
        }

        match app.mode {
            Mode::DisplayOptions => {
                if let Some(options) = stock.options.as_mut() {
                    if main_chunks[1].width >= 44 && main_chunks[1].height >= 14 {
                        frame.render_stateful_widget(OptionsWidget {}, main_chunks[1], options);
                    } else {
                        let mut padded = main_chunks[1];
                        padded = add_padding(padded, 1, PaddingDirection::Left);
                        padded = add_padding(padded, 1, PaddingDirection::Top);
                        main_chunks[1] = padded;

                        frame.render_widget(
                            Paragraph::new(Line::from(Span::styled(
                                "Increase screen size to display options",
                                style(),
                            ))),
                            main_chunks[1],
                        );
                    }
                }
            }
            Mode::ConfigureChart => {
                if main_chunks[1].width >= 44 && main_chunks[1].height >= 14 {
                    let state = &mut stock.chart_configuration;

                    let chart_type = stock.chart_type;

                    frame.render_stateful_widget(
                        ChartConfigurationWidget { chart_type },
                        main_chunks[1],
                        state,
                    );
                } else {
                    let mut padded = main_chunks[1];
                    padded = add_padding(padded, 1, PaddingDirection::Left);
                    padded = add_padding(padded, 1, PaddingDirection::Top);
                    main_chunks[1] = padded;

                    frame.render_widget(
                        Paragraph::new(Line::from(Span::styled(
                            "Increase screen size to display configuration screen",
                            style(),
                        )))
                        .wrap(Wrap { trim: false }),
                        main_chunks[1],
                    );
                }
            }
            _ => {}
        }
    }
}

fn draw_add_stock(frame: &mut Frame, app: &mut App, area: Rect) {
    frame.render_stateful_widget(AddStockWidget {}, area, &mut app.add_stock);
}

fn draw_summary(frame: &mut Frame, app: &mut App, mut area: Rect) {
    let border = block::new(" Summary ");
    frame.render_widget(border, area);
    area = add_padding(area, 1, PaddingDirection::All);
    area = add_padding(area, 1, PaddingDirection::Right);

    let show_volumes = *SHOW_VOLUMES.read() && app.chart_type != ChartType::Kagi;
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
        .constraints([
            Constraint::Length(1),
            Constraint::Length((num_to_render * stock_widget_height as usize) as u16),
            Constraint::Min(0),
        ])
        .split(area)
        .to_vec();

    // header[0]
    // header[1] - (Optional) help icon
    let header = if app.hide_help {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(0)])
            .split(layout[0])
            .to_vec()
    } else {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(0), Constraint::Length(8)])
            .split(layout[0])
            .to_vec()
    };

    // Draw help icon
    if !app.hide_help {
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled("Help '?'", style())))
                .style(style().fg(THEME.text_normal()))
                .alignment(Alignment::Center),
            header[1],
        );
    }

    let contraints = app.stocks[scroll_offset..num_to_render + scroll_offset]
        .iter()
        .map(|_| Constraint::Length(stock_widget_height))
        .collect::<Vec<_>>();

    let stock_layout = Layout::default()
        .constraints(contraints)
        .split(layout[1])
        .to_vec();

    // Make sure only displayed stocks have network activity
    app.stocks.iter().enumerate().for_each(|(idx, s)| {
        if idx >= scroll_offset && idx < num_to_render + scroll_offset {
            s.stock_service.resume();
        } else {
            s.stock_service.pause();
        }
    });

    for (idx, stock) in app.stocks[scroll_offset..num_to_render + scroll_offset]
        .iter_mut()
        .enumerate()
    {
        frame.render_stateful_widget(StockSummaryWidget {}, stock_layout[idx], stock);
    }

    // Draw time frame & paging
    {
        let mut current = layout[2];
        current = add_padding(current, 1, PaddingDirection::Left);
        frame.render_widget(Clear, current);
        frame.render_widget(Block::default().style(style()), current);

        let offset = current.height - 2;
        current = add_padding(current, offset, PaddingDirection::Top);

        frame.render_widget(
            Block::default()
                .borders(Borders::TOP)
                .border_style(style().fg(THEME.border_secondary())),
            current,
        );

        current = add_padding(current, 1, PaddingDirection::Top);
        layout[2] = current;

        let time_frames = TimeFrame::tab_names()
            .iter()
            .map(|s| Line::from(*s))
            .collect::<Vec<_>>();

        // botton_layout[0] - time frame
        // botton_layout[1] - paging indicator
        let bottom_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(0), Constraint::Length(3)])
            .split(layout[2])
            .to_vec();

        let tabs = Tabs::new(time_frames)
            .select(app.summary_time_frame.idx())
            .style(style().fg(THEME.text_secondary()))
            .highlight_style(style().fg(THEME.text_primary()));

        frame.render_widget(tabs, bottom_layout[0]);

        let more_up = scroll_offset > 0;
        let more_down = scroll_offset + num_to_render < app.stocks.len();

        let up_arrow = Span::styled(
            "ᐱ",
            style().fg(if more_up {
                THEME.text_normal()
            } else {
                THEME.gray()
            }),
        );
        let down_arrow = Span::styled(
            "ᐯ",
            style().fg(if more_down {
                THEME.text_normal()
            } else {
                THEME.gray()
            }),
        );

        frame.render_widget(
            Paragraph::new(Line::from(vec![up_arrow, Span::raw(" "), down_arrow])),
            bottom_layout[1],
        );
    }
}

fn draw_help(frame: &mut Frame, app: &App, area: Rect) {
    let mut layout = area;

    if layout.width < HELP_WIDTH as u16 || layout.height < HELP_HEIGHT as u16 {
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(
                "Increase screen size to display help",
                style(),
            ))),
            layout,
        );
    } else {
        layout = app.help.get_rect(layout);

        frame.render_widget(app.help, layout);
    }
}

fn draw_debug(frame: &mut Frame, app: &mut App, area: Rect) {
    app.debug.mode = app.mode;

    let debug_text = Line::from(Span::styled(format!("{:?}", app.debug), style()));
    let debug_paragraph = Paragraph::new(debug_text).wrap(Wrap { trim: true });

    frame.render_widget(debug_paragraph, area);
}

pub fn add_padding(mut rect: Rect, n: u16, direction: PaddingDirection) -> Rect {
    match direction {
        PaddingDirection::Top => {
            rect.y += n;
            rect.height = rect.height.saturating_sub(n);
            rect
        }
        PaddingDirection::Bottom => {
            rect.height = rect.height.saturating_sub(n);
            rect
        }
        PaddingDirection::Left => {
            rect.x += n;
            rect.width = rect.width.saturating_sub(n);
            rect
        }
        PaddingDirection::Right => {
            rect.width = rect.width.saturating_sub(n);
            rect
        }
        PaddingDirection::All => {
            rect.y += n;
            rect.height = rect.height.saturating_sub(n * 2);

            rect.x += n;
            rect.width = rect.width.saturating_sub(n * 2);

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
