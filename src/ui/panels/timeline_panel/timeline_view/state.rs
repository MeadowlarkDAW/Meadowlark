use meadowlark_engine::engine::EngineTempoMap;
use std::cell::RefCell;
use std::rc::Rc;

use crate::state_system::source_state::project_track_state::{ClipState, ClipType};
use crate::state_system::source_state::{
    AppState, PaletteColor, ProjectState, SnapMode, TimelineTool, DEFAULT_TIMELINE_ZOOM,
};
use crate::state_system::time::{TempoMap, Timestamp};

use super::zoom_value_to_normal;

//#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
//pub(super) struct ClipID(pub u64);

pub struct TimelineViewWorkingState {
    pub(super) horizontal_zoom: f64,
    pub(super) horizontal_zoom_normalized: f64,
    pub(super) scroll_beats_x: f64,
    pub(super) scroll_pixels_y: f32,

    pub(super) view_width_pixels: f32,
    pub(super) view_height_pixels: f32,
    pub(super) scale_factor: f64,

    pub(super) lane_states: Vec<TimelineLaneState>,

    pub(super) loop_start_beats_x: f64,
    pub(super) loop_end_beats_x: f64,
    pub loop_active: bool,

    pub(super) playhead_beats_x: f64,
    pub(super) playhead_seek_beats_x: f64,
    pub transport_playing: bool,

    pub selected_tool: TimelineTool,
    pub snap_active: bool,
    pub snap_mode: SnapMode,

    pub(super) track_index_to_lane_index: Vec<usize>,

    pub(super) any_clips_selected: bool,
}

impl TimelineViewWorkingState {
    pub fn new() -> Self {
        Self {
            horizontal_zoom: DEFAULT_TIMELINE_ZOOM,
            scroll_beats_x: 0.0,
            scroll_pixels_y: 0.0,
            view_width_pixels: 0.0,
            view_height_pixels: 0.0,
            scale_factor: 1.0,
            lane_states: Vec::new(),
            loop_start_beats_x: 0.0,
            loop_end_beats_x: 0.0,
            loop_active: false,
            playhead_beats_x: 0.0,
            playhead_seek_beats_x: 0.0,
            transport_playing: false,
            track_index_to_lane_index: Vec::new(),
            horizontal_zoom_normalized: zoom_value_to_normal(DEFAULT_TIMELINE_ZOOM),
            selected_tool: TimelineTool::Pointer,
            snap_active: true,
            snap_mode: SnapMode::Line,
            any_clips_selected: false,
        }
    }

