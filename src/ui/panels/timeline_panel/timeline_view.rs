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

use std::cell::RefCell;
use std::rc::Rc;
use vizia::prelude::*;

use crate::state_system::actions::{AppAction, TimelineAction};
use crate::state_system::source_state::AudioClipCopyableState;
use crate::state_system::time::{MusicalTime, Timestamp};
use crate::state_system::working_state::timeline_view_state::{
    zoom_normal_to_value, zoom_value_to_normal, TimelineLaneType, TimelineViewState,
    CLIP_DRAG_THRESHOLD_POINTS, CLIP_RESIZE_HANDLE_WIDTH_POINTS, DRAG_ZOOM_SCALAR,
    MARKER_REGION_HEIGHT, POINTS_PER_BEAT,
};

mod culler;
mod renderer;
mod style;

pub use style::TimelineViewStyle;

use culler::{ClipRegion, TimelineViewCuller};
use renderer::{render_timeline_view, RendererCache};

struct DraggingClip {
    lane_index: usize,
    track_index: usize,
    clip_index: usize,

    selected: bool,

    region: ClipRegion,

    drag_start_units_x: f64,

    passed_drag_threshold: bool,
}

pub struct TimelineView {
    /// This is only allowed to be borrowed mutably within the
    /// `state_system::handle_action` method.
    shared_state: Rc<RefCell<TimelineViewState>>,

    style: TimelineViewStyle,

    is_dragging_marker_region: bool,
    is_dragging_with_middle_click: bool,
    drag_start_beats_x: f64,
    drag_start_pixel_x_offset: f64,
    drag_start_horizontal_zoom_normalized: f64,

    dragging_clip: Option<DraggingClip>,

    culler: TimelineViewCuller,

    scale_factor: f64,
    view_width_pixels: f32,
    view_height_pixels: f32,

    clip_top_height_pixels: f32,
    clip_threshold_height_pixels: f32,
    clip_resize_handle_width_pixels: f32,
    clip_drag_threshold_pixels: f32,

    renderer_cache: RefCell<RendererCache>,
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
            drag_start_beats_x: 0.0,
            drag_start_pixel_x_offset: 0.0,
            drag_start_horizontal_zoom_normalized: 0.0,
            dragging_clip: None,
            culler: TimelineViewCuller::new(),
            scale_factor: 1.0,
            view_width_pixels: 0.0,
            view_height_pixels: 0.0,
            clip_top_height_pixels: 0.0,
            clip_threshold_height_pixels: 0.0,
            clip_resize_handle_width_pixels: 0.0,
            clip_drag_threshold_pixels: 0.0,
            renderer_cache: RefCell::new(RendererCache::new()),
        }
        .build(cx, move |cx| {})
    }
}

