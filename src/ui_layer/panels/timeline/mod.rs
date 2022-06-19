mod grid;
mod keymap;
mod lanes;

use self::{grid::TimelineGridHeader, lanes::lane_content};
use crate::ui_layer::Panel;
use grid::TimelineGrid;
use keymap::timeline_keymap;
use lanes::lane_header;
use vizia::prelude::*;

pub fn timeline(cx: &mut Context) {
    timeline_keymap(cx);

    VStack::new(cx, |cx| {
        Panel::new(
            cx,
            |cx| {
                Label::new(cx, "TIMELINE").class("small");
            },
            |cx| {
                // Timeline content
                VStack::new(cx, |cx| {
                    // Left area of the timeline content
                    HStack::new(cx, |cx| {
                        // Above the lane headers
                        HStack::new(cx, |cx| {
                            Element::new(cx);
                        })
                        .class("lane_header");

                        // Header of the timeline
                        TimelineGridHeader::new(cx);
                    })
                    .class("timeline_content_header");

                    // Right area of the timeline content
                    ScrollView::new(cx, 0.0, 0.0, true, true, |cx| {
                        HStack::new(cx, |cx| {
                            lane_header(cx);
                            ZStack::new(cx, |cx| {
                                TimelineGrid::new(cx);
                                lane_content(cx);
                            });
                        });
                    })
                    .class("timeline_content");
                });
            },
        )
        .class("timeline");
    })
    .row_between(Pixels(1.0))
    .class("timeline");
}
