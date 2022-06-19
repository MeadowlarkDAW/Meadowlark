use vizia::prelude::*;

use crate::ui_layer::Panel;

pub fn timeline(cx: &mut Context) {
    VStack::new(cx, |cx| {
        Panel::new(
            cx,
            |cx| {
                Label::new(cx, "TIMELINE").class("small");
            },
            |_| {},
        )
        .class("timeline");
    })
    .row_between(Pixels(1.0))
    .class("timeline");
}
