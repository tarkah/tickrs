use std::process;

use crossterm::event::Event;

use crate::common::{ChartType, TimeFrame};
use crate::widget::chart_configuration::ChartConfigurationState;
use crate::widget::options::SelectionMode;
use crate::{cleanup_terminal, widget, ENABLE_PRE_POST, SHOW_VOLUMES, SHOW_X_LABELS};

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Mode {
    AddStock,
    ConfigureChart,
    DisplayStock,
    DisplayOptions,
    DisplaySummary,
    Help,
}

pub struct App {
    pub num_to_render: usize,
    pub mode: Mode,
    pub stocks: Vec<widget::StockState>,
    pub add_stock: widget::AddStockState,
    pub help: widget::HelpWidget,
    pub current_tab: usize,
    pub hide_help: bool,
    pub debug: DebugInfo,
    pub previous_mode: Mode,
    pub time_frame: TimeFrame,
    pub summary_scroll_state: SummaryScrollState,
    pub chart_type: ChartType,
}

impl App {
    pub fn exit_app(&mut self) {
        cleanup_terminal();
        process::exit(0);
    }

    pub fn close(&mut self) {
        self.mode = self.previous_mode;
    }

    pub fn scroll_left(&mut self) {
        if let Some(stock) = self.stocks.get_mut(self.current_tab) {
            if let Some(chart_state) = stock.chart_state_mut() {
                chart_state.scroll_left();
            }
        }
    }

    pub fn scroll_right(&mut self) {
        if let Some(stock) = self.stocks.get_mut(self.current_tab) {
            if let Some(chart_state) = stock.chart_state_mut() {
                chart_state.scroll_right();
            }
        }
    }

    pub fn mode_summary_toggle(&mut self) {
        match self.mode {
            Mode::DisplayStock => self.mode = Mode::DisplaySummary,
            Mode::DisplaySummary => self.mode = Mode::DisplayStock,
            _ => {}
        }
    }

    pub fn mode_help(&mut self) {
        self.previous_mode = self.mode;
        self.mode = Mode::Help;
    }

    pub fn mode_add_stock(&mut self) {
        self.previous_mode = self.mode;
        self.mode = Mode::AddStock;
    }

    pub fn mode_display_options(&mut self) {
        if self.toggle_options() {
            self.previous_mode = self.mode;
            self.mode = Mode::DisplayOptions;
        }
    }

    pub fn mode_configure_chart(&mut self) {
        if self.toggle_configure() {
            self.previous_mode = self.mode;
            self.mode = Mode::ConfigureChart;
        }
    }

    pub fn toggle_chart_type(&mut self) {
        self.chart_type = self.chart_type.toggle();

        for stock in self.stocks.iter_mut() {
            stock.set_chart_type(self.chart_type);
        }
    }

    pub fn add_stock(&mut self) {
        let mut stock = self.add_stock.enter(self.chart_type);
        stock.set_time_frame(self.time_frame);
        self.stocks.push(stock);

        self.scroll_bottom();
        self.select_tab_last();
    }

    pub fn remove_stock(&mut self) {
        self.stocks.remove(self.current_tab);

        if self.current_tab > self.stocks.len() .saturating_sub(1) {
            self.select_tab_last();
        }

        if self.stocks.is_empty() {
            self.mode_add_stock();
        }
    }

    pub fn move_tab_left(&mut self) {
        if self.current_tab == 0 {
            return;
        }

        let new_idx = self.current_tab - 1;
        self.stocks.swap(self.current_tab, new_idx);
        self.current_tab = new_idx;
    }

    pub fn move_tab_right(&mut self) {
        if self.current_tab == self.stocks.len() - 1 {
            return;
        }

        let new_idx = self.current_tab + 1;
        self.stocks.swap(self.current_tab, new_idx);
        self.current_tab = new_idx;
    }

    pub fn select_tab_left(&mut self) {
        if self.current_tab > 0 {
            self.current_tab -= 1;
        }
    }

    pub fn select_tab_right(&mut self) {
        if self.current_tab < self.stocks.len().saturating_sub(1) {
            self.current_tab += 1;
        }
    }

    pub fn select_tab_first(&mut self) {
        self.current_tab = 0
    }

    pub fn select_tab_last(&mut self) {
        self.current_tab = self.stocks.len().saturating_sub(1);
    }

    pub fn scroll_selection(&mut self) {
        self.summary_scroll_state.offset = self.current_tab;
    }

