use vizia::*;

pub fn bottom_bar(cx: &mut Context) {
    HStack::new(cx, |_| {}).text("Bottom Bar").class("bottom_bar");
}
