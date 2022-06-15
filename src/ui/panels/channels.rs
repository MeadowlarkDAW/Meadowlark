use vizia::prelude::*;

use crate::ui::{PanelEvent, PanelState, ResizableStack};

#[derive(Debug, Clone, Copy, PartialEq, Data)]
pub enum ChannelRackOrientation {
    Horizontal,
    Vertical,
}

impl Default for ChannelRackOrientation {
    fn default() -> Self {
        Self::Horizontal
    }
}

impl From<ChannelRackOrientation> for bool {
    fn from(orientation: ChannelRackOrientation) -> bool {
        match orientation {
            ChannelRackOrientation::Vertical => true,
            ChannelRackOrientation::Horizontal => false,
        }
    }
}

pub fn channels(cx: &mut Context) {
    VStack::new(cx, |cx| {
        // Although this is a vstack we're using css to switch between horizontal and vertical layouts
        VStack::new(cx, |cx| {
            VStack::new(cx, |cx| {
                // TODO - Make this resizable when channel rack orientation is vertical
                VStack::new(cx, |cx| {
                    // Header
                    HStack::new(cx, |cx| {
                        Label::new(cx, "CHANNEL RACK").class("small");

                        Button::new(
                            cx,
                            |cx| {
                                cx.emit(PanelEvent::ToggleChannelRackOrientation);
                                cx.emit(PanelEvent::ShowPatterns);
                            },
                            |cx| Label::new(cx, "A"),
                        )
                        .child_space(Stretch(1.0))
                        .width(Pixels(24.0))
                        .left(Stretch(1.0));

                        Button::new(
                            cx,
                            |cx| cx.emit(PanelEvent::TogglePatterns),
                            |cx| Label::new(cx, "B"),
                        )
                        .child_space(Stretch(1.0))
                        .width(Pixels(24.0))
                        .right(Pixels(10.0));
                    })
                    .class("header");

                    // Contents
                    VStack::new(cx, |_| {}).class("level3");
                })
                .row_between(Pixels(1.0))
                .width(Pixels(225.0))
                .class("instruments");

                VStack::new(cx, |cx| {
                    HStack::new(cx, |cx| {
                        Label::new(cx, "PATTERNS").class("small");
                    })
                    .class("header");

                    // Contents
                    VStack::new(cx, |_| {}).class("level3");
                })
                .row_between(Pixels(1.0))
                .class("patterns")
                .checked(PanelState::hide_patterns);
            })
            .overflow(Overflow::Hidden);

            VStack::new(cx, |cx| {
                // TODO - De-duplicate this code
                HStack::new(cx, |cx| {
                    Label::new(cx, "PATTERNS").class("small").text_wrap(false);
                })
                .class("header");

                // Contents
                VStack::new(cx, |_| {}).class("level3");
            })
            .row_between(Pixels(1.0))
            .class("patterns")
            .checked(PanelState::hide_patterns);
        })
        .toggle_class("vertical", PanelState::channel_rack_orientation.map(|&val| val.into()))
        .toggle_class("hidden", PanelState::hide_patterns)
        .class("channels");
    })
    .class("channel_rack");
}
