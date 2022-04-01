use vizia::*;

use crate::state::{
    ui_state::{TempoMapUiState, UiState},
    StateSystem,
};

pub fn tempo_controls(cx: &mut Context) -> Handle<VStack> {
    VStack::new(cx, |cx| {
        Label::new(cx, "TEMPO").class("control_header");
        HStack::new(cx, |cx| {
            Label::new(
                cx,
                StateSystem::ui_state.then(UiState::tempo_map).then(TempoMapUiState::bpm),
            );
            Label::new(cx, "TAP").width(Pixels(50.0)).height(Pixels(30.0)).width(Pixels(40.0));
            Label::new(cx, "4/4").width(Pixels(50.0)).height(Pixels(30.0)).width(Pixels(40.0));
            Label::new(cx, "Groove").width(Pixels(50.0)).height(Pixels(30.0)).width(Pixels(50.0));
        })
        .class("control_stack");
    })
    .class("control_block")
}
