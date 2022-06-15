use vizia::prelude::*;

pub fn timeline(cx: &mut Context) {
    VStack::new(cx, |cx| {
        HStack::new(cx, |cx| {
            Label::new(cx, "TIMELINE").class("small");
        })
        .class("header");

        // Contents
        VStack::new(cx, |_| {}).class("level3");
    })
    .row_between(Pixels(1.0))
    .class("timeline");
}
