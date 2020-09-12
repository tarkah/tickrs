use crate::cleanup_terminal;
use crate::widget::options;
use crate::{app, SHOW_X_LABELS};

use crossbeam_channel::Sender;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub fn handle_keys_display_stock(
    key_event: KeyEvent,
    mut app: &mut app::App,
    request_redraw: &Sender<()>,
) {
    if key_event.modifiers.is_empty() {
        match key_event.code {
            KeyCode::Left => {
                app.stocks[app.current_tab].time_frame_down();
                let _ = request_redraw.try_send(());
            }
            KeyCode::Right => {
                app.stocks[app.current_tab].time_frame_up();
                let _ = request_redraw.try_send(());
            }
            KeyCode::Char('/') => {
                app.previous_mode = app.mode;
                app.mode = app::Mode::AddStock;
                let _ = request_redraw.try_send(());
            }
            KeyCode::Char('k') => {
                app.stocks.remove(app.current_tab);

                if app.current_tab != 0 {
                    app.current_tab -= 1;
                }

                if app.stocks.is_empty() {
                    app.previous_mode = app.mode;
                    app.mode = app::Mode::AddStock;
                }
                let _ = request_redraw.try_send(());
            }
            KeyCode::Char('q') => {
                cleanup_terminal();
                std::process::exit(0);
            }
            KeyCode::Char('s') => {
                app.mode = app::Mode::DisplaySummary;

                for stock in app.stocks.iter_mut() {
                    if stock.time_frame != app.summary_time_frame {
                        stock.set_time_frame(app.summary_time_frame);
                    }
                }
                let _ = request_redraw.try_send(());
            }
            KeyCode::Char('?') => {
                app.previous_mode = app.mode;
                app.mode = app::Mode::Help;
                let _ = request_redraw.try_send(());
            }
            KeyCode::Char('o') => {
                app.stocks[app.current_tab].toggle_options();
                app.mode = app::Mode::DisplayOptions;
                let _ = request_redraw.try_send(());
            }
            KeyCode::Char('x') => {
                let mut show_x_labels = SHOW_X_LABELS.write().unwrap();
                *show_x_labels = !*show_x_labels;
                let _ = request_redraw.try_send(());
            }
            KeyCode::Tab => {
                if app.current_tab == app.stocks.len() - 1 {
                    app.current_tab = 0;
                } else {
                    app.current_tab += 1;
                }
                let _ = request_redraw.try_send(());
            }
            KeyCode::BackTab => {
                if app.current_tab == 0 {
                    app.current_tab = app.stocks.len() - 1;
                } else {
                    app.current_tab -= 1;
                }
                let _ = request_redraw.try_send(());
            }
            _ => {}
        }
    } else if key_event.modifiers == KeyModifiers::CONTROL {
        if let KeyCode::Char('c') = key_event.code {
            cleanup_terminal();
            std::process::exit(0);
        }
    } else if key_event.modifiers == KeyModifiers::SHIFT {
        if let KeyCode::Char('?') = key_event.code {
            app.previous_mode = app.mode;
            app.mode = app::Mode::Help;
            let _ = request_redraw.try_send(());
        }
    }
}

pub fn handle_keys_add_stock(
    key_event: KeyEvent,
    mut app: &mut app::App,
    request_redraw: &Sender<()>,
) {
    if key_event.modifiers.is_empty() || key_event.modifiers == KeyModifiers::SHIFT {
        match key_event.code {
            KeyCode::Enter => {
                let stock = app.add_stock.enter();

                app.stocks.push(stock);
                app.current_tab = app.stocks.len() - 1;

                app.add_stock.reset();
                app.mode = app.previous_mode;
                let _ = request_redraw.try_send(());
            }
            KeyCode::Char(c) => {
                app.add_stock.add_char(c);
                let _ = request_redraw.try_send(());
            }
            KeyCode::Backspace => {
                app.add_stock.del_char();
                let _ = request_redraw.try_send(());
            }
            KeyCode::Esc => {
                app.add_stock.reset();
                if !app.stocks.is_empty() {
                    app.mode = app.previous_mode;
                }
                let _ = request_redraw.try_send(());
            }
            _ => {}
        }
    } else if key_event.modifiers == KeyModifiers::CONTROL {
        if let KeyCode::Char('c') = key_event.code {
            cleanup_terminal();
            std::process::exit(0);
        }
    }
}

