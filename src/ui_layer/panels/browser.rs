use vizia::prelude::*;

use crate::{
    program_layer::{
        program_state::{PanelEvent, PanelState},
        ProgramLayer, ProgramState,
    },
    ui_layer::ResizableStack,
};

pub fn browser(cx: &mut Context) {
    ResizableStack::new(
        cx,
        ProgramLayer::state.then(ProgramState::panels.then(PanelState::browser_width)),
        |cx, width| {
            cx.emit(PanelEvent::SetBrowserWidth(width));
        },
        |cx| {
            HStack::new(cx, |cx| {
                Label::new(cx, "BROWSER").class("small");
            })
            .class("header");

            // Contents
            VStack::new(cx, |_| {}).class("level3");
        },
    )
    .row_between(Pixels(1.0))
    .width(Pixels(160.0))
    .class("browser")
    .display(ProgramLayer::state.then(ProgramState::panels.then(PanelState::show_browser)));
}
