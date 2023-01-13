use crate::state_system::source_state::PaletteColor;

use super::state::{TimelineViewClipState, TimelineViewWorkingState};
use super::POINTS_PER_BEAT;

pub(super) struct TimelineViewCuller {
    pub visible_lanes: Vec<VisibleLaneState>,

    pub scale_factor: f64,
    pub view_width_pixels: f32,
    pub view_height_pixels: f32,

    pub pixels_per_beat: f64,
    pub beats_per_pixel: f64,
    pub view_end_beats_x: f64,
    pub marker_width_buffer: f64,

    pub loop_start_pixels_x: Option<f32>,
    pub loop_end_pixels_x: Option<f32>,

    pub playhead_seek_pixels_x: Option<f32>,
    pub playhead_pixels_x: Option<f32>,

    resize_handle_half_width_pixels: f32,
}

impl TimelineViewCuller {
    pub fn new() -> Self {
        Self {
            visible_lanes: Vec::new(),
            scale_factor: 1.0,
            view_width_pixels: 0.0,
            view_height_pixels: 0.0,
            pixels_per_beat: 1.0,
            beats_per_pixel: 1.0,
            view_end_beats_x: 0.0,
            marker_width_buffer: 0.0,
            loop_start_pixels_x: None,
            loop_end_pixels_x: None,
            playhead_seek_pixels_x: None,
            playhead_pixels_x: None,
            resize_handle_half_width_pixels: 0.0,
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

        self.pixels_per_beat = shared_state.horizontal_zoom * POINTS_PER_BEAT * self.scale_factor;
        self.beats_per_pixel = self.pixels_per_beat.recip();
        self.view_end_beats_x = shared_state.scroll_beats_x
            + (f64::from(self.view_width_pixels) * self.beats_per_pixel);

        // Leave a bit of buffer room for the markers/playhead so they will rendered even
        // if their centers lie outside of the view.
        self.marker_width_buffer = 40.0 * self.scale_factor * self.beats_per_pixel;

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
                shared_state.scroll_beats_x,
                self.view_end_beats_x,
                self.pixels_per_beat,
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
                    shared_state.scroll_beats_x,
                    self.view_end_beats_x,
                    self.pixels_per_beat,
                );

                break;
            }
        }
    }

    pub fn cull_markers(&mut self, shared_state: &TimelineViewWorkingState) {
        self.loop_start_pixels_x = if shared_state.loop_start_beats_x
            >= shared_state.scroll_beats_x - self.marker_width_buffer
            && shared_state.loop_start_beats_x <= self.view_end_beats_x + self.marker_width_buffer
        {
            Some(
                ((shared_state.loop_start_beats_x - shared_state.scroll_beats_x)
                    * self.pixels_per_beat) as f32,
            )
        } else {
            None
        };
        self.loop_end_pixels_x = if shared_state.loop_end_beats_x
            >= shared_state.scroll_beats_x - self.marker_width_buffer
            && shared_state.loop_end_beats_x <= self.view_end_beats_x + self.marker_width_buffer
        {
            Some(
                ((shared_state.loop_end_beats_x - shared_state.scroll_beats_x)
                    * self.pixels_per_beat) as f32,
            )
        } else {
            None
        };
    }

    pub fn cull_playhead(&mut self, shared_state: &TimelineViewWorkingState) {
        self.playhead_seek_pixels_x = if shared_state.playhead_seek_beats_x
            >= shared_state.scroll_beats_x - self.marker_width_buffer
            && shared_state.playhead_seek_beats_x
                <= self.view_end_beats_x + self.marker_width_buffer
        {
            Some(
                ((shared_state.playhead_seek_beats_x - shared_state.scroll_beats_x)
                    * self.pixels_per_beat) as f32,
            )
        } else {
            None
        };

        self.playhead_pixels_x = if !shared_state.transport_playing {
            None
        } else if shared_state.playhead_beats_x
            >= shared_state.scroll_beats_x - self.marker_width_buffer
            && shared_state.playhead_beats_x <= self.view_end_beats_x + self.marker_width_buffer
        {
            Some(
                ((shared_state.playhead_beats_x - shared_state.scroll_beats_x)
                    * self.pixels_per_beat) as f32,
            )
        } else {
            None
        };
    }

    pub fn mouse_is_over_clip(
        &self,
        cursor_x: f32,
        cursor_y: f32,
        clip_top_height_pixels: f32,
        clip_threshold_height_pixels: f32,
        clip_resize_handle_width_pixels: f32,
    ) -> Option<MouseOverClipRes> {
        for visible_lane in self.visible_lanes.iter() {
            if cursor_y >= visible_lane.view_start_pixels_y
                && cursor_y < visible_lane.view_end_pixels_y
            {
                for visible_clip in visible_lane.visible_clips.iter() {
                    if cursor_x >= visible_clip.view_start_pixels_x
                        && cursor_x < visible_clip.view_end_pixels_x
                    {
                        let is_in_top_part = if visible_lane.view_end_pixels_y
                            - visible_lane.view_start_pixels_y
                            <= clip_threshold_height_pixels
                        {
                            // The lane is collapsed to the compact view.
                            true
                        } else {
                            cursor_y <= visible_lane.view_start_pixels_y + clip_top_height_pixels
                        };

                        let region = if is_in_top_part {
                            if cursor_x
                                <= visible_clip.view_start_pixels_x
                                    + clip_resize_handle_width_pixels
                            {
                                ClipRegion::ResizeLeft
                            } else if cursor_x
                                >= visible_clip.view_end_pixels_x - clip_resize_handle_width_pixels
                            {
                                ClipRegion::ResizeRight
                            } else {
                                ClipRegion::TopPart
                            }
                        } else {
                            ClipRegion::BottomPart
                        };

                        return Some(MouseOverClipRes {
                            lane_index: visible_lane.lane_index,
                            track_index: visible_lane.track_index,
                            clip_index: visible_clip.clip_index,
                            selected: visible_clip.selected,
                            region,
                        });
                    }
                }
            }
        }

        None
    }
}

