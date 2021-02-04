use itertools::Itertools;
use tui::buffer::Buffer;
use tui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use tui::style::{Color, Modifier, Style};
use tui::symbols::{bar, Marker};
use tui::widgets::{
    Axis, BarChart, Block, Borders, Chart, Dataset, GraphType, Paragraph, StatefulWidget, Tabs,
    Text, Widget,
};

use super::{block, OptionsState};
use crate::api::model::{ChartMeta, CompanyData};
use crate::common::*;
use crate::draw::{add_padding, PaddingDirection};
use crate::service::{self, Service};
use crate::{
    ENABLE_PRE_POST, HIDE_PREV_CLOSE, HIDE_TOGGLE, SHOW_VOLUMES, SHOW_X_LABELS, TIME_FRAME,
    TRUNC_PRE,
};

const NUM_LOADING_TICKS: usize = 4;

pub struct StockState {
    pub symbol: String,
    pub stock_service: service::stock::StockService,
    pub profile: Option<CompanyData>,
    pub current_regular_price: f64,
    pub current_post_price: Option<f64>,
    pub prev_close_price: Option<f64>,
    pub reg_mkt_volume: Option<String>,
    pub prices: [Vec<Price>; 7],
    pub time_frame: TimeFrame,
    pub show_options: bool,
    pub options: Option<OptionsState>,
    pub loading_tick: usize,
    pub prev_state_loaded: bool,
    pub chart_meta: Option<ChartMeta>,
}

impl StockState {
    pub fn new(symbol: String) -> StockState {
        let time_frame = *TIME_FRAME;

        let stock_service = service::stock::StockService::new(symbol.clone(), time_frame);

        StockState {
            symbol,
            stock_service,
            profile: None,
            current_regular_price: 0.0,
            current_post_price: None,
            prev_close_price: None,
            reg_mkt_volume: None,
            prices: [vec![], vec![], vec![], vec![], vec![], vec![], vec![]],
            time_frame,
            show_options: false,
            options: None,
            loading_tick: NUM_LOADING_TICKS,
            prev_state_loaded: false,
            chart_meta: None,
        }
    }

    pub fn symbol(&self) -> &str {
        &self.symbol
    }

    pub fn time_frame_up(&mut self) {
        self.set_time_frame(self.time_frame.up());
    }

    pub fn time_frame_down(&mut self) {
        self.set_time_frame(self.time_frame.down());
    }

    pub fn set_time_frame(&mut self, time_frame: TimeFrame) {
        self.time_frame = time_frame;

        self.stock_service.update_time_frame(time_frame);
    }

    pub fn prices(&self) -> impl Iterator<Item = &Price> {
        let (start, end) = self.start_end();

        self.prices[self.time_frame.idx()].iter().filter(move |p| {
            if self.time_frame == TimeFrame::Day1 {
                (p.date > start && p.date < end) || p.date == start || p.date == end
            } else {
                true
            }
        })
    }

    pub fn volumes(&self) -> Vec<u64> {
        let (start, end) = self.start_end();

        let mut prices = self.prices();

        if self.time_frame == TimeFrame::Day1 {
            let times = MarketHours(start, end);

            times
                .map(|t| {
                    if let Some(p) = prices.find(|p| p.date == t) {
                        p.volume
                    } else {
                        0
                    }
                })
                .collect()
        } else {
            prices.map(|p| p.volume).collect()
        }
    }

    pub fn current_price(&self) -> f64 {
        let enable_pre_post = { *ENABLE_PRE_POST.read().unwrap() };

        if enable_pre_post && self.current_post_price.is_some() {
            self.current_post_price.unwrap()
        } else {
            self.current_regular_price
        }
    }

    pub fn update(&mut self) {
        let updates = self.stock_service.updates();

        for update in updates {
            match update {
                service::stock::Update::NewPrice((regular, post, vol)) => {
                    self.current_regular_price = regular;
                    self.current_post_price = post;
                    self.reg_mkt_volume = Some(vol);
                }
                service::stock::Update::Prices((time_frame, chart_meta, prices)) => {
                    self.prices[time_frame.idx()] = prices;

                    if time_frame == TimeFrame::Day1 {
                        self.prev_close_price = Some(chart_meta.chart_previous_close);
                    }

                    self.chart_meta = Some(chart_meta);
                }
                service::stock::Update::CompanyData(data) => {
                    self.profile = Some(data);
                }
            }
        }
    }

