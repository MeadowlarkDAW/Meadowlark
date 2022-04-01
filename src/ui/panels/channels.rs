use vizia::*;

pub fn channels(cx: &mut Context) {
    HStack::new(cx, |cx| {
        VStack::new(cx, |cx| {}).text("Instruments").class("instruments");

        VStack::new(cx, |cx| {}).text("Patterns").class("patterns");
    })
    .class("channels");
}
