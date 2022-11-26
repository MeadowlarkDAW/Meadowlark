use vizia::prelude::*;

use crate::state_system::{AppAction, StateSystem, TrackHeadersPanelAction};
use crate::ui::views::track_header_view::TrackHeaderView;

pub fn track_headers_panel(cx: &mut Context) {
    VStack::new(cx, |cx| {
        Element::new(cx).height(Pixels(26.0)).width(Stretch(1.0)).class("top_spacer");

        ScrollView::new(cx, 0.0, 0.0, false, true, |cx| {
            List::new(cx, StateSystem::track_headers, |cx, index, entry| {
                TrackHeaderView::new(cx, entry, move |cx, height| {
                    cx.emit(AppAction::TrackHeadersPanel(
                        TrackHeadersPanelAction::ResizeTrackByIndex { index, height },
                    ))
                });
            })
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
