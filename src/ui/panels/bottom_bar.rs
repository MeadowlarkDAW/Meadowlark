use vizia::prelude::*;

use crate::ui::icon::{Icon, IconCode};

pub fn bottom_bar(cx: &mut Context) {
    HStack::new(cx, |cx| {
        Button::new(cx, |_| {}, |cx| Icon::new(cx, IconCode::Home, 28.0, 22.0)).class("icon_btn");

        Button::new(cx, |_| {}, |cx| Icon::new(cx, IconCode::Terminal, 28.0, 22.0))
            .class("icon_btn")
            .left(Stretch(1.0));
    })
    .class("bottom_bar");
}
