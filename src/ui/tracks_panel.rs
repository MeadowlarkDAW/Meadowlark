use gtk::gio::Menu;
use gtk::prelude::*;
use gtk::{
    Box, Button, CenterBox, Label, Notebook, Orientation, PopoverMenuBar, Separator, ToggleButton,
};

pub fn setup() -> CenterBox {
    let tracks_panel_center_box =
        CenterBox::builder().name("tracks_panel").orientation(Orientation::Vertical).build();

    // --- Start region ------------------------------------------------------

    tracks_panel_center_box
}
