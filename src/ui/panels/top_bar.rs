use vizia::*;

use crate::ui::{icons::IconCode, Icon, PanelEvent};

pub fn top_bar(cx: &mut Context) {
    HStack::new(cx, |cx| {
        Button::new(
            cx,
            |cx| cx.emit(PanelEvent::TogglePianoRoll),
            |cx| Icon::new(cx, IconCode::Piano, 24.0, 16.0),
        )
        .left(Stretch(1.0))
        .right(Pixels(20.0));
    })
    .class("top_bar");
}
