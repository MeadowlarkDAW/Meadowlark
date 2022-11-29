use vizia::prelude::*;

mod timeline_toolbar;
mod timeline_view;

pub mod track_header_view;
pub mod track_headers_panel;

pub fn timeline_panel(cx: &mut Context) {
    VStack::new(cx, |cx| {
        timeline_toolbar::timeline_toolbar(cx);

        HStack::new(cx, |cx| {
            track_headers_panel::track_headers_panel(cx);

            Element::new(cx).width(Stretch(1.0));
        });
    })
    .width(Stretch(1.0));
}
