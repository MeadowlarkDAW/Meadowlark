use vizia::*;

pub fn top_bar(cx: &mut Context) {
    HStack::new(cx, |cx| {}).text("Top Bar").class("top_bar");
}