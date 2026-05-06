use crossterm::event::KeyCode::{self, Backspace, Char, Enter, Esc};
use crossterm::event::KeyModifiers;

use crate::app::App;
use crate::event::{NONE, SHIFT};

pub fn handle_key_bindings(modifiers: KeyModifiers, key: KeyCode, app: &mut App) {
    match (modifiers, key) {
        (NONE, Enter) => submit(app),
        (NONE, Esc | Char('/')) => close(app),
        (NONE, Backspace) => del_char(app),
        (NONE | SHIFT, Char(c)) => add_char(app, c),
        _ => {}
    }
}

fn submit(app: &mut App) {
    app.add_stock();
    close(app);
}

fn add_char(app: &mut App, c: char) {
    app.add_stock.add_char(c);
}

fn del_char(app: &mut App) {
    app.add_stock.del_char();
}

fn close(app: &mut App) {
    // Only close the screen if there are stocks to show
    if !app.stocks.is_empty() {
        app.add_stock.reset();
        app.close();
    }
}
