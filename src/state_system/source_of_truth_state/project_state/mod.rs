use crate::ui::panels::timeline_panel::track_header_view::DEFAULT_TRACK_HEADER_HEIGHT;

pub mod palette;
pub mod project_track_state;

pub use palette::PaletteColor;
pub use project_track_state::{ProjectTrackState, TrackRouteType, TrackType};

pub static DEFAULT_TIMELINE_ZOOM: f64 = 0.25;

/// This struct contains all of the state in a given project which can
/// be considered the "source of truth". All other state is derived from
/// the project state.
///
/// This project state is also what gets turned into a "save file".
///
/// Only the `StateSystem` struct is allowed to mutate this.
pub struct ProjectState {
    pub master_track_color: PaletteColor,
    pub master_track_lane_height: f32,
    pub master_track_volume_normalized: f32,
    pub master_track_pan_normalized: f32,

    pub tracks: Vec<ProjectTrackState>,

    /// The horizontal zoom level. 0.25 = default zoom
    pub timeline_horizontal_zoom: f64,

    /// The x position of the left side of the view. When the timeline is in
    /// musical mode, this is in units of beats. When the timeline is in
    /// H:M:S mode, this is in units of seconds.
    pub timeline_scroll_units_x: f64,

    /// The mode in which the timeline displays its contents.
    pub timeline_mode: TimelineMode,
}

impl ProjectState {
    pub fn test_project() -> Self {
        Self {
            master_track_color: PaletteColor::Unassigned,
            master_track_lane_height: DEFAULT_TRACK_HEADER_HEIGHT,
            master_track_volume_normalized: 1.0,
            master_track_pan_normalized: 0.5,

            tracks: vec![
                ProjectTrackState {
                    name: "Spicy Synth".into(),
                    color: PaletteColor::Color0,
                    lane_height: DEFAULT_TRACK_HEADER_HEIGHT,
                    type_: TrackType::Synth,
                    volume_normalized: 1.0,
                    pan_normalized: 0.5,
                    routed_to: TrackRouteType::ToMaster,
                },
                ProjectTrackState {
                    name: "Drum Hits".into(),
                    color: PaletteColor::Color1,
                    lane_height: DEFAULT_TRACK_HEADER_HEIGHT,
                    type_: TrackType::Audio,
                    volume_normalized: 1.0,
                    pan_normalized: 0.5,
                    routed_to: TrackRouteType::ToMaster,
                },
            ],

            timeline_horizontal_zoom: DEFAULT_TIMELINE_ZOOM,
            timeline_scroll_units_x: 0.0,
            timeline_mode: TimelineMode::Musical,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimelineMode {
    /// In this mode, the timeline displays content in units of measures,
    /// bars, beats, and sub-beats.
    Musical,
    /// In this mode, the timeline displays content in units of hours,
    /// minutes, seconds, and milliseconds.
    HMS,
}
