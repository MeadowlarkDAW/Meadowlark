use meadowlark_engine::engine::EngineTempoMap;

use crate::state_system::source_state::project_track_state::AudioClipState;
use crate::state_system::source_state::{
    AppState, AudioClipCopyableState, PaletteColor, ProjectState, SnapMode, TimelineTool,
    TrackType, DEFAULT_TIMELINE_ZOOM,
};
use crate::state_system::time::{TempoMap, Timestamp};

pub static MIN_ZOOM: f64 = 0.025; // TODO: Find a good value for this.
pub static MAX_ZOOM: f64 = 8.0; // TODO: Find a good value for this.

pub static POINTS_PER_BEAT: f64 = 100.0;
pub static MARKER_REGION_HEIGHT: f32 = 28.0;
pub static DRAG_ZOOM_SCALAR: f64 = 0.00029;
pub static DRAG_ZOOM_EXP: f64 = 3.75;

pub static CLIP_RESIZE_HANDLE_WIDTH_POINTS: f32 = 3.0;
pub static CLIP_DRAG_THRESHOLD_POINTS: f32 = 5.0;

/// The zoom threshold at which major lines represent measures and minor lines
/// represent bars.
pub static ZOOM_THRESHOLD_BARS: f64 = 0.125;
/// The zoom threshold at which major lines represent bars and minor lines represent
/// beats.
pub static ZOOM_THRESHOLD_BEATS: f64 = 0.5;
/// The zoom threshold at which major lines represent beats and minor lines represent
/// quarter-notes.
pub static ZOOM_THRESHOLD_QUARTER_BEATS: f64 = 2.0;
/// The zoom threshold at which major lines represent beats and minor lines represent
/// eight-notes.
pub static ZOOM_THRESHOLD_EIGTH_BEATS: f64 = 4.0;
/// The zoom threshold at which major lines represent beats and minor lines represent
/// sixteenth-notes.
pub static ZOOM_THRESHOLD_SIXTEENTH_BEATS: f64 = 8.0;

pub struct TimelineViewState {
    pub horizontal_zoom: f64,
    pub horizontal_zoom_normalized: f64,
    pub scroll_beats_x: f64,
    pub scroll_pixels_y: f32,

    pub view_width_pixels: f32,
    pub view_height_pixels: f32,
    pub scale_factor: f64,

    pub lane_states: Vec<TimelineLaneState>,

    pub loop_start_beats_x: f64,
    pub loop_end_beats_x: f64,
    pub loop_active: bool,

    pub playhead_beats_x: f64,
    pub playhead_seek_beats_x: f64,
    pub transport_playing: bool,

    pub selected_tool: TimelineTool,
    pub snap_active: bool,
    pub snap_mode: SnapMode,

    pub track_index_to_lane_index: Vec<usize>,

    pub any_clips_selected: bool,
}

impl TimelineViewState {
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
            match &track_state.type_ {
                TrackType::Audio(audio_track_state) => {
                    let clips: Vec<TimelineViewAudioClipState> = audio_track_state
                        .clips
                        .iter()
                        .enumerate()
                        .map(|(clip_index, clip_state)| {
                            TimelineViewAudioClipState::new(
                                clip_state.clone(),
                                &project_state.tempo_map,
                            )
                        })
                        .collect();

                    self.lane_states.push(TimelineLaneState {
                        track_index,
                        height: track_state.lane_height,
                        color: track_state.color,
                        selected_clip_indexes: Vec::new(),
                        type_: TimelineLaneType::Audio(TimelineAudioLaneState { clips }),
                    });
                }
                TrackType::Synth => {
                    // TODO
                }
            }

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

    pub fn insert_audio_clip(
        &mut self,
        track_index: usize,
        clip_state: AudioClipState,
        tempo_map: &TempoMap,
    ) {
        if let Some(lane_i) = self.track_index_to_lane_index.get(track_index) {
            let lane_state = self.lane_states.get_mut(*lane_i).unwrap();

            if let TimelineLaneType::Audio(audio_lane_state) = &mut lane_state.type_ {
                audio_lane_state.clips.push(TimelineViewAudioClipState::new(clip_state, tempo_map));
            }
        }
    }

