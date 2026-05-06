use crossterm::event::KeyCode::{Backspace, Char, Down, Enter, Esc, Left, Right, Up};
use crossterm::event::{KeyCode, KeyModifiers};

use crate::app::App;
use crate::event::NONE;

pub fn handle_key_bindings(modifiers: KeyModifiers, key: KeyCode, app: &mut App) {
    match (modifiers, key) {
        (NONE, Enter) => submit(app),
        (NONE, Esc | Char('q') | Char('e')) => close(app),
        (NONE, Down | Char('j')) => select_down(app),
        (NONE, Left | Char('h')) => select_left(app),
        (NONE, Right | Char('l')) => select_right(app),
        (NONE, Up | Char('k')) => select_up(app),
        (NONE, Backspace) => del_char(app),
        (NONE, Char(c)) => add_char(app, c),
        _ => {}
    }
}

fn del_char(app: &mut App) {
    app.chart_config_mut().del_char();
}

fn add_char(app: &mut App, c: char) {
    if c.is_numeric() || c == '.' {
        app.chart_config_mut().add_char(c);
    }
}

fn close(app: &mut App) {
    app.toggle_configure();
    app.close();
}

fn select_down(app: &mut App) {
    app.chart_config_mut().selection_down();
}

fn select_up(app: &mut App) {
    app.chart_config_mut().selection_up();
}

fn select_left(app: &mut App) {
    app.chart_config_mut().back_tab();
}

fn select_right(app: &mut App) {
    app.chart_config_mut().tab();
}

fn submit(app: &mut App) {
    let time_frame = app.time_frame;
    app.chart_config_mut().enter(time_frame);
}