pub fn handle_keys_display_summary(
    key_event: KeyEvent,
    mut app: &mut app::App,
    request_redraw: &Sender<()>,
) {
    if key_event.modifiers.is_empty() {
        match key_event.code {
            KeyCode::Left => {
                app.summary_time_frame = app.summary_time_frame.down();

                for stock in app.stocks.iter_mut() {
                    stock.set_time_frame(app.summary_time_frame);
                }
                let _ = request_redraw.try_send(());
            }
            KeyCode::Right => {
                app.summary_time_frame = app.summary_time_frame.up();

                for stock in app.stocks.iter_mut() {
                    stock.set_time_frame(app.summary_time_frame);
                }
                let _ = request_redraw.try_send(());
            }
            KeyCode::Char('q') => {
                cleanup_terminal();
                std::process::exit(0);
            }
            KeyCode::Char('s') => {
                app.mode = app::Mode::DisplayStock;
                let _ = request_redraw.try_send(());
            }
            KeyCode::Char('?') => {
                app.previous_mode = app.mode;
                app.mode = app::Mode::Help;
                let _ = request_redraw.try_send(());
            }
            KeyCode::Char('/') => {
                app.previous_mode = app.mode;
                app.mode = app::Mode::AddStock;
                let _ = request_redraw.try_send(());
            }
            _ => {}
        }
    } else if key_event.modifiers == KeyModifiers::CONTROL {
        if let KeyCode::Char('c') = key_event.code {
            cleanup_terminal();
            std::process::exit(0);
        }
    } else if key_event.modifiers == KeyModifiers::SHIFT {
        if let KeyCode::Char('?') = key_event.code {
            app.previous_mode = app.mode;
            app.mode = app::Mode::Help;
            let _ = request_redraw.try_send(());
        }
    }
}

pub fn handle_keys_help(key_event: KeyEvent, mut app: &mut app::App, request_redraw: &Sender<()>) {
    if key_event.modifiers.is_empty() {
        match key_event.code {
            KeyCode::Esc | KeyCode::Char('?') => {
                app.mode = app.previous_mode;
                let _ = request_redraw.try_send(());
            }
            _ => {}
        }
    } else if key_event.modifiers == KeyModifiers::CONTROL {
        if let KeyCode::Char('c') = key_event.code {
            cleanup_terminal();
            std::process::exit(0);
        }
    }
}

pub fn handle_keys_display_options(
    key_event: KeyEvent,
    mut app: &mut app::App,
    request_redraw: &Sender<()>,
) {
    if key_event.modifiers.is_empty() {
        match key_event.code {
            KeyCode::Esc | KeyCode::Char('o') => {
                app.stocks[app.current_tab].toggle_options();
                app.mode = app::Mode::DisplayStock;
                let _ = request_redraw.try_send(());
            }
            KeyCode::Char('q') => {
                cleanup_terminal();
                std::process::exit(0);
            }
            KeyCode::Tab => {
                app.stocks[app.current_tab]
                    .options
                    .as_mut()
                    .unwrap()
                    .toggle_option_type();
                let _ = request_redraw.try_send(());
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
                let _ = request_redraw.try_send(());
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
                let _ = request_redraw.try_send(());
            }
            KeyCode::Left => {
                app.stocks[app.current_tab]
                    .options
                    .as_mut()
                    .unwrap()
                    .selection_mode_left();
                let _ = request_redraw.try_send(());
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
                    let _ = request_redraw.try_send(());
                }
            }
            KeyCode::Char('?') => {
                app.previous_mode = app.mode;
                app.mode = app::Mode::Help;
                let _ = request_redraw.try_send(());
            }
            _ => {}
        }
    } else if key_event.modifiers == KeyModifiers::CONTROL {
        if let KeyCode::Char('c') = key_event.code {
            cleanup_terminal();
            std::process::exit(0);
        }
    } else if key_event.modifiers == KeyModifiers::SHIFT {
        if let KeyCode::Char('?') = key_event.code {
            app.previous_mode = app.mode;
            app.mode = app::Mode::Help;
            let _ = request_redraw.try_send(());
        }
    }
}
