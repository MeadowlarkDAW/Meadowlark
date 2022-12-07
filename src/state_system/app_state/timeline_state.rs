use crate::ui::panels::timeline_panel::track_header_view::DEFAULT_TRACK_HEADER_HEIGHT;

use super::TracksState;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimelineMode {
    /// In this mode, the timeline displays content in units of measures,
    /// bars, beats, and sub-beats.
    Musical,
    /// In this mode, the timeline displays content in units of hours,
    /// minutes, seconds, and milliseconds.
    HMS,
}

#[derive(Debug, Clone)]
pub struct TimelineState {
    /// The horizontal zoom level. 1.0 = default zoom
    pub horizontal_zoom: f64,

    /// The x position of the left side of the view. When the timeline is in
    /// musical mode, this is in units of beats. When the timeline is in
    /// H:M:S mode, this is in units of seconds.
    pub scroll_units_x: f64,

    /// The mode in which the timeline displays its contents.
    pub mode: TimelineMode,

    pub lane_states: Vec<TimelineLaneState>,

    track_index_to_lane_index: Vec<usize>,
}

impl TimelineState {
    pub fn new(tracks_state: &TracksState) -> Self {
        let mut new_self = Self {
            horizontal_zoom: 1.0,
            scroll_units_x: 0.0,
            mode: TimelineMode::Musical,
            lane_states: Vec::new(),
            track_index_to_lane_index: Vec::new(),
        };

        new_self.sync_lanes_from_track_state(tracks_state);
        new_self
    }

    pub fn sync_lanes_from_track_state(&mut self, tracks_state: &TracksState) {
        self.lane_states.clear();
        self.track_index_to_lane_index.clear();

        let mut lane_index = 0;
        for track in tracks_state.tracks.iter() {
            // TODO: Automation lanes.

            self.lane_states.push(TimelineLaneState { height: track.lane_height });
            self.track_index_to_lane_index.push(lane_index);

            lane_index += 1;
        }
    }

    pub fn set_track_height(&mut self, track_index: usize, height: f32) -> Option<usize> {
        if let Some(lane_i) = self.track_index_to_lane_index.get(track_index) {
            let lane_state = self.lane_states.get_mut(*lane_i).unwrap();

            lane_state.height = height;

            Some(*lane_i)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone)]
pub struct TimelineLaneState {
    pub height: f32,
}

impl TimelineLaneState {
    pub fn new() -> Self {
        Self { height: DEFAULT_TRACK_HEADER_HEIGHT }
    }
}
