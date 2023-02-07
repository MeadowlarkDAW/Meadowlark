use vizia::prelude::*;

use crate::{
    state_system::{
        source_state::{PaletteColor, TrackType},
        SourceState,
    },
    ui::generic_views::virtual_slider::VirtualSliderState,
};

pub static DEFAULT_TRACK_HEADER_HEIGHT: f32 = 52.0;
pub static MIN_TRACK_HEADER_HEIGHT: f32 = 30.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SelectedTrack {
    Master,
    Track(usize),
}

#[derive(Debug, Lens, Clone)]
pub struct TrackHeadersPanelState {
    pub master_track_header: TrackHeaderState,
    pub track_headers: Vec<TrackHeaderState>,

    #[lens(ignore)]
    selected_track: Option<SelectedTrack>,
}

impl TrackHeadersPanelState {
    pub fn new(state: &SourceState) -> Self {
        if let Some(project_state) = &state.project {
            let master_track_header = TrackHeaderState {
                name: "Master".into(),
                color: project_state.master_track_color,
                height: project_state.master_track_lane_height,
                type_: TrackHeaderType::Master,
                volume: VirtualSliderState::from_value(
                    project_state.master_track_volume_normalized,
                    1.0,
                ),
                pan: VirtualSliderState::from_value(project_state.master_track_pan_normalized, 0.5),
                selected: false,
            };

            let track_headers: Vec<TrackHeaderState> = project_state
                .tracks
                .iter()
                .map(|track_state| TrackHeaderState {
                    name: track_state.name.clone(),
                    color: track_state.color,
                    height: track_state.lane_height,
                    type_: match track_state.type_ {
                        TrackType::Audio(_) => TrackHeaderType::Audio,
                        TrackType::Synth => TrackHeaderType::Synth,
                    },
                    volume: VirtualSliderState::from_value(track_state.volume_normalized, 1.0),
                    pan: VirtualSliderState::from_value(track_state.pan_normalized, 0.5),
                    selected: false,
                })
                .collect();

            Self { master_track_header, track_headers, selected_track: None }
        } else {
            Self {
                master_track_header: TrackHeaderState {
                    name: "Master".into(),
                    color: PaletteColor::Unassigned,
                    height: DEFAULT_TRACK_HEADER_HEIGHT,
                    type_: TrackHeaderType::Master,
                    volume: VirtualSliderState::from_value(1.0, 1.0),
                    pan: VirtualSliderState::from_value(0.5, 0.5),
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Data)]
pub enum TrackHeaderType {
    Audio,
    Synth,
    Master,
}

#[derive(Debug, Lens, Clone)]
pub struct TrackHeaderState {
    pub name: String,
    pub color: PaletteColor,
    pub height: f32,
    pub type_: TrackHeaderType,
    pub selected: bool,
    pub volume: VirtualSliderState,
    pub pan: VirtualSliderState,
}
