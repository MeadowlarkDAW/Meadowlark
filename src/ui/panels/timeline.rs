use vizia::*;

pub fn timeline(cx: &mut Context) {
    VStack::new(cx, |cx| {
        HStack::new(cx, |cx| {}).class("toolbar");
    })
    .text("Timeline")
    .class("timeline");
}