    fn options_enabled(&self) -> bool {
        self.chart_meta
            .as_ref()
            .map(|m| m.instrument_type.as_deref())
            .flatten()
            != Some("CRYPTOCURRENCY")
    }

    pub fn toggle_options(&mut self) -> bool {
        if !self.options_enabled() {
            return false;
        }

        self.show_options = !self.show_options;

        if self.options.is_some() {
            self.options.take();
        } else {
            self.options = Some(OptionsState::new(self.symbol.clone()));
        }

        true
    }

    pub fn start_end(&self) -> (i64, i64) {
        let enable_pre_post = { *ENABLE_PRE_POST.read().unwrap() };

        let pre = self
            .chart_meta
            .as_ref()
            .and_then(|c| c.current_trading_period.as_ref())
            .map(|p| &p.pre);

        let regular = self
            .chart_meta
            .as_ref()
            .and_then(|c| c.current_trading_period.as_ref())
            .map(|p| &p.regular);

        let post = self
            .chart_meta
            .as_ref()
            .and_then(|c| c.current_trading_period.as_ref())
            .map(|p| &p.post);

        let mut pre_start = pre.map(|p| p.start).unwrap_or(32400);
        let reg_start = regular.map(|p| p.start).unwrap_or(52200);
        let reg_end = regular.map(|p| p.end).unwrap_or(75600);
        let post_end = post.map(|p| p.end).unwrap_or(90000);

        // Pre market really only has activity 30 min before open
        if reg_start - pre_start >= 1800 && *TRUNC_PRE {
            pre_start = reg_start - 1800;
        }

        let start = if !enable_pre_post {
            reg_start
        } else {
            pre_start
        };

        let end = if !enable_pre_post { reg_end } else { post_end };

        (start, end)
    }

    pub fn regular_start_end_idx(&self) -> (Option<usize>, Option<usize>) {
        let reg_start = self
            .chart_meta
            .as_ref()
            .and_then(|m| m.current_trading_period.as_ref())
            .map(|c| c.regular.start);

        let reg_end = self
            .chart_meta
            .as_ref()
            .and_then(|m| m.current_trading_period.as_ref())
            .map(|c| c.regular.end);

        let start_idx = self
            .prices()
            .enumerate()
            .find(|(_, p)| Some(p.date) >= reg_start)
            .map(|(idx, _)| idx);

        let end_idx = self
            .prices()
            .enumerate()
            .find(|(_, p)| Some(p.date) >= reg_end)
            .map(|(idx, _)| idx);

        (start_idx, end_idx)
    }

    pub fn current_trading_period(&self) -> TradingPeriod {
        let (reg_start, reg_end) = self.regular_start_end_idx();

        if self.time_frame != TimeFrame::Day1 {
            TradingPeriod::Regular
        } else if reg_start.is_some() && reg_end.is_some() {
            TradingPeriod::Post
        } else if reg_start.is_some() {
            TradingPeriod::Regular
        } else {
            TradingPeriod::Pre
        }
    }

    pub fn min_max(&self) -> (f64, f64) {
        let mut data: Vec<_> = self.prices().map(cast_historical_as_price).collect();
        data.pop();
        data.push(self.current_price());
        data = remove_zeros(data);

        data.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let mut min = data.first().cloned().unwrap_or(0.0);
        let mut max = data.last().cloned().unwrap_or(1.0);

        if self.current_price().le(&min) {
            min = self.current_price();
        }

        if self.current_price().gt(&max) {
            max = self.current_price();
        }

        if self.time_frame == TimeFrame::Day1 && !*HIDE_PREV_CLOSE {
            if let Some(prev_close) = self.prev_close_price {
                if prev_close.le(&min) {
                    min = prev_close;
                }

                if prev_close.gt(&max) {
                    max = prev_close;
                }
            }
        }

        (min, max)
    }

