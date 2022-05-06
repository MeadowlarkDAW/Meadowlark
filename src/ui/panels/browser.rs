use vizia::prelude::*;

use crate::ui::ResizableStack;

pub fn browser(cx: &mut Context) {
    ResizableStack::new(cx, |cx| {
        Label::new(cx, "This is the browser");
    })
    .width(Pixels(160.0))
    .class("browser");
}
