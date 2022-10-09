use gtk::gio::Menu;
use gtk::prelude::*;
use gtk::{
    Box, Button, CenterBox, Label, Notebook, Orientation, PopoverMenuBar, Separator, ToggleButton,
};

pub fn setup() -> CenterBox {
    let timeline_panel_center_box =
        CenterBox::builder().name("timeline_panel").orientation(Orientation::Vertical).build();

    // --- Start region ------------------------------------------------------

    timeline_panel_center_box
}
