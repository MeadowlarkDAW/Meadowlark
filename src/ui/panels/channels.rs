use vizia::*;

use crate::ui::ResizableStack;

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

#[derive(Lens)]
pub struct ChannelRackData {
    orientation: ChannelRackOrientation,
    show_patterns: bool,
}

pub enum ChannelRackEvent {
    ToggleOrientation,
    TogglePatterns,
    ShowPatterns,
}

impl Model for ChannelRackData {
    fn event(&mut self, _: &mut Context, event: &mut Event) {
        if let Some(channel_rack_event) = event.message.downcast() {
            match channel_rack_event {
                ChannelRackEvent::ToggleOrientation => {
                    if self.orientation == ChannelRackOrientation::Horizontal {
                        self.orientation = ChannelRackOrientation::Vertical;
                    } else {
                        self.orientation = ChannelRackOrientation::Horizontal;
                    }
                }

                ChannelRackEvent::TogglePatterns => {
                    self.show_patterns ^= true;
                }

                ChannelRackEvent::ShowPatterns => {
                    self.show_patterns = false;
                }
            }
        }
    }
}

pub fn channels(cx: &mut Context) {
    ChannelRackData { orientation: ChannelRackOrientation::Horizontal, show_patterns: false }
        .build(cx);

    VStack::new(cx, |cx| {
        // Although this is a vstack we're using css to switch between horizontal and vertical layouts
        VStack::new(cx, |cx| {
            VStack::new(cx, |cx| {
                // TODO - Make this resizable when channel rack orientation is vertical
                VStack::new(cx, |cx| {
                    HStack::new(cx, |cx| {
                        Button::new(
                            cx,
                            |cx| {
                                cx.emit(ChannelRackEvent::ToggleOrientation);
                                cx.emit(ChannelRackEvent::ShowPatterns);
                            },
                            |cx| Label::new(cx, "A"),
                        )
                        .child_space(Stretch(1.0))
                        .width(Pixels(24.0));

                        Button::new(
                            cx,
                            |cx| cx.emit(ChannelRackEvent::TogglePatterns),
                            |cx| Label::new(cx, "B"),
                        )
                        .child_space(Stretch(1.0))
                        .width(Pixels(24.0));
                    })
                    .class("header");
                    Label::new(cx, "Channels");
                })
                .width(Pixels(225.0))
                .class("instruments");

                VStack::new(cx, |cx| {
                    Element::new(cx).class("header");
                })
                .text("Patterns")
                .class("patterns")
                .checked(ChannelRackData::show_patterns);
            });

            VStack::new(cx, |cx| {
                Element::new(cx).class("header");
            })
            .text("Patterns")
            .class("patterns")
            .checked(ChannelRackData::show_patterns);
        })
        .toggle_class("vertical", ChannelRackData::orientation.map(|&val| val.into()))
        .toggle_class("hidden", ChannelRackData::show_patterns)
        .class("channels");
    })
    .class("channel_rack");
}
