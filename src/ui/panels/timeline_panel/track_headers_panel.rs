use vizia::prelude::*;

use super::track_header_view::{
    BoundTrackHeaderState, BoundTrackHeaderType, TrackHeaderEvent, TrackHeaderView,
};
use crate::state_system::{
    app_state::TrackType, AppAction, AppState, BoundUiState, StateSystem, TrackAction,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SelectedTrack {
    Master,
    Track(usize),
}

#[derive(Debug, Lens, Clone)]
pub struct BoundTrackHeadersPanelState {
    pub master_track_header: BoundTrackHeaderState,
    pub track_headers: Vec<BoundTrackHeaderState>,

    #[lens(ignore)]
    selected_track: Option<SelectedTrack>,
}

impl BoundTrackHeadersPanelState {
    pub fn new(app_state: &AppState) -> Self {
        let master_track_header = BoundTrackHeaderState {
            name: "Master".into(),
            color: app_state.tracks_state.master_track_color,
            height: app_state.tracks_state.master_track_lane_height,
            type_: BoundTrackHeaderType::Master,
            selected: false,
        };

        let track_headers: Vec<BoundTrackHeaderState> = app_state
            .tracks_state
            .tracks
            .iter()
            .map(|track_state| BoundTrackHeaderState {
                name: track_state.name.clone(),
                color: track_state.color,
                height: track_state.lane_height,
                type_: match track_state.type_ {
                    TrackType::Audio => BoundTrackHeaderType::Audio,
                    TrackType::Synth => BoundTrackHeaderType::Synth,
                },
                selected: false,
            })
            .collect();

        Self { master_track_header, track_headers, selected_track: None }
    }

    pub fn select_master_track(&mut self) {
        match self.selected_track {
            Some(SelectedTrack::Master) => {
                // Track is already selected.
                return;
            }
            Some(SelectedTrack::Track(old_index)) => {
                if let Some(track_state) = &mut self.track_headers.get_mut(old_index) {
                    track_state.selected = false;
                }
            }
            _ => {}
        }

        self.master_track_header.selected = true;
        self.selected_track = Some(SelectedTrack::Master);
    }

    pub fn select_track_by_index(&mut self, index: usize) {
        match self.selected_track {
            Some(SelectedTrack::Master) => self.master_track_header.selected = false,
            Some(SelectedTrack::Track(old_index)) => {
                if old_index == index {
                    // Track is already selected.
                    return;
                } else {
                    if let Some(track_state) = &mut self.track_headers.get_mut(old_index) {
                        track_state.selected = false;
                    }
                }
            }
            _ => {}
        }

        if let Some(track_state) = self.track_headers.get_mut(index) {
            track_state.selected = true;
            self.selected_track = Some(SelectedTrack::Track(index));
        } else {
            self.selected_track = None;
        }
    }
}

pub fn track_headers_panel(cx: &mut Context) {
    VStack::new(cx, |cx| {
        Element::new(cx).height(Pixels(26.0)).width(Stretch(1.0)).class("top_spacer");

        ScrollView::new(cx, 0.0, 0.0, false, true, |cx| {
            List::new(
                cx,
                StateSystem::bound_ui_state
                    .then(BoundUiState::track_headers_panel)
                    .then(BoundTrackHeadersPanelState::track_headers),
                |cx, index, entry| {
                    TrackHeaderView::new(cx, entry, false, move |cx, event| match event {
                        TrackHeaderEvent::Selected => {
                            cx.emit(AppAction::Track(TrackAction::SelectTrackByIndex { index }));
                        }
                        TrackHeaderEvent::DragResized(height) => {
                            cx.emit(AppAction::Track(TrackAction::ResizeTrackLaneByIndex {
                                index,
                                height,
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
            StateSystem::bound_ui_state
                .then(BoundUiState::track_headers_panel)
                .then(BoundTrackHeadersPanelState::master_track_header),
            true,
            move |cx, event| match event {
                TrackHeaderEvent::Selected => {
                    cx.emit(AppAction::Track(TrackAction::SelectMasterTrack));
                }
                TrackHeaderEvent::DragResized(height) => {
                    cx.emit(AppAction::Track(TrackAction::ResizeMasterTrackLane { height }));
                }
            },
        );
    })
    .class("track_headers_panel")
    .width(Pixels(250.0))
    .height(Stretch(1.0));
}
