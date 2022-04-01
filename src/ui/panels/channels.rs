use vizia::*;

use crate::ui::ResizableStack;

pub fn channels(cx: &mut Context) {
    ResizableStack::new(cx, |cx| {
        ResizableStack::new(cx, |cx| {}).text("Instruments").class("instruments");

        VStack::new(cx, |cx| {}).text("Patterns").class("patterns");
    })
    .layout_type(LayoutType::Row)
    .width(Pixels(425.0))
    .class("channels");

    // HStack::new(cx, |cx| {

    // })
    // .class("channels");
}
