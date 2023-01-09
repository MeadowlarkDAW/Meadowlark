use crate::state_system::source_state::PaletteColor;

use super::state::{TimelineViewClipState, TimelineViewWorkingState};
use super::POINTS_PER_BEAT;

pub(super) struct TimelineViewCuller {
    pub visible_lanes: Vec<VisibleLaneState>,

    pub scale_factor: f64,
    pub view_width_pixels: f32,
    pub view_height_pixels: f32,

    pub pixels_per_unit: f64,
    pub units_per_pixel: f64,
    pub view_end_units_x: f64,
    pub marker_width_buffer: f64,

    pub loop_start_pixels_x: Option<f32>,
    pub loop_end_pixels_x: Option<f32>,

    pub playhead_seek_pixels_x: Option<f32>,
    pub playhead_pixels_x: Option<f32>,
}

impl TimelineViewCuller {
    pub fn new() -> Self {
        Self {
            visible_lanes: Vec::new(),
            scale_factor: 1.0,
            view_width_pixels: 0.0,
            view_height_pixels: 0.0,
            pixels_per_unit: 1.0,
            units_per_pixel: 1.0,
            view_end_units_x: 0.0,
            marker_width_buffer: 0.0,
            loop_start_pixels_x: None,
            loop_end_pixels_x: None,
            playhead_seek_pixels_x: None,
            playhead_pixels_x: None,
        }
    }

    pub fn update_view_area(
        &mut self,
        view_width_pixels: f32,
        view_height_pixels: f32,
        scale_factor: f64,
        shared_state: &TimelineViewWorkingState,
    ) {
        self.view_width_pixels = view_width_pixels;
        self.view_height_pixels = view_height_pixels;
        self.scale_factor = scale_factor;

        self.pixels_per_unit = shared_state.horizontal_zoom * POINTS_PER_BEAT * self.scale_factor;
        self.units_per_pixel = self.pixels_per_unit.recip();
        self.view_end_units_x = shared_state.scroll_units_x
            + (f64::from(self.view_width_pixels) * self.units_per_pixel);

        // Leave a bit of buffer room for the markers/playhead so they will rendered even
        // if their centers lie outside of the view.
        self.marker_width_buffer = 40.0 * self.scale_factor * self.units_per_pixel;

        self.cull_all_lanes(shared_state);
        self.cull_markers(shared_state);
        self.cull_playhead(shared_state);
    }

    pub fn cull_all_lanes(&mut self, shared_state: &TimelineViewWorkingState) {
        self.visible_lanes.clear();

        // TODO: Vertical scrolling
        let scroll_pixels_y = 0.0;
        let scroll_end_pixels_y = scroll_pixels_y + self.view_height_pixels;
        let mut current_lane_pixels_y = 0.0;

        for (lane_index, lane_state) in shared_state.lane_states.iter().enumerate() {
            let lane_end_pixels_y =
                current_lane_pixels_y + (lane_state.height * self.scale_factor as f32);
            if lane_end_pixels_y < scroll_pixels_y {
                continue;
            } else if current_lane_pixels_y > scroll_end_pixels_y {
                break;
            }

            let mut visible_lane_state = VisibleLaneState {
                lane_index,
                track_index: lane_state.track_index,
                color: lane_state.color,
                view_start_pixels_y: current_lane_pixels_y - scroll_pixels_y,
                view_end_pixels_y: lane_end_pixels_y - scroll_pixels_y,
                visible_clips: Vec::new(),
            };
            visible_lane_state.cull_clips(
                &lane_state.clips,
                shared_state.scroll_units_x,
                self.view_end_units_x,
                self.pixels_per_unit,
            );

            self.visible_lanes.push(visible_lane_state);

            current_lane_pixels_y = lane_end_pixels_y;
        }
    }

