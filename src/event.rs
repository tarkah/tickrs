use crate::app;
use crate::cleanup_terminal;
use crate::draw;
use crate::widget::options;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use tui::backend::Backend;
use tui::Terminal;

pub fn handle_keys_display_stock<B: Backend>(
    key_event: KeyEvent,
    mut terminal: &mut Terminal<B>,
    mut app: &mut app::App,
) {
    if key_event.modifiers.is_empty() {
        match key_event.code {
            KeyCode::Left => {
                app.stocks[app.current_tab].time_frame_down();
                draw::draw_main(&mut terminal, &mut app);
            }
            KeyCode::Right => {
                app.stocks[app.current_tab].time_frame_up();
                draw::draw_main(&mut terminal, &mut app);
            }
            KeyCode::Char('/') => {
                app.mode = app::Mode::AddStock;
                draw::draw_main(&mut terminal, &mut app);
            }
            KeyCode::Char('k') => {
                app.stocks.remove(app.current_tab);

                if app.current_tab != 0 {
                    app.current_tab -= 1;
                }

                if app.stocks.is_empty() {
                    app.mode = app::Mode::AddStock;
                }

                draw::draw_main(&mut terminal, &mut app);
            }
            KeyCode::Char('q') => {
                cleanup_terminal();
                std::process::exit(0);
            }
            KeyCode::Char('s') => {
                app.mode = app::Mode::DisplaySummary;
                draw::draw_summary(&mut terminal, &mut app);
            }
            KeyCode::Char('?') => {
                app.pre_help_mode = app::Mode::DisplayStock;
                app.mode = app::Mode::Help;
                draw::draw_help(&mut terminal, &mut app);
            }
            KeyCode::Char('o') => {
                app.stocks[app.current_tab].toggle_options();
                app.mode = app::Mode::DisplayOptions;
                draw::draw_main(&mut terminal, &mut app);
            }
            KeyCode::Tab => {
                if app.current_tab == app.stocks.len() - 1 {
                    app.current_tab = 0;
                    draw::draw_main(&mut terminal, &mut app);
                } else {
                    app.current_tab += 1;
                    draw::draw_main(&mut terminal, &mut app);
                }
            }
            KeyCode::BackTab => {
                if app.current_tab == 0 {
                    app.current_tab = app.stocks.len() - 1;
                    draw::draw_main(&mut terminal, &mut app);
                } else {
                    app.current_tab -= 1;
                    draw::draw_main(&mut terminal, &mut app);
                }
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
            app.pre_help_mode = app::Mode::DisplayStock;
            app.mode = app::Mode::Help;
            draw::draw_help(&mut terminal, &mut app);
        }
    }
}

pub fn handle_keys_add_stock<B: Backend>(
    key_event: KeyEvent,
    mut terminal: &mut Terminal<B>,
    mut app: &mut app::App,
) {
    if key_event.modifiers.is_empty() || key_event.modifiers == KeyModifiers::SHIFT {
        match key_event.code {
            KeyCode::Enter => {
                let stock = app.add_stock.enter();

                app.stocks.push(stock);
                app.current_tab = app.stocks.len() - 1;

                app.add_stock.reset();
                app.mode = app::Mode::DisplayStock;

                draw::draw_main(&mut terminal, &mut app);
            }
            KeyCode::Char(c) => {
                app.add_stock.add_char(c);
                draw::draw_main(&mut terminal, &mut app);
            }
            KeyCode::Backspace => {
                app.add_stock.del_char();
                draw::draw_main(&mut terminal, &mut app);
            }
            KeyCode::Esc => {
                app.add_stock.reset();
                if !app.stocks.is_empty() {
                    app.mode = app::Mode::DisplayStock;
                }
                draw::draw_main(&mut terminal, &mut app);
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

pub fn handle_keys_display_summary<B: Backend>(
    key_event: KeyEvent,
    mut terminal: &mut Terminal<B>,
    mut app: &mut app::App,
) {
    if key_event.modifiers.is_empty() {
        match key_event.code {
            KeyCode::Char('q') => {
                cleanup_terminal();
                std::process::exit(0);
            }
            KeyCode::Char('s') => {
                app.mode = app::Mode::DisplayStock;
                draw::draw_main(&mut terminal, &mut app);
            }
            KeyCode::Char('?') => {
                app.pre_help_mode = app::Mode::DisplaySummary;
                app.mode = app::Mode::Help;
                draw::draw_help(&mut terminal, &mut app);
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
            app.pre_help_mode = app::Mode::DisplaySummary;
            app.mode = app::Mode::Help;
            draw::draw_help(&mut terminal, &mut app);
        }
    }
}

pub fn handle_keys_help<B: Backend>(
    key_event: KeyEvent,
    mut terminal: &mut Terminal<B>,
    mut app: &mut app::App,
) {
    if key_event.modifiers.is_empty() {
        match key_event.code {
            KeyCode::Esc | KeyCode::Char('?') => {
                app.mode = app.pre_help_mode;

                match app.mode {
                    app::Mode::DisplaySummary => draw::draw_summary(&mut terminal, &mut app),
                    _ => draw::draw_main(&mut terminal, &mut app),
                };
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

pub fn handle_keys_display_options<B: Backend>(
    key_event: KeyEvent,
    mut terminal: &mut Terminal<B>,
    mut app: &mut app::App,
) {
    if key_event.modifiers.is_empty() {
        match key_event.code {
            KeyCode::Esc | KeyCode::Char('o') => {
                app.stocks[app.current_tab].toggle_options();
                app.mode = app::Mode::DisplayStock;
                draw::draw_main(&mut terminal, &mut app);
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

                draw::draw_main(&mut terminal, &mut app);
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

                draw::draw_main(&mut terminal, &mut app);
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

                draw::draw_main(&mut terminal, &mut app);
            }
            KeyCode::Left => {
                app.stocks[app.current_tab]
                    .options
                    .as_mut()
                    .unwrap()
                    .selection_mode_left();

                draw::draw_main(&mut terminal, &mut app);
            }
            KeyCode::Right => {
                if app.stocks[app.current_tab]
                    .options
                    .as_mut()
                    .unwrap()
                    .data
                    .is_some()
                {
                    app.stocks[app.current_tab]
                        .options
                        .as_mut()
                        .unwrap()
                        .selection_mode_right();

                    draw::draw_main(&mut terminal, &mut app);
                }
            }
            KeyCode::Char('?') => {
                app.pre_help_mode = app::Mode::DisplayOptions;
                app.mode = app::Mode::Help;
                draw::draw_help(&mut terminal, &mut app);
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
            app.pre_help_mode = app::Mode::DisplayOptions;
            app.mode = app::Mode::Help;
            draw::draw_help(&mut terminal, &mut app);
        }
    }
}
