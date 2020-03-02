use crossbeam_channel::{select, tick, unbounded, Receiver};

use crossterm::cursor;
use crossterm::event::Event;
use crossterm::execute;
use crossterm::terminal;

use tui::backend::CrosstermBackend;
use tui::Terminal;

use std::io::{self, Write};
use std::panic;
use std::time::Duration;

mod app;
mod cli;
mod draw;
mod event;
mod service;
mod task;
mod time_frame;
mod widget;

pub use crate::time_frame::TimeFrame;

fn main() {
    let opts = cli::get_opts();

    better_panic::install();

    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend).unwrap();

    setup_panic_hook();
    setup_terminal();

    let ticker = tick(Duration::from_millis(1000));
    let ui_events = setup_ui_events();
    let ctrl_c_events = setup_ctrl_c();

    let starting_stocks: Vec<_> = opts
        .symbols
        .into_iter()
        .map(widget::StockWidget::new)
        .collect();

    let starting_mode = if starting_stocks.is_empty() {
        app::Mode::AddStock
    } else {
        app::Mode::DisplayStock
    };

    let mut app = app::App {
        mode: starting_mode,
        stocks: starting_stocks,
        add_stock: widget::AddStockWidget::new(),
        help: widget::HelpWidget {},
        current_tab: 0,
        hide_help: opts.hide_help,
    };

    for stock in app.stocks.iter_mut() {
        stock.update();
    }

    draw::draw(&mut terminal, &mut app);

    loop {
        select! {
            recv(ctrl_c_events) -> _ => {
                cleanup_terminal();
            }
            recv(ticker) -> _ => {
                for stock in app.stocks.iter_mut() {
                    stock.update();
                }

                if app.mode == app::Mode::DisplayStock {
                    draw::draw(&mut terminal, &mut app);
                } else if app.mode == app::Mode::Help {
                    draw::draw_help(&mut terminal, &mut app);
                }
            }
            recv(ui_events) -> message => {
                if let Ok(Event::Key(key_event)) = message {
                    match app.mode {
                        app::Mode::AddStock => {
                            event::handle_keys_add_stock(key_event, &mut terminal, &mut app);
                        }
                        app::Mode::DisplayStock => {
                            event::handle_keys_display_stock(key_event, &mut terminal, &mut app);
                        }
                        app::Mode::Help => {
                            event::handle_keys_help(key_event, &mut terminal, &mut app);
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

    terminal::enable_raw_mode().unwrap();
}

fn cleanup_terminal() {
    let mut stdout = io::stdout();

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

fn setup_ctrl_c() -> Receiver<()> {
    let (sender, receiver) = unbounded();
    ctrlc::set_handler(move || {
        sender.send(()).unwrap();
    })
    .unwrap();

    receiver
}

fn setup_panic_hook() {
    panic::set_hook(Box::new(|panic_info| {
        cleanup_terminal();
        better_panic::Settings::auto().create_panic_handler()(panic_info);
    }));
}