pub(super) struct MouseOverClipRes {
    pub lane_index: usize,
    pub track_index: usize,
    pub clip_index: usize,

    pub selected: bool,

    pub region: ClipRegion,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum ClipRegion {
    TopPart,
    BottomPart,
    ResizeLeft,
    ResizeRight,
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
        view_start_beats_x: f64,
        view_end_beats_x: f64,
        pixels_per_beat: f64,
    ) {
        self.visible_clips.clear();

        // TODO: Use something more efficient than a linear search?
        for (clip_index, clip_state) in clips.iter().enumerate() {
            if view_end_beats_x >= clip_state.timeline_start_beats_x
                && clip_state.timeline_end_beats_x >= view_start_beats_x
            {
                let clip_view_start_pixels_x = ((clip_state.timeline_start_beats_x
                    - view_start_beats_x)
                    * pixels_per_beat) as f32;
                let clip_view_end_pixels_x = ((clip_state.timeline_end_beats_x
                    - view_start_beats_x)
                    * pixels_per_beat) as f32;

                self.visible_clips.push(VisibleClipState {
                    clip_index,
                    view_start_pixels_x: clip_view_start_pixels_x,
                    view_end_pixels_x: clip_view_end_pixels_x,
                    color: self.color, // TODO: Support clips that are a different color from the track color.
                    selected: clip_state.selected,
                });

                // Sort clips by their starting position, so that when two clips overlap, the
                // later one will always be rendered above the earlier one.
                //
                // In addition, clips that are selected should appear above clips that are not
                // selected.
                self.visible_clips.sort_unstable_by(|a, b| {
                    if a.selected == b.selected {
                        a.view_start_pixels_x
                            .partial_cmp(&b.view_start_pixels_x)
                            .unwrap_or(std::cmp::Ordering::Equal)
                    } else if a.selected {
                        std::cmp::Ordering::Greater
                    } else {
                        std::cmp::Ordering::Less
                    }
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