    pub fn high_low(&self) -> (f64, f64) {
        let mut data = self.prices().cloned().collect::<Vec<_>>();

        data.sort_by(|a, b| a.high.partial_cmp(&b.high).unwrap());
        let mut max = data.last().map(|d| d.high).unwrap_or(0.0);

        data = remove_zeros_lows(data);
        data.sort_by(|a, b| a.low.partial_cmp(&b.low).unwrap());
        let mut min = data.first().map(|d| d.low).unwrap_or(0.0);

        if self.current_price().le(&min) {
            min = self.current_price();
        }

        if self.current_price().gt(&max) {
            max = self.current_price();
        }

        (max, min)
    }

    pub fn x_bounds(&self, start: i64, end: i64) -> [f64; 2] {
        let num_points = ((end - start) / 60) as f64;

        match self.time_frame {
            TimeFrame::Day1 => [0.0, num_points],
            _ => [0.0, (self.prices().count() + 1) as f64],
        }
    }

    pub fn x_labels(&self, width: u16, start: i64, end: i64) -> Vec<String> {
        let mut labels = vec![];

        let dates = if self.time_frame == TimeFrame::Day1 {
            MarketHours(start, end).collect()
        } else {
            self.prices().map(|p| p.date).collect::<Vec<_>>()
        };

        if dates.is_empty() {
            return labels;
        }

        let label_len = dates
            .get(0)
            .map_or(0, |d| self.time_frame.format_time(*d).len())
            + 5;
        let num_labels = width as usize / label_len;
        let chunk_size = (dates.len() as f32 / (num_labels - 1) as f32).ceil() as usize;

        for (idx, chunk) in dates.chunks(chunk_size).enumerate() {
            if idx == 0 {
                labels.push(
                    chunk
                        .get(0)
                        .map_or("".to_string(), |d| self.time_frame.format_time(*d)),
                );
            }

            labels.push(
                chunk
                    .get(chunk.len() - 1)
                    .map_or("".to_string(), |d| self.time_frame.format_time(*d)),
            );
        }

        labels
    }

    pub fn y_bounds(&self, min: f64, max: f64) -> [f64; 2] {
        [(min - 0.05), (max + 0.05)]
    }

    pub fn y_labels(&self, min: f64, max: f64) -> Vec<String> {
        if self.loaded() {
            vec![
                format!("{:>8.2}", (min - 0.05)),
                format!("{:>8.2}", ((min - 0.05) + (max + 0.05)) / 2.0),
                format!("{:>8.2}", max + 0.05),
            ]
        } else {
            vec![
                "       ".to_string(),
                "       ".to_string(),
                "       ".to_string(),
            ]
        }
    }

    pub fn pct_change(&self) -> f64 {
        if self.prices().count() == 0 {
            return 0.0;
        }

        let baseline = if self.time_frame == TimeFrame::Day1 {
            if let Some(prev_close) = self.prev_close_price {
                prev_close
            } else {
                self.prices().next().map(|d| d.close).unwrap()
            }
        } else {
            self.prices().next().map(|d| d.close).unwrap()
        };

        self.current_price() / baseline - 1.0
    }

    pub fn loaded(&self) -> bool {
        self.prices().count() > 0 && self.current_price() > 0.0 && self.profile.is_some()
    }

    pub fn loading_tick(&mut self) {
        let loaded = self.loaded();

        if !loaded {
            // Reset tick
            if self.prev_state_loaded {
                self.prev_state_loaded = false;
                self.loading_tick = NUM_LOADING_TICKS;
            }

            self.loading_tick = (self.loading_tick + 1) % (NUM_LOADING_TICKS + 1);
        } else if !self.prev_state_loaded {
            self.prev_state_loaded = true;
        }
    }
}

pub struct StockWidget {}

impl StatefulWidget for StockWidget {
    type State = StockState;

    #[allow(clippy::clippy::unnecessary_unwrap)]
    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let pct_change = state.pct_change();

        let show_x_labels = SHOW_X_LABELS.read().map_or(false, |l| *l);
        let enable_pre_post = *ENABLE_PRE_POST.read().unwrap();
        let show_volumes = *SHOW_VOLUMES.read().unwrap();

        let loaded = state.loaded();
        state.loading_tick();

