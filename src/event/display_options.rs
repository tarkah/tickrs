use crossterm::event::KeyCode::{self, BackTab, Char, Down, Esc, Left, Right, Tab, Up};
use crossterm::event::KeyModifiers;

use crate::app::App;
use crate::event::{NONE, SHIFT};

pub fn handle_key_bindings(modifiers: KeyModifiers, key: KeyCode, app: &mut App) {
    match (modifiers, key) {
        (NONE, Down | Char('j')) => app.select_options_next(),
        (NONE, Left | Char('h')) => app.select_options_left(),
        (NONE, Right | Char('l')) => app.select_options_right(),
        (NONE, Up | Char('k')) => app.select_options_previous(),
        (NONE, Esc | Char('o') | Char('q')) => close(app),
        (NONE, Tab) | (SHIFT, BackTab) => app.toggle_option_type(),
        _ => {}
    }
}

fn close(app: &mut App) {
    app.toggle_options();
    app.close();
}
