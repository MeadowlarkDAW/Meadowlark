use vizia::*;

use crate::ui::ResizableStack;

pub fn browser(cx: &mut Context) {
    ResizableStack::new(cx, |_| {})
        .width(Pixels(160.0))
        .text("Browser/Properties")
        .class("browser");
}
