use crate::ui_layer::{AppData, AppEvent, TrackData};
use vizia::prelude::*;

pub fn tracks(cx: &mut Context) {
    // Timeline::new(cx, |cx| {
    HStack::new(cx, |cx| {
        // Track group
        List::new(cx, AppData::track_data, |cx, index, item| {
            HStack::new(cx, |cx| {
                // Header
                HStack::new(cx, |cx| {
                    Label::new(cx, item.then(TrackData::name)).class("track_name");
                    Element::new(cx)
                        .background_color(item.then(TrackData::color))
                        .class("track_bar");
                })
                .class("track_header")
                .class("level4")
                .on_press(move |cx| {
                    cx.emit(AppEvent::SelectTrack(index));
                    cx.focus();
                });

                // Content
                HStack::new(cx, |cx| {
                    Label::new(cx, "pattern");
                    Label::new(cx, "track");
                })
                .class("track_content");
            })
            .class("track")
            .toggle_class("selected", item.then(TrackData::selected));
        })
        .class("track_group");
    });
    // });
}
