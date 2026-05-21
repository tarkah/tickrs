use crossterm::event::KeyCode::{self, Char, Esc};
use crossterm::event::KeyModifiers;

use crate::app::App;
use crate::event::NONE;

pub fn handle_key_bindings(modifiers: KeyModifiers, key: KeyCode, app: &mut App) {
    if let (NONE, Esc | Char('?') | Char('q')) = (modifiers, key) {
        app.close();
    }
}
