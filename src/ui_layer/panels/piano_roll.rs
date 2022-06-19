use vizia::prelude::*;

use crate::ui_layer::Panel;

pub fn piano_roll(cx: &mut Context) {
    VStack::new(cx, |cx| {
        Panel::new(
            cx,
            |cx| {
                Label::new(cx, "PIANO ROLL").class("small");
            },
            |_| {},
        )
        .class("piano_roll");
    })
    .row_between(Pixels(1.0))
    .class("piano_roll");
}
