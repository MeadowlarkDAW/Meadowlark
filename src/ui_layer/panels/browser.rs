use vizia::prelude::*;

use crate::{
    program_layer::{
        program_state::{PanelEvent, PanelState},
        ProgramLayer, ProgramState,
    },
    ui_layer::{Panel, ResizableStack},
};

pub fn browser(cx: &mut Context) {
    ResizableStack::new(
        cx,
        ProgramLayer::state.then(ProgramState::panels.then(PanelState::browser_width)),
        |cx, width| {
            cx.emit(PanelEvent::SetBrowserWidth(width));
        },
        |cx| {
            Panel::new(
                cx,
                |cx| {
                    Label::new(cx, "BROWSER").class("small");
                },
                |_| {},
            );
        },
    )
    .class("browser")
    .display(ProgramLayer::state.then(ProgramState::panels.then(PanelState::show_browser)));
}
