use tui::buffer::Buffer;
use tui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use tui::style::Modifier;
use tui::text::{Span, Spans};
use tui::widgets::{Block, Borders, Paragraph, StatefulWidget, Widget};

use super::chart::{PricesCandlestickChart, PricesKagiChart, PricesLineChart, VolumeBarChart};
use super::stock::StockState;
use super::{CachableWidget, CacheState};
use crate::common::{format_decimals, ChartType};
use crate::draw::{add_padding, PaddingDirection};
use crate::theme::style;
use crate::{ENABLE_PRE_POST, SHOW_VOLUMES, THEME};

pub struct StockSummaryWidget {}

impl StatefulWidget for StockSummaryWidget {
    type State = StockState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        self.render_cached(area, buf, state);
    }
}

impl CachableWidget<StockState> for StockSummaryWidget {
    fn cache_state_mut(state: &mut StockState) -> &mut CacheState {
        &mut state.cache_state
    }

    fn render(self, mut area: Rect, buf: &mut Buffer, state: &mut <Self as StatefulWidget>::State) {
        let data = state.prices().collect::<Vec<_>>();

        let pct_change = state.pct_change(&data);

        let chart_type = state.chart_type;
        let enable_pre_post = *ENABLE_PRE_POST.read().unwrap();
        let show_volumes = *SHOW_VOLUMES.read().unwrap() && chart_type != ChartType::Kagi;

        let loaded = state.loaded();

        let (company_name, currency) = match state.profile.as_ref() {
            Some(profile) => (
                profile.price.short_name.as_str(),
                profile.price.currency.as_deref().unwrap_or("USD"),
            ),
            None => ("", ""),
        };

        let loading_indicator = ".".repeat(state.loading_tick);

        let title = &format!(
            " {}{}",
            state.symbol,
            if state.profile.is_some() {
                format!(" - {}", company_name)
            } else {
                "".to_string()
            }
        );
        Block::default()
            .title(Span::styled(
                format!(
                    " {}{} ",
                    &title[..24.min(title.len())],
                    if loaded {
                        "".to_string()
                    } else {
                        format!("{:<4}", loading_indicator)
                    }
                ),
                style().fg(THEME.text_normal()),
            ))
            .borders(Borders::TOP)
            .border_style(style().fg(THEME.border_secondary()))
            .render(area, buf);
        area = add_padding(area, 1, PaddingDirection::Top);

        let mut layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(25), Constraint::Min(0)].as_ref())
            .split(area);

        {
            layout[0] = add_padding(layout[0], 1, PaddingDirection::Left);
            layout[0] = add_padding(layout[0], 2, PaddingDirection::Right);

            let (high, low) = state.high_low(&data);
            let current_fmt = format_decimals(state.current_price());
            let high_fmt = format_decimals(high);
            let low_fmt = format_decimals(low);

            let vol = state.reg_mkt_volume.clone().unwrap_or_default();

            let prices = vec![
                Spans::from(vec![
                    Span::styled("C: ", style().fg(THEME.text_normal())),
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
                ]),
                Spans::from(vec![
                    Span::styled("H: ", style().fg(THEME.text_normal())),
                    Span::styled(
                        if loaded { high_fmt } else { "".to_string() },
                        style().fg(THEME.text_secondary()),
                    ),
                ]),
                Spans::from(vec![
                    Span::styled("L: ", style().fg(THEME.text_normal())),
                    Span::styled(
                        if loaded { low_fmt } else { "".to_string() },
                        style().fg(THEME.text_secondary()),
                    ),
                ]),
                Spans::default(),
                Spans::from(vec![
                    Span::styled("Volume: ", style().fg(THEME.text_normal())),
                    Span::styled(
                        if loaded { vol } else { "".to_string() },
                        style().fg(THEME.text_secondary()),
                    ),
                ]),
            ];

            let pct = vec![Span::styled(
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
            )];

            Paragraph::new(prices)
                .style(style())
                .alignment(Alignment::Left)
                .render(layout[0], buf);

            Paragraph::new(Spans::from(pct))
                .style(style())
                .alignment(Alignment::Right)
                .render(layout[0], buf);
        }

        // graph_chunks[0] = prices
        // graph_chunks[1] = volume
        let graph_chunks = if show_volumes {
            Layout::default()
                .constraints([Constraint::Min(5), Constraint::Length(1)].as_ref())
                .split(layout[1])
        } else {
            Layout::default()
                .constraints([Constraint::Min(0)].as_ref())
                .split(layout[1])
        };

        // Draw prices line chart
        match chart_type {
            ChartType::Line => {
                PricesLineChart {
                    data: &data,
                    enable_pre_post,
                    is_profit: pct_change >= 0.0,
                    is_summary: true,
                    loaded,
                    show_x_labels: false,
                }
                .render(graph_chunks[0], buf, state);
            }
            ChartType::Candlestick => {
                PricesCandlestickChart {
                    data: &data,
                    loaded,
                    show_x_labels: false,
                    is_summary: true,
                }
                .render(graph_chunks[0], buf, state);
            }
            ChartType::Kagi => {
                PricesKagiChart {
                    data: &data,
                    loaded,
                    show_x_labels: false,
                    is_summary: true,
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
                show_x_labels: false,
            }
            .render(graph_chunks[1], buf, state);
        }
    }
}
