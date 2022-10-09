use gtk::gio::Menu;
use gtk::prelude::*;
use gtk::{
    Align, Box, Button, CenterBox, Label, Notebook, Orientation, PopoverMenuBar, Separator, Stack,
    StackSwitcher, ToggleButton,
};

pub fn setup() -> CenterBox {
    let browser_panel_center_box =
        CenterBox::builder().name("browser_panel").orientation(Orientation::Vertical).build();

    // --- Start region ------------------------------------------------------

    let start_box = Box::builder().orientation(Orientation::Vertical).build();

    let title_text = Label::builder()
        .label("BROWSER")
        .css_classes(vec!["panel_title".into()])
        .halign(Align::Start)
        .build();

    start_box.append(&title_text);

    let tabs_group = Box::builder()
        .orientation(Orientation::Horizontal)
        .margin_start(8)
        .margin_end(8)
        .homogeneous(true)
        .build();

    // TODO: Make this a functional widget.
    let audio_tab_btn = ToggleButton::builder()
        .icon_name("mdk-audio-symbolic")
        .css_classes(vec!["browser_panel_tab".into()])
        .margin_top(4)
        .build();

    // TODO: Make this a functional widget.
    let instruments_tab_btn = ToggleButton::builder()
        .icon_name("mdk-instrument-symbolic")
        .css_classes(vec!["browser_panel_tab".into()])
        .margin_top(4)
        .group(&audio_tab_btn)
        .build();

    // TODO: Make this a functional widget.
    let fx_tab_btn = ToggleButton::builder()
        .icon_name("mdk-fx-symbolic")
        .css_classes(vec!["browser_panel_tab".into()])
        .margin_top(4)
        .group(&audio_tab_btn)
        .build();

    // TODO: Make this a functional widget.
    let file_browser_tab_btn = ToggleButton::builder()
        .icon_name("mdk-folder-symbolic")
        .css_classes(vec!["browser_panel_tab".into()])
        .margin_top(4)
        .group(&audio_tab_btn)
        .build();

    tabs_group.append(&audio_tab_btn);
    tabs_group.append(&instruments_tab_btn);
    tabs_group.append(&fx_tab_btn);
    tabs_group.append(&file_browser_tab_btn);

    start_box.append(&tabs_group);

    browser_panel_center_box.set_start_widget(Some(&start_box));

    browser_panel_center_box
}
