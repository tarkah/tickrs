use itertools::Itertools;
use tui::buffer::Buffer;
use tui::layout::Rect;
use tui::style::Style;
use tui::symbols::bar;
use tui::widgets::{BarChart, Block, Borders, StatefulWidget, Widget};

use crate::common::{Price, TimeFrame};
use crate::widget::StockState;
use crate::THEME;

pub struct VolumeBarChart<'a> {
    pub data: &'a [Price],
    pub loaded: bool,
    pub show_x_labels: bool,
}

impl<'a> StatefulWidget for VolumeBarChart<'a> {
    type State = StockState;

    #[allow(clippy::clippy::unnecessary_unwrap)]
    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let mut volume_chunks = area;
        volume_chunks.height += 1;

        let x_offset = if !self.loaded {
            8
        } else if self.show_x_labels {
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

        let volumes = state.volumes(&self.data);
        let vol_count = volumes.len();

        if vol_count > 0 {
            let volumes = self
                .data
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
                .style(Style::default().fg(THEME.gray).bg(THEME.background()))
                .data(&volumes)
                .render(volume_chunks, buf);
        }
    }
}