impl View for TimelineView {
    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        event.map(|timeline_view_event, _| match timeline_view_event {
            TimelineViewEvent::PlayheadMoved => {
                self.culler.cull_playhead(&*self.shared_state.borrow());
                cx.needs_redraw();
            }
            TimelineViewEvent::Navigated => {
                self.culler.update_view_area(
                    self.view_width_pixels,
                    self.view_height_pixels,
                    self.scale_factor,
                    &*self.shared_state.borrow(),
                );
                cx.needs_redraw();
            }
            TimelineViewEvent::TransportStateChanged => {
                self.culler.cull_playhead(&*self.shared_state.borrow());
                cx.needs_redraw();
            }
            TimelineViewEvent::TrackHeightSet { index } => {
                self.culler.cull_all_lanes(&*self.shared_state.borrow());
                cx.needs_redraw();
            }
            TimelineViewEvent::SyncedFromProjectState => {
                self.culler.update_view_area(
                    self.view_width_pixels,
                    self.view_height_pixels,
                    self.scale_factor,
                    &*self.shared_state.borrow(),
                );
                cx.needs_redraw();
            }
            TimelineViewEvent::ClipUpdated { track_index, clip_index } => {
                let lane_index = {
                    self.shared_state
                        .borrow()
                        .track_index_to_lane_index
                        .get(*track_index)
                        .map(|i| *i)
                };
                if let Some(lane_index) = lane_index {
                    self.culler.cull_lane(lane_index, &*self.shared_state.borrow());
                }

                // TODO: Don't need to redraw if the clip remained outside of the visible area.
                cx.needs_redraw();
            }
            TimelineViewEvent::ClipInserted { track_index, clip_index } => {
                let lane_index = {
                    self.shared_state
                        .borrow()
                        .track_index_to_lane_index
                        .get(*track_index)
                        .map(|i| *i)
                };
                if let Some(lane_index) = lane_index {
                    self.culler.cull_lane(lane_index, &*self.shared_state.borrow());
                }

                // TODO: Don't need to redraw if the clip is outside the visible area.
                cx.needs_redraw();
            }
            TimelineViewEvent::ClipRemoved { track_index, clip_index } => {
                let lane_index = {
                    self.shared_state
                        .borrow()
                        .track_index_to_lane_index
                        .get(*track_index)
                        .map(|i| *i)
                };
                if let Some(lane_index) = lane_index {
                    self.culler.cull_lane(lane_index, &*self.shared_state.borrow());
                }

                // TODO: Don't need to redraw if the clip was outside the visible area.
                cx.needs_redraw();
            }
            TimelineViewEvent::ClipSelectionChanged => {
                // TODO: Optimize by only culling the lanes which have clips that were
                // selected/deselected.
                self.culler.cull_all_lanes(&*self.shared_state.borrow());

                cx.needs_redraw();
            }
            TimelineViewEvent::ClipStatesChanged { track_index } => {
                let lane_index = {
                    self.shared_state
                        .borrow()
                        .track_index_to_lane_index
                        .get(*track_index)
                        .map(|i| *i)
                };
                if let Some(lane_index) = lane_index {
                    self.culler.cull_lane(lane_index, &*self.shared_state.borrow());
                }

                cx.needs_redraw();
            }
            TimelineViewEvent::LoopStateUpdated => {
                self.culler.cull_markers(&*self.shared_state.borrow());

                // TODO: Don't need to redraw the whole view.
                cx.needs_redraw();
            }
            TimelineViewEvent::ToolsChanged => {}
        });

