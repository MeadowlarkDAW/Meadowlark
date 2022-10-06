use gtk::gio::Menu;
use gtk::prelude::*;
use gtk::{Box, Button, CenterBox, Label, Orientation, PopoverMenuBar, Separator};

pub fn setup() -> Box {
    let tabs_box = Box::builder()
        .name("browser_panel_tabs")
        .orientation(Orientation::Vertical)
        .width_request(40)
        .build();

    tabs_box
}
