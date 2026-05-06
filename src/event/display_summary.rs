use crossterm::event::{KeyCode, KeyModifiers};

use crate::app::App;
use crate::event::display_common;

pub fn handle_key_bindings(modifiers: KeyModifiers, key: KeyCode, app: &mut App) {
    display_common::handle_key_bindings(modifiers, key, app);
}
