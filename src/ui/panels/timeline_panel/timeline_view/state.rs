use dropseed::engine::DSTempoMap;

use crate::state_system::actions::ScrollUnits;
use crate::state_system::source_state::project_track_state::{ClipState, ClipType};
use crate::state_system::source_state::{
    PaletteColor, ProjectState, TimelineMode, DEFAULT_TIMELINE_ZOOM,
};
use crate::state_system::time::{TempoMap, Timestamp};

use super::zoom_value_to_normal;

pub struct TimelineViewState {
    pub(super) horizontal_zoom: f64,
    pub(super) horizontal_zoom_normalized: f64,
    pub(super) scroll_units_x: f64,
    pub(super) scroll_pixels_y: f32,
    pub(super) mode: TimelineMode,

    pub(super) view_width_pixels: f32,
    pub(super) view_height_pixels: f32,
    pub(super) scale_factor: f64,

    pub(super) lane_states: Vec<TimelineLaneState>,

    pub(super) loop_start_units_x: f64,
    pub(super) loop_end_units_x: f64,
    pub(super) loop_active: bool,

    pub(super) playhead_units_x: f64,
    pub(super) playhead_seek_units_x: f64,
    pub(super) transport_playing: bool,

    pub(super) track_index_to_lane_index: Vec<usize>,
}

impl TimelineViewState {
    pub fn new() -> Self {
        Self {
            horizontal_zoom: DEFAULT_TIMELINE_ZOOM,
            scroll_units_x: 0.0,
            scroll_pixels_y: 0.0,
            mode: TimelineMode::Musical,
            view_width_pixels: 0.0,
            view_height_pixels: 0.0,
            scale_factor: 1.0,
            lane_states: Vec::new(),
            loop_start_units_x: 0.0,
            loop_end_units_x: 0.0,
            loop_active: false,
            playhead_units_x: 0.0,
            playhead_seek_units_x: 0.0,
            transport_playing: false,
            track_index_to_lane_index: Vec::new(),
            horizontal_zoom_normalized: zoom_value_to_normal(DEFAULT_TIMELINE_ZOOM),
        }
    }

    pub fn sync_from_project_state(&mut self, project_state: &ProjectState) {
        self.lane_states.clear();
        self.track_index_to_lane_index.clear();
        self.mode = project_state.timeline_mode;

        self.navigate(
            project_state.timeline_horizontal_zoom,
            project_state.timeline_scroll_units_x,
        );

        let mut lane_index = 0;
        for track_state in project_state.tracks.iter() {
            let clips: Vec<TimelineViewClipState> = track_state
                .clips
                .iter()
                .map(|(clip_id, clip_state)| {
                    TimelineViewClipState::new(
                        clip_state,
                        &project_state.tempo_map,
                        project_state.timeline_mode,
                        *clip_id,
                    )
                })
                .collect();

            self.lane_states.push(TimelineLaneState {
                height: track_state.lane_height,
                color: track_state.color,
                clips,
            });

            self.track_index_to_lane_index.push(lane_index);

            // TODO: Automation lanes

            lane_index += 1;
        }

        self.set_loop_state(
            project_state.loop_start,
            project_state.loop_end,
            project_state.loop_active,
        );

        self.set_playhead_seek_pos(project_state.playhead_last_seeked);
    }

    pub fn insert_clip(
        &mut self,
        track_index: usize,
        clip_state: &ClipState,
        clip_id: u64,
        tempo_map: &TempoMap,
    ) {
        if let Some(lane_i) = self.track_index_to_lane_index.get(track_index) {
            let lane_state = self.lane_states.get_mut(*lane_i).unwrap();

            let timeline_view_clip_state =
                TimelineViewClipState::new(clip_state, tempo_map, self.mode, clip_id);

            // TODO: Use a more efficient binary search?
            let mut index = 0;
            for (i, clip) in lane_state.clips.iter().enumerate() {
                if clip.timeline_start_x >= timeline_view_clip_state.timeline_start_x {
                    index = i;
                    break;
                }
            }
            lane_state.clips.insert(index, timeline_view_clip_state);
        }
    }

    pub fn remove_clip(&mut self, track_index: usize, clip_id: u64) {
        if let Some(lane_i) = self.track_index_to_lane_index.get(track_index) {
            let lane_state = self.lane_states.get_mut(*lane_i).unwrap();

            let mut found_i = None;
            for (i, clip_state) in lane_state.clips.iter_mut().enumerate() {
                if clip_state.clip_id == clip_id {
                    found_i = Some(i);
                    break;
                }
            }

            if let Some(i) = found_i {
                lane_state.clips.remove(i);
            }
        }
    }

    pub fn update_clip(
        &mut self,
        track_index: usize,
        clip_state: &ClipState,
        clip_id: u64,
        tempo_map: &TempoMap,
    ) {
        if let Some(lane_i) = self.track_index_to_lane_index.get(track_index) {
            let lane_state = self.lane_states.get_mut(*lane_i).unwrap();

            for state in lane_state.clips.iter_mut() {
                if state.clip_id == clip_id {
                    *state = TimelineViewClipState::new(clip_state, tempo_map, self.mode, clip_id);
                    break;
                }
            }
        }
    }

