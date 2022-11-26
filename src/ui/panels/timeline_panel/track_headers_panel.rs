use vizia::prelude::*;

use crate::ui::views::track_header::{TrackColor, TrackHeader, TrackType};

pub fn track_headers_panel(cx: &mut Context) {
    VStack::new(cx, |cx| {
        Element::new(cx).height(Pixels(26.0)).width(Stretch(1.0)).class("top_spacer");

        VStack::new(cx, |cx| {
            TrackHeader::new(cx, TrackColor::MasterTrack, 60.0, "Master".into(), TrackType::Master);
            TrackHeader::new(cx, TrackColor::Color0, 60.0, "Spicy Synth".into(), TrackType::Synth);
            TrackHeader::new(cx, TrackColor::Color1, 60.0, "Drum Hits".into(), TrackType::Audio);

            Element::new(cx).top(Stretch(1.0));
        })
        .top(Pixels(2.0))
        .child_space(Pixels(2.0))
        .width(Stretch(1.0))
        .height(Stretch(1.0))
        .class("track_headers_panel")
        .row_between(Pixels(2.0));
    })
    .width(Pixels(240.0))
    .height(Stretch(1.0));
}
