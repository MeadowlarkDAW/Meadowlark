use vizia::prelude::*;

use crate::ui::PanelState;

pub fn piano_roll(cx: &mut Context) {
    VStack::new(cx, |cx| {
        HStack::new(cx, |_| {}).class("toolbar");
    })
    .class("piano_roll")
    .toggle_class("hidden", PanelState::hide_piano_roll);
}
