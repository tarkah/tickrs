use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use tui::buffer::{Buffer, Cell};
use tui::layout::Rect;
use tui::widgets::StatefulWidget;

pub use self::add_stock::{AddStockState, AddStockWidget};
pub use self::help::{HelpWidget, HELP_HEIGHT, HELP_WIDTH};
pub use self::options::{OptionsState, OptionsWidget};
pub use self::stock::{StockState, StockWidget};
pub use self::stock_summary::StockSummaryWidget;

mod add_stock;
pub mod block;
mod chart;
mod help;
pub mod options;
mod stock;
mod stock_summary;

pub trait CachableWidget<T: Hash>: StatefulWidget<State = T> + Sized {
    fn cache_state_mut(state: &mut <Self as StatefulWidget>::State) -> &mut CacheState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut <Self as StatefulWidget>::State);

    fn render_cached(
        self,
        area: Rect,
        buf: &mut Buffer,
        state: &mut <Self as StatefulWidget>::State,
    ) {
        // Hash our state
        let mut hasher = DefaultHasher::default();
        state.hash(&mut hasher);
        let hash = hasher.finish();

        // Get previously cached values
        let CacheState {
            prev_area,
            prev_content,
            prev_hash,
        } = <Self as CachableWidget<T>>::cache_state_mut(state).clone();

        // If current hash and layout matches previous, use cached buffer instead of re-rendering
        if hash == prev_hash && prev_area == area {
            for (idx, cell) in buf.content.iter_mut().enumerate() {
                let x = idx as u16 % buf.area.width;
                let y = idx as u16 / buf.area.width;

                if x >= area.x && x < area.x + area.width && y >= area.y && y < area.y + area.height
                {
                    if let Some(cached_cell) = prev_content.get(idx) {
                        *cell = cached_cell.clone();
                    }
                }
            }
        }
        // Otherwise re-render and store those values in the cache
        else {
            <Self as CachableWidget<T>>::render(self, area, buf, state);

            let cached_state = <Self as CachableWidget<T>>::cache_state_mut(state);
            cached_state.prev_hash = hash;
            cached_state.prev_area = area;
            cached_state.prev_content = buf.content.clone();
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct CacheState {
    prev_area: Rect,
    prev_hash: u64,
    prev_content: Vec<Cell>,
}
