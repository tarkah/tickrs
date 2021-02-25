pub use self::prices_candlestick::PricesCandlestickChart;
pub use self::prices_kagi::PricesKagiChart;
pub use self::prices_line::PricesLineChart;
pub use self::volume_bar::VolumeBarChart;

mod prices_candlestick;
pub mod prices_kagi;
mod prices_line;
mod volume_bar;

const SCROLL_STEP: usize = 2;

#[derive(Debug, Default, Clone, Copy, Hash)]
pub struct ChartState {
    pub max_offset: Option<usize>,
    pub offset: Option<usize>,
    queued_scroll: Option<ChartScrollDirection>,
}

impl ChartState {
    pub fn scroll_left(&mut self) {
        self.queued_scroll = Some(ChartScrollDirection::Left);
    }

    pub fn scroll_right(&mut self) {
        self.queued_scroll = Some(ChartScrollDirection::Right);
    }

    fn scroll(&mut self, direction: ChartScrollDirection, max_offset: usize) {
        if max_offset == 0 {
            return;
        }

        let new_offset = match direction {
            ChartScrollDirection::Left => self.offset.unwrap_or(0) + SCROLL_STEP,
            ChartScrollDirection::Right => self.offset.unwrap_or(0).saturating_sub(SCROLL_STEP),
        };

        self.offset = if new_offset == 0 {
            None
        } else {
            Some(new_offset.min(max_offset))
        };
    }

    fn offset(&mut self, max_offset: usize) -> usize {
        if max_offset == 0 {
            self.max_offset.take();
            self.offset.take();
        }

        self.max_offset = Some(max_offset);

        max_offset - self.offset.map(|o| o.min(max_offset)).unwrap_or(0)
    }
}

#[derive(Debug, Clone, Copy, Hash)]
enum ChartScrollDirection {
    Left,
    Right,
}
