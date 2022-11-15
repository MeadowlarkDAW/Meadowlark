use vizia::prelude::*;

use crate::ui::views::{Icon, IconCode};

pub fn bottom_bar(cx: &mut Context) {
    HStack::new(cx, |cx| {
        Button::new(cx, |_| {}, |cx| Icon::new(cx, IconCode::Home, 22.0, 20.0)).class("icon_btn");

        Button::new(cx, |_| {}, |cx| Icon::new(cx, IconCode::Terminal, 22.0, 20.0))
            .class("icon_btn")
            .left(Stretch(1.0));
    })
    .height(Pixels(26.0))
    .class("bottom_bar");
}
