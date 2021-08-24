use tuix::*;

use crate::ui::{AppData, TempoEvent, TransportEvent};

use super::ControlBar;

/// Widget for the TEMPO control bar
#[derive(Default)]
pub struct TransportControlBar {}

impl Widget for TransportControlBar {
    type Ret = Entity;
    type Data = ();
    fn on_build(&mut self, state: &mut State, entity: Entity) -> Self::Ret {
        let controls = ControlBar::new("TRANSPORT").build(state, entity, |builder| builder);

        // Playhead position
        Label::new("5.2.3")
            //.bind(AppData::beats_per_minute, |value| value.to_string())
            .build(state, controls, |builder| {
                builder.set_name("playhead position")
            });

        // Play/ Pause button

        CheckButton::new()
            .on_checked(|data, state, checkbutton| {
                checkbutton.set_text(state, "PAUSE");
                checkbutton.emit(state, TransportEvent::Play);
            })
            .on_unchecked(|data, state, checkbutton| {
                checkbutton.set_text(state, "PLAY");
                checkbutton.emit(state, TransportEvent::Pause);
            })
            .bind(AppData::is_playing, |is_playing| *is_playing)
            .build(state, controls, |builder| builder);

        // Stop button
        Button::with_label("STOP")
            .on_press(|_, state, button| {
                button.emit(state, TransportEvent::Stop);
            })
            .bind(AppData::is_playing, |data| ())
            .build(state, controls, |builder| builder);

        entity
            .class(state, "control_bar")
            .set_name(state, "transport controls")
    }
}