        let (company_name, currency) = match state.profile.as_ref() {
            Some(profile) => (
                profile.price.short_name.as_str(),
                profile.price.currency.as_deref().unwrap_or("USD"),
            ),
            None => ("", ""),
        };

        let loading_indicator = ".".repeat(state.loading_tick);

        // Draw widget block
        {
            block::new(
                &format!(
                    " {}{:<4} ",
                    state.symbol,
                    if loaded {
                        format!(" - {}", company_name)
                    } else if state.profile.is_some() {
                        format!(" - {}{:<4}", company_name, loading_indicator)
                    } else {
                        loading_indicator
                    }
                ),
                None,
            )
            .render(area, buf);
        }

        // chunks[0] - Top Padding
        // chunks[1] - Company Info
        // chunks[2] - Graph - fill remaining space
        // chunks[3] - Time Frame Tabs
        // chunks[4] - Bottom Padding
        let mut chunks = Layout::default()
            .constraints(
                [
                    Constraint::Length(2),
                    Constraint::Length(6),
                    Constraint::Min(0),
                    Constraint::Length(2),
                    Constraint::Length(1),
                ]
                .as_ref(),
            )
            .split(area);

        chunks[1] = add_padding(chunks[1], 2, PaddingDirection::Left);
        chunks[1] = add_padding(chunks[1], 2, PaddingDirection::Right);

        chunks[2] = add_padding(chunks[2], 2, PaddingDirection::Left);
        chunks[2] = add_padding(chunks[2], 2, PaddingDirection::Right);

        chunks[3] = add_padding(chunks[3], 2, PaddingDirection::Left);
        chunks[3] = add_padding(chunks[3], 2, PaddingDirection::Right);

