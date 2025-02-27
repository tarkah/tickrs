use std::hash::{Hash, Hasher};

use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::Modifier;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, StatefulWidget, Tabs, Widget, Wrap};

use super::chart::{
    ChartState, PricesCandlestickChart, PricesKagiChart, PricesLineChart, VolumeBarChart,
};
use super::chart_configuration::ChartConfigurationState;
use super::{block, CachableWidget, CacheState, OptionsState};
use crate::api::model::{ChartMeta, CompanyData};
use crate::common::*;
use crate::draw::{add_padding, PaddingDirection};
use crate::service::{self, Service};
use crate::theme::style;
use crate::{
    DEFAULT_TIMESTAMPS, ENABLE_PRE_POST, HIDE_PREV_CLOSE, HIDE_TOGGLE, OPTS, SHOW_VOLUMES,
    SHOW_X_LABELS, THEME, TIME_FRAME, TRUNC_PRE,
};

const NUM_LOADING_TICKS: usize = 4;

pub struct StockState {
    pub symbol: String,
    pub chart_type: ChartType,
    pub stock_service: service::stock::StockService,
    pub profile: Option<CompanyData>,
    pub current_regular_price: f64,
    pub current_post_price: Option<f64>,
    pub prev_close_price: Option<f64>,
    pub reg_mkt_volume: Option<String>,
    pub prices: [Vec<Price>; 7],
    pub time_frame: TimeFrame,
    pub show_options: bool,
    pub show_configure: bool,
    pub options: Option<OptionsState>,
    pub chart_configuration: ChartConfigurationState,
    pub loading_tick: usize,
    pub prev_state_loaded: bool,
    pub chart_meta: Option<ChartMeta>,
    pub chart_state: Option<ChartState>,
    pub cache_state: CacheState,
}

impl Hash for StockState {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.symbol.hash(state);
        self.chart_type.hash(state);
        self.current_regular_price.to_bits().hash(state);
        // Only fetched once, so just need to check if Some
        self.profile.is_some().hash(state);
        self.current_post_price.map(|f| f.to_bits()).hash(state);
        self.prev_close_price.map(|f| f.to_bits()).hash(state);
        self.reg_mkt_volume.hash(state);
        self.prices.hash(state);
        self.time_frame.hash(state);
        self.show_options.hash(state);
        self.show_configure.hash(state);
        self.chart_configuration.hash(state);
        self.loading_tick.hash(state);
        self.prev_state_loaded.hash(state);
        self.chart_meta.hash(state);

        if let Some(chart_state) = self.chart_state.as_ref() {
            chart_state.hash(state);
        }

        // Hash globals since they affect "state" of how widget is rendered
        DEFAULT_TIMESTAMPS.read().get(&self.time_frame).hash(state);
        ENABLE_PRE_POST.read().hash(state);
        HIDE_PREV_CLOSE.hash(state);
        HIDE_TOGGLE.hash(state);
        SHOW_VOLUMES.read().hash(state);
        SHOW_X_LABELS.read().hash(state);
        TRUNC_PRE.hash(state);
    }
}

