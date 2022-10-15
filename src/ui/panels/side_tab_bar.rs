use vizia::prelude::*;

use crate::{
    state_system::{AppEvent, BoundUiState, StateSystem},
    ui::icon::{Icon, IconCode},
};

pub fn side_tab_bar(cx: &mut Context) {
    VStack::new(cx, |cx| {
        Button::new(
            cx,
            |cx| cx.emit(AppEvent::ToggleBrowserPanelShown),
            |cx| Icon::new(cx, IconCode::Folder, 32.0, 26.0),
        )
        .class("icon_btn")
        .toggle_class(
            "side_tab_toggled",
            StateSystem::bound_ui_state.then(BoundUiState::browser_panel_shown),
        );

        Button::new(cx, |_| {}, |cx| Icon::new(cx, IconCode::PianoRoll, 32.0, 26.0))
            .class("icon_btn");

        Button::new(cx, |_| {}, |cx| Icon::new(cx, IconCode::Properties, 32.0, 26.0))
            .class("icon_btn");
    })
    .height(Stretch(1.0))
    .row_between(Pixels(5.0))
    .class("side_tab_bar");
}
