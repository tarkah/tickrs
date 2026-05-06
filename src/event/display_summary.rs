use crossterm::event::{
    KeyCode, KeyModifiers,
    MouseEventKind::{self, ScrollDown, ScrollUp},
};

use crate::app::App;
use crate::event::display_common;

pub fn handle_key_bindings(modifiers: KeyModifiers, key: KeyCode, app: &mut App) {
    display_common::handle_key_bindings(modifiers, key, app);
}

pub fn handle_mouse_events(mouse_event: MouseEventKind, app: &mut App) {
    match mouse_event {
        ScrollDown => app.summary_scroll_state.scroll_down(),
        ScrollUp => app.summary_scroll_state.scroll_up(),
        _ => {}
    }
}