impl StockState {
    pub fn new(symbol: String, chart_type: ChartType) -> StockState {
        let time_frame = *TIME_FRAME;

        let stock_service = service::stock::StockService::new(symbol.clone(), time_frame);
        let kagi_options = OPTS.kagi_options.get(&symbol).cloned().unwrap_or_default();

        StockState {
            symbol,
            chart_type,
            stock_service,
            profile: None,
            current_regular_price: 0.0,
            current_post_price: None,
            prev_close_price: None,
            reg_mkt_volume: None,
            prices: [vec![], vec![], vec![], vec![], vec![], vec![], vec![]],
            time_frame,
            show_options: false,
            show_configure: false,
            options: None,
            chart_configuration: ChartConfigurationState {
                kagi_options,
                ..Default::default()
            },
            loading_tick: NUM_LOADING_TICKS,
            prev_state_loaded: false,
            chart_meta: None,
            cache_state: Default::default(),
            chart_state: None,
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

        // Resets chart state where applicable
        self.set_chart_type(self.chart_type);
    }

    pub fn prices(&self) -> impl Iterator<Item = Price> {
        let (start, end) = self.start_end();

        let prices = self.prices[self.time_frame.idx()].clone();

        let max_time = prices.last().map(|p| p.date).unwrap_or(end);

        let default_timestamps = {
            let defaults = DEFAULT_TIMESTAMPS.read();
            defaults.get(&self.time_frame).cloned()
        };

        let prices = if self.time_frame == TimeFrame::Day1 {
            let times = MarketHours(
                start,
                if max_time < start {
                    end.max(start)
                } else {
                    max_time.min(end)
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
        let enable_pre_post = { *ENABLE_PRE_POST.read() };

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
                    self.profile = Some(*data);
                }
            }
        }
    }

    fn options_enabled(&self) -> bool {
        !self.is_crypto() && !self.is_index()
    }

    fn configure_enabled(&self) -> bool {
        self.chart_type == ChartType::Kagi
    }

    fn is_crypto(&self) -> bool {
        self.chart_meta
            .as_ref()
            .and_then(|m| m.instrument_type.as_deref())
            == Some("CRYPTOCURRENCY")
    }

    fn is_index(&self) -> bool {
        self.chart_meta
            .as_ref()
            .and_then(|m| m.instrument_type.as_deref())
            == Some("INDEX")
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

    pub fn toggle_configure(&mut self) -> bool {
        if !self.configure_enabled() {
            return false;
        }

        self.show_configure = !self.show_configure;

        self.chart_configuration.reset_form(self.time_frame);

        true
    }

    pub fn start_end(&self) -> (i64, i64) {
        let enable_pre_post = { *ENABLE_PRE_POST.read() };

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
        let (mut max, mut min) = self.high_low(data);

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
        data.push(Price {
            close: self.current_price(),
            open: self.current_price(),
            high: self.current_price(),
            low: self.current_price(),
            ..Default::default()
        });
        data.retain(|p| p.close.gt(&0.0));

        let high = data
            .iter()
            .max_by(|a, b| a.high.partial_cmp(&b.high).unwrap())
            .map(|p| p.high)
            .unwrap_or(1.0);
        let low = data
            .iter()
            .min_by(|a, b| a.low.partial_cmp(&b.low).unwrap())
            .map(|p| p.low)
            .unwrap_or(0.0);

        (high, low)
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
            .first()
            .map_or(0, |d| self.time_frame.format_time(*d).len())
            + 5;

        let num_labels = width as usize / label_len;

        if num_labels == 0 {
            return labels;
        }

        for i in 0..num_labels {
            let idx = i * (dates.len() - 1) / (num_labels.max(2) - 1);

            let timestamp = dates.get(idx).unwrap();

            let label = Span::styled(
                self.time_frame.format_time(*timestamp),
                style().fg(THEME.text_normal()),
            );

            labels.push(label);
        }

        labels
    }

    pub fn y_bounds(&self, min: f64, max: f64) -> [f64; 2] {
        [(min), (max)]
    }

    pub fn y_labels(&self, min: f64, max: f64) -> Vec<Span> {
        if self.loaded() {
            vec![
                Span::styled(
                    format!("{:>8}", format_decimals(min)),
                    style().fg(THEME.text_normal()),
                ),
                Span::styled(
                    format!("{:>8}", format_decimals((min + max) / 2.0)),
                    style().fg(THEME.text_normal()),
                ),
                Span::styled(
                    format!("{:>8}", format_decimals(max)),
                    style().fg(THEME.text_normal()),
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
        !self.prices[self.time_frame.idx()].is_empty() && self.current_price() > 0.0
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

    pub fn set_chart_type(&mut self, chart_type: ChartType) {
        self.chart_state.take();

        if chart_type == ChartType::Kagi {
            self.chart_state = Some(Default::default());
        }

        self.chart_type = chart_type;
    }

    pub fn chart_state_mut(&mut self) -> Option<&mut ChartState> {
        self.chart_state.as_mut()
    }

    pub fn chart_config_mut(&mut self) -> &mut ChartConfigurationState {
        &mut self.chart_configuration
    }
}

pub struct StockWidget {}

impl StatefulWidget for StockWidget {
    type State = StockState;

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

        let chart_type = state.chart_type;
        let show_x_labels = *SHOW_X_LABELS.read();
        let enable_pre_post = *ENABLE_PRE_POST.read();
        let show_volumes = *SHOW_VOLUMES.read() && chart_type != ChartType::Kagi;

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
        let mut chunks: Vec<Rect> = Layout::default()
            .constraints(
                [
                    Constraint::Length(6),
                    Constraint::Min(0),
                    Constraint::Length(2),
                ]
                .as_ref(),
            )
            .split(area)
            .to_vec();

        // Draw company info
        {
            // info_chunks[0] - Prices / volumes
            // info_chunks[1] - Toggle block
            let mut info_chunks: Vec<Rect> = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Min(23), Constraint::Length(29)].as_ref())
                .split(chunks[0])
                .to_vec();
            info_chunks[0] = add_padding(info_chunks[0], 1, PaddingDirection::Top);

            let (high, low) = state.high_low(&data);
            let current_fmt = format_decimals(state.current_price());
            let high_fmt = format_decimals(high);
            let low_fmt = format_decimals(low);

            let vol = state.reg_mkt_volume.clone().unwrap_or_default();

            let company_info = vec![
                Line::from(vec![
                    Span::styled("C: ", style()),
                    Span::styled(
                        if loaded {
                            format!("{} {}", current_fmt, currency)
                        } else {
                            "".to_string()
                        },
                        style()
                            .add_modifier(Modifier::BOLD)
                            .fg(THEME.text_primary()),
                    ),
                    Span::styled(
                        if loaded {
                            format!("  {:.2}%", pct_change * 100.0)
                        } else {
                            "".to_string()
                        },
                        style()
                            .add_modifier(Modifier::BOLD)
                            .fg(if pct_change >= 0.0 {
                                THEME.profit()
                            } else {
                                THEME.loss()
                            }),
                    ),
                ]),
                Line::from(vec![
                    Span::styled("H: ", style()),
                    Span::styled(
                        if loaded { high_fmt } else { "".to_string() },
                        style().fg(THEME.text_secondary()),
                    ),
                ]),
                Line::from(vec![
                    Span::styled("L: ", style()),
                    Span::styled(
                        if loaded { low_fmt } else { "".to_string() },
                        style().fg(THEME.text_secondary()),
                    ),
                ]),
                Line::default(),
                Line::from(vec![
                    Span::styled("Volume: ", style()),
                    Span::styled(
                        if loaded { vol } else { "".to_string() },
                        style().fg(THEME.text_secondary()),
                    ),
                ]),
            ];

            Paragraph::new(company_info)
                .style(style().fg(THEME.text_normal()))
                .alignment(Alignment::Left)
                .wrap(Wrap { trim: true })
                .render(info_chunks[0], buf);

            if !*HIDE_TOGGLE {
                let toggle_block = block::new(" Toggle ");
                toggle_block.render(info_chunks[1], buf);

                info_chunks[1] = add_padding(info_chunks[1], 1, PaddingDirection::All);
                info_chunks[1] = add_padding(info_chunks[1], 1, PaddingDirection::Left);

                let toggle_chunks: Vec<Rect> = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([
                        Constraint::Length(12),
                        Constraint::Length(2),
                        Constraint::Length(12),
                    ])
                    .split(info_chunks[1])
                    .to_vec();

                let mut left_info = vec![Line::from(Span::styled("Summary  's'", style()))];
                let mut right_info = vec![];

                if loaded {
                    left_info.push(Line::from(Span::styled(
                        format!("{: <8} 'c'", chart_type.as_str()),
                        style(),
                    )));

                    left_info.push(Line::from(Span::styled(
                        "Volumes  'v'",
                        style()
                            .bg(if show_volumes {
                                THEME.highlight_unfocused()
                            } else {
                                THEME.background()
                            })
                            .fg(if chart_type == ChartType::Kagi {
                                THEME.gray()
                            } else {
                                THEME.text_normal()
                            }),
                    )));

                    left_info.push(Line::from(Span::styled(
                        "X Labels 'x'",
                        style().bg(if show_x_labels {
                            THEME.highlight_unfocused()
                        } else {
                            THEME.background()
                        }),
                    )));

                    right_info.push(Line::from(Span::styled(
                        "Pre Post 'p'",
                        style().bg(if enable_pre_post {
                            THEME.highlight_unfocused()
                        } else {
                            THEME.background()
                        }),
                    )));

                    right_info.push(Line::from(Span::styled(
                        "Edit     'e'",
                        style()
                            .bg(if state.show_configure {
                                THEME.highlight_unfocused()
                            } else {
                                THEME.background()
                            })
                            .fg(if state.configure_enabled() {
                                THEME.text_normal()
                            } else {
                                THEME.gray()
                            }),
                    )));
                }

                if state.options_enabled() && loaded {
                    right_info.push(Line::from(Span::styled(
                        "Options  'o'",
                        style().bg(if state.show_options {
                            THEME.highlight_unfocused()
                        } else {
                            THEME.background()
                        }),
                    )));
                }

                Paragraph::new(left_info)
                    .style(style().fg(THEME.text_normal()))
                    .alignment(Alignment::Left)
                    .render(toggle_chunks[0], buf);

                Paragraph::new(right_info)
                    .style(style().fg(THEME.text_normal()))
                    .alignment(Alignment::Left)
                    .render(toggle_chunks[2], buf);
            }
        }

        // graph_chunks[0] = prices
        // graph_chunks[1] = volume
        let graph_chunks: Vec<Rect> = if show_volumes {
            Layout::default()
                .constraints([Constraint::Min(6), Constraint::Length(5)].as_ref())
                .split(chunks[1])
                .to_vec()
        } else {
            Layout::default()
                .constraints([Constraint::Min(0)].as_ref())
                .split(chunks[1])
                .to_vec()
        };

        // Draw prices line chart
        match chart_type {
            ChartType::Line => {
                PricesLineChart {
                    data: &data,
                    enable_pre_post,
                    is_profit: pct_change >= 0.0,
                    is_summary: false,
                    loaded,
                    show_x_labels,
                }
                .render(graph_chunks[0], buf, state);
            }
            ChartType::Candlestick => {
                PricesCandlestickChart {
                    data: &data,
                    loaded,
                    show_x_labels,
                    is_summary: false,
                }
                .render(graph_chunks[0], buf, state);
            }
            ChartType::Kagi => {
                PricesKagiChart {
                    data: &data,
                    loaded,
                    show_x_labels,
                    is_summary: false,
                    kagi_options: state.chart_configuration.kagi_options.clone(),
                }
                .render(graph_chunks[0], buf, state);
            }
        }

        // Draw volumes bar chart
        if show_volumes {
            VolumeBarChart {
                data: &data,
                loaded,
                show_x_labels,
            }
            .render(graph_chunks[1], buf, state);
        }

        // Draw time frame tabs & optional chart scroll indicators
        {
            Block::default()
                .borders(Borders::TOP)
                .border_style(style().fg(THEME.border_secondary()))
                .render(chunks[2], buf);
            chunks[2] = add_padding(chunks[2], 1, PaddingDirection::Top);

            // layout[0] - timeframe
            // layout[1] - scroll indicators
            let layout: Vec<Rect> = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(if state.chart_state.is_some() {
                    [Constraint::Min(0), Constraint::Length(3)].as_ref()
                } else {
                    [Constraint::Min(0)].as_ref()
                })
                .split(chunks[2])
                .to_vec();

            let tab_names = TimeFrame::tab_names()
                .iter()
                .map(|s| Line::from(*s))
                .collect();

            Tabs::new(tab_names)
                .select(state.time_frame.idx())
                .style(style().fg(THEME.text_secondary()))
                .highlight_style(style().fg(THEME.text_primary()))
                .render(layout[0], buf);

            if let Some(chart_state) = state.chart_state.as_ref() {
                let more_left = chart_state.offset.unwrap_or_default()
                    < chart_state.max_offset.unwrap_or_default();
                let more_right = chart_state.offset.is_some();

                let left_arrow = Span::styled(
                    "ᐸ",
                    style().fg(if more_left {
                        THEME.text_normal()
                    } else {
                        THEME.gray()
                    }),
                );
                let right_arrow = Span::styled(
                    "ᐳ",
                    style().fg(if more_right {
                        THEME.text_normal()
                    } else {
                        THEME.gray()
                    }),
                );

                Paragraph::new(Line::from(vec![left_arrow, Span::raw(" "), right_arrow]))
                    .render(layout[1], buf);
            }
        }
    }
}
