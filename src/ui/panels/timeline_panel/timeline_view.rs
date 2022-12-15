//! Note, the `TimelineView` is not an idiomatic Vizia view. It is a single monolithic view
//! with its own custom rendering and input logic. Unlike most of the views in Meadowlark,
//! the `TimelineView` does not make use of lenses or any kind of data-binding. Instead, it
//! uses a system of custom events and reference-counted pointers.
//!
//! Here are my reasons for designing it this way as opposed to the idiomatic way of
//! composing a bunch of smaller views together with lenses:
//!
//! * Clips on the timeline have an unusual layout that won't work very well with Vizia's
//! built-in layout system. Mainly because of horizontal zooming and because clips
//! aren't necessarily bound to a specific lane.
//! * The state of the timeline can be very large, and the use of lenses would be
//! inefficient at detecting exactly which parts of the state have changed. In addition, if
//! we tried composing a bunch of smaller views together, we could end up with thousands of
//! views in a large project.
//! * When a clip is very long and/or the timeline is zoomed in very far, the pixel length
//! of a clip will be very, very long, which wouldn't work well with generic culling and
//! render caching systems. A custom culling and render caching system is needed here.
//! * I want better control of how to handle both the input and rendering logic for
//! overlapping clips.

use dropseed::plugin_api::transport::TempoMap;
use meadowlark_core_types::time::Timestamp;
use std::cell::RefCell;
use std::rc::Rc;
use vizia::resource::FontOrId;
use vizia::{prelude::*, vg::Color};

use crate::state_system::actions::{Action, ScrollUnits, TimelineAction};
use crate::state_system::source_state::project_track_state::{ClipState, ClipType};
use crate::state_system::source_state::{
    PaletteColor, ProjectState, TimelineMode, DEFAULT_TIMELINE_ZOOM,
};

static POINTS_PER_BEAT: f64 = 100.0;
static MARKER_REGION_HEIGHT: f32 = 28.0;
static DRAG_ZOOM_SCALAR: f64 = 0.00029;
static DRAG_ZOOM_EXP: f64 = 3.75;

pub static MIN_ZOOM: f64 = 0.025; // TODO: Find a good value for this.
pub static MAX_ZOOM: f64 = 8.0; // TODO: Find a good value for this.

/// The zoom threshold at which major lines represent measures and minor lines
/// represent bars.
static ZOOM_THRESHOLD_BARS: f64 = 0.125;
/// The zoom threshold at which major lines represent bars and minor lines represent
/// beats.
static ZOOM_THRESHOLD_BEATS: f64 = 0.5;
/// The zoom threshold at which major lines represent beats and minor lines represent
/// quarter-notes.
static ZOOM_THRESHOLD_QUARTER_BEATS: f64 = 2.0;
/// The zoom threshold at which major lines represent beats and minor lines represent
/// eight-notes.
static ZOOM_THRESHOLD_EIGTH_BEATS: f64 = 4.0;
/// The zoom threshold at which major lines represent beats and minor lines represent
/// sixteenth-notes.
static ZOOM_THRESHOLD_SIXTEENTH_BEATS: f64 = 8.0;

pub struct TimelineViewState {
    horizontal_zoom: f64,
    horizontal_zoom_normalized: f64,
    scroll_units_x: f64,
    scroll_pixels_y: f32,
    mode: TimelineMode,

    view_width_points: f32,
    view_height_points: f32,
    scale_factor: f64,

    lane_states: Vec<TimelineLaneState>,

    loop_start_units_x: f64,
    loop_end_units_x: f64,
    loop_active: bool,

    track_index_to_lane_index: Vec<usize>,
}

impl TimelineViewState {
    pub fn new() -> Self {
        Self {
            horizontal_zoom: DEFAULT_TIMELINE_ZOOM,
            scroll_units_x: 0.0,
            scroll_pixels_y: 0.0,
            mode: TimelineMode::Musical,
            view_width_points: 0.0,
            view_height_points: 0.0,
            scale_factor: 1.0,
            lane_states: Vec::new(),
            loop_start_units_x: 0.0,
            loop_end_units_x: 0.0,
            loop_active: false,
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
}

pub enum TimelineViewEvent {
    Navigated,
    TrackHeightSet { index: usize },
    SyncedFromProjectState,
    ClipUpdated { track_index: usize, clip_id: u64 },
    ClipInserted { track_index: usize, clip_id: u64 },
    ClipRemoved { track_index: usize, clip_id: u64 },
    LoopStateUpdated,
}

struct TimelineLaneState {
    height: f32,
    color: PaletteColor,

    // TODO: Store clips in a format that can more efficiently check if a clip is
    // visible within a range?
    clips: Vec<TimelineViewClipState>,
}

enum TimelineViewClipType {
    Audio,
}

struct TimelineViewClipState {
    type_: TimelineViewClipType,

    /// The x position of the start of the clip. When the timeline is in musical
    /// mode, this is in units of beats. When the timeline is in H:M:S mode, this
    /// is in units of seconds.
    timeline_start_x: f64,
    /// The x position of the end of the clip. When the timeline is in musical
    /// mode, this is in units of beats. When the timeline is in H:M:S mode, this
    /// is in units of seconds.
    timeline_end_x: f64,

    clip_id: u64,
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
                                            + audio_clip_state.length.to_seconds_f64(),
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
                    timeline_start_x,
                    timeline_end_x,
                    clip_id,
                }
            }
        }
    }
}

