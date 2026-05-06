use crossbeam_channel::Sender;
use crossterm::event::{KeyCode::Char, KeyEvent, KeyModifiers};

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

pub fn handle_key_bindings(
    mode: Mode,
    key_event: KeyEvent,
    app: &mut App,
    request_redraw: &Sender<()>,
) {
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

    let _ = request_redraw.try_send(());
}
