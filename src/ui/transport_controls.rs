use vizia::*;

use crate::state::{
    ui_state::{TimelineTransportUiState, UiState},
    AppEvent, StateSystem,
};

const ICON_STOP: &str = "\u{25a0}";
const ICON_PLAY: &str = "\u{25b6}";
const ICON_PAUSE: &str = "\u{2389}";

pub fn transport_controls(cx: &mut Context) {
    VStack::new(cx, |cx| {
        Label::new(cx, "TRANSPORT").class("control_header");
        HStack::new(cx, |cx| {
            // Playhead Position Label
            Binding::new(
                cx,
                StateSystem::ui_state
                    .then(UiState::timeline_transport)
                    .then(TimelineTransportUiState::playhead),
                |cx, playhead| {
                    let beats = playhead.get(cx).as_beats_f64();
                    Label::new(
                        cx,
                        &format!(
                            "{}.{}.{}",
                            (beats / 4.0) as i32 + 1,
                            (beats as i32 % 4) + 1,
                            ((beats * 4.0) as i32 % 4) + 1
                        ),
                    )
                    .class("transport_playhead")
                    .background_color(Color::rgba(255, 255, 0, 0));
                },
            );

            // Play/Pause Button
            // TODO - Replace with a toggle button when one exists
            Binding::new(
                cx,
                StateSystem::ui_state
                    .then(UiState::timeline_transport)
                    .then(TimelineTransportUiState::is_playing),
                |cx, is_playing| {
                    if *is_playing.get(cx) {
                        Label::new(cx, ICON_PAUSE)
                    } else {
                        Label::new(cx, ICON_PLAY)
                    }
                    .on_press(|cx| cx.emit(AppEvent::Play))
                    .font("icon")
                    .class("play_button");
                },
            );

            // Stop Button
            Button::new(
                cx,
                |cx| cx.emit(AppEvent::Stop),
                |cx| {
                    Label::new(cx, ICON_STOP)
                        .font("icons")
                        .font_size(24.0)
                        .width(Stretch(1.0))
                        .child_space(Stretch(1.0))
                },
            )
            .class("stop_button");
        })
        .class("control_stack");
    })
    .child_space(Pixels(10.0));
}