        // Draw company info
        {
            let mut info_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Min(0), Constraint::Length(15)].as_ref())
                .split(chunks[1]);
            info_chunks[1].y -= 1;
            info_chunks[1].height += 1;

            let (high, low) = state.high_low();
            let vol = state.reg_mkt_volume.clone().unwrap_or_default();

            let company_info = [
                Text::styled("c: ", Style::default()),
                Text::styled(
                    if loaded {
                        format!("{:.2} {}", state.current_price(), currency)
                    } else {
                        "".to_string()
                    },
                    Style::default().modifier(Modifier::BOLD).fg(Color::Yellow),
                ),
                Text::styled(
                    if loaded {
                        format!("  {:.2}%\n", pct_change * 100.0)
                    } else {
                        "\n".to_string()
                    },
                    Style::default()
                        .modifier(Modifier::BOLD)
                        .fg(if pct_change >= 0.0 {
                            Color::Green
                        } else {
                            Color::Red
                        }),
                ),
                Text::styled("h: ", Style::default()),
                Text::styled(
                    if loaded {
                        format!("{:.2}\n", high)
                    } else {
                        "\n".to_string()
                    },
                    Style::default().fg(Color::LightCyan),
                ),
                Text::styled("l: ", Style::default()),
                Text::styled(
                    if loaded {
                        format!("{:.2}\n\n", low)
                    } else {
                        "\n\n".to_string()
                    },
                    Style::default().fg(Color::LightCyan),
                ),
                Text::styled("v: ", Style::default()),
                Text::styled(
                    if loaded { vol } else { "".to_string() },
                    Style::default().fg(Color::LightCyan),
                ),
            ];

            Paragraph::new(company_info.iter())
                .style(Style::default().fg(Color::White))
                .alignment(Alignment::Left)
                .wrap(true)
                .render(info_chunks[0], buf);

            if !*HIDE_TOGGLE {
                let toggle_block = block::new(" Toggle ", None);
                toggle_block.render(info_chunks[1], buf);
                info_chunks[1].x += 2;
                info_chunks[1].width -= 2;
                info_chunks[1].y += 1;
                info_chunks[1].height -= 1;

                let mut toggle_info = vec![Text::styled("Summary  's'", Style::default())];

                if loaded {
                    toggle_info.push(Text::styled(
                        "\nVolumes  'v'",
                        Style::default().bg(if show_volumes {
                            Color::DarkGray
                        } else {
                            Color::Reset
                        }),
                    ));

                    toggle_info.push(Text::styled(
                        "\nX Labels 'x'",
                        Style::default().bg(if show_x_labels {
                            Color::DarkGray
                        } else {
                            Color::Reset
                        }),
                    ));

                    toggle_info.push(Text::styled(
                        "\nPre Post 'p'",
                        Style::default().bg(if enable_pre_post {
                            Color::DarkGray
                        } else {
                            Color::Reset
                        }),
                    ));
                }

                if state.options_enabled() && loaded {
                    toggle_info.push(Text::styled(
                        "\nOptions  'o'",
                        Style::default().bg(if state.show_options {
                            Color::DarkGray
                        } else {
                            Color::Reset
                        }),
                    ));
                }

                Paragraph::new(toggle_info.iter())
                    .style(Style::default().fg(Color::White))
                    .alignment(Alignment::Left)
                    .wrap(false)
                    .render(info_chunks[1], buf);
            }
        }

        // Draw graph
        {
            let (min, max) = state.min_max();
            let (start, end) = state.start_end();

            let mut prices: Vec<_> = state.prices().map(cast_historical_as_price).collect();

            prices.pop();
            prices.push(state.current_price());
            zeros_as_pre(&mut prices);

            // Need more than one price for GraphType::Line to work
            let graph_type = if prices.len() <= 2 {
                GraphType::Scatter
            } else {
                GraphType::Line
            };

            let x_labels = if show_x_labels {
                state.x_labels(chunks[2].width, start, end)
            } else {
                vec![]
            };

            let trading_period = state.current_trading_period();

            let (reg_prices, pre_prices, post_prices) = if loaded {
                let (start_idx, end_idx) = state.regular_start_end_idx();

                if enable_pre_post && state.time_frame == TimeFrame::Day1 {
                    (
                        prices
                            .iter()
                            .enumerate()
                            .filter(|(idx, _)| {
                                if let Some(start) = start_idx {
                                    *idx >= start
                                } else {
                                    false
                                }
                            })
                            .filter(|(idx, _)| {
                                if let Some(end) = end_idx {
                                    *idx <= end
                                } else {
                                    true
                                }
                            })
                            .map(cast_as_dataset)
                            .collect::<Vec<(f64, f64)>>(),
                        {
                            let pre_end_idx_noninclusive = if let Some(start_idx) = start_idx {
                                if start_idx == 0 {
                                    0
                                } else {
                                    start_idx
                                }
                            } else {
                                prices.len()
                            };

                            if pre_end_idx_noninclusive > 0 {
                                Some(
                                    prices
                                        .iter()
                                        .enumerate()
                                        .filter(|(idx, _)| *idx <= pre_end_idx_noninclusive)
                                        .map(cast_as_dataset)
                                        .collect::<Vec<(f64, f64)>>(),
                                )
                            } else {
                                None
                            }
                        },
                        {
                            let post_start_idx_noninclusive = end_idx.unwrap_or(prices.len());

                            if post_start_idx_noninclusive < prices.len() {
                                Some(
                                    prices
                                        .iter()
                                        .enumerate()
                                        .filter(|(idx, _)| *idx >= post_start_idx_noninclusive)
                                        .map(cast_as_dataset)
                                        .collect::<Vec<(f64, f64)>>(),
                                )
                            } else {
                                None
                            }
                        },
                    )
                } else {
                    (
                        prices
                            .iter()
                            .enumerate()
                            .map(cast_as_dataset)
                            .collect::<Vec<(f64, f64)>>(),
                        None,
                        None,
                    )
                }
            } else {
                (vec![], None, None)
            };

            let prev_close_line = if state.time_frame == TimeFrame::Day1
                && loaded
                && !*HIDE_PREV_CLOSE
                && state.prev_close_price.is_some()
            {
                let num_points = (end - start) / 60 + 1;

                Some(
                    (0..num_points)
                        .map(|i| ((i + 1) as f64, state.prev_close_price.unwrap()))
                        .collect::<Vec<_>>(),
                )
            } else {
                None
            };

            let mut datasets = vec![Dataset::default()
                .marker(Marker::Braille)
                .style(Style::default().fg(
                    if trading_period != TradingPeriod::Regular && enable_pre_post {
                        Color::DarkGray
                    } else if pct_change >= 0.0 {
                        Color::Green
                    } else {
                        Color::Red
                    },
                ))
                .graph_type(graph_type)
                .data(&reg_prices)];

            if let Some(data) = pre_prices.as_ref() {
                datasets.insert(
                    0,
                    Dataset::default()
                        .marker(Marker::Braille)
                        .style(
                            Style::default().fg(if trading_period != TradingPeriod::Pre {
                                Color::DarkGray
                            } else if pct_change >= 0.0 {
                                Color::Green
                            } else {
                                Color::Red
                            }),
                        )
                        .graph_type(GraphType::Line)
                        .data(&data),
                );
            }

            if let Some(data) = post_prices.as_ref() {
                datasets.insert(
                    0,
                    Dataset::default()
                        .marker(Marker::Braille)
                        .style(
                            Style::default().fg(if trading_period != TradingPeriod::Post {
                                Color::DarkGray
                            } else if pct_change >= 0.0 {
                                Color::Green
                            } else {
                                Color::Red
                            }),
                        )
                        .graph_type(GraphType::Line)
                        .data(&data),
                );
            }

            if let Some(data) = prev_close_line.as_ref() {
                datasets.insert(
                    0,
                    Dataset::default()
                        .marker(Marker::Braille)
                        .style(Style::default().fg(Color::DarkGray))
                        .graph_type(GraphType::Line)
                        .data(&data),
                );
            }

            // graph_chunks[0] = prices
            // graph_chunks[1] = volume
            let graph_chunks = if show_volumes {
                Layout::default()
                    .constraints([Constraint::Min(6), Constraint::Length(5)].as_ref())
                    .split(chunks[2])
            } else {
                Layout::default()
                    .constraints([Constraint::Min(0)].as_ref())
                    .split(chunks[2])
            };

            if show_volumes {
                let mut volume_chunks = graph_chunks[1];
                volume_chunks.height += 1;

                let x_offset = if !loaded {
                    8
                } else if show_x_labels {
                    match state.time_frame {
                        TimeFrame::Day1 => 9,
                        TimeFrame::Week1 => 12,
                        _ => 11,
                    }
                } else {
                    9
                };
                volume_chunks.x += x_offset;
                volume_chunks.width -= x_offset + 1;

                let width = volume_chunks.width;
                let num_bars = width as usize;

                let volumes = state.volumes();
                let vol_count = volumes.len();

                if vol_count > 0 {
                    let volumes = state
                        .prices()
                        .map(|p| [p.volume].repeat(num_bars))
                        .flatten()
                        .chunks(vol_count)
                        .into_iter()
                        .map(|c| ("", c.into_iter().sum::<u64>() / vol_count as u64))
                        .collect::<Vec<_>>();

                    volume_chunks.x -= 1;

                    Block::default()
                        .borders(Borders::LEFT)
                        .border_style(Style::default().fg(Color::Blue))
                        .render(volume_chunks, buf);

                    volume_chunks.x += 1;

                    BarChart::default()
                        .bar_gap(0)
                        .bar_set(bar::NINE_LEVELS)
                        .style(Style::default().fg(Color::DarkGray))
                        .data(&volumes)
                        .render(volume_chunks, buf);
                }
            }

            Chart::<String, String>::default()
                .block(
                    Block::default()
                        .borders(Borders::TOP)
                        .border_style(Style::default()),
                )
                .x_axis({
                    let axis = Axis::default().bounds(state.x_bounds(start, end));

                    if show_x_labels && loaded {
                        axis.labels(&x_labels)
                            .style(Style::default().fg(Color::LightBlue))
                    } else {
                        axis
                    }
                })
                .y_axis(
                    Axis::default()
                        .bounds(state.y_bounds(min, max))
                        .labels(&state.y_labels(min, max))
                        .style(Style::default().fg(Color::LightBlue)),
                )
                .datasets(&datasets)
                .render(graph_chunks[0], buf);
        }

        // Draw time frame tabs
        {
            Tabs::default()
                .block(Block::default().borders(Borders::TOP))
                .titles(&TimeFrame::tab_names())
                .select(state.time_frame.idx())
                .style(Style::default().fg(Color::Cyan))
                .highlight_style(Style::default().fg(Color::Yellow))
                .render(chunks[3], buf);
        }
    }
}
