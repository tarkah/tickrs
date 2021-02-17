use std::hash::{Hash, Hasher};

use itertools::Itertools;
use tui::buffer::Buffer;
use tui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use tui::style::{Modifier, Style};
use tui::symbols::{bar, Marker};
use tui::text::{Span, Spans};
use tui::widgets::{
    Axis, BarChart, Block, Borders, Chart, Dataset, GraphType, Paragraph, StatefulWidget, Tabs,
    Widget, Wrap,
};

use super::{block, CachableWidget, CacheState, OptionsState};
use crate::api::model::{ChartMeta, CompanyData};
use crate::common::*;
use crate::draw::{add_padding, PaddingDirection};
use crate::service::{self, Service};
use crate::{
    DEFAULT_TIMESTAMPS, ENABLE_PRE_POST, HIDE_PREV_CLOSE, HIDE_TOGGLE, SHOW_VOLUMES, SHOW_X_LABELS,
    THEME, TIME_FRAME, TRUNC_PRE,
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
    pub cache_state: CacheState,
}

impl Hash for StockState {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.symbol.hash(state);
        self.current_regular_price.to_bits().hash(state);
        // Only fetched once, so just need to check if Some
        self.profile.is_some().hash(state);
        self.current_post_price.map(|f| f.to_bits()).hash(state);
        self.prev_close_price.map(|f| f.to_bits()).hash(state);
        self.reg_mkt_volume.hash(state);
        self.prices.hash(state);
        self.time_frame.hash(state);
        self.show_options.hash(state);
        self.loading_tick.hash(state);
        self.prev_state_loaded.hash(state);
        self.chart_meta.hash(state);

