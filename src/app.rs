use crossterm::event::Event;

use crate::common::TimeFrame;
use crate::service::default_timestamps::DefaultTimestampService;
use crate::service::Service;
use crate::{widget, DEFAULT_TIMESTAMPS};

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum Mode {
    AddStock,
    DisplayStock,
    DisplayOptions,
    DisplaySummary,
    Help,
}

pub struct App {
    pub mode: Mode,
    pub stocks: Vec<widget::StockState>,
    pub add_stock: widget::AddStockState,
    pub help: widget::HelpWidget,
    pub current_tab: usize,
    pub hide_help: bool,
    pub debug: DebugInfo,
    pub previous_mode: Mode,
    pub summary_time_frame: TimeFrame,
    pub default_timestamp_service: DefaultTimestampService,
    pub summary_scroll_state: SummaryScrollState,
}

impl App {
    pub fn update(&self) {
        let mut timestamp_updates = self.default_timestamp_service.updates();

        if let Some(new_defaults) = timestamp_updates.pop() {
            *DEFAULT_TIMESTAMPS.write().unwrap() = new_defaults;
        }
    }
}

#[derive(Debug)]
pub struct DebugInfo {
    pub enabled: bool,
    pub dimensions: (u16, u16),
    pub last_event: Option<Event>,
    pub mode: Mode,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct SummaryScrollState {
    pub offset: usize,
    pub queued_scroll: Option<ScrollDirection>,
}

#[derive(Debug, Clone, Copy)]
pub enum ScrollDirection {
    Up,
    Down,
}
