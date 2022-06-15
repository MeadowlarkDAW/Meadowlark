use vizia::prelude::*;

const MATERIAL_CLOSE: &str = "\u{e5cd}";

pub fn bottom_bar(cx: &mut Context) {
    HStack::new(cx, |cx| {
        // TODO - Replace with list bound to app data
        HStack::new(cx, |cx| {
            HStack::new(cx, |cx| {
                Label::new(cx, "MEADOWLARK PROJECT");
                Label::new(cx, MATERIAL_CLOSE).font("material").font_size(10.0).left(Pixels(5.0));
            })
            .class("tab")
            .class("selected");

            HStack::new(cx, |cx| {
                Label::new(cx, "OTHER PROJECT");
                Label::new(cx, MATERIAL_CLOSE).font("material").font_size(10.0).left(Pixels(5.0));
            })
            .class("tab");
        })
        .child_left(Pixels(1.0))
        .child_right(Pixels(1.0))
        .col_between(Pixels(1.0));
    })
    .class("bottom_bar");
}