    pub fn sync_from_project_state(&mut self, app_state: &AppState, project_state: &ProjectState) {
        self.lane_states.clear();
        self.track_index_to_lane_index.clear();
        self.selected_tool = app_state.selected_timeline_tool;
        self.snap_active = app_state.timeline_snap_active;
        self.snap_mode = app_state.timeline_snap_mode;

        self.navigate(
            project_state.timeline_horizontal_zoom,
            project_state.timeline_scroll_beats_x,
        );

        let mut lane_index = 0;
        for (track_index, track_state) in project_state.tracks.iter().enumerate() {
            let clips: Vec<TimelineViewClipState> = track_state
                .clips
                .iter()
                .enumerate()
                .map(|(clip_index, clip_state)| {
                    TimelineViewClipState::new(Rc::clone(clip_state), &project_state.tempo_map)
                })
                .collect();

            self.lane_states.push(TimelineLaneState {
                track_index,
                height: track_state.lane_height,
                color: track_state.color,
                clips,
                selected_clip_indexes: Vec::new(),
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
        clip_state: Rc<RefCell<ClipState>>,
        tempo_map: &TempoMap,
    ) {
        if let Some(lane_i) = self.track_index_to_lane_index.get(track_index) {
            let lane_state = self.lane_states.get_mut(*lane_i).unwrap();

            lane_state.clips.push(TimelineViewClipState::new(clip_state, tempo_map));
        }
    }

    pub fn remove_clip(&mut self, track_index: usize, clip_index: usize) {
        if let Some(lane_i) = self.track_index_to_lane_index.get(track_index) {
            let lane_state = self.lane_states.get_mut(*lane_i).unwrap();

            if clip_index < lane_state.clips.len() {
                lane_state.clips.remove(clip_index);

                let mut selected_i = None;
                for (i, clip_i) in lane_state.selected_clip_indexes.iter_mut().enumerate() {
                    if *clip_i == clip_index {
                        selected_i = Some(i);
                    } else if *clip_i > clip_index {
                        *clip_i -= 1;
                    }
                }
                if let Some(i) = selected_i {
                    lane_state.selected_clip_indexes.remove(i);
                }
            }
        }
    }

    pub fn sync_clip(&mut self, track_index: usize, clip_index: usize, tempo_map: &TempoMap) {
        if let Some(lane_i) = self.track_index_to_lane_index.get(track_index) {
            let lane_state = self.lane_states.get_mut(*lane_i).unwrap();

            lane_state.clips[clip_index].sync_with_source_state(tempo_map);
        }
    }

    pub fn navigate(
        &mut self,
        // The horizontal zoom level. 0.25 = default zoom
        horizontal_zoom: f64,
        // The x position of the left side of the timeline view.
        scroll_beats_x: f64,
    ) {
        self.horizontal_zoom = horizontal_zoom;
        self.horizontal_zoom_normalized = zoom_value_to_normal(horizontal_zoom);

        self.scroll_beats_x = scroll_beats_x.max(0.0);
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
        self.loop_start_beats_x = match loop_start {
            Timestamp::Musical(x) => x.as_beats_f64().max(0.0),
            Timestamp::Superclock(x) => {
                // TODO
                0.0
            }
        };
        self.loop_end_beats_x = match loop_end {
            Timestamp::Musical(x) => x.as_beats_f64().max(0.0),
            Timestamp::Superclock(x) => {
                // TODO
                0.0
            }
        };
        self.loop_active = loop_active;
    }

    pub fn set_playhead_seek_pos(&mut self, playhead: Timestamp) {
        self.playhead_seek_beats_x = match playhead {
            Timestamp::Musical(x) => x.as_beats_f64().max(0.0),
            Timestamp::Superclock(x) => {
                // TODO
                0.0
            }
        };
    }

    pub fn update_playhead_position(&mut self, playhead_frame: u64, tempo_map: &TempoMap) {
        self.playhead_beats_x = tempo_map.frame_to_beat(playhead_frame).to_float();
    }

    pub fn use_current_playhead_as_seek_pos(&mut self) {
        self.playhead_seek_beats_x = self.playhead_beats_x;
    }

    pub fn select_single_clip(&mut self, track_index: usize, clip_index: usize) {
        self.deselect_all_clips();

        if let Some(lane_i) = self.track_index_to_lane_index.get(track_index) {
            let lane_state = self.lane_states.get_mut(*lane_i).unwrap();

            if let Some(clip_state) = lane_state.clips.get_mut(clip_index) {
                clip_state.selected = true;

                lane_state.selected_clip_indexes.push(clip_index);

                self.any_clips_selected = true;
            } else {
                self.any_clips_selected = false;
            }
        } else {
            self.any_clips_selected = false;
        }
    }

    pub fn deselect_all_clips(&mut self) {
        for lane_state in self.lane_states.iter_mut() {
            for clip_i in lane_state.selected_clip_indexes.iter() {
                lane_state.clips[*clip_i].selected = false;
            }
        }

        self.any_clips_selected = false;
    }
}

pub(super) struct TimelineLaneState {
    pub track_index: usize,
    pub height: f32,
    pub color: PaletteColor,

    // TODO: Store clips in a format that can more efficiently check if a clip is
    // visible within a range?
    pub clips: Vec<TimelineViewClipState>,

    pub selected_clip_indexes: Vec<usize>,
}

pub(super) struct TimelineViewClipState {
    /// Only the `state_system::handle_action()` method is allowed to mutate this.
    pub source_clip_state: Rc<RefCell<ClipState>>,

    /// The x position of the start of the clip.
    pub timeline_start_beats_x: f64,
    /// The x position of the end of the clip.
    pub timeline_end_beats_x: f64,

    pub selected: bool,
}

impl TimelineViewClipState {
    pub fn new(state: Rc<RefCell<ClipState>>, tempo_map: &TempoMap) -> Self {
        let mut new_self = Self {
            source_clip_state: state,
            timeline_start_beats_x: 0.0,
            timeline_end_beats_x: 0.0,
            selected: false,
        };

        new_self.sync_with_source_state(tempo_map);

        new_self
    }

    pub fn sync_with_source_state(&mut self, tempo_map: &TempoMap) {
        let source_state = self.source_clip_state.borrow();

        match &source_state.type_ {
            ClipType::Audio(audio_clip_state) => {
                let (timeline_start_beats_x, timeline_end_beats_x) =
                    match source_state.timeline_start {
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
                    };

                self.timeline_start_beats_x = timeline_start_beats_x;
                self.timeline_end_beats_x = timeline_end_beats_x;
            }
        }
    }
}
