use std::cell::RefCell;
use std::rc::Rc;
use vizia::prelude::*;

use crate::state_system::actions::{AppAction, InternalAction};
use crate::state_system::working_state::timeline_view_state::TimelineViewState;

mod timeline_toolbar;
mod timeline_view;

pub mod track_header_view;
pub mod track_headers_panel;

pub use timeline_view::TimelineViewEvent;

use timeline_view::{TimelineView, TimelineViewStyle};

pub fn timeline_panel(
    cx: &mut Context,
    shared_timeline_view_state: Rc<RefCell<TimelineViewState>>,
) {
    VStack::new(cx, |cx| {
        timeline_toolbar::timeline_toolbar(cx);

        HStack::new(cx, |cx| {
            track_headers_panel::track_headers_panel(cx);

            let timeline_view_id =
                TimelineView::new(cx, shared_timeline_view_state, TimelineViewStyle::default())
                    .width(Stretch(1.0))
                    .height(Stretch(1.0))
                    .entity;

            cx.emit(AppAction::_Internal(InternalAction::TimelineViewID(timeline_view_id)));
        });
    })
    .width(Stretch(1.0));
}
