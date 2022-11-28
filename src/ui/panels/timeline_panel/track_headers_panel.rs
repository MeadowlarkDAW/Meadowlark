use vizia::prelude::*;

use super::track_header_view::TrackHeaderView;
use crate::state_system::{AppAction, BoundUiState, StateSystem, TrackAction};

pub fn track_headers_panel(cx: &mut Context) {
    VStack::new(cx, |cx| {
        Element::new(cx).height(Pixels(26.0)).width(Stretch(1.0)).class("top_spacer");

        ScrollView::new(cx, 0.0, 0.0, false, true, |cx| {
            List::new(
                cx,
                StateSystem::bound_ui_state.then(BoundUiState::track_headers),
                |cx, index, entry| {
                    TrackHeaderView::new(cx, entry, move |cx, height| {
                        cx.emit(AppAction::Track(TrackAction::ResizeTrackLaneByIndex {
                            index,
                            height,
                        }))
                    });
                },
            )
            .top(Pixels(2.0))
            .child_space(Pixels(2.0))
            .width(Stretch(1.0))
            .height(Auto)
            .row_between(Pixels(2.0));
        })
        .class("hidden_scrollbar")
        .height(Stretch(1.0));
    })
    .class("track_headers_panel")
    .width(Pixels(250.0))
    .height(Stretch(1.0));
}
