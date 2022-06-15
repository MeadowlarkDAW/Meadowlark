use vizia::prelude::*;

use crate::ui::ResizableStack;

pub fn browser(cx: &mut Context) {
    ResizableStack::new(cx, |cx| {
        HStack::new(cx, |cx| {
            Label::new(cx, "BROWSER").class("small");
        })
        .class("header");

        // Contents
        VStack::new(cx, |_| {}).class("level3");
    })
    .row_between(Pixels(1.0))
    .width(Pixels(160.0))
    .class("browser");
}
