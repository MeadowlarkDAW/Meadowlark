use rusty_daw_core::MusicalTime;
use vizia::*;

use crate::state::{
    ui_state::{
        TempoMapUiState, TimelineSelectionEvent, TimelineSelectionUiState, TimelineTrackUiState,
        UiState,
    },
    StateSystem,
};

use super::{timeline_view::TimelineViewState, Clip};

pub fn track<D>(cx: &mut Context, track_id: usize, track_data: D)
where
    D: 'static + Lens<Target = TimelineTrackUiState> + Copy,
{
    //let track_height = track_data.get(cx).height;
    // This ZStack isn't strictly necessary but the bindings mess with the list so this is a temporary fix
    ZStack::new(cx, move |cx| {
        Binding::new(
            cx,
            StateSystem::ui_state.then(UiState::tempo_map).then(TempoMapUiState::bpm),
            move |cx, bpm| {
                Binding::new(cx, TimelineViewState::root, move |cx, track_view_state| {
                    let start_beats = track_view_state.get(cx).start_time.as_beats_f64();
                    let end_beats = track_view_state.get(cx).end_time.as_beats_f64();
                    let timeline_beats = end_beats - start_beats;
                    let timeline_width = track_view_state.get(cx).width as f64;
                    HStack::new(cx, move |cx| {
                        let clip_data = track_data.get(cx).audio_clips.clone();

                        if cx.current.child_iter(&cx.tree.clone()).count() != clip_data.len() {
                            for child in cx.current.child_iter(&cx.tree.clone()) {
                                cx.remove(child);
                            }

                            cx.style.needs_relayout = true;
                            cx.style.needs_redraw = true;
                        }

                        for (clip_id, clip) in clip_data.iter().enumerate() {
                            let clip_start = clip.timeline_start.as_beats_f64();
                            let clip_name = clip.name.clone();
                            let duration =
                                clip.duration.to_musical(*bpm.get(cx) as f64).as_beats_f64();
                            let clip_end = clip_start + duration;

                            let clip_start_pos =
                                timeline_width * (clip_start - start_beats) / timeline_beats;
                            let clip_end_pos =
                                timeline_width * (clip_end - start_beats) / timeline_beats;

                            let clip_width = clip_end_pos.floor() - clip_start_pos.floor();

                            let should_display =
                                clip_start >= start_beats || clip_end >= start_beats;

                            Clip::new(
                                cx,
                                track_id,
                                clip_id,
                                clip_name,
                                MusicalTime::from_beats_f64(clip_start),
                                MusicalTime::from_beats_f64(clip_end),
                            )
                            //.display(if should_display { Display::Flex } else { Display::None })
                            .left(Pixels(clip_start_pos.floor() as f32 + 1.0))
                            .width(Pixels(clip_width as f32 - 1.0))
                            .z_order(2)
                            .class("clip");
                        }
                    })
                    .background_color(Color::rgb(68, 60, 60));

                    // Selection - TODO
                    Binding::new(cx, TimelineSelectionUiState::root, move |cx, selection| {
                        let select_start = selection.get(cx).select_start.as_beats_f64();
                        let select_end = selection.get(cx).select_end.as_beats_f64();
                        let track_start = selection.get(cx).track_start;
                        let track_end = selection.get(cx).track_end;
                        let should_display = track_id >= track_start && track_id <= track_end;

                        let select_start_pos =
                            timeline_width * (select_start - start_beats).max(0.0) / timeline_beats;
                        let select_end_pos =
                            timeline_width * (select_end - start_beats).max(0.0) / timeline_beats;

                        let select_width = select_end_pos.floor() - select_start_pos.floor();

                        Element::new(cx)
                            .display(if should_display { Display::Flex } else { Display::None })
                            .background_color(Color::rgba(50, 200, 250, 100))
                            .width(Pixels(select_width as f32 - 1.0))
                            .left(Pixels(select_start_pos.floor() as f32 + 1.0));
                    });
                });
            },
        );
    })
    .height(Pixels(200.0))
    .background_color(Color::rgb(68, 60, 60))
    //.background_color(Color::rgb(200, 60, 60))
    .on_over(move |cx| {
        cx.emit(TimelineSelectionEvent::SetHoveredTrack(track_id));
    });
}