struct VisibleLaneState {
    index: usize,
    view_start_pixels_y: f32,
    view_end_pixels_y: f32,
    color: PaletteColor,

    visible_clips: Vec<VisibleClipState>,
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
        for (index, clip_state) in clips.iter().enumerate() {
            if view_end_units_x >= clip_state.timeline_start_x
                && clip_state.timeline_end_x >= view_start_units_x
            {
                let clip_view_start_pixels_x =
                    ((clip_state.timeline_start_x - view_start_units_x) * pixels_per_unit) as f32;
                let clip_view_end_pixels_x =
                    ((clip_state.timeline_end_x - view_start_units_x) * pixels_per_unit) as f32;

                self.visible_clips.push(VisibleClipState {
                    index,
                    view_start_pixels_x: clip_view_start_pixels_x,
                    view_end_pixels_x: clip_view_end_pixels_x,
                    color: self.color, // TODO: Support clips that are a different color from the track color.
                });
            }
        }
    }
}

struct VisibleClipState {
    index: usize,
    view_start_pixels_x: f32,
    view_end_pixels_x: f32,
    color: PaletteColor,
}

#[derive(Debug, Clone)]
pub struct TimelineViewStyle {
    pub background_color_1: Color,
    pub background_color_2: Color,

    pub major_line_color: Color,
    pub major_line_color_2: Color,
    pub minor_line_color_1: Color,
    pub minor_line_color_2: Color,

    pub major_line_width: f32,
    pub major_line_width_2: f32,
    pub minor_line_width: f32,

    pub line_marker_label_color: Color,
    pub line_marker_bg_color: Color,
    pub line_marker_label_size: f32,

    pub clip_body_alpha: f32,
    pub clip_top_height: f32,
    pub clip_threshold_height: f32,
    pub clip_border_color: Color,
    pub clip_border_width: f32,
    pub clip_border_radius: f32,

    pub loop_marker_width: f32,
    pub loop_marker_active_color: Color,
    pub loop_marker_inactive_color: Color,
    pub loop_marker_flag_size: f32,
}

impl Default for TimelineViewStyle {
    fn default() -> Self {
        Self {
            background_color_1: Color::rgb(0x2a, 0x2b, 0x2a),
            background_color_2: Color::rgb(0x28, 0x28, 0x28),

            major_line_color: Color::rgb(0x1a, 0x1c, 0x1c),
            major_line_color_2: Color::rgb(0x21, 0x21, 0x21),
            minor_line_color_1: Color::rgb(0x1f, 0x1f, 0x1f),
            minor_line_color_2: Color::rgb(0x1e, 0x1f, 0x1e),

            major_line_width: 2.0,
            major_line_width_2: 2.0,
            minor_line_width: 1.0,

            line_marker_label_color: Color::rgb(0x7d, 0x7e, 0x81),
            line_marker_bg_color: Color::rgb(0x22, 0x22, 0x22),
            line_marker_label_size: 12.0,

            clip_body_alpha: 0.15,
            clip_top_height: 14.0,
            clip_threshold_height: 35.0,
            clip_border_color: Color::rgb(0x20, 0x20, 0x20),
            clip_border_width: 1.0,
            clip_border_radius: 2.0,

            loop_marker_width: 1.0,
            loop_marker_active_color: Color::rgb(0x8b, 0x8b, 0x8b),
            loop_marker_inactive_color: Color::rgb(0x44, 0x44, 0x44),
            loop_marker_flag_size: 10.0,
        }
    }
}

struct CustomDrawCache {
    do_full_redraw: bool,
}

impl CustomDrawCache {
    fn new() -> Self {
        Self { do_full_redraw: true }
    }
}

fn zoom_normal_to_value(zoom_normal: f64) -> f64 {
    if zoom_normal >= 1.0 {
        MAX_ZOOM
    } else if zoom_normal <= 0.0 {
        MIN_ZOOM
    } else {
        (zoom_normal.powf(DRAG_ZOOM_EXP) * (MAX_ZOOM - MIN_ZOOM)) + MIN_ZOOM
    }
}

fn zoom_value_to_normal(zoom: f64) -> f64 {
    if zoom >= MAX_ZOOM {
        1.0
    } else if zoom <= MIN_ZOOM {
        0.0
    } else {
        ((zoom - MIN_ZOOM) / (MAX_ZOOM - MIN_ZOOM)).powf(1.0 / DRAG_ZOOM_EXP)
    }
}

pub struct TimelineView {
    /// Only the `StateSystem` struct is allowed to borrow this mutably.
    shared_state: Rc<RefCell<TimelineViewState>>,

    style: TimelineViewStyle,

    is_dragging_marker_region: bool,
    is_dragging_with_middle_click: bool,
    drag_start_scroll_x: f64,
    drag_start_pixel_x_offset: f64,
    drag_start_horizontal_zoom_normalized: f64,

    visible_lanes: Vec<VisibleLaneState>,

