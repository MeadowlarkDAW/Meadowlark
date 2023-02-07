use vizia::prelude::*;

use crate::{
    state_system::{
        working_state::browser_panel_state::BrowserPanelState, AppAction, BrowserPanelAction,
        StateSystem, WorkingState,
    },
    ui::generic_views::{icon, IconCode},
};

pub fn side_tab_bar(cx: &mut Context) {
    const ICON_FRAME_SIZE: f32 = 26.0;
    const ICON_SIZE: f32 = 22.0;

    VStack::new(cx, |cx| {
        Button::new(
            cx,
            |cx| {
                cx.emit(AppAction::BrowserPanel(BrowserPanelAction::SetPanelShown(
                    !StateSystem::working_state
                        .then(WorkingState::browser_panel_lens)
                        .then(BrowserPanelState::panel_shown)
                        .get(cx),
                )))
            },
            |cx| icon(cx, IconCode::Folder, ICON_FRAME_SIZE, ICON_SIZE),
        )
        .class("side_tab_btn")
        .toggle_class(
            "side_tab_toggled",
            StateSystem::working_state
                .then(WorkingState::browser_panel_lens)
                .then(BrowserPanelState::panel_shown),
        );

        Button::new(cx, |_| {}, |cx| icon(cx, IconCode::Piano, ICON_FRAME_SIZE, ICON_SIZE))
            .class("side_tab_btn");

        Button::new(cx, |_| {}, |cx| icon(cx, IconCode::Properties, ICON_FRAME_SIZE, ICON_SIZE))
            .class("side_tab_btn");
    })
    .height(Stretch(1.0))
    .row_between(Pixels(6.0))
    .width(Pixels(32.0))
    .class("side_tab_bar");
}
