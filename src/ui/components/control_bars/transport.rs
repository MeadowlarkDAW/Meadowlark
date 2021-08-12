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
            .build(state, controls, |builder| builder);

        // Play button
        Button::with_label("PLAY")
            .on_press(|_, state, button| {
                button.emit(state, TransportEvent::Play);
            })
            .build(state, controls, |builder| builder);

        // Stop button
        Button::with_label("STOP")
            .on_press(|_, state, button| {
                button.emit(state, TransportEvent::Stop);
            })
            .build(state, controls, |builder| builder);

        entity.class(state, "control_bar")
    }
}