    loop_start_marker_x: Option<f32>,
    loop_end_marker_x: Option<f32>,

    view_width_points: f32,
    view_height_points: f32,
    scale_factor: f64,
    custom_draw_cache: RefCell<CustomDrawCache>,
}

impl TimelineView {
    pub fn new<'a>(
        cx: &'a mut Context,
        shared_state: Rc<RefCell<TimelineViewState>>,
        style: TimelineViewStyle,
    ) -> Handle<'a, Self> {
        Self {
            shared_state,
            style,
            is_dragging_marker_region: false,
            is_dragging_with_middle_click: false,
            drag_start_scroll_x: 0.0,
            drag_start_pixel_x_offset: 0.0,
            drag_start_horizontal_zoom_normalized: 0.0,
            visible_lanes: Vec::new(),
            loop_start_marker_x: None,
            loop_end_marker_x: None,
            view_width_points: 0.0,
            view_height_points: 0.0,
            scale_factor: 1.0,
            custom_draw_cache: RefCell::new(CustomDrawCache::new()),
        }
        .build(cx, move |cx| {})
    }

    fn cull_all_lanes(&mut self) {
        let shared_state = self.shared_state.borrow();

        self.visible_lanes.clear();

        let view_start_units_x = shared_state.scroll_units_x;
        let pixels_per_unit = shared_state.horizontal_zoom * POINTS_PER_BEAT * self.scale_factor;
        let view_end_units_x =
            view_start_units_x + (f64::from(self.view_width_points) / pixels_per_unit);

        // TODO: Vertical scrolling
        let scroll_pixels_y = 0.0;
        let scroll_end_pixels_y = scroll_pixels_y + self.view_height_points;
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
                index: lane_index,
                color: lane_state.color,
                view_start_pixels_y: current_lane_pixels_y - scroll_pixels_y,
                view_end_pixels_y: lane_end_pixels_y - scroll_pixels_y,
                visible_clips: Vec::new(),
            };
            visible_lane_state.cull_clips(
                &lane_state.clips,
                view_start_units_x,
                view_end_units_x,
                pixels_per_unit,
            );

            self.visible_lanes.push(visible_lane_state);

            current_lane_pixels_y = lane_end_pixels_y;
        }
    }

    fn cull_lane(&mut self, lane_index: usize) {
        let shared_state = self.shared_state.borrow();

        let view_start_units_x = shared_state.scroll_units_x;
        let pixels_per_unit = shared_state.horizontal_zoom * POINTS_PER_BEAT * self.scale_factor;
        let view_end_units_x =
            view_start_units_x + (f64::from(self.view_width_points) / pixels_per_unit);

        for visible_lane in self.visible_lanes.iter_mut() {
            if visible_lane.index == lane_index {
                let clips = &shared_state.lane_states[lane_index].clips;

                visible_lane.cull_clips(
                    clips,
                    view_start_units_x,
                    view_end_units_x,
                    pixels_per_unit,
                );

                break;
            }
        }
    }

    fn cull_markers(&mut self) {
        let shared_state = self.shared_state.borrow();

        let pixels_per_unit = shared_state.horizontal_zoom * POINTS_PER_BEAT * self.scale_factor;

        // Leave a bit of buffer room so the markers will still be rendered even if their centers
        // lie outside of the bounds.
        let width_buffer =
            f64::from(self.style.loop_marker_flag_size) * self.scale_factor * 2.0 / pixels_per_unit;

        let view_start_units_x = shared_state.scroll_units_x;
        let view_end_units_x =
            view_start_units_x + (f64::from(self.view_width_points) / pixels_per_unit);

        self.loop_start_marker_x = if shared_state.loop_start_units_x
            >= view_start_units_x - width_buffer
            && shared_state.loop_start_units_x <= view_end_units_x + width_buffer
        {
            Some(((shared_state.loop_start_units_x - view_start_units_x) * pixels_per_unit) as f32)
        } else {
            None
        };
        self.loop_end_marker_x = if shared_state.loop_end_units_x
            >= view_start_units_x - width_buffer
            && shared_state.loop_end_units_x <= view_end_units_x + width_buffer
        {
            Some(((shared_state.loop_end_units_x - view_start_units_x) * pixels_per_unit) as f32)
        } else {
            None
        };
    }
}

