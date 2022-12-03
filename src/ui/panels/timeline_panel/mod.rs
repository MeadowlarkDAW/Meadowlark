use vizia::prelude::*;

mod timeline_toolbar;
mod timeline_view;

pub mod track_header_view;
pub mod track_headers_panel;

use timeline_view::{TimelineView, TimelineViewStyle};

use crate::state_system::app_state::TimelineState;

pub fn timeline_panel(cx: &mut Context, timeline_state: &TimelineState) {
    VStack::new(cx, |cx| {
        timeline_toolbar::timeline_toolbar(cx);

        HStack::new(cx, |cx| {
            track_headers_panel::track_headers_panel(cx);

            TimelineView::new(cx, timeline_state, TimelineViewStyle::default())
                .width(Stretch(1.0))
                .height(Stretch(1.0));
        });
    })
    .width(Stretch(1.0));
}
