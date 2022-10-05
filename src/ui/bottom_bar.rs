use gtk::prelude::*;
use gtk::{Box, Button, CenterBox, Orientation, Separator};

pub fn setup() -> CenterBox {
    let bottom_bar_center_box = CenterBox::builder().name("bottom_bar").height_request(26).build();

    // --- Start region ------------------------------------------------------

    let start_box = Box::builder().orientation(Orientation::Horizontal).spacing(4).build();

    // TODO: Make this a functional widget.
    let home_btn = Button::builder().icon_name("go-home-symbolic").margin_start(4).build();
    start_box.append(&home_btn);

    start_box.append(&Separator::new(Orientation::Vertical));

    bottom_bar_center_box.set_start_widget(Some(&start_box));

    // --- End region --------------------------------------------------------

    let end_box = Box::builder().orientation(Orientation::Horizontal).spacing(4).build();

    end_box.append(&Separator::new(Orientation::Vertical));

    // TODO: Make this a functional widget.
    let terminal_btn = Button::builder().icon_name("system-run-symbolic").margin_end(4).build();
    end_box.append(&terminal_btn);

    bottom_bar_center_box.set_end_widget(Some(&end_box));

    bottom_bar_center_box
}