        event.map(|window_event, meta| match window_event {
            WindowEvent::GeometryChanged(_) => {
                let current = cx.current();
                let width = cx.cache.get_width(current);
                let height = cx.cache.get_height(current);
                let scale_factor = cx.style.dpi_factor as f64;

                if self.view_width_pixels != width
                    || self.view_height_pixels != height && self.scale_factor != scale_factor
                {
                    self.view_width_pixels = width;
                    self.view_height_pixels = height;
                    self.scale_factor = scale_factor;

                    self.clip_top_height_pixels = self.style.clip_top_height * scale_factor as f32;
                    self.clip_threshold_height_pixels =
                        self.style.clip_threshold_height * scale_factor as f32;
                    self.clip_resize_handle_width_pixels =
                        CLIP_RESIZE_HANDLE_WIDTH_POINTS * scale_factor as f32;
                    self.clip_drag_threshold_pixels =
                        CLIP_DRAG_THRESHOLD_POINTS * scale_factor as f32;

                    self.culler.update_view_area(
                        self.view_width_pixels,
                        self.view_height_pixels,
                        self.scale_factor,
                        &*self.shared_state.borrow(),
                    );
                    self.renderer_cache.borrow_mut().do_full_redraw = true;

                    cx.needs_redraw();
                }
            }
            WindowEvent::MouseDown(button) => {
                let scale_factor = cx.style.dpi_factor as f32;

                if *button == MouseButton::Left {
                    let shared_state = self.shared_state.borrow();
                    let current = cx.current();
                    let bounds = cx.cache.get_bounds(current);

                    let clip_start_y = bounds.y + (MARKER_REGION_HEIGHT * scale_factor);

                    if cx.mouse.left.pos_down.1 >= bounds.y
                        && cx.mouse.left.pos_down.1
                            <= bounds.y + (MARKER_REGION_HEIGHT * scale_factor)
                        && bounds.width() != 0.0
                        && !self.is_dragging_with_middle_click
                    {
                        self.is_dragging_marker_region = true;
                        self.drag_start_horizontal_zoom_normalized =
                            zoom_value_to_normal(shared_state.horizontal_zoom);
                        self.drag_start_beats_x = cursor_x_to_beats(
                            cx.mouse.left.pos_down.0,
                            bounds.x,
                            shared_state.scroll_beats_x,
                            shared_state.horizontal_zoom,
                            scale_factor,
                        );
                        self.drag_start_pixel_x_offset =
                            f64::from(cx.mouse.left.pos_down.0 - bounds.x);

                        meta.consume();
                        cx.capture();
                        cx.focus_with_visibility(false);

                        // TODO: Lock the pointer in place once Vizia gets that ability.
                    } else if let Some(hovered_clip) = self.culler.mouse_is_over_clip(
                        cx.mouse.cursorx - bounds.x,
                        cx.mouse.cursory - clip_start_y,
                        self.clip_top_height_pixels,
                        self.clip_threshold_height_pixels,
                        self.clip_resize_handle_width_pixels,
                    ) {
                        if !hovered_clip.selected {
                            // The user clicked on an unselected clip, so select it.
                            cx.emit(AppAction::Timeline(TimelineAction::SelectSingleClip {
                                track_index: hovered_clip.track_index,
                                clip_index: hovered_clip.clip_index,
                            }));
                        }

                        let drag_start_units_x =
                            match &shared_state.lane_states[hovered_clip.lane_index].type_ {
                                TimelineLaneType::Audio(audio_lane_state) => {
                                    audio_lane_state.clips[hovered_clip.clip_index]
                                        .timeline_start_beats_x
                                }
                            };

                        self.dragging_clip = Some(DraggingClip {
                            lane_index: hovered_clip.lane_index,
                            track_index: hovered_clip.track_index,
                            clip_index: hovered_clip.clip_index,
                            selected: hovered_clip.selected,
                            region: hovered_clip.region,
                            drag_start_units_x,
                            passed_drag_threshold: false,
                        });

                        meta.consume();
                        cx.capture();
                        cx.focus_with_visibility(false);
                    } else {
                        // The user clicked in an area without a clip, so deselect all
                        // selected clips.
                        cx.emit(AppAction::Timeline(TimelineAction::DeselectAllClips));
                    }
                } else if *button == MouseButton::Middle {
                    let shared_state = self.shared_state.borrow();
                    let current = cx.current();
                    let bounds = cx.cache.get_bounds(current);

                    if bounds.width() != 0.0 && !self.is_dragging_marker_region {
                        self.is_dragging_with_middle_click = true;
                        self.drag_start_horizontal_zoom_normalized =
                            zoom_value_to_normal(shared_state.horizontal_zoom);
                        self.drag_start_beats_x = cursor_x_to_beats(
                            cx.mouse.middle.pos_down.0,
                            bounds.x,
                            shared_state.scroll_beats_x,
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

                    if let Some(dragged_clip) = self.dragging_clip.take() {
                        let shared_state = self.shared_state.borrow();

                        match &shared_state.lane_states[dragged_clip.lane_index].type_ {
                            TimelineLaneType::Audio(audio_lane_state) => {
                                let cloned_state: AudioClipCopyableState = audio_lane_state.clips
                                    [dragged_clip.clip_index]
                                    .clip_state
                                    .copyable;

                                cx.emit(AppAction::Timeline(
                                    TimelineAction::SetAudioClipCopyableStates {
                                        track_index: dragged_clip.track_index,
                                        changed_clips: vec![(
                                            dragged_clip.clip_index,
                                            cloned_state,
                                        )],
                                    },
                                ));
                            }
                        }
                    }

                    if !self.is_dragging_with_middle_click {
                        cx.release();
                    }

                    let is_select = {
                        let (dx, dy) = cx.mouse.delta(MouseButton::Left);
                        dx.abs() < self.clip_drag_threshold_pixels
                            && dy.abs() <= self.clip_drag_threshold_pixels
                    };
                    if is_select {
                        let scale_factor = cx.style.dpi_factor as f32;
                        let current = cx.current();
                        let bounds = cx.cache.get_bounds(current);

                        let clip_start_y = bounds.y + (MARKER_REGION_HEIGHT * scale_factor);

                        if cx.mouse.cursory >= bounds.y && cx.mouse.cursory < clip_start_y {
                            // The user selected inside the marker region. Seek the
                            // playhead to that position.

                            // TODO
                        }
                    }
                } else if *button == MouseButton::Middle {
                    self.is_dragging_with_middle_click = false;

                    meta.consume();

                    if !self.is_dragging_marker_region && self.dragging_clip.is_none() {
                        cx.release();
                    }
                }
            }
            WindowEvent::MouseMove(x, y) => {
                if let Some(dragged_clip) = &mut self.dragging_clip {
                    let shared_state = self.shared_state.borrow();

                    let (cursor_delta_x, cursor_delta_y) = cx.mouse.delta(MouseButton::Left);

                    dragged_clip.passed_drag_threshold |=
                        cursor_delta_x.abs() >= self.clip_drag_threshold_pixels;

                    if dragged_clip.passed_drag_threshold {
                        let scale_factor = cx.style.dpi_factor as f32;
                        let current = cx.current();
                        let bounds = cx.cache.get_bounds(current);

                        let offset_x_beats = f64::from(cursor_delta_x)
                            / (POINTS_PER_BEAT
                                * shared_state.horizontal_zoom
                                * f64::from(scale_factor));

                        match dragged_clip.region {
                            ClipRegion::TopPart => {
                                let new_start_beats_x =
                                    dragged_clip.drag_start_units_x + offset_x_beats;
                                let new_timestamp = Timestamp::Musical(
                                    MusicalTime::from_beats_f64(new_start_beats_x),
                                );

                                match &shared_state.lane_states[dragged_clip.lane_index].type_ {
                                    TimelineLaneType::Audio(audio_lane_state) => {
                                        let mut cloned_state: AudioClipCopyableState =
                                            audio_lane_state.clips[dragged_clip.clip_index]
                                                .clip_state
                                                .copyable;

                                        cloned_state.timeline_start = new_timestamp;

                                        cx.emit(AppAction::Timeline(
                                            TimelineAction::GestureAudioClipCopyableStates {
                                                track_index: dragged_clip.track_index,
                                                changed_clips: vec![(
                                                    dragged_clip.clip_index,
                                                    cloned_state,
                                                )],
                                            },
                                        ));
                                    }
                                }
                            }
                            ClipRegion::BottomPart => {}
                            ClipRegion::ResizeLeft => {}
                            ClipRegion::ResizeRight => {}
                        }
                    }
                } else if self.is_dragging_marker_region || self.is_dragging_with_middle_click {
                    let shared_state = self.shared_state.borrow();
                    let scale_factor = cx.style.dpi_factor as f64;

                    let (cursor_delta_x, cursor_delta_y) = if self.is_dragging_marker_region {
                        cx.mouse.delta(MouseButton::Left)
                    } else {
                        cx.mouse.delta(MouseButton::Middle)
                    };

                    let delta_zoom_normal =
                        -f64::from(cursor_delta_y) * DRAG_ZOOM_SCALAR / scale_factor;
                    let new_zoom_normal = (self.drag_start_horizontal_zoom_normalized
                        + delta_zoom_normal)
                        .clamp(0.0, 1.0);
                    let horizontal_zoom = zoom_normal_to_value(new_zoom_normal);

                    // Calculate the new scroll position offset for the left side of the view so
                    // that zooming is centered around the point where the mouse button last
                    // pressed down.
                    let zoom_x_offset = self.drag_start_pixel_x_offset
                        / (POINTS_PER_BEAT * horizontal_zoom * scale_factor);

                    let pan_offset_x_beats = f64::from(cursor_delta_x)
                        / (POINTS_PER_BEAT * horizontal_zoom * scale_factor);

                    let scroll_beats_x =
                        (self.drag_start_beats_x - pan_offset_x_beats - zoom_x_offset).max(0.0);

                    cx.emit(AppAction::Timeline(TimelineAction::Navigate {
                        horizontal_zoom,
                        scroll_beats_x,
                    }));
                }
            }
            _ => {}
        });
    }

    fn draw(&self, cx: &mut DrawContext, canvas: &mut Canvas) {
        let mut cache = self.renderer_cache.borrow_mut();
        let state = self.shared_state.borrow();

        render_timeline_view(cx, canvas, &mut *cache, &*state, &self.culler, &self.style);
    }
}

pub enum TimelineViewEvent {
    PlayheadMoved,
    Navigated,
    TransportStateChanged,
    TrackHeightSet { index: usize },
    SyncedFromProjectState,
    ClipUpdated { track_index: usize, clip_index: usize },
    ClipInserted { track_index: usize, clip_index: usize },
    ClipRemoved { track_index: usize, clip_index: usize },
    ClipSelectionChanged,
    ClipStatesChanged { track_index: usize },
    LoopStateUpdated,
    ToolsChanged,
}

fn cursor_x_to_beats(
    cursor_x: f32,
    view_x: f32,
    scroll_beats_x: f64,
    horizontal_zoom: f64,
    scale_factor: f32,
) -> f64 {
    assert_ne!(horizontal_zoom, 0.0);
    scroll_beats_x
        + (f64::from(cursor_x - view_x)
            / (horizontal_zoom * POINTS_PER_BEAT * f64::from(scale_factor)))
}
