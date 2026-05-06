use crossterm::event::KeyCode::{
    self, BackTab, Char, Delete, Down, End, Esc, Home, Left, Right, Tab, Up,
};
use crossterm::event::KeyModifiers;

use crate::app::App;
use crate::event::{CONTROL, NONE, SHIFT};

pub fn handle_key_bindings(modifiers: KeyModifiers, key: KeyCode, app: &mut App) {
    match (modifiers, key) {
        (CONTROL, Down | Right | Char('j') | Char('l')) => app.move_tab_right(),
        (CONTROL, Up | Left | Char('k') | Char('h')) => app.move_tab_left(),
        (NONE, Down | Right | Char('j') | Char('l')) => app.select_tab_right(),
        (NONE, Up | Left | Char('k') | Char('h')) => app.select_tab_left(),
        (NONE, Char('/')) => app.mode_add_stock(),
        (NONE, Char('<')) => app.scroll_left(),
        (NONE, Char('>')) => app.scroll_right(),
        (NONE, Char('?')) => app.mode_help(),
        (NONE, Char('c')) => app.toggle_chart_type(),
        (NONE, Char('e')) => app.mode_configure_chart(),
        (NONE, Char('o')) => app.mode_display_options(),
        (NONE, Char('p')) => app.toggle_pre_post(),
        (NONE, Char('q') | Esc) => app.exit_app(),
        (NONE, Char('r') | Delete) => app.remove_stock(),
        (NONE, Char('s')) => app.mode_summary_toggle(),
        (NONE, Char('v')) => app.toggle_volume(),
        (NONE, Char('x')) => app.toggle_x_labels(),
        (NONE, End) => goto_bottom(app),
        (NONE, Home) => goto_top(app),
        (NONE, Tab) => app.time_frame_up(),
        (SHIFT, BackTab) => app.time_frame_down(),
        _ => {}
    }
}

fn goto_bottom(app: &mut App){
    app.scroll_bottom();
    app.select_tab_last();
}

fn goto_top(app: &mut App){
    app.scroll_top();  
    app.select_tab_first();
}