    pub fn remove_audio_clip(&mut self, track_index: usize, clip_index: usize) {
        if let Some(lane_i) = self.track_index_to_lane_index.get(track_index) {
            let lane_state = self.lane_states.get_mut(*lane_i).unwrap();

            if let TimelineLaneType::Audio(audio_lane_state) = &mut lane_state.type_ {
                if clip_index < audio_lane_state.clips.len() {
                    audio_lane_state.clips.remove(clip_index);

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
    }

    pub fn sync_audio_clip_copyable_state(
        &mut self,
        track_index: usize,
        clip_index: usize,
        new_state: &AudioClipCopyableState,
        tempo_map: &TempoMap,
    ) {
        if let Some(lane_i) = self.track_index_to_lane_index.get(track_index) {
            let lane_state = self.lane_states.get_mut(*lane_i).unwrap();

            if let TimelineLaneType::Audio(audio_lane_state) = &mut lane_state.type_ {
                audio_lane_state.clips[clip_index]
                    .sync_with_new_copyable_state(new_state, tempo_map);
            }
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

            match &mut lane_state.type_ {
                TimelineLaneType::Audio(audio_lane_state) => {
                    if let Some(clip_state) = audio_lane_state.clips.get_mut(clip_index) {
                        clip_state.selected = true;

                        lane_state.selected_clip_indexes.push(clip_index);

                        self.any_clips_selected = true;
                    }
                }
            }
        }
    }

    pub fn deselect_all_clips(&mut self) {
        for lane_state in self.lane_states.iter_mut() {
            match &mut lane_state.type_ {
                TimelineLaneType::Audio(audio_lane_state) => {
                    for clip_i in lane_state.selected_clip_indexes.iter() {
                        audio_lane_state.clips[*clip_i].selected = false;
                    }
                }
            }
        }

        self.any_clips_selected = false;
    }
}

pub struct TimelineLaneState {
    pub track_index: usize,
    pub height: f32,
    pub color: PaletteColor,

    pub selected_clip_indexes: Vec<usize>,

    pub type_: TimelineLaneType,
}

pub enum TimelineLaneType {
    Audio(TimelineAudioLaneState),
}

pub struct TimelineAudioLaneState {
    // TODO: Store clips in a format that can more efficiently check if a clip is
    // visible within a range?
    pub clips: Vec<TimelineViewAudioClipState>,
}

pub struct TimelineViewAudioClipState {
    pub clip_state: AudioClipState,

    /// The x position of the start of the clip.
    pub timeline_start_beats_x: f64,
    /// The x position of the end of the clip.
    pub timeline_end_beats_x: f64,

    pub selected: bool,
}

impl TimelineViewAudioClipState {
    pub fn new(clip_state: AudioClipState, tempo_map: &TempoMap) -> Self {
        let (timeline_start_beats_x, timeline_end_beats_x) =
            match clip_state.copyable.timeline_start {
                Timestamp::Musical(start_time) => (
                    start_time.as_beats_f64(),
                    tempo_map
                        .seconds_to_musical(
                            tempo_map.musical_to_seconds(start_time)
                                + clip_state.copyable.clip_length.to_seconds_f64(),
                        )
                        .as_beats_f64(),
                ),
                Timestamp::Superclock(start_time) => {
                    // TODO
                    (0.0, 0.0)
                }
            };

        Self { clip_state, timeline_start_beats_x, timeline_end_beats_x, selected: false }
    }

    pub fn sync_with_new_copyable_state(
        &mut self,
        new_state: &AudioClipCopyableState,
        tempo_map: &TempoMap,
    ) {
        let (timeline_start_beats_x, timeline_end_beats_x) = match new_state.timeline_start {
            Timestamp::Musical(start_time) => (
                start_time.as_beats_f64(),
                tempo_map
                    .seconds_to_musical(
                        tempo_map.musical_to_seconds(start_time)
                            + new_state.clip_length.to_seconds_f64(),
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

        self.clip_state.copyable = *new_state;
    }
}

pub fn zoom_normal_to_value(zoom_normal: f64) -> f64 {
    if zoom_normal >= 1.0 {
        MAX_ZOOM
    } else if zoom_normal <= 0.0 {
        MIN_ZOOM
    } else {
        (zoom_normal.powf(DRAG_ZOOM_EXP) * (MAX_ZOOM - MIN_ZOOM)) + MIN_ZOOM
    }
}

pub fn zoom_value_to_normal(zoom: f64) -> f64 {
    if zoom >= MAX_ZOOM {
        1.0
    } else if zoom <= MIN_ZOOM {
        0.0
    } else {
        ((zoom - MIN_ZOOM) / (MAX_ZOOM - MIN_ZOOM)).powf(1.0 / DRAG_ZOOM_EXP)
    }
}