        // Hash globals since they affect "state" of how widget is rendered
        DEFAULT_TIMESTAMPS
            .read()
            .unwrap()
            .get(&self.time_frame)
            .hash(state);
        ENABLE_PRE_POST.read().unwrap().hash(state);
        HIDE_PREV_CLOSE.hash(state);
        HIDE_TOGGLE.hash(state);
        SHOW_VOLUMES.read().unwrap().hash(state);
        SHOW_X_LABELS.read().unwrap().hash(state);
        TRUNC_PRE.hash(state);
    }
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
            cache_state: Default::default(),
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

    pub fn prices(&self) -> impl Iterator<Item = Price> {
        let (start, end) = self.start_end();

        let prices = self.prices[self.time_frame.idx()].clone();

        let max_time = prices.last().map(|p| p.date).unwrap_or(end);

        let default_timestamps = {
            let defaults = DEFAULT_TIMESTAMPS.read().unwrap();
            defaults.get(&self.time_frame).cloned()
        };

        let prices = if self.time_frame == TimeFrame::Day1 {
            let times = MarketHours(
                start,
                if max_time < start {
                    end.max(start)
                } else {
                    max_time
                },
            );

            times
                .map(|t| {
                    if let Some(p) = prices.iter().find(|p| {
                        let min_rounded = p.date - p.date % 60;

                        min_rounded == t
                    }) {
                        *p
                    } else {
                        Price {
                            date: t,
                            ..Default::default()
                        }
                    }
                })
                .collect::<Vec<_>>()
        } else if self.is_crypto() {
            prices
        } else if let Some(default_timestamps) = default_timestamps {
            default_timestamps
                .into_iter()
                .map(|t| {
                    if let Some(p) = prices.iter().find(|p| {
                        let a_rounded = p.date - p.date % self.time_frame.round_by();
                        let b_rounded = t - t % self.time_frame.round_by();

                        a_rounded == b_rounded
                    }) {
                        *p
                    } else {
                        Price {
                            date: t,
                            ..Default::default()
                        }
                    }
                })
                .collect::<Vec<_>>()
        } else {
            prices
        };

        prices.into_iter()
    }

    pub fn volumes(&self, data: &[Price]) -> Vec<u64> {
        let (start, end) = self.start_end();

        if self.time_frame == TimeFrame::Day1 {
            let times = MarketHours(start, end.max(start));

            times
                .map(|t| {
                    if let Some(p) = data.iter().find(|p| p.date == t) {
                        p.volume
                    } else {
                        0
                    }
                })
                .collect()
        } else {
            data.iter().map(|p| p.volume).collect()
        }
    }

    pub fn current_price(&self) -> f64 {
        let enable_pre_post = { *ENABLE_PRE_POST.read().unwrap() };

        if enable_pre_post && self.current_post_price.is_some() {
            self.current_post_price
                .unwrap_or(self.current_regular_price)
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
        !self.is_crypto()
    }

    fn is_crypto(&self) -> bool {
        self.chart_meta
            .as_ref()
            .and_then(|m| m.instrument_type.as_deref())
            == Some("CRYPTOCURRENCY")
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

    pub fn regular_start_end_idx(&self, data: &[Price]) -> (Option<usize>, Option<usize>) {
        let reg_start = self
            .chart_meta
            .as_ref()
            .and_then(|m| m.current_trading_period.as_ref())
            .map(|c| c.regular.start);

        let reg_end = self
            .chart_meta
            .as_ref()
            .and_then(|m| m.current_trading_period.as_ref())
            // Last data point is always 1 minute before "end" time
            .map(|c| c.regular.end - 60);

        let start_idx = data
            .iter()
            .enumerate()
            .find(|(_, p)| Some(p.date) >= reg_start)
            .map(|(idx, _)| idx);

        let end_idx = data
            .iter()
            .enumerate()
            .find(|(_, p)| Some(p.date) >= reg_end)
            .map(|(idx, _)| idx);

        (start_idx, end_idx)
    }

    pub fn current_trading_period(&self, data: &[Price]) -> TradingPeriod {
        let (reg_start, reg_end) = self.regular_start_end_idx(data);

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

    pub fn min_max(&self, data: &[Price]) -> (f64, f64) {
        let mut data: Vec<_> = data.iter().map(cast_historical_as_price).collect();
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

    pub fn high_low(&self, data: &[Price]) -> (f64, f64) {
        let mut data = data.to_vec();

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

    pub fn x_bounds(&self, start: i64, end: i64, data: &[Price]) -> [f64; 2] {
        let num_points = ((end - start) / 60) as f64;

        match self.time_frame {
            TimeFrame::Day1 => [0.0, num_points],
            _ => [0.0, (data.len() + 1) as f64],
        }
    }

    pub fn x_labels(&self, width: u16, start: i64, end: i64, data: &[Price]) -> Vec<Span> {
        let mut labels = vec![];

        let dates = if self.time_frame == TimeFrame::Day1 {
            MarketHours(start, end.max(start)).collect()
        } else {
            data.iter().map(|p| p.date).collect::<Vec<_>>()
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
                labels.push(chunk.get(0).map_or(Span::raw("".to_string()), |d| {
                    Span::styled(
                        self.time_frame.format_time(*d),
                        Style::default().fg(THEME.text_normal),
                    )
                }));
            }

            labels.push(
                chunk
                    .get(chunk.len() - 1)
                    .map_or(Span::raw("".to_string()), |d| {
                        Span::styled(
                            self.time_frame.format_time(*d),
                            Style::default().fg(THEME.text_normal),
                        )
                    }),
            );
        }

        labels
    }

    pub fn y_bounds(&self, min: f64, max: f64) -> [f64; 2] {
        [(min - 0.05), (max + 0.05)]
    }

    pub fn y_labels(&self, min: f64, max: f64) -> Vec<Span> {
        if self.loaded() {
            vec![
                Span::styled(
                    format!("{:>8.2}", (min - 0.05)),
                    Style::default().fg(THEME.text_normal),
                ),
                Span::styled(
                    format!("{:>8.2}", ((min - 0.05) + (max + 0.05)) / 2.0),
                    Style::default().fg(THEME.text_normal),
                ),
                Span::styled(
                    format!("{:>8.2}", max + 0.05),
                    Style::default().fg(THEME.text_normal),
                ),
            ]
        } else {
            vec![
                Span::raw("       ".to_string()),
                Span::raw("       ".to_string()),
                Span::raw("       ".to_string()),
            ]
        }
    }

    pub fn pct_change(&self, data: &[Price]) -> f64 {
        if data.iter().filter(|p| p.close > 0.0).count() == 0 {
            return 0.0;
        }

        let baseline = if self.time_frame == TimeFrame::Day1 {
            if let Some(prev_close) = self.prev_close_price {
                prev_close
            } else {
                data.iter()
                    .find(|p| p.close > 0.0)
                    .map(|d| d.close)
                    .unwrap()
            }
        } else {
            data.iter()
                .find(|p| p.close > 0.0)
                .map(|d| d.close)
                .unwrap()
        };

        self.current_price() / baseline - 1.0
    }

    pub fn loaded(&self) -> bool {
        !self.prices[self.time_frame.idx()].is_empty()
            && self.current_price() > 0.0
            && self.profile.is_some()
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
        self.render_cached(area, buf, state);
    }
}

impl CachableWidget<StockState> for StockWidget {
    fn cache_state_mut(state: &mut StockState) -> &mut CacheState {
        &mut state.cache_state
    }

    fn render(self, mut area: Rect, buf: &mut Buffer, state: &mut <Self as StatefulWidget>::State) {
        let data = state.prices().collect::<Vec<_>>();

        let pct_change = state.pct_change(&data);

        let show_x_labels = SHOW_X_LABELS.read().map_or(false, |l| *l);
        let enable_pre_post = *ENABLE_PRE_POST.read().unwrap();
        let show_volumes = *SHOW_VOLUMES.read().unwrap();

        let loaded = state.loaded();

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
            block::new(&format!(
                " {}{:<4} ",
                state.symbol,
                if loaded {
                    format!(" - {}", company_name)
                } else if state.profile.is_some() {
                    format!(" - {}{:<4}", company_name, loading_indicator)
                } else {
                    loading_indicator
                }
            ))
            .render(area, buf);
            area = add_padding(area, 1, PaddingDirection::All);
            area = add_padding(area, 1, PaddingDirection::Left);
            area = add_padding(area, 1, PaddingDirection::Right);
        }

        // chunks[0] - Company Info
        // chunks[1] - Graph - fill remaining space
        // chunks[2] - Time Frame Tabs
        let chunks = Layout::default()
            .constraints(
                [
                    Constraint::Length(7),
                    Constraint::Min(0),
                    Constraint::Length(2),
                ]
                .as_ref(),
            )
            .split(area);

        // Draw company info
        {
            // info_chunks[0] - Prices / volumes
            // info_chunks[1] - Toggle block
            let mut info_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Min(0), Constraint::Length(15)].as_ref())
                .split(chunks[0]);
            info_chunks[0] = add_padding(info_chunks[0], 1, PaddingDirection::Top);

            let (high, low) = state.high_low(&data);
            let vol = state.reg_mkt_volume.clone().unwrap_or_default();

            let company_info = vec![
                Spans::from(vec![
                    Span::styled("c: ", Style::default()),
                    Span::styled(
                        if loaded {
                            format!("{:.2} {}", state.current_price(), currency)
                        } else {
                            "".to_string()
                        },
                        Style::default()
                            .add_modifier(Modifier::BOLD)
                            .fg(THEME.text_primary),
                    ),
                    Span::styled(
                        if loaded {
                            format!("  {:.2}%", pct_change * 100.0)
                        } else {
                            "".to_string()
                        },
                        Style::default()
                            .add_modifier(Modifier::BOLD)
                            .fg(if pct_change >= 0.0 {
                                THEME.profit
                            } else {
                                THEME.loss
                            }),
                    ),
                ]),
                Spans::from(vec![
                    Span::styled("h: ", Style::default()),
                    Span::styled(
                        if loaded {
                            format!("{:.2}", high)
                        } else {
                            "".to_string()
                        },
                        Style::default().fg(THEME.text_secondary),
                    ),
                ]),
                Spans::from(vec![
                    Span::styled("l: ", Style::default()),
                    Span::styled(
                        if loaded {
                            format!("{:.2}", low)
                        } else {
                            "".to_string()
                        },
                        Style::default().fg(THEME.text_secondary),
                    ),
                ]),
                Spans::default(),
                Spans::from(vec![
                    Span::styled("v: ", Style::default()),
                    Span::styled(
                        if loaded { vol } else { "".to_string() },
                        Style::default().fg(THEME.text_secondary),
                    ),
                ]),
            ];

            Paragraph::new(company_info)
                .style(
                    Style::default()
                        .fg(THEME.text_normal)
                        .bg(THEME.background()),
                )
                .alignment(Alignment::Left)
                .wrap(Wrap { trim: true })
                .render(info_chunks[0], buf);

            if !*HIDE_TOGGLE {
                let toggle_block = block::new(" Toggle ");
                toggle_block.render(info_chunks[1], buf);
                info_chunks[1] = add_padding(info_chunks[1], 2, PaddingDirection::Left);
                info_chunks[1] = add_padding(info_chunks[1], 1, PaddingDirection::Top);
                info_chunks[1] = add_padding(info_chunks[1], 1, PaddingDirection::Right);
                info_chunks[1] = add_padding(info_chunks[1], 1, PaddingDirection::Bottom);

                let mut toggle_info =
                    vec![Spans::from(Span::styled("Summary  's'", Style::default()))];

                if loaded {
                    toggle_info.push(Spans::from(Span::styled(
                        "Volumes  'v'",
                        Style::default().bg(if show_volumes {
                            THEME.highlight_unfocused
                        } else {
                            THEME.background()
                        }),
                    )));

                    toggle_info.push(Spans::from(Span::styled(
                        "X Labels 'x'",
                        Style::default().bg(if show_x_labels {
                            THEME.highlight_unfocused
                        } else {
                            THEME.background()
                        }),
                    )));

                    toggle_info.push(Spans::from(Span::styled(
                        "Pre Post 'p'",
                        Style::default().bg(if enable_pre_post {
                            THEME.highlight_unfocused
                        } else {
                            THEME.background()
                        }),
                    )));
                }

                if state.options_enabled() && loaded {
                    toggle_info.push(Spans::from(Span::styled(
                        "Options  'o'",
                        Style::default().bg(if state.show_options {
                            THEME.highlight_unfocused
                        } else {
                            THEME.background()
                        }),
                    )));
                }

                Paragraph::new(toggle_info)
                    .style(
                        Style::default()
                            .fg(THEME.text_normal)
                            .bg(THEME.background()),
                    )
                    .alignment(Alignment::Left)
                    .render(info_chunks[1], buf);
            }
        }

        // Draw graph
        {
            let (min, max) = state.min_max(&data);
            let (start, end) = state.start_end();

            let mut prices: Vec<_> = data.iter().map(cast_historical_as_price).collect();

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
                state.x_labels(chunks[1].width, start, end, &data)
            } else {
                vec![]
            };

            let trading_period = state.current_trading_period(&data);

            let (reg_prices, pre_prices, post_prices) = if loaded {
                let (start_idx, end_idx) = state.regular_start_end_idx(&data);

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
                            let pre_end_idx = if let Some(start_idx) = start_idx {
                                start_idx
                            } else {
                                prices.len()
                            };

                            if pre_end_idx > 0 {
                                Some(
                                    prices
                                        .iter()
                                        .enumerate()
                                        .filter(|(idx, _)| *idx <= pre_end_idx)
                                        .map(cast_as_dataset)
                                        .collect::<Vec<(f64, f64)>>(),
                                )
                            } else {
                                None
                            }
                        },
                        {
                            if let Some(post_start_idx) = end_idx {
                                Some(
                                    prices
                                        .iter()
                                        .enumerate()
                                        .filter(|(idx, _)| *idx >= post_start_idx)
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
                .style(
                    Style::default()
                        .fg(
                            if trading_period != TradingPeriod::Regular && enable_pre_post {
                                THEME.foreground_inactive
                            } else if pct_change >= 0.0 {
                                THEME.profit
                            } else {
                                THEME.loss
                            },
                        )
                        .bg(THEME.background()),
                )
                .graph_type(graph_type)
                .data(&reg_prices)];

            if let Some(data) = post_prices.as_ref() {
                datasets.push(
                    Dataset::default()
                        .marker(Marker::Braille)
                        .style(
                            Style::default()
                                .fg(if trading_period != TradingPeriod::Post {
                                    THEME.foreground_inactive
                                } else if pct_change >= 0.0 {
                                    THEME.profit
                                } else {
                                    THEME.loss
                                })
                                .bg(THEME.background()),
                        )
                        .graph_type(GraphType::Line)
                        .data(&data),
                );
            }

            if let Some(data) = pre_prices.as_ref() {
                datasets.insert(
                    0,
                    Dataset::default()
                        .marker(Marker::Braille)
                        .style(
                            Style::default()
                                .fg(if trading_period != TradingPeriod::Pre {
                                    THEME.foreground_inactive
                                } else if pct_change >= 0.0 {
                                    THEME.profit
                                } else {
                                    THEME.loss
                                })
                                .bg(THEME.background()),
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
                        .style(
                            Style::default()
                                .fg(THEME.foreground_inactive)
                                .bg(THEME.background()),
                        )
                        .graph_type(GraphType::Line)
                        .data(&data),
                );
            }

            // graph_chunks[0] = prices
            // graph_chunks[1] = volume
            let graph_chunks = if show_volumes {
                Layout::default()
                    .constraints([Constraint::Min(6), Constraint::Length(5)].as_ref())
                    .split(chunks[1])
            } else {
                Layout::default()
                    .constraints([Constraint::Min(0)].as_ref())
                    .split(chunks[1])
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

                let volumes = state.volumes(&data);
                let vol_count = volumes.len();

                if vol_count > 0 {
                    let volumes = data
                        .iter()
                        .map(|p| [p.volume].repeat(num_bars))
                        .flatten()
                        .chunks(vol_count)
                        .into_iter()
                        .map(|c| ("", c.sum::<u64>() / vol_count as u64))
                        .collect::<Vec<_>>();

                    volume_chunks.x -= 1;

                    Block::default()
                        .borders(Borders::LEFT)
                        .border_style(Style::default().fg(THEME.border_axis))
                        .render(volume_chunks, buf);

                    volume_chunks.x += 1;

                    BarChart::default()
                        .bar_gap(0)
                        .bar_set(bar::NINE_LEVELS)
                        .style(
                            Style::default()
                                .fg(THEME.foreground_inactive)
                                .bg(THEME.background()),
                        )
                        .data(&volumes)
                        .render(volume_chunks, buf);
                }
            }

            Chart::new(datasets)
                .style(Style::default().bg(THEME.background()))
                .block(
                    Block::default()
                        .style(Style::default().fg(THEME.border_secondary))
                        .borders(Borders::TOP)
                        .border_style(Style::default()),
                )
                .x_axis({
                    let axis = Axis::default().bounds(state.x_bounds(start, end, &data));

                    if show_x_labels && loaded {
                        axis.labels(x_labels)
                            .style(Style::default().fg(THEME.border_axis))
                    } else {
                        axis
                    }
                })
                .y_axis(
                    Axis::default()
                        .bounds(state.y_bounds(min, max))
                        .labels(state.y_labels(min, max))
                        .style(Style::default().fg(THEME.border_axis)),
                )
                .render(graph_chunks[0], buf);
        }

        // Draw time frame tabs
        {
            let tab_names = TimeFrame::tab_names()
                .iter()
                .map(|s| Spans::from(*s))
                .collect();

            Tabs::new(tab_names)
                .block(
                    Block::default().borders(Borders::TOP).border_style(
                        Style::default()
                            .fg(THEME.border_secondary)
                            .bg(THEME.background()),
                    ),
                )
                .select(state.time_frame.idx())
                .style(Style::default().fg(THEME.text_secondary))
                .highlight_style(Style::default().fg(THEME.text_primary))
                .render(chunks[2], buf);
        }
    }
}
