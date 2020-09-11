extern crate tickrs_api as api;

use crossbeam_channel::{select, tick, unbounded, Receiver, Sender};

use crossterm::cursor;
use crossterm::event::{Event, MouseEvent};
use crossterm::execute;
use crossterm::terminal;

use lazy_static::lazy_static;

use tui::backend::CrosstermBackend;
use tui::Terminal;

use std::io::{self, Write};
use std::panic;
use std::sync::RwLock;
use std::thread;
use std::time::Duration;

mod app;
mod cli;
mod common;
mod draw;
mod event;
mod service;
mod task;
mod widget;

use crate::app::DebugInfo;
use crate::common::TimeFrame;

lazy_static! {
    pub static ref OPTS: cli::Opt = cli::get_opts();
    pub static ref UPDATE_INTERVAL: u64 = OPTS.update_interval;
    pub static ref TIME_FRAME: TimeFrame = OPTS.time_frame;
    pub static ref HIDE_TOGGLE: bool = OPTS.hide_toggle;
    pub static ref REDRAW_REQUEST: (Sender<()>, Receiver<()>) = unbounded();
    pub static ref SHOW_X_LABELS: RwLock<bool> = RwLock::new(OPTS.show_x_labels);
}

fn main() {
    let opts = OPTS.clone();

    better_panic::install();

    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend).unwrap();

    setup_panic_hook();
    setup_terminal();

    let redraw_request = REDRAW_REQUEST.1.clone();
    let ticker = tick(Duration::from_secs(*UPDATE_INTERVAL));
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

    let mut app = app::App {
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
    };

    for stock in app.stocks.iter_mut() {
        stock.update();
    }

    draw::draw(&mut terminal, &mut app);

    loop {
        select! {
            recv(ticker) -> _ => {
                for stock in app.stocks.iter_mut() {
                    stock.update();

                    if let Some(options) = stock.options.as_mut() {
                        options.update();
                    }
                }

                draw::draw(&mut terminal, &mut app);
            }
            recv(redraw_request) -> _ => {
                // Wait some ms and collect all redraw requests so we don't
                // redraw on each one. This is mostly important when the app is first
                // launched with more than one symbol supplied, since each will initially
                // request a redraw when the API data is first received.
                thread::sleep(Duration::from_millis(200));
                let _ = redraw_request.try_iter().collect::<Vec<_>>();

                for stock in app.stocks.iter_mut() {
                    stock.update();

                    if let Some(options) = stock.options.as_mut() {
                        options.update();
                    }
                }

                draw::draw(&mut terminal, &mut app);
            }
            recv(ui_events) -> message => {
                if app.debug.enabled {
                    if let Ok(event) = message {
                        app.debug.last_event = Some(event);
                        draw::draw(&mut terminal, &mut app);
                    }
                }

                if let Ok(Event::Key(key_event)) = message {
                    match app.mode {
                        app::Mode::AddStock => {
                            event::handle_keys_add_stock(key_event, &mut terminal, &mut app);
                        }
                        app::Mode::DisplayStock => {
                            event::handle_keys_display_stock(key_event, &mut terminal, &mut app);
                        }
                        app::Mode::DisplaySummary => {
                            event::handle_keys_display_summary(key_event, &mut terminal, &mut app);
                        }
                        app::Mode::Help => {
                            event::handle_keys_help(key_event, &mut terminal, &mut app);
                        }
                        app::Mode::DisplayOptions => {
                            event::handle_keys_display_options(key_event, &mut terminal, &mut app);
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
                        draw::draw(&mut terminal, &mut app);
                    }
                } else if let Ok(Event::Resize(_,_)) = message {
                    draw::draw(&mut terminal, &mut app);
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