    pub fn cull_lane(&mut self, lane_index: usize, shared_state: &TimelineViewWorkingState) {
        for visible_lane in self.visible_lanes.iter_mut() {
            if visible_lane.lane_index == lane_index {
                visible_lane.cull_clips(
                    &shared_state.lane_states[lane_index].clips,
                    shared_state.scroll_units_x,
                    self.view_end_units_x,
                    self.pixels_per_unit,
                );

                break;
            }
        }
    }

    pub fn cull_markers(&mut self, shared_state: &TimelineViewWorkingState) {
        self.loop_start_pixels_x = if shared_state.loop_start_units_x
            >= shared_state.scroll_units_x - self.marker_width_buffer
            && shared_state.loop_start_units_x <= self.view_end_units_x + self.marker_width_buffer
        {
            Some(
                ((shared_state.loop_start_units_x - shared_state.scroll_units_x)
                    * self.pixels_per_unit) as f32,
            )
        } else {
            None
        };
        self.loop_end_pixels_x = if shared_state.loop_end_units_x
            >= shared_state.scroll_units_x - self.marker_width_buffer
            && shared_state.loop_end_units_x <= self.view_end_units_x + self.marker_width_buffer
        {
            Some(
                ((shared_state.loop_end_units_x - shared_state.scroll_units_x)
                    * self.pixels_per_unit) as f32,
            )
        } else {
            None
        };
    }

    pub fn cull_playhead(&mut self, shared_state: &TimelineViewWorkingState) {
        self.playhead_seek_pixels_x = if shared_state.playhead_seek_units_x
            >= shared_state.scroll_units_x - self.marker_width_buffer
            && shared_state.playhead_seek_units_x
                <= self.view_end_units_x + self.marker_width_buffer
        {
            Some(
                ((shared_state.playhead_seek_units_x - shared_state.scroll_units_x)
                    * self.pixels_per_unit) as f32,
            )
        } else {
            None
        };

        self.playhead_pixels_x = if !shared_state.transport_playing {
            None
        } else if shared_state.playhead_units_x
            >= shared_state.scroll_units_x - self.marker_width_buffer
            && shared_state.playhead_units_x <= self.view_end_units_x + self.marker_width_buffer
        {
            Some(
                ((shared_state.playhead_units_x - shared_state.scroll_units_x)
                    * self.pixels_per_unit) as f32,
            )
        } else {
            None
        };
    }
}

pub(super) struct VisibleLaneState {
    pub lane_index: usize,
    pub track_index: usize,
    pub view_start_pixels_y: f32,
    pub view_end_pixels_y: f32,
    pub color: PaletteColor,

    pub visible_clips: Vec<VisibleClipState>,
}

impl VisibleLaneState {
    fn cull_clips(
        &mut self,
        clips: &[TimelineViewClipState],
        view_start_units_x: f64,
        view_end_units_x: f64,
        pixels_per_unit: f64,
    ) {
        self.visible_clips.clear();

        // TODO: Use something more efficient than a linear search?
        for (clip_index, clip_state) in clips.iter().enumerate() {
            if view_end_units_x >= clip_state.timeline_start_x
                && clip_state.timeline_end_x >= view_start_units_x
            {
                let clip_view_start_pixels_x =
                    ((clip_state.timeline_start_x - view_start_units_x) * pixels_per_unit) as f32;
                let clip_view_end_pixels_x =
                    ((clip_state.timeline_end_x - view_start_units_x) * pixels_per_unit) as f32;

                self.visible_clips.push(VisibleClipState {
                    clip_index,
                    view_start_pixels_x: clip_view_start_pixels_x,
                    view_end_pixels_x: clip_view_end_pixels_x,
                    color: self.color, // TODO: Support clips that are a different color from the track color.
                    selected: clip_state.selected,
                });
            }
        }
    }
}

pub(super) struct VisibleClipState {
    pub clip_index: usize,
    pub view_start_pixels_x: f32,
    pub view_end_pixels_x: f32,
    pub color: PaletteColor,
    pub selected: bool,
}
