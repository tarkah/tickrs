use app::ScrollDirection;
use crossbeam_channel::Sender;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::app::{self, Mode};
use crate::common::ChartType;
use crate::widget::options;
use crate::{cleanup_terminal, ENABLE_PRE_POST, SHOW_VOLUMES, SHOW_X_LABELS};

fn handle_keys_add_stock(keycode: KeyCode, mut app: &mut app::App) {
    match keycode {
        KeyCode::Enter => {
            let mut stock = app.add_stock.enter(app.chart_type);

            if app.previous_mode == app::Mode::DisplaySummary {
                stock.set_time_frame(app.summary_time_frame);
            }

            app.stocks.push(stock);
            app.current_tab = app.stocks.len() - 1;

            app.add_stock.reset();
            app.mode = app.previous_mode;
        }
        KeyCode::Char(c) => {
            app.add_stock.add_char(c);
        }
        KeyCode::Backspace => {
            app.add_stock.del_char();
        }
        KeyCode::Esc => {
            app.add_stock.reset();
            if !app.stocks.is_empty() {
                app.mode = app.previous_mode;
            }
        }
        _ => {}
    }
}

fn handle_keys_display_stock(keycode: KeyCode, modifiers: KeyModifiers, mut app: &mut app::App) {
    match (keycode, modifiers) {
        (KeyCode::Left, KeyModifiers::CONTROL) => {
            let new_idx = if app.current_tab == 0 {
                app.stocks.len() - 1
            } else {
                app.current_tab - 1
            };
            app.stocks.swap(app.current_tab, new_idx);
            app.current_tab = new_idx;
        }
        (KeyCode::Right, KeyModifiers::CONTROL) => {
            let new_idx = (app.current_tab + 1) % app.stocks.len();
            app.stocks.swap(app.current_tab, new_idx);
            app.current_tab = new_idx;
        }
        (KeyCode::BackTab, KeyModifiers::SHIFT) => {
            if app.current_tab == 0 {
                app.current_tab = app.stocks.len() - 1;
            } else {
                app.current_tab -= 1;
            }
        }
        (KeyCode::Left, KeyModifiers::NONE) => {
            app.stocks[app.current_tab].time_frame_down();
        }
        (KeyCode::Right, KeyModifiers::NONE) => {
            app.stocks[app.current_tab].time_frame_up();
        }
        (KeyCode::Left, KeyModifiers::SHIFT) | (KeyCode::Char('<'), _) => {
            if let Some(stock) = app.stocks.get_mut(app.current_tab) {
                if let Some(chart_state) = stock.chart_state_mut() {
                    chart_state.scroll_left();
                }
            }
        }
        (KeyCode::Right, KeyModifiers::SHIFT) | (KeyCode::Char('>'), _) => {
            if let Some(stock) = app.stocks.get_mut(app.current_tab) {
                if let Some(chart_state) = stock.chart_state_mut() {
                    chart_state.scroll_right();
                }
            }
        }
        (KeyCode::Char('/'), KeyModifiers::NONE) => {
            app.previous_mode = app.mode;
            app.mode = app::Mode::AddStock;
        }
        (KeyCode::Char('k'), KeyModifiers::NONE) => {
            app.stocks.remove(app.current_tab);

            if app.current_tab != 0 {
                app.current_tab -= 1;
            }

            if app.stocks.is_empty() {
                app.previous_mode = app.mode;
                app.mode = app::Mode::AddStock;
            }
        }
        (KeyCode::Char('s'), KeyModifiers::NONE) => {
            app.mode = app::Mode::DisplaySummary;

            for stock in app.stocks.iter_mut() {
                if stock.time_frame != app.summary_time_frame {
                    stock.set_time_frame(app.summary_time_frame);
                }
            }
        }
        (KeyCode::Char('o'), KeyModifiers::NONE) => {
            if app.stocks[app.current_tab].toggle_options() {
                app.mode = app::Mode::DisplayOptions;
            }
        }
        (KeyCode::Char('e'), KeyModifiers::NONE) => {
            if app.stocks[app.current_tab].toggle_configure() {
                app.mode = app::Mode::ConfigureChart;
            }
        }
        (KeyCode::Tab, KeyModifiers::NONE) => {
            if app.current_tab == app.stocks.len() - 1 {
                app.current_tab = 0;
            } else {
                app.current_tab += 1;
            }
        }
        _ => {}
    }
}

fn handle_keys_display_summary(keycode: KeyCode, mut app: &mut app::App) {
    match keycode {
        KeyCode::Left => {
            app.summary_time_frame = app.summary_time_frame.down();

            for stock in app.stocks.iter_mut() {
                stock.set_time_frame(app.summary_time_frame);
            }
        }
        KeyCode::Right => {
            app.summary_time_frame = app.summary_time_frame.up();

            for stock in app.stocks.iter_mut() {
                stock.set_time_frame(app.summary_time_frame);
            }
        }
        KeyCode::Up => {
            app.summary_scroll_state.queued_scroll = Some(ScrollDirection::Up);
        }
        KeyCode::Down => {
            app.summary_scroll_state.queued_scroll = Some(ScrollDirection::Down);
        }
        KeyCode::Char('s') => {
            app.mode = app::Mode::DisplayStock;
        }
        KeyCode::Char('/') => {
            app.previous_mode = app.mode;
            app.mode = app::Mode::AddStock;
        }
        _ => {}
    }
}