impl View for TimelineView {
    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        event.map(|timeline_view_event, _| match timeline_view_event {
            TimelineViewEvent::Navigated => {
                self.cull_all_lanes();
                self.cull_markers();
                cx.needs_redraw();
            }
            TimelineViewEvent::TrackHeightSet { index } => {
                self.cull_all_lanes();
                cx.needs_redraw();
            }
            TimelineViewEvent::SyncedFromProjectState => {
                self.cull_all_lanes();
                self.cull_markers();
                cx.needs_redraw();
            }
            TimelineViewEvent::ClipUpdated { track_index, clip_id } => {
                let lane_index = {
                    self.shared_state
                        .borrow()
                        .track_index_to_lane_index
                        .get(*track_index)
                        .map(|i| *i)
                };
                if let Some(lane_index) = lane_index {
                    self.cull_lane(lane_index);
                }

                // TODO: Don't need to redraw if the clip remained outside of the visible area.
                cx.needs_redraw();
            }
            TimelineViewEvent::ClipInserted { track_index, clip_id } => {
                let lane_index = {
                    self.shared_state
                        .borrow()
                        .track_index_to_lane_index
                        .get(*track_index)
                        .map(|i| *i)
                };
                if let Some(lane_index) = lane_index {
                    self.cull_lane(lane_index);
                }

                // TODO: Don't need to redraw if the clip is outside the visible area.
                cx.needs_redraw();
            }
            TimelineViewEvent::ClipRemoved { track_index, clip_id } => {
                let lane_index = {
                    self.shared_state
                        .borrow()
                        .track_index_to_lane_index
                        .get(*track_index)
                        .map(|i| *i)
                };
                if let Some(lane_index) = lane_index {
                    self.cull_lane(lane_index);
                }

                // TODO: Don't need to redraw if the clip was outside the visible area.
                cx.needs_redraw();
            }
            TimelineViewEvent::LoopStateUpdated => {
                // TODO: Don't need to redraw the whole view.
                cx.needs_redraw();
            }
        });

