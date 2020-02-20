use crate::widget;

#[derive(PartialEq)]
pub enum Mode {
    AddStock,
    DisplayStock,
}

pub struct App {
    pub mode: Mode,
    pub stocks: Vec<widget::StockWidget>,
    pub add_stock: widget::AddStockWidget,
    pub current_tab: usize,
}
