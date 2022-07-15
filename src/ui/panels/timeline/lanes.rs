use crate::ui::{
    state::{LaneState, LaneStates, TimelineGridState},
    UiData, UiEvent, UiState,
};
use vizia::prelude::*;

pub const DEFAULT_LANE_HEIGHT_PX: f32 = 100.0;

pub fn lane_header(cx: &mut Context) {
    List::new(
        cx,
        UiData::state.then(
            UiState::timeline_grid.then(TimelineGridState::lane_states.then(LaneStates::lanes)),
        ),
        move |cx, index, item| {
            // Lane header
            HStack::new(cx, move |cx| {
                // Lane name
                Label::new(
                    cx,
                    item.then(LaneState::name).map(move |x| match x {
                        Some(lane) => (*lane).clone(),
                        None => format!("lane {}", index),
                    }),
                )
                .class("lane_name");

                // Lane bar
                Element::new(cx)
                    .bind(item.then(LaneState::color), move |handle, color| {
                        handle.bind(item.then(LaneState::disabled), move |handle, disabled| {
                            if !disabled.get(handle.cx) {
                                handle.background_color(color.map(|x| match x {
                                    Some(color) => (*color).clone().into(),
                                    None => Color::from("#888888"),
                                }));
                            } else {
                                handle.background_color(Color::from("#444444"));
                            }
                        });
                    })
                    .class("lane_bar");
            })
            .on_press(move |cx| {
                cx.emit(UiEvent::SelectLane(index));
                cx.focus();
            })
            .bind(item.then(LaneState::height), move |handle, height| {
                let factor = match height.get(handle.cx) {
                    Some(height) => height as f32,
                    None => 1.0,
                };
                handle.bind(
                    UiData::state
                        .then(UiState::timeline_grid.then(TimelineGridState::vertical_zoom_level)),
                    move |handle, zoom_y| {
                        let zoom_y = zoom_y.get(handle.cx) as f32;
                        handle.height(Pixels(factor * DEFAULT_LANE_HEIGHT_PX * zoom_y));
                    },
                );
            })
            .class("lane_header")
            .toggle_class("selected", item.then(LaneState::selected))
            .toggle_class("disabled", item.then(LaneState::disabled));
        },
    )
    .class("lane_headers");
}

pub fn lane_content(cx: &mut Context) {
    // TODO: Implement
}
