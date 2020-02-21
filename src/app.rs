use crate::widget;

#[derive(PartialEq)]
pub enum Mode {
    AddStock,
    DisplayStock,
    Help,
}

pub struct App {
    pub mode: Mode,
    pub stocks: Vec<widget::StockWidget>,
    pub add_stock: widget::AddStockWidget,
    pub help: widget::HelpWidget,
    pub current_tab: usize,
    pub hide_help: bool,
}
