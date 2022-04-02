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

impl Into<bool> for ChannelRackOrientation {
    fn into(self) -> bool {
        match self {
            Self::Vertical => true,
            Self::Horizontal => false,
        }
    }
}

#[derive(Lens)]
pub struct ChannelRackData {
    orientation: ChannelRackOrientation,
}

pub enum ChannelRackEvent {
    ToggleOrientation,
}

impl Model for ChannelRackData {
    fn event(&mut self, cx: &mut Context, event: &mut Event) {
        if let Some(channel_rack_event) = event.message.downcast() {
            match channel_rack_event {
                ChannelRackEvent::ToggleOrientation => {
                    if self.orientation == ChannelRackOrientation::Horizontal {
                        self.orientation = ChannelRackOrientation::Vertical;
                    } else {
                        self.orientation = ChannelRackOrientation::Horizontal;
                    }
                }
            }
        }
    }
}

pub fn channels(cx: &mut Context) {
    ChannelRackData { orientation: ChannelRackOrientation::Horizontal }.build(cx);

    VStack::new(cx, |cx| {
        VStack::new(cx, |cx| {
            VStack::new(cx, |cx| {
                Button::new(
                    cx,
                    |cx| cx.emit(ChannelRackEvent::ToggleOrientation),
                    |cx| Label::new(cx, "TEST"),
                )
                .space(Pixels(5.0));
                Label::new(cx, "Channels");
            })
            .width(Pixels(225.0))
            .class("instruments");

            VStack::new(cx, |cx| {}).text("Patterns").class("patterns");
        });

        VStack::new(cx, |cx| {}).text("Patterns").class("patterns");
    })
    // .width(ChannelRackData::orientation.map(|val|{
    //     match val {
    //         ChannelRackOrientation::Horizontal => {
    //             Pixels(425.0)
    //         }
    //         ChannelRackOrientation::Vertical => {
    //             Pixels(225.0)
    //         }
    //     }
    // }))
    .toggle_class("vertical", ChannelRackData::orientation.map(|&val| val.into()))
    .class("channels");

    // HStack::new(cx, |cx| {
    //     ResizableStack::new(cx, |cx| {}).text("Instruments").class("instruments");

    //     VStack::new(cx, |cx| {}).text("Patterns").class("patterns");
    // })
    // .class("channels")
}
