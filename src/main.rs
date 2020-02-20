use crossbeam_channel::{select, tick, unbounded, Receiver};

use crossterm::cursor;
use crossterm::event::{Event, KeyCode, KeyModifiers};
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
mod service;
mod task;
mod time_frame;
mod widget;

pub use crate::time_frame::TimeFrame;

fn main() {
    better_panic::install();

    let opts = cli::get_opts();

    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend).unwrap();

    setup_panic_hook();
    setup_terminal();

    let ticker = tick(Duration::from_secs_f64(1.0));
    let ui_events = setup_ui_events();
    let ctrl_c_events = setup_ctrl_c();

    let starting_stocks: Vec<_> = opts
        .stocks
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
        current_tab: 0,
    };

    for stock in app.stocks.iter_mut() {
        stock.update();
    }

    draw::draw(&mut terminal, &mut app);

    loop {
        select! {
            recv(ctrl_c_events) -> _ => {
                break;
            }
            recv(ticker) -> _ => {
                for stock in app.stocks.iter_mut() {
                    stock.update();
                }

                if app.mode == app::Mode::DisplayStock {
                    draw::draw(&mut terminal, &mut app);
                }
            }
            recv(ui_events) -> message => {
                match app.mode {
                    app::Mode::AddStock => {
                        if let Ok(Event::Key(key_event)) = message {
                            if key_event.modifiers.is_empty() || key_event.modifiers == KeyModifiers::SHIFT {
                                match key_event.code {
                                    KeyCode::Enter => {
                                        let stock = app.add_stock.enter();

                                        app.stocks.push(stock);
                                        app.current_tab = app.stocks.len() - 1;

                                        app.add_stock.reset();
                                        app.mode = app::Mode::DisplayStock;

                                        draw::draw(&mut terminal, &mut app);

                                    }
                                    KeyCode::Char(c) => {
                                        app.add_stock.add_char(c);
                                        draw::draw(&mut terminal, &mut app);
                                    }
                                    KeyCode::Backspace => {
                                        app.add_stock.del_char();
                                        draw::draw(&mut terminal, &mut app);
                                    }
                                    KeyCode::Esc => {
                                        app.add_stock.reset();
                                        if !app.stocks.is_empty() {
                                            app.mode = app::Mode::DisplayStock;
                                        }
                                        draw::draw(&mut terminal, &mut app);
                                    }
                                    _ => {}
                                }
                            } else if key_event.modifiers == KeyModifiers::CONTROL {
                                if let KeyCode::Char('c') = key_event.code {
                                        break
                                }
                            }
                        }
                    }
                    app::Mode::DisplayStock => {
                        if let Ok(Event::Key(key_event)) = message {
                            if key_event.modifiers.is_empty() {
                                match key_event.code {
                                    KeyCode::Left => {
                                        app.stocks[app.current_tab].time_frame_down();
                                        draw::draw(&mut terminal, &mut app);
                                    },
                                    KeyCode::Right => {
                                        app.stocks[app.current_tab].time_frame_up();
                                        draw::draw(&mut terminal, &mut app);
                                    },
                                    KeyCode::Char('/') => {
                                        app.mode = app::Mode::AddStock;
                                        draw::draw(&mut terminal, &mut app);
                                    }
                                    KeyCode::Char('k') => {
                                        app.stocks.remove(app.current_tab);

                                        if app.current_tab != 0 {
                                            app.current_tab -= 1;
                                        }

                                        if app.stocks.is_empty() {
                                            app.mode = app::Mode::AddStock;
                                        }

                                        draw::draw(&mut terminal, &mut app);
                                    }
                                    KeyCode::Tab => {
                                        if app.current_tab == app.stocks.len() - 1 {
                                            app.current_tab = 0;
                                            draw::draw(&mut terminal, &mut app);
                                        } else {
                                            app.current_tab += 1;
                                            draw::draw(&mut terminal, &mut app);
                                        }
                                    }
                                    KeyCode::BackTab => {
                                        if app.current_tab == 0 {
                                            app.current_tab = app.stocks.len() - 1;
                                            draw::draw(&mut terminal, &mut app);
                                        } else {
                                            app.current_tab -= 1;
                                            draw::draw(&mut terminal, &mut app);
                                        }
                                    }
                                    _ => {}
                                }
                            } else if key_event.modifiers == KeyModifiers::CONTROL {
                                if let KeyCode::Char('c') = key_event.code {
                                        break
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    cleanup_terminal();
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
