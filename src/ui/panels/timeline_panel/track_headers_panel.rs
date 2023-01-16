use vizia::prelude::*;

use super::track_header_view::{
    BoundTrackHeaderState, BoundTrackHeaderType, TrackHeaderEvent, TrackHeaderView,
    DEFAULT_TRACK_HEADER_HEIGHT,
};
use crate::{
    state_system::{
        source_state::{PaletteColor, TrackType},
        AppAction, SourceState, StateSystem, TrackAction, WorkingState,
    },
    ui::generic_views::virtual_slider::VirtualSliderLens,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SelectedTrack {
    Master,
    Track(usize),
}

#[derive(Debug, Lens, Clone)]
pub struct TrackHeadersPanelLens {
    pub master_track_header: BoundTrackHeaderState,
    pub track_headers: Vec<BoundTrackHeaderState>,

    #[lens(ignore)]
    selected_track: Option<SelectedTrack>,
}

impl TrackHeadersPanelLens {
    pub fn new(state: &SourceState) -> Self {
        if let Some(project_state) = &state.project {
            let master_track_header = BoundTrackHeaderState {
                name: "Master".into(),
                color: project_state.master_track_color,
                height: project_state.master_track_lane_height,
                type_: BoundTrackHeaderType::Master,
                volume: VirtualSliderLens::from_value(
                    project_state.master_track_volume_normalized,
                    1.0,
                ),
                pan: VirtualSliderLens::from_value(project_state.master_track_pan_normalized, 0.5),
                selected: false,
            };

            let track_headers: Vec<BoundTrackHeaderState> = project_state
                .tracks
                .iter()
                .map(|track_state| BoundTrackHeaderState {
                    name: track_state.name.clone(),
                    color: track_state.color,
                    height: track_state.lane_height,
                    type_: match track_state.type_ {
                        TrackType::Audio(_) => BoundTrackHeaderType::Audio,
                        TrackType::Synth => BoundTrackHeaderType::Synth,
                    },
                    volume: VirtualSliderLens::from_value(track_state.volume_normalized, 1.0),
                    pan: VirtualSliderLens::from_value(track_state.pan_normalized, 0.5),
                    selected: false,
                })
                .collect();

            Self { master_track_header, track_headers, selected_track: None }
        } else {
            Self {
                master_track_header: BoundTrackHeaderState {
                    name: "Master".into(),
                    color: PaletteColor::Unassigned,
                    height: DEFAULT_TRACK_HEADER_HEIGHT,
                    type_: BoundTrackHeaderType::Master,
                    volume: VirtualSliderLens::from_value(1.0, 1.0),
                    pan: VirtualSliderLens::from_value(0.5, 0.5),
                    selected: false,
                },
                track_headers: Vec::new(),
                selected_track: None,
            }
        }
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
        Element::new(cx).height(Pixels(28.0)).width(Stretch(1.0)).class("top_spacer");

        ScrollView::new(cx, 0.0, 0.0, false, true, |cx| {
            List::new(
                cx,
                StateSystem::working_state
                    .then(WorkingState::track_headers_panel_lens)
                    .then(TrackHeadersPanelLens::track_headers),
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
                .then(TrackHeadersPanelLens::master_track_header),
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
    .width(Pixels(250.0))
    .height(Stretch(1.0));
}
