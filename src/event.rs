use crossterm::event::{KeyCode::Char, KeyEvent, KeyModifiers, MouseEventKind};

use crate::app::{App, Mode};

mod add_stock;
mod configure_chart;
mod display_common;
mod display_options;
mod display_stock;
mod display_summary;
mod help;

const CONTROL: KeyModifiers = KeyModifiers::CONTROL;
const NONE: KeyModifiers = KeyModifiers::NONE;
const SHIFT: KeyModifiers = KeyModifiers::SHIFT;

pub fn handle_key_bindings(mode: Mode, key_event: KeyEvent, app: &mut App) {
    let modifiers = key_event.modifiers;
    let key = key_event.code;

    if let (CONTROL, Char('c')) = (modifiers, key) {
        app.exit_app();
    }

    match mode {
        Mode::AddStock => add_stock::handle_key_bindings(modifiers, key, app),
        Mode::ConfigureChart => configure_chart::handle_key_bindings(modifiers, key, app),
        Mode::DisplayOptions => display_options::handle_key_bindings(modifiers, key, app),
        Mode::DisplayStock => display_stock::handle_key_bindings(modifiers, key, app),
        Mode::DisplaySummary => display_summary::handle_key_bindings(modifiers, key, app),
        Mode::Help => help::handle_key_bindings(modifiers, key, app),
    };
}

pub fn handle_mouse_events(mode: Mode, mouse_event: MouseEventKind, app: &mut App) {
    if mode == Mode::DisplaySummary {
        display_summary::handle_mouse_events(mouse_event, app);
    }
}