        event.map(|window_event, meta| match window_event {
            WindowEvent::GeometryChanged(_) => {
                let current = cx.current();
                let width = cx.cache.get_width(current);
                let height = cx.cache.get_height(current);
                let scale_factor = cx.scale_factor() as f64;

                if self.view_width_points != width
                    || self.view_height_points != height && self.scale_factor != scale_factor
                {
                    self.view_width_points = width;
                    self.view_height_points = height;
                    self.scale_factor = scale_factor;

                    self.cull_all_lanes();
                    self.cull_markers();
                    self.custom_draw_cache.borrow_mut().do_full_redraw = true;

                    cx.needs_redraw();
                }
            }
            WindowEvent::MouseDown(button) => {
                let scale_factor = cx.scale_factor();

                if *button == MouseButton::Left {
                    let shared_state = self.shared_state.borrow();
                    let current = cx.current();
                    let bounds = cx.cache.get_bounds(current);

                    if cx.mouse.left.pos_down.1 >= bounds.y
                        && cx.mouse.left.pos_down.1
                            <= bounds.y + (MARKER_REGION_HEIGHT * scale_factor)
                        && bounds.width() != 0.0
                        && !self.is_dragging_with_middle_click
                    {
                        self.is_dragging_marker_region = true;
                        self.drag_start_horizontal_zoom_normalized =
                            zoom_value_to_normal(shared_state.horizontal_zoom);
                        self.drag_start_scroll_x = cursor_x_to_beats(
                            cx.mouse.left.pos_down.0,
                            bounds.x,
                            shared_state.scroll_units_x,
                            shared_state.horizontal_zoom,
                            scale_factor,
                        );
                        self.drag_start_pixel_x_offset =
                            f64::from(cx.mouse.left.pos_down.0 - bounds.x);

                        meta.consume();
                        cx.capture();
                        cx.focus_with_visibility(false);

                        // TODO: Lock the pointer in place once Vizia gets that ability.
                    }
                } else if *button == MouseButton::Middle {
                    let shared_state = self.shared_state.borrow();
                    let current = cx.current();
                    let bounds = cx.cache.get_bounds(current);

                    if bounds.width() != 0.0 && !self.is_dragging_marker_region {
                        self.is_dragging_with_middle_click = true;
                        self.drag_start_horizontal_zoom_normalized =
                            zoom_value_to_normal(shared_state.horizontal_zoom);
                        self.drag_start_scroll_x = cursor_x_to_beats(
                            cx.mouse.middle.pos_down.0,
                            bounds.x,
                            shared_state.scroll_units_x,
                            shared_state.horizontal_zoom,
                            scale_factor,
                        );
                        self.drag_start_pixel_x_offset =
                            f64::from(cx.mouse.middle.pos_down.0 - bounds.x);

                        meta.consume();
                        cx.capture();
                        cx.focus_with_visibility(false);

                        // TODO: Lock the pointer in place once Vizia gets that ability.
                    }
                }
            }
            WindowEvent::MouseUp(button) => {
                if *button == MouseButton::Left {
                    self.is_dragging_marker_region = false;

                    meta.consume();

                    if !self.is_dragging_with_middle_click {
                        cx.release();
                    }
                } else if *button == MouseButton::Middle {
                    self.is_dragging_with_middle_click = false;

                    meta.consume();

                    if !self.is_dragging_marker_region {
                        cx.release();
                    }
                }
            }
            WindowEvent::MouseMove(x, y) => {
                if self.is_dragging_marker_region || self.is_dragging_with_middle_click {
                    let shared_state = self.shared_state.borrow();
                    let scale_factor = f64::from(cx.scale_factor());

                    let (offset_x_pixels, offset_y_pixels) = if self.is_dragging_marker_region {
                        cx.mouse.delta(MouseButton::Left)
                    } else {
                        cx.mouse.delta(MouseButton::Middle)
                    };

                    let delta_zoom_normal =
                        -f64::from(offset_y_pixels) * DRAG_ZOOM_SCALAR / scale_factor;
                    let new_zoom_normal = (self.drag_start_horizontal_zoom_normalized
                        + delta_zoom_normal)
                        .clamp(0.0, 1.0);
                    let horizontal_zoom = zoom_normal_to_value(new_zoom_normal);

                    // Calculate the new scroll position offset for the left side of the view so
                    // that zooming is centered around the point where the mouse button last
                    // pressed down.
                    let zoom_x_offset = self.drag_start_pixel_x_offset
                        / (POINTS_PER_BEAT * horizontal_zoom * scale_factor);

                    if shared_state.mode == TimelineMode::Musical {
                        let pan_offset_x_beats = f64::from(offset_x_pixels)
                            / (POINTS_PER_BEAT * horizontal_zoom * scale_factor);

                        let scroll_units_x =
                            (self.drag_start_scroll_x - pan_offset_x_beats - zoom_x_offset)
                                .max(0.0);

                        cx.emit(Action::Timeline(TimelineAction::Navigate {
                            horizontal_zoom,
                            scroll_units_x: ScrollUnits::Musical(scroll_units_x),
                        }));
                    } else {
                        // TODO
                    };

                    cx.needs_redraw();
                }
            }
            _ => {}
        });
    }

    fn draw(&self, cx: &mut DrawContext, canvas: &mut Canvas) {
        use vizia::vg::{Baseline, Paint, Path};

        // TODO: Make this work at different DPI scale factors.

        static MAJOR_LINE_TOP_PADDING: f32 = 14.0; // TODO: Make this part of the style?
        static LINE_MARKER_LABEL_TOP_OFFSET: f32 = 19.0; // TODO: Make this part of the style?
        static LINE_MARKER_LABEL_LEFT_OFFSET: f32 = 7.0; // TODO: Make this part of the style?

        let bounds = cx.bounds();
        let view_width_points = bounds.width();
        let scale_factor = cx.style.dpi_factor as f32;
        let shared_state = self.shared_state.borrow();
        let mut custom_draw_cache = self.custom_draw_cache.borrow_mut();

        // TODO: Actually cache drawing into a texture.
        let do_full_redraw = {
            let mut res = custom_draw_cache.do_full_redraw;
            custom_draw_cache.do_full_redraw = false;

            // Vizia doesn't always send a `GeometryChanged` event before drawing. (Might be
            // a bug?)
            if self.view_width_points != bounds.width()
                || self.view_height_points != bounds.height()
            {
                res = true;
            }

            res
        };

        // Make sure content doesn't render outside of the view bounds.
        canvas.scissor(bounds.x, bounds.y, bounds.width(), bounds.height());

        let mut bg_path = Path::new();
        bg_path.rect(bounds.x, bounds.y, bounds.width(), bounds.height());
        canvas.fill_path(&mut bg_path, &Paint::color(self.style.background_color_1));

        // -- Draw the line markers on the top ----------------------------------------

        let mut bg_path = Path::new();
        bg_path.rect(
            bounds.x,
            bounds.y,
            bounds.width(),
            (MARKER_REGION_HEIGHT + 3.0) * scale_factor,
        );
        canvas.fill_path(&mut bg_path, &Paint::color(self.style.line_marker_bg_color));

        // -- Draw the vertical gridlines ---------------------------------------------

        let major_line_start_y = bounds.y + (MAJOR_LINE_TOP_PADDING * scale_factor);
        let major_line_height = bounds.height() - (MAJOR_LINE_TOP_PADDING * scale_factor);

        let minor_line_start_y = bounds.y + ((MARKER_REGION_HEIGHT + 3.0) * scale_factor);
        let minor_line_height = bounds.height() - (MARKER_REGION_HEIGHT * scale_factor);

        let major_line_width = self.style.major_line_width * scale_factor;
        let major_line_width_offset = (major_line_width / 2.0).floor();
        let major_line_width_2 = self.style.major_line_width_2 * scale_factor;
        let major_line_width_2_offset = (major_line_width_2 / 2.0).floor();
        let major_line_paint = Paint::color(self.style.major_line_color);
        let major_line_paint_2 = Paint::color(self.style.major_line_color_2);

        let minor_line_width = self.style.minor_line_width * scale_factor;
        let minor_line_width_offset = (minor_line_width / 2.0).floor();
        let minor_line_paint_1 = Paint::color(self.style.minor_line_color_1);
        let minor_line_paint_2 = Paint::color(self.style.minor_line_color_1);

        let mut line_marker_label_paint = Paint::color(self.style.line_marker_label_color);
        let line_marker_font_id = {
            let id =
                cx.resource_manager.fonts.get("inter-bold").expect("Could not get inter-bold font");
            if let FontOrId::Id(id) = id {
                *id
            } else {
                panic!("inter-bold did not have a font ID");
            }
        };
        line_marker_label_paint.set_font(&[line_marker_font_id]);
        line_marker_label_paint.set_font_size(self.style.line_marker_label_size * scale_factor);
        line_marker_label_paint.set_text_baseline(Baseline::Middle);
        let line_marker_label_y =
            (bounds.y + (LINE_MARKER_LABEL_TOP_OFFSET * scale_factor)).round();

        let beat_delta_x = (POINTS_PER_BEAT * shared_state.horizontal_zoom) as f32 * scale_factor;
        let first_beat_x = bounds.x
            - ((shared_state.scroll_units_x.fract()
                * POINTS_PER_BEAT
                * shared_state.horizontal_zoom) as f32
                * scale_factor);
        let first_beat = shared_state.scroll_units_x.floor() as i64;

        enum MajorValueDeltaType {
            WholeUnits(i64),
            Fractional(i64),
        }

        let draw_vertical_gridlines =
            |canvas: &mut Canvas,
             first_major_value: i64,
             first_major_x: f32,
             major_value_delta: MajorValueDeltaType,
             mut major_value_fraction_count: i64,
             major_delta_x: f32,
             num_minor_subdivisions: usize,
             view_end_x: f32,
             start_with_secondary_color: bool,
             mut color_region_count: i64,
             major_values_per_color_region: i64| {
                let minor_delta_x = major_delta_x / num_minor_subdivisions as f32;
                let color_region_width =
                    (major_delta_x * major_values_per_color_region as f32).round();
                let secondary_color_paint = Paint::color(self.style.background_color_2);

                // If starting on a secondary color region, make sure that it is
                // drawn with the right width.
                if start_with_secondary_color && color_region_count > 0 {
                    let first_color_region_start_x = first_major_x.round();
                    let first_color_region_width = (major_delta_x
                        * (major_values_per_color_region - color_region_count) as f32)
                        .round();

                    let mut color_region_path = Path::new();
                    color_region_path.rect(
                        first_color_region_start_x,
                        minor_line_start_y,
                        first_color_region_width,
                        minor_line_height,
                    );

                    canvas.fill_path(&mut color_region_path, &secondary_color_paint);
                }

                let mut current_major_value = first_major_value;
                let mut current_major_x = first_major_x;
                let mut is_secondary_color = start_with_secondary_color;
                while current_major_x <= view_end_x {
                    // Draw the secondary color regions.
                    if color_region_count == 0 && is_secondary_color {
                        let color_region_start_x = current_major_x.round();

                        let mut color_region_path = Path::new();
                        color_region_path.rect(
                            color_region_start_x,
                            minor_line_start_y,
                            color_region_width,
                            minor_line_height,
                        );

                        canvas.fill_path(&mut color_region_path, &secondary_color_paint);
                    }

                    // Draw the minor line subdivisions.
                    for i in 1..num_minor_subdivisions {
                        let line_x = (current_major_x + (minor_delta_x * i as f32)).round();

                        // We draw rectangles instead of lines because those are more
                        // efficient to draw.
                        let mut minor_line_path = Path::new();
                        minor_line_path.rect(
                            line_x - minor_line_width_offset,
                            minor_line_start_y,
                            minor_line_width,
                            minor_line_height,
                        );

                        if is_secondary_color {
                            canvas.fill_path(&mut minor_line_path, &minor_line_paint_2);
                        } else {
                            canvas.fill_path(&mut minor_line_path, &minor_line_paint_1);
                        }
                    }

                    // Round to the nearest pixel so lines are sharp.
                    let line_x = current_major_x.round();

                    let (line_width_offset, line_width, line_paint) = match major_value_delta {
                        MajorValueDeltaType::WholeUnits(_) => {
                            (major_line_width_offset, major_line_width, &major_line_paint)
                        }
                        MajorValueDeltaType::Fractional(_) => {
                            if major_value_fraction_count == 0 {
                                (major_line_width_offset, major_line_width, &major_line_paint)
                            } else {
                                (major_line_width_2_offset, major_line_width_2, &major_line_paint_2)
                            }
                        }
                    };

                    // We draw rectangles instead of lines because those are more
                    // efficient to draw.
                    let mut major_line_path = Path::new();
                    major_line_path.rect(
                        line_x - line_width_offset,
                        major_line_start_y,
                        line_width,
                        major_line_height,
                    );

                    canvas.fill_path(&mut major_line_path, &line_paint);

                    let text = match major_value_delta {
                        MajorValueDeltaType::WholeUnits(_) => format!("{}", current_major_value),
                        MajorValueDeltaType::Fractional(_) => {
                            format!("{}.{}", current_major_value, major_value_fraction_count)
                        }
                    };

                    canvas
                        .fill_text(
                            current_major_x + LINE_MARKER_LABEL_LEFT_OFFSET,
                            line_marker_label_y,
                            text,
                            &line_marker_label_paint,
                        )
                        .unwrap();

                    match major_value_delta {
                        MajorValueDeltaType::WholeUnits(delta) => current_major_value += delta,
                        MajorValueDeltaType::Fractional(num_fractions) => {
                            major_value_fraction_count += 1;
                            if major_value_fraction_count == num_fractions {
                                major_value_fraction_count = 0;
                                current_major_value += 1;
                            }
                        }
                    }

                    current_major_x += major_delta_x;

                    color_region_count += 1;
                    if color_region_count == major_values_per_color_region {
                        color_region_count = 0;
                        is_secondary_color = !is_secondary_color;
                    }
                }
            };

        // TODO: Account for different time signatures.
        // TODO: Account for time signature changes.
        let beats_per_bar: i64 = 4;
        let bars_per_measure: i64 = 4;
        let beats_per_measure: i64 = beats_per_bar * bars_per_measure;

        if shared_state.horizontal_zoom < ZOOM_THRESHOLD_BARS {
            // The zoom threshold at which major lines represent measures and minor lines
            // represent bars.

            let measure_delta_x = beat_delta_x * beats_per_measure as f32;

            let num_bars = first_beat / beats_per_bar;
            let num_measures = first_beat / beats_per_measure;
            let first_measure_beat = num_measures * beats_per_measure;
            let first_measure_beat_x =
                first_beat_x - (((first_beat - first_measure_beat) as f32) * beat_delta_x);

            // Draw one extra to make sure that the text of the last marker is rendered.
            let view_end_x = bounds.x + bounds.width() + measure_delta_x;

            let start_with_secondary_color = (num_measures % 2) == 1;

            draw_vertical_gridlines(
                canvas,
                (num_measures * bars_per_measure) + 1,
                first_measure_beat_x,
                MajorValueDeltaType::WholeUnits(bars_per_measure),
                0,
                measure_delta_x,
                (beats_per_measure / beats_per_bar) as usize,
                view_end_x,
                start_with_secondary_color,
                0,
                1,
            );
        } else if shared_state.horizontal_zoom < ZOOM_THRESHOLD_BEATS {
            // The zoom threshold at which major lines represent bars and minor lines represent
            // beats.

            let bar_delta_x = beat_delta_x * beats_per_bar as f32;

            let num_bars = first_beat / beats_per_bar;
            let first_bar_beat = num_bars * beats_per_bar;
            let first_bar_beat_x =
                first_beat_x - (((first_beat - first_bar_beat) as f32) * beat_delta_x);

            // Draw one extra to make sure that the text of the last marker is rendered.
            let view_end_x = bounds.x + bounds.width() + bar_delta_x;

            let num_measures = first_beat / beats_per_measure;
            let start_with_secondary_color = (num_measures % 2) == 1;
            let color_region_count = num_bars % bars_per_measure;

            draw_vertical_gridlines(
                canvas,
                num_bars + 1,
                first_bar_beat_x,
                MajorValueDeltaType::WholeUnits(1),
                0,
                bar_delta_x,
                beats_per_bar as usize,
                view_end_x,
                start_with_secondary_color,
                color_region_count,
                bars_per_measure,
            );
        } else {
            // The zoom threshold at which major lines represent beats and minor lines represent
            // beat subdivisions.

            let num_subbeat_divisions =
                if shared_state.horizontal_zoom < ZOOM_THRESHOLD_QUARTER_BEATS {
                    4
                } else if shared_state.horizontal_zoom < ZOOM_THRESHOLD_EIGTH_BEATS {
                    8
                } else {
                    16
                }; // TODO: More subdivisions?

            // Draw one extra to make sure that the text of the last marker is rendered.
            let view_end_x = bounds.x + bounds.width() + beat_delta_x;

            let num_bars = first_beat / beats_per_bar;
            let first_bar_beat = num_bars * beats_per_bar;
            let first_bar_beat_x =
                first_beat_x - (((first_beat - first_bar_beat) as f32) * beat_delta_x);
            let bar_fraction_count = first_beat - first_bar_beat;

            let num_measures = first_beat / beats_per_measure;
            let start_with_secondary_color = (num_measures % 2) == 1;
            let color_region_count = first_beat % beats_per_measure;

            draw_vertical_gridlines(
                canvas,
                num_bars + 1,
                first_beat_x,
                MajorValueDeltaType::Fractional(beats_per_bar),
                bar_fraction_count,
                beat_delta_x,
                num_subbeat_divisions,
                view_end_x,
                start_with_secondary_color,
                color_region_count,
                beats_per_measure,
            );
        }

        // -- Draw the loop markers ---------------------------------------------------

        if self.loop_start_marker_x.is_some() || self.loop_end_marker_x.is_some() {
            let loop_marker_width = self.style.loop_marker_width * scale_factor;
            let loop_marker_width_offset = (loop_marker_width / 2.0).floor();

            let loop_marker_color = if shared_state.loop_active {
                self.style.loop_marker_active_color
            } else {
                self.style.loop_marker_inactive_color
            };
            let loop_marker_paint = Paint::color(loop_marker_color);

            let flag_size = self.style.loop_marker_flag_size * scale_factor;

            if let Some(x) = self.loop_start_marker_x {
                let mut line_path = Path::new();
                let line_x = (bounds.x + x - loop_marker_width_offset).round();
                line_path.rect(line_x, bounds.y, loop_marker_width, bounds.height());
                canvas.fill_path(&mut line_path, &loop_marker_paint);

                let mut flag_path = Path::new();
                let flag_x = line_x + loop_marker_width;
                flag_path.move_to(flag_x, bounds.y);
                flag_path.line_to(flag_x, bounds.y + flag_size);
                flag_path.line_to(flag_x + flag_size, bounds.y);
                flag_path.close();
                canvas.fill_path(&mut flag_path, &loop_marker_paint);
            }
            if let Some(x) = self.loop_end_marker_x {
                let mut line_path = Path::new();
                let line_x = (bounds.x + x - loop_marker_width_offset).round();
                line_path.rect(line_x, bounds.y, loop_marker_width, bounds.height());
                canvas.fill_path(&mut line_path, &loop_marker_paint);

                let mut flag_path = Path::new();
                let flag_x = line_x;
                flag_path.move_to(flag_x, bounds.y);
                flag_path.line_to(flag_x, bounds.y + flag_size);
                flag_path.line_to(flag_x - flag_size, bounds.y);
                flag_path.close();
                canvas.fill_path(&mut flag_path, &loop_marker_paint);
            }
        }

        // -- Draw lanes --------------------------------------------------------------

        // Draw the first line above the first track.
        //
        // We draw rectangles instead of lines because those are more
        // efficient to draw.
        let y = (bounds.y + ((MARKER_REGION_HEIGHT + 3.0) * scale_factor)).round()
            - major_line_width_offset;
        let mut first_line_path = Path::new();
        first_line_path.rect(bounds.x, y, view_width_points, major_line_width);
        canvas.fill_path(&mut first_line_path, &major_line_paint);

        let clip_top_height = (self.style.clip_top_height * scale_factor).round();
        let clip_threshold_height = (self.style.clip_threshold_height * scale_factor).round();

        let clip_border_width = self.style.clip_border_width * scale_factor;
        let clip_border_width_offset = clip_border_width / 2.0;

        let mut clip_border_paint = Paint::color(self.style.clip_border_color);
        clip_border_paint.set_line_width(clip_border_width);
        let clip_border_radius = self.style.clip_border_radius * scale_factor;

        let start_y: f32 = bounds.y + ((MARKER_REGION_HEIGHT + 3.0) * scale_factor);
        if !self.visible_lanes.is_empty() {
            let mut current_lane_y: f32 = start_y + self.visible_lanes[0].view_start_pixels_y;

            for visible_lane in self.visible_lanes.iter() {
                let lane_state = &shared_state.lane_states[visible_lane.index];

                let lane_end_y = current_lane_y + (lane_state.height * scale_factor);

                // We draw rectangles instead of lines because those are more
                // efficient to draw.
                let horizontal_line_y = lane_end_y.round() - major_line_width_offset;
                let mut line_path = Path::new();
                line_path.rect(bounds.x, horizontal_line_y, view_width_points, major_line_width);
                canvas.fill_path(&mut line_path, &major_line_paint);

                // Draw clips

                let clip_start_y =
                    (current_lane_y - major_line_width_offset + major_line_width).round();
                let clip_height = (lane_end_y - major_line_width_offset - clip_start_y).round()
                    - clip_border_width;
                let clip_start_y = clip_start_y + clip_border_width_offset;

                for visible_clip in visible_lane.visible_clips.iter() {
                    let x = (bounds.x + visible_clip.view_start_pixels_x).round()
                        + clip_border_width_offset;
                    let end_x = (bounds.x + visible_clip.view_end_pixels_x).round()
                        - clip_border_width_offset;
                    let width = end_x.min(bounds.right()) - x;

                    let clip_top_color: Color = visible_clip.color.into_color().into();

                    if clip_height < clip_threshold_height {
                        let mut top_path = Path::new();

                        if clip_border_radius == 0.0 {
                            top_path.rect(x, clip_start_y, width, clip_height);
                        } else {
                            top_path.rounded_rect(
                                x,
                                clip_start_y,
                                width,
                                clip_height,
                                clip_border_radius,
                            );
                        }
                        canvas.fill_path(&mut top_path, &Paint::color(clip_top_color));
                        canvas.stroke_path(&mut top_path, &clip_border_paint);
                    } else {
                        let clip_body_color = Color::rgbaf(
                            clip_top_color.r,
                            clip_top_color.g,
                            clip_top_color.b,
                            self.style.clip_body_alpha,
                        );

                        let mut body_path = Path::new();
                        if clip_border_radius == 0.0 {
                            body_path.rect(x, clip_start_y, width, clip_height);
                        } else {
                            body_path.rounded_rect(
                                x,
                                clip_start_y,
                                width,
                                clip_height,
                                clip_border_radius,
                            );
                        }
                        canvas.fill_path(&mut body_path, &Paint::color(clip_body_color));

                        let mut top_path = Path::new();
                        if clip_border_radius == 0.0 {
                            top_path.rect(x, clip_start_y, width, clip_top_height);
                        } else {
                            top_path.rounded_rect_varying(
                                x,
                                clip_start_y,
                                width,
                                clip_top_height,
                                clip_border_radius,
                                clip_border_radius,
                                0.0,
                                0.0,
                            );
                        }
                        canvas.fill_path(&mut top_path, &Paint::color(clip_top_color));

                        canvas.stroke_path(&mut body_path, &clip_border_paint);
                    }
                }

                current_lane_y = lane_end_y;
            }
        }

        canvas.reset_scissor();
    }
}

pub enum InternalTimelineViewEvent {}

fn cursor_x_to_beats(
    cursor_x: f32,
    view_x: f32,
    scroll_units_x: f64,
    horizontal_zoom: f64,
    scale_factor: f32,
) -> f64 {
    assert_ne!(horizontal_zoom, 0.0);
    scroll_units_x
        + (f64::from(cursor_x - view_x)
            / (horizontal_zoom * POINTS_PER_BEAT * f64::from(scale_factor)))
}
