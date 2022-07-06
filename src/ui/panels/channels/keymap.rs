use vizia::prelude::*;

use super::ChannelEvent;

pub fn channels_keymap(cx: &mut Context) {
    Keymap::from(vec![
        // CTRL + N => Insert new channel.
        (
            KeyChord::new(Modifiers::CTRL, Code::KeyN),
            KeymapEntry::new(ChannelEvent::AddChannel, |cx| {
                cx.emit(ChannelEvent::AddChannel);
            }),
        ),
    ])
    .build(cx);
}