fn handle_keys_display_options(keycode: KeyCode, mut app: &mut app::App) {
    match keycode {
        KeyCode::Esc | KeyCode::Char('o') | KeyCode::Char('q') => {
            app.stocks[app.current_tab].toggle_options();
            app.mode = app::Mode::DisplayStock;
        }
        KeyCode::Tab => {
            app.stocks[app.current_tab]
                .options
                .as_mut()
                .unwrap()
                .toggle_option_type();
        }
        KeyCode::Up => {
            match app.stocks[app.current_tab]
                .options
                .as_mut()
                .unwrap()
                .selection_mode
            {
                options::SelectionMode::Dates => {
                    app.stocks[app.current_tab]
                        .options
                        .as_mut()
                        .unwrap()
                        .previous_date();
                }
                options::SelectionMode::Options => {
                    app.stocks[app.current_tab]
                        .options
                        .as_mut()
                        .unwrap()
                        .previous_option();
                }
            }
        }
        KeyCode::Down => {
            match app.stocks[app.current_tab]
                .options
                .as_mut()
                .unwrap()
                .selection_mode
            {
                options::SelectionMode::Dates => {
                    app.stocks[app.current_tab]
                        .options
                        .as_mut()
                        .unwrap()
                        .next_date();
                }
                options::SelectionMode::Options => {
                    app.stocks[app.current_tab]
                        .options
                        .as_mut()
                        .unwrap()
                        .next_option();
                }
            }
        }
        KeyCode::Left => {
            app.stocks[app.current_tab]
                .options
                .as_mut()
                .unwrap()
                .selection_mode_left();
        }
        KeyCode::Right => {
            if app.stocks[app.current_tab]
                .options
                .as_mut()
                .unwrap()
                .data()
                .is_some()
            {
                app.stocks[app.current_tab]
                    .options
                    .as_mut()
                    .unwrap()
                    .selection_mode_right();
            }
        }
        _ => {}
    }
}

pub fn handle_keys_configure_chart(keycode: KeyCode, mut app: &mut app::App) {
    match keycode {
        KeyCode::Esc | KeyCode::Char('e') => {
            app.stocks[app.current_tab].toggle_configure();
            app.mode = app::Mode::DisplayStock;
        }
        KeyCode::Up => {
            let config = app.stocks[app.current_tab].chart_config_mut();
            config.selection_up();
        }
        KeyCode::Down => {
            let config = app.stocks[app.current_tab].chart_config_mut();
            config.selection_down();
        }
        KeyCode::Tab => {
            let config = app.stocks[app.current_tab].chart_config_mut();
            config.tab();
        }
        KeyCode::Enter => {
            let time_frame = app.stocks[app.current_tab].time_frame;
            let config = app.stocks[app.current_tab].chart_config_mut();
            config.enter(time_frame);
        }
        KeyCode::Char(c) => {
            if c.is_numeric() || c == '.' {
                let config = app.stocks[app.current_tab].chart_config_mut();
                config.add_char(c);
            }
        }
        KeyCode::Backspace => {
            let config = app.stocks[app.current_tab].chart_config_mut();
            config.del_char();
        }
        _ => {}
    }
}

pub fn handle_key_bindings(
    mode: Mode,
    key_event: KeyEvent,
    mut app: &mut app::App,
    request_redraw: &Sender<()>,
) {
    match (mode, key_event.modifiers, key_event.code) {
        (_, KeyModifiers::CONTROL, KeyCode::Char('c')) => {
            cleanup_terminal();
            std::process::exit(0);
        }
        (Mode::AddStock, modifiers, keycode) => {
            if modifiers.is_empty() || modifiers == KeyModifiers::SHIFT {
                handle_keys_add_stock(keycode, app)
            }
        }
        (Mode::Help, modifiers, keycode) => {
            if modifiers.is_empty()
                && (matches!(
                    keycode,
                    KeyCode::Esc | KeyCode::Char('?') | KeyCode::Char('q')
                ))
            {
                app.mode = app.previous_mode;
            }
        }
        (mode, KeyModifiers::NONE, KeyCode::Char('q')) if mode != Mode::DisplayOptions => {
            cleanup_terminal();
            std::process::exit(0);
        }
        (_, KeyModifiers::NONE, KeyCode::Char('?')) => {
            app.previous_mode = app.mode;
            app.mode = app::Mode::Help;
        }
        (_, KeyModifiers::NONE, KeyCode::Char('c')) => {
            app.chart_type = app.chart_type.toggle();

            for stock in app.stocks.iter_mut() {
                stock.set_chart_type(app.chart_type);
            }
        }
        (_, KeyModifiers::NONE, KeyCode::Char('v')) => {
            if app.chart_type != ChartType::Kagi {
                let mut show_volumes = SHOW_VOLUMES.write().unwrap();
                *show_volumes = !*show_volumes;
            }
        }
        (_, KeyModifiers::NONE, KeyCode::Char('p')) => {
            let mut guard = ENABLE_PRE_POST.write().unwrap();
            *guard = !*guard;
        }
        (Mode::DisplaySummary, modifiers, keycode) => {
            if modifiers.is_empty() {
                handle_keys_display_summary(keycode, app)
            }
        }
        (_, KeyModifiers::NONE, KeyCode::Char('x')) => {
            let mut show_x_labels = SHOW_X_LABELS.write().unwrap();
            *show_x_labels = !*show_x_labels;
        }
        (Mode::DisplayOptions, modifiers, keycode) => {
            if modifiers.is_empty() {
                handle_keys_display_options(keycode, app)
            }
        }
        (Mode::ConfigureChart, modifiers, keycode) => {
            if modifiers.is_empty() {
                handle_keys_configure_chart(keycode, app)
            }
        }
        (Mode::DisplayStock, modifiers, keycode) => {
            handle_keys_display_stock(keycode, modifiers, app)
        }
    }
    let _ = request_redraw.try_send(());
}
