use vizia::*;

pub fn channels(cx: &mut Context) {
    HStack::new(cx, |cx| {
        VStack::new(cx, |cx| {}).class("instruments");

        VStack::new(cx, |cx| {}).class("patterns");
    })
    .class("channels");
}
