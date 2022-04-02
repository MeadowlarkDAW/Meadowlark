use vizia::*;

pub fn timeline(cx: &mut Context) {
    VStack::new(cx, |cx| {
        HStack::new(cx, |_| {}).class("toolbar");
    })
    .text("Timeline")
    .class("timeline");
}
