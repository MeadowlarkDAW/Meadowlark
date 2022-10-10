use gtk::gio::Menu;
use gtk::prelude::*;
use gtk::{
    Align, Box, Button, CenterBox, Label, Orientation, PopoverMenuBar, Separator, ToggleButton,
};

use crate::state::AppState;

pub fn setup(app_state: &AppState) -> Box {
    let tabs_box = Box::builder()
        .name("side_bar_tabs")
        .orientation(Orientation::Vertical)
        .width_request(40)
        .spacing(3)
        .build();

    let browser_panel_btn = ToggleButton::builder()
        .icon_name("mdk-folder-symbolic")
        .css_classes(vec!["side_bar_tab".into()])
        .margin_top(4)
        .active(app_state.browser_panel_shown)
        .build();
    browser_panel_btn.connect_clicked(move |button| {
        button
            .activate_action("app.toggle_browser_panel", Some(&button.is_active().to_variant()))
            .unwrap();
    });

    // TODO: Make this a functional widget.
    let piano_roll_panel_btn = ToggleButton::builder()
        .icon_name("mdk-piano-roll-symbolic")
        .css_classes(vec!["side_bar_tab".into()])
        .margin_top(4)
        .build();

    // TODO: Make this a functional widget.
    let properties_panel_btn = ToggleButton::builder()
        .icon_name("mdk-properties-panel-symbolic")
        .css_classes(vec!["side_bar_tab".into()])
        .margin_top(4)
        .build();

    tabs_box.append(&browser_panel_btn);
    tabs_box.append(&piano_roll_panel_btn);
    tabs_box.append(&properties_panel_btn);
    tabs_box
}
