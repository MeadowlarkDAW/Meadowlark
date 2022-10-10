use gtk::gio::Menu;
use gtk::prelude::*;
use gtk::{Box, Button, CenterBox, Image, Label, Orientation, PopoverMenuBar, Separator};

pub struct TopBarWidgets {
    box_: CenterBox,

    undo_btn: Button,
    redo_btn: Button,

    loop_toggle_btn: Button,
    play_pause_btn: Button,
    record_btn: Button,

    bpm_text: Label,
    time_signature_text: Label,
}

impl TopBarWidgets {
    pub fn new() -> Self {
        let box_ = CenterBox::builder()
            .name("top_bar")
            .orientation(Orientation::Horizontal)
            .height_request(38)
            .build();

        // --- Start region ------------------------------------------------------

        let start_box = Box::builder().orientation(Orientation::Horizontal).spacing(3).build();

        start_box.append(&menu_bar());

        start_box.append(&Separator::new(Orientation::Vertical));

        // TODO: Make this a functional widget.
        let save_btn = Button::from_icon_name("mdk-save-symbolic");
        // TODO: Make this a functional widget.
        let undo_btn = Button::from_icon_name("mdk-undo-symbolic");
        // TODO: Make this a functional widget.
        let redo_btn = Button::from_icon_name("mdk-redo-symbolic");
        start_box.append(&save_btn);
        start_box.append(&undo_btn);
        start_box.append(&redo_btn);

        //start_box.append(&Separator::new(Orientation::Vertical));

        box_.set_start_widget(Some(&start_box));

        // --- Center region -----------------------------------------------------

        let center_box = Box::builder().orientation(Orientation::Horizontal).spacing(4).build();

        //center_box.append(&Separator::new(Orientation::Vertical));

        // TODO: Make this a functional widget.
        let loop_toggle_btn = Button::from_icon_name("mdk-loop-symbolic");
        // TODO: Make this a functional widget.
        let stop_btn = Button::from_icon_name("mdk-play-symbolic");
        // TODO: Make this a functional widget.
        let play_pause_btn = Button::from_icon_name("mdk-stop-symbolic");
        // TODO: Make this a functional widget.
        let record_btn = Button::builder()
            .icon_name("mdk-record-symbolic")
            .css_classes(vec!["record_btn".into()])
            .build();
        center_box.append(&loop_toggle_btn);
        center_box.append(&stop_btn);
        center_box.append(&play_pause_btn);
        center_box.append(&record_btn);

        //center_box.append(&Separator::new(Orientation::Vertical));

        box_.set_center_widget(Some(&center_box));

        // --- End region --------------------------------------------------------

        let end_box = Box::builder().orientation(Orientation::Horizontal).spacing(4).build();

        //end_box.append(&Separator::new(Orientation::Vertical));

        // TODO: Make this a functional widget.
        let bpm_text = Label::builder().label("120.0 bpm").margin_end(8).build();
        // TODO: Make this a functional widget.
        let tap_btn =
            Button::builder().label("TAP").css_classes(vec!["toolbar-text-btn".into()]).build();
        end_box.append(&bpm_text);
        end_box.append(&tap_btn);

        end_box.append(&Separator::new(Orientation::Vertical));

        let groove_btn_contents =
            Box::builder().orientation(Orientation::Horizontal).spacing(4).build();
        groove_btn_contents.append(&Label::new(Some("GRV")));
        groove_btn_contents.append(&Image::from_icon_name("mdk-menu-small-symbolic"));

        let groove_btn = Button::builder()
            .child(&groove_btn_contents)
            .css_classes(vec!["toolbar-text-btn".into()])
            .build();

        // TODO: Make this a functional widget.
        //let groove_btn =
        //Button::builder().label("GRV").css_classes(vec!["toolbar-text-btn".into()]).build();
        // TODO: Make this a functional widget.
        let time_signature_text =
            Label::builder().label("4 / 4").margin_start(8).margin_end(16).build();
        end_box.append(&groove_btn);
        end_box.append(&time_signature_text);

        box_.set_end_widget(Some(&end_box));

        Self {
            box_,
            undo_btn,
            redo_btn,
            loop_toggle_btn,
            play_pause_btn,
            record_btn,
            bpm_text,
            time_signature_text,
        }
    }

    pub fn container_widget(&self) -> &CenterBox {
        &self.box_
    }
}

fn menu_bar() -> PopoverMenuBar {
    let file_section_model = Menu::new();
    file_section_model.append(Some("Open"), None);

    let edit_section_model = Menu::new();

    let view_section_model = Menu::new();

    let help_section_model = Menu::new();
    help_section_model.append(Some("About"), Some("win.open-about-dialog"));

    let menu_bar_model = Menu::new();
    menu_bar_model.append_submenu(Some("File"), &file_section_model);
    menu_bar_model.append_submenu(Some("Edit"), &edit_section_model);
    menu_bar_model.append_submenu(Some("View"), &view_section_model);
    menu_bar_model.append_submenu(Some("Help"), &help_section_model);

    gtk::PopoverMenuBar::from_model(Some(&menu_bar_model))
}
