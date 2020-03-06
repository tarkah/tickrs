use crate::widget;

#[derive(PartialEq)]
pub enum Mode {
    AddStock,
    DisplayStock,
    Help,
}

pub struct App {
    pub mode: Mode,
    pub stocks: Vec<widget::StockState>,
    pub add_stock: widget::AddStockState,
    pub help: widget::HelpWidget,
    pub current_tab: usize,
    pub hide_help: bool,
}
