extern crate tickrs_api as api;

use std::collections::HashMap;
use std::io::{self, Write};
use std::sync::{Arc, Mutex, RwLock};
use std::time::Duration;
use std::{panic, thread};

use crossbeam_channel::{bounded, select, unbounded, Receiver, Sender};
use crossterm::event::{Event, MouseEvent};
use crossterm::{cursor, execute, terminal, write_ansi_code};
use lazy_static::lazy_static;
use service::default_timestamps::DefaultTimestampService;
use tui::backend::CrosstermBackend;
use tui::Terminal;

use crate::app::DebugInfo;
use crate::common::TimeFrame;

mod app;
mod cli;
mod common;
mod draw;
mod event;
mod service;
mod task;
mod widget;

lazy_static! {
    static ref CLIENT: api::Client = api::Client::new();
    pub static ref OPTS: cli::Opt = cli::get_opts();
    pub static ref UPDATE_INTERVAL: u64 = OPTS.update_interval;
    pub static ref TIME_FRAME: TimeFrame = OPTS.time_frame;
    pub static ref HIDE_TOGGLE: bool = OPTS.hide_toggle;
    pub static ref HIDE_PREV_CLOSE: bool = OPTS.hide_prev_close;
    pub static ref REDRAW_REQUEST: (Sender<()>, Receiver<()>) = bounded(1);
    pub static ref DATA_RECEIVED: (Sender<()>, Receiver<()>) = bounded(1);
    pub static ref SHOW_X_LABELS: RwLock<bool> = RwLock::new(OPTS.show_x_labels);
    pub static ref ENABLE_PRE_POST: RwLock<bool> = RwLock::new(OPTS.enable_pre_post);
    pub static ref TRUNC_PRE: bool = OPTS.trunc_pre;
    pub static ref SHOW_VOLUMES: RwLock<bool> = RwLock::new(OPTS.show_volumes);
    pub static ref DEFAULT_TIMESTAMPS: RwLock<HashMap<TimeFrame, Vec<i64>>> = Default::default();
}

fn main() {
    let opts = OPTS.clone();

    better_panic::install();

    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend).unwrap();

    setup_panic_hook();
    setup_terminal();

    let request_redraw = REDRAW_REQUEST.0.clone();
    let data_received = DATA_RECEIVED.1.clone();
    let ui_events = setup_ui_events();

    let starting_stocks: Vec<_> = opts
        .symbols
        .into_iter()
        .map(widget::StockState::new)
        .collect();

    let starting_mode = if starting_stocks.is_empty() {
        app::Mode::AddStock
    } else if opts.summary {
        app::Mode::DisplaySummary
    } else {
        app::Mode::DisplayStock
    };

    let default_timestamp_service = DefaultTimestampService::new();

    let app = Arc::new(Mutex::new(app::App {
        mode: starting_mode,
        stocks: starting_stocks,
        add_stock: widget::AddStockState::new(),
        help: widget::HelpWidget {},
        current_tab: 0,
        hide_help: opts.hide_help,
        debug: DebugInfo {
            enabled: std::env::var("SHOW_DEBUG")
                .ok()
                .unwrap_or_else(|| String::from("0"))
                == "1",
            dimensions: (0, 0),
            cursor_location: None,
            last_event: None,
            mode: starting_mode,
        },
        previous_mode: if opts.summary {
            app::Mode::DisplaySummary
        } else {
            app::Mode::DisplayStock
        },
        summary_time_frame: opts.time_frame,
        default_timestamp_service,
    }));

    let move_app = app.clone();

    // Redraw thread
    thread::spawn(move || {
        let app = move_app;

        let redraw_requested = REDRAW_REQUEST.1.clone();

        loop {
            select! {
                recv(redraw_requested) -> _ => {
                    let mut app = app.lock().unwrap();

                    draw::draw(&mut terminal, &mut app);
                }
                // Default redraw on every duration
                default(Duration::from_millis(500)) => {
                    let mut app = app.lock().unwrap();

                    // Drive animation of loading icon
                    for stock in app.stocks.iter_mut() {
                        stock.loading_tick();
                    }

                    draw::draw(&mut terminal, &mut app);
                }
            }
        }
    });

    loop {
        select! {
            // Notified that new data has been fetched from API, update widgets
            // so they can update their state with this new information
            recv(data_received) -> _ => {
                let mut app = app.lock().unwrap();

                app.update();

                for stock in app.stocks.iter_mut() {
                    stock.update();

                    if let Some(options) = stock.options.as_mut() {
                        options.update();
                    }
                }
            }
            recv(ui_events) -> message => {
                let mut app = app.lock().unwrap();

                if app.debug.enabled {
                    if let Ok(event) = message {
                        app.debug.last_event = Some(event);
                    }
                }

                if let Ok(Event::Key(key_event)) = message {
                    match app.mode {
                        app::Mode::AddStock => {
                            event::handle_keys_add_stock(key_event, &mut app, &request_redraw);
                        }
                        app::Mode::DisplayStock => {
                            event::handle_keys_display_stock(key_event,&mut app, &request_redraw);
                        }
                        app::Mode::DisplaySummary => {
                            event::handle_keys_display_summary(key_event, &mut app, &request_redraw);
                        }
                        app::Mode::Help => {
                            event::handle_keys_help(key_event, &mut app, &request_redraw);
                        }
                        app::Mode::DisplayOptions => {
                            event::handle_keys_display_options(key_event, &mut app, &request_redraw);
                        }
                    }
                } else if let Ok(Event::Mouse(event)) = message {
                    if app.debug.enabled {
                        match event {
                            MouseEvent::Down(_, row, column, ..) => app.debug.cursor_location = Some((row, column)),
                            MouseEvent::Up(_, row, column, ..) => app.debug.cursor_location = Some((row, column)),
                            MouseEvent::Drag(_, row, column, ..) => app.debug.cursor_location = Some((row, column)),
                            _ => {}
                        }
                    }
                }
            }
        }
    }
}

fn setup_terminal() {
    let mut stdout = io::stdout();

    execute!(stdout, terminal::EnterAlternateScreen).unwrap();
    execute!(stdout, cursor::Hide).unwrap();

    execute!(stdout, terminal::Clear(terminal::ClearType::All)).unwrap();

    execute!(stdout, crossterm::event::EnableMouseCapture).unwrap();

    terminal::enable_raw_mode().unwrap();
}

fn cleanup_terminal() {
    let mut stdout = io::stdout();

    execute!(stdout, crossterm::event::DisableMouseCapture).unwrap();

    execute!(stdout, cursor::MoveTo(0, 0)).unwrap();
    execute!(stdout, terminal::Clear(terminal::ClearType::All)).unwrap();

    execute!(stdout, terminal::LeaveAlternateScreen).unwrap();
    execute!(stdout, cursor::Show).unwrap();

    terminal::disable_raw_mode().unwrap();
}

fn setup_ui_events() -> Receiver<Event> {
    let (sender, receiver) = unbounded();
    std::thread::spawn(move || loop {
        sender.send(crossterm::event::read().unwrap()).unwrap();
    });

    receiver
}

fn setup_panic_hook() {
    panic::set_hook(Box::new(|panic_info| {
        cleanup_terminal();
        better_panic::Settings::auto().create_panic_handler()(panic_info);
    }));
}
