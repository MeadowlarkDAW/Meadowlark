use vizia::*;

pub fn top_bar(cx: &mut Context) {
    HStack::new(cx, |_| {}).text("Top Bar").class("top_bar");
}
