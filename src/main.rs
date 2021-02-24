extern crate tickrs_api as api;

use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use std::time::Duration;
use std::{io, panic, thread};

use crossbeam_channel::{bounded, select, unbounded, Receiver, Sender};
use crossterm::event::{Event, MouseEvent, MouseEventKind};
use crossterm::{cursor, execute, terminal};
use lazy_static::lazy_static;
use service::default_timestamps::DefaultTimestampService;
use tui::backend::CrosstermBackend;
use tui::Terminal;

use crate::app::DebugInfo;
use crate::common::{ChartType, TimeFrame};

mod app;
mod common;
mod draw;
mod event;
mod opts;
mod service;
mod task;
mod theme;
mod widget;

lazy_static! {
    static ref CLIENT: api::Client = api::Client::new();
    static ref DEBUG_LEVEL: app::EnvConfig = app::EnvConfig::load();
    pub static ref OPTS: opts::Opts = opts::resolve_opts();
    pub static ref UPDATE_INTERVAL: u64 = OPTS.update_interval.unwrap_or(1);
    pub static ref TIME_FRAME: TimeFrame = OPTS.time_frame.unwrap_or(TimeFrame::Day1);
    pub static ref HIDE_TOGGLE: bool = OPTS.hide_toggle;
    pub static ref HIDE_PREV_CLOSE: bool = OPTS.hide_prev_close;
    pub static ref REDRAW_REQUEST: (Sender<()>, Receiver<()>) = bounded(1);
    pub static ref DATA_RECEIVED: (Sender<()>, Receiver<()>) = bounded(1);
    pub static ref SHOW_X_LABELS: RwLock<bool> = RwLock::new(OPTS.show_x_labels);
    pub static ref ENABLE_PRE_POST: RwLock<bool> = RwLock::new(OPTS.enable_pre_post);
    pub static ref TRUNC_PRE: bool = OPTS.trunc_pre;
    pub static ref SHOW_VOLUMES: RwLock<bool> = RwLock::new(OPTS.show_volumes);
    pub static ref DEFAULT_TIMESTAMPS: RwLock<HashMap<TimeFrame, Vec<i64>>> = Default::default();
    pub static ref THEME: theme::Theme = OPTS.theme.unwrap_or_default();
}

fn main() {
    better_panic::install();

    let opts = OPTS.clone();

    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend).unwrap();

    setup_panic_hook();
    setup_terminal();

    let request_redraw = REDRAW_REQUEST.0.clone();
    let data_received = DATA_RECEIVED.1.clone();
    let ui_events = setup_ui_events();

    let starting_chart_type = opts.chart_type.unwrap_or(ChartType::Line);

    let starting_stocks: Vec<_> = opts
        .symbols
        .unwrap_or_default()
        .into_iter()
        .map(|symbol| widget::StockState::new(symbol, starting_chart_type))
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
            enabled: DEBUG_LEVEL.show_debug,
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
        summary_time_frame: opts.time_frame.unwrap_or(TimeFrame::Day1),
        default_timestamp_service,
        summary_scroll_state: Default::default(),
        chart_type: starting_chart_type,
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

                match message {
                    Ok(Event::Key(key_event)) => {
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
                    }
                    Ok(Event::Mouse(MouseEvent { kind, row, column,.. })) => {
                        if app.debug.enabled {
                            match kind {
                                MouseEventKind::Down(_) => app.debug.cursor_location = Some((row, column)),
                                MouseEventKind::Up(_) => app.debug.cursor_location = Some((row, column)),
                                MouseEventKind::Drag(_) => app.debug.cursor_location = Some((row, column)),
                                _ => {}
                            }
                        }
                    }
                    Ok(Event::Resize(..)) => {
                        let _ = request_redraw.try_send(());
                    }
                    _ => {}
                }
            }
        }
    }
}

fn setup_terminal() {
    let mut stdout = io::stdout();

    execute!(stdout, cursor::Hide).unwrap();
    execute!(stdout, terminal::EnterAlternateScreen).unwrap();

    execute!(stdout, terminal::Clear(terminal::ClearType::All)).unwrap();

    if DEBUG_LEVEL.debug_mouse {
        execute!(stdout, crossterm::event::EnableMouseCapture).unwrap();
    }

    terminal::enable_raw_mode().unwrap();
}

fn cleanup_terminal() {
    let mut stdout = io::stdout();

    if DEBUG_LEVEL.debug_mouse {
        execute!(stdout, crossterm::event::DisableMouseCapture).unwrap();
    }

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
