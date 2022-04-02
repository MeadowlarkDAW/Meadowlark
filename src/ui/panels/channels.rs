use vizia::*;

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
}

pub enum ChannelRackEvent {
    ToggleOrientation,
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
            }
        }
    }
}

pub fn channels(cx: &mut Context) {
    ChannelRackData { orientation: ChannelRackOrientation::Horizontal }.build(cx);

    VStack::new(cx, |cx| {
        VStack::new(cx, |cx| {
            // TODO - Make this resizable when channel rack orientation is vertical
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

            VStack::new(cx, |_| {}).text("Patterns").class("patterns");
        });

        VStack::new(cx, |_| {}).text("Patterns").class("patterns");
    })
    .toggle_class("vertical", ChannelRackData::orientation.map(|&val| val.into()))
    .class("channels");
}