    pub fn navigate(
        &mut self,
        // The horizontal zoom level. 0.25 = default zoom
        horizontal_zoom: f64,
        // The x position of the left side of the timeline view.
        scroll_units_x: ScrollUnits,
    ) {
        self.horizontal_zoom = horizontal_zoom;
        self.horizontal_zoom_normalized = zoom_value_to_normal(horizontal_zoom);

        self.scroll_units_x = match scroll_units_x {
            ScrollUnits::Musical(x) => {
                if self.mode == TimelineMode::Musical {
                    x.max(0.0)
                } else {
                    // TODO
                    0.0
                }
            }
            ScrollUnits::HMS(x) => {
                // TODO
                0.0
            }
        };
    }

    pub fn set_track_height(&mut self, track_index: usize, height: f32) {
        if let Some(lane_i) = self.track_index_to_lane_index.get(track_index) {
            let lane_state = self.lane_states.get_mut(*lane_i).unwrap();

            lane_state.height = height;
        }
    }

    pub fn set_loop_state(
        &mut self,
        loop_start: Timestamp,
        loop_end: Timestamp,
        loop_active: bool,
    ) {
        self.loop_start_units_x = match loop_start {
            Timestamp::Musical(x) => {
                if self.mode == TimelineMode::Musical {
                    x.as_beats_f64().max(0.0)
                } else {
                    // TODO
                    0.0
                }
            }
            Timestamp::Superclock(x) => {
                // TODO
                0.0
            }
        };
        self.loop_end_units_x = match loop_end {
            Timestamp::Musical(x) => {
                if self.mode == TimelineMode::Musical {
                    x.as_beats_f64().max(0.0)
                } else {
                    // TODO
                    0.0
                }
            }
            Timestamp::Superclock(x) => {
                // TODO
                0.0
            }
        };
        self.loop_active = loop_active;
    }

    pub fn set_playhead_seek_pos(&mut self, playhead: Timestamp) {
        self.playhead_seek_units_x = match playhead {
            Timestamp::Musical(x) => {
                if self.mode == TimelineMode::Musical {
                    x.as_beats_f64().max(0.0)
                } else {
                    // TODO
                    0.0
                }
            }
            Timestamp::Superclock(x) => {
                // TODO
                0.0
            }
        };
    }

    pub fn update_playhead_position(&mut self, playhead_frame: u64, tempo_map: &TempoMap) {
        self.playhead_units_x = if self.mode == TimelineMode::Musical {
            tempo_map.frame_to_beat(playhead_frame).to_float()
        } else {
            // TODO
            0.0
        };
    }

    pub fn use_current_playhead_as_seek_pos(&mut self) {
        self.playhead_seek_units_x = self.playhead_units_x;
    }

    pub fn set_transport_playing(&mut self, playing: bool) {
        self.transport_playing = playing;
    }
}

pub(super) struct TimelineLaneState {
    pub height: f32,
    pub color: PaletteColor,

    // TODO: Store clips in a format that can more efficiently check if a clip is
    // visible within a range?
    pub clips: Vec<TimelineViewClipState>,
}

pub(super) enum TimelineViewClipType {
    Audio,
}

pub(super) struct TimelineViewClipState {
    pub type_: TimelineViewClipType,

    pub name: String,

    /// The x position of the start of the clip. When the timeline is in musical
    /// mode, this is in units of beats. When the timeline is in H:M:S mode, this
    /// is in units of seconds.
    pub timeline_start_x: f64,
    /// The x position of the end of the clip. When the timeline is in musical
    /// mode, this is in units of beats. When the timeline is in H:M:S mode, this
    /// is in units of seconds.
    pub timeline_end_x: f64,

    pub clip_id: u64,
}

impl TimelineViewClipState {
    pub fn new(state: &ClipState, tempo_map: &TempoMap, mode: TimelineMode, clip_id: u64) -> Self {
        match &state.type_ {
            ClipType::Audio(audio_clip_state) => {
                let (timeline_start_x, timeline_end_x) = match mode {
                    TimelineMode::Musical => {
                        match state.timeline_start {
                            Timestamp::Musical(start_time) => (
                                start_time.as_beats_f64(),
                                tempo_map
                                    .seconds_to_musical(
                                        tempo_map.musical_to_seconds(start_time)
                                            + audio_clip_state.clip_length.to_seconds_f64(),
                                    )
                                    .as_beats_f64(),
                            ),
                            Timestamp::Superclock(start_time) => {
                                // TODO
                                (0.0, 0.0)
                            }
                        }
                    }
                    TimelineMode::HMS => {
                        // TODO
                        (0.0, 0.0)
                    }
                };

                Self {
                    type_: TimelineViewClipType::Audio,
                    name: state.name.clone(),
                    timeline_start_x,
                    timeline_end_x,
                    clip_id,
                }
            }
        }
    }
}
