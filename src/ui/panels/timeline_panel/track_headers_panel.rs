use vizia::prelude::*;

use super::track_header_view::{TrackHeaderEvent, TrackHeaderView};
use crate::state_system::{
    working_state::track_headers_panel_state::TrackHeadersPanelState, AppAction, StateSystem,
    TrackAction, WorkingState,
};

pub fn track_headers_panel(cx: &mut Context) {
    VStack::new(cx, |cx| {
        Element::new(cx).height(Pixels(28.0)).width(Stretch(1.0)).class("top_spacer");

        ScrollView::new(cx, 0.0, 0.0, false, true, |cx| {
            List::new(
                cx,
                StateSystem::working_state
                    .then(WorkingState::track_headers_panel_lens)
                    .then(TrackHeadersPanelState::track_headers),
                |cx, index, entry| {
                    TrackHeaderView::new(cx, entry, false, move |cx, event| match event {
                        TrackHeaderEvent::Selected => {
                            cx.emit(AppAction::Track(TrackAction::SelectTrack { index }));
                        }
                        TrackHeaderEvent::Resized(height) => {
                            cx.emit(AppAction::Track(TrackAction::SetTrackHeight {
                                index,
                                height,
                            }));
                        }
                        TrackHeaderEvent::SetVolumeNormalized(volume_normalized) => {
                            cx.emit(AppAction::Track(TrackAction::SetTrackVolumeNormalized {
                                index,
                                volume_normalized,
                            }));
                        }
                        TrackHeaderEvent::SetPanNormalized(pan_normalized) => {
                            cx.emit(AppAction::Track(TrackAction::SetTrackPanNormalized {
                                index,
                                pan_normalized,
                            }));
                        }
                    });
                },
            )
            .top(Pixels(2.0))
            .width(Stretch(1.0))
            .height(Auto)
            .row_between(Pixels(2.0));
        })
        .class("hidden_scrollbar")
        .height(Stretch(1.0));

        // Draw a separator between the tracks and the master track.
        Element::new(cx).width(Stretch(1.0)).height(Pixels(3.0)).class("spacer");

        TrackHeaderView::new(
            cx,
            StateSystem::working_state
                .then(WorkingState::track_headers_panel_lens)
                .then(TrackHeadersPanelState::master_track_header),
            true,
            move |cx, event| match event {
                TrackHeaderEvent::Selected => {
                    cx.emit(AppAction::Track(TrackAction::SelectMasterTrack));
                }
                TrackHeaderEvent::Resized(height) => {
                    cx.emit(AppAction::Track(TrackAction::SetMasterTrackHeight { height }));
                }
                TrackHeaderEvent::SetVolumeNormalized(volume_normalized) => {
                    cx.emit(AppAction::Track(TrackAction::SetMasterTrackVolumeNormalized(
                        volume_normalized,
                    )));
                }
                TrackHeaderEvent::SetPanNormalized(pan_normalized) => {
                    cx.emit(AppAction::Track(TrackAction::SetMasterTrackPanNormalized(
                        pan_normalized,
                    )));
                }
            },
        );
    })
    .class("track_headers_panel")
    .width(Pixels(270.0))
    .height(Stretch(1.0));
}