    pub fn scroll_top(&mut self) {
        self.summary_scroll_state.offset = 0;
    }

    pub fn scroll_bottom(&mut self) {
        self.summary_scroll_state.offset = self.stocks.len().saturating_sub(self.num_to_render);
    }

    pub fn toggle_configure(&mut self) -> bool {
        self.stocks[self.current_tab].toggle_configure()
    }

    pub fn toggle_options(&mut self) -> bool {
        self.stocks[self.current_tab].toggle_options()
    }

    pub fn toggle_volume(&mut self) {
        if self.chart_type == ChartType::Kagi {
            return;
        }

        let mut show_volumes = SHOW_VOLUMES.write();
        *show_volumes = !*show_volumes;
    }

    pub fn toggle_pre_post(&mut self) {
        let mut guard = ENABLE_PRE_POST.write();
        *guard = !*guard;
    }

    pub fn toggle_x_labels(&mut self) {
        let mut show_x_labels = SHOW_X_LABELS.write();
        *show_x_labels = !*show_x_labels;
    }

    pub fn toggle_option_type(&mut self) {
        self.stocks[self.current_tab]
            .options
            .as_mut()
            .unwrap()
            .toggle_option_type();
    }

    pub fn select_options_previous(&mut self) {
        match self.stocks[self.current_tab]
            .options
            .as_mut()
            .unwrap()
            .selection_mode
        {
            SelectionMode::Dates => {
                self.stocks[self.current_tab]
                    .options
                    .as_mut()
                    .unwrap()
                    .previous_date();
            }
            SelectionMode::Options => {
                self.stocks[self.current_tab]
                    .options
                    .as_mut()
                    .unwrap()
                    .previous_option();
            }
        }
    }

    pub fn select_options_next(&mut self) {
        match self.stocks[self.current_tab]
            .options
            .as_mut()
            .unwrap()
            .selection_mode
        {
            SelectionMode::Dates => {
                self.stocks[self.current_tab]
                    .options
                    .as_mut()
                    .unwrap()
                    .next_date();
            }
            SelectionMode::Options => {
                self.stocks[self.current_tab]
                    .options
                    .as_mut()
                    .unwrap()
                    .next_option();
            }
        }
    }

    pub fn select_options_left(&mut self) {
        self.stocks[self.current_tab]
            .options
            .as_mut()
            .unwrap()
            .selection_mode_left();
    }

    pub fn select_options_right(&mut self) {
        if self.stocks[self.current_tab]
            .options
            .as_mut()
            .unwrap()
            .data()
            .is_some()
        {
            self.stocks[self.current_tab]
                .options
                .as_mut()
                .unwrap()
                .selection_mode_right();
        }
    }

    pub fn chart_config_mut(&mut self) -> &mut ChartConfigurationState {
        self.stocks[self.current_tab].chart_config_mut()
    }

    pub fn time_frame_up(&mut self) {
        self.set_time_frame(self.time_frame.up());
    }

    pub fn time_frame_down(&mut self) {
        self.set_time_frame(self.time_frame.down());
    }

    pub fn set_time_frame(&mut self, time_frame: TimeFrame) {
        self.time_frame = time_frame;

        for stock in self.stocks.iter_mut() {
            stock.set_time_frame(time_frame);
        }
    }
}

pub struct EnvConfig {
    pub show_debug: bool,
    pub debug_mouse: bool,
}

impl EnvConfig {
    #[inline]
    fn env_match(key: &str, default: &str, expected: &str) -> bool {
        std::env::var(key).ok().unwrap_or_else(|| default.into()) == expected
    }

    pub fn load() -> Self {
        Self {
            show_debug: Self::env_match("SHOW_DEBUG", "0", "1"),
            debug_mouse: Self::env_match("DEBUG_MOUSE", "0", "1"),
        }
    }
}

#[derive(Debug)]
pub struct DebugInfo {
    pub enabled: bool,
    pub dimensions: (u16, u16),
    pub cursor_location: Option<(u16, u16)>,
    pub last_event: Option<Event>,
    pub mode: Mode,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct SummaryScrollState {
    pub offset: usize,
    pub queued_scroll: Option<ScrollDirection>,
}

impl SummaryScrollState {
    pub fn scroll_down(&mut self) {
        self.queued_scroll = Some(ScrollDirection::Down);
    }

    pub fn scroll_up(&mut self) {
        self.queued_scroll = Some(ScrollDirection::Up);
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ScrollDirection {
    Up,
    Down,
}
