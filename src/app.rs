use crossterm::event::Event;

use crate::common::TimeFrame;
use crate::widget;

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
}

#[derive(Debug)]
pub struct DebugInfo {
    pub enabled: bool,
    pub dimensions: (u16, u16),
    pub cursor_location: Option<(u16, u16)>,
    pub last_event: Option<Event>,
    pub mode: Mode,
}
