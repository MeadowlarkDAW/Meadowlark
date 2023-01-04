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

use crate::state_system::actions::{AppAction, ScrollUnits, TimelineAction};
use crate::state_system::source_state::TimelineMode;

mod culler;
mod renderer;
mod state;
mod style;

pub use state::TimelineViewState;
pub use style::TimelineViewStyle;

use culler::TimelineViewCuller;
use renderer::{render_timeline_view, RendererCache};

pub static MIN_ZOOM: f64 = 0.025; // TODO: Find a good value for this.
pub static MAX_ZOOM: f64 = 8.0; // TODO: Find a good value for this.

static POINTS_PER_BEAT: f64 = 100.0;
static MARKER_REGION_HEIGHT: f32 = 28.0;
static DRAG_ZOOM_SCALAR: f64 = 0.00029;
static DRAG_ZOOM_EXP: f64 = 3.75;

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

pub struct TimelineView {
    /// This is only allowed to be borrowed mutably within the
    /// `state_system::handle_action` method.
    shared_state: Rc<RefCell<TimelineViewState>>,

    style: TimelineViewStyle,

    is_dragging_marker_region: bool,
    is_dragging_with_middle_click: bool,
    drag_start_scroll_x: f64,
    drag_start_pixel_x_offset: f64,
    drag_start_horizontal_zoom_normalized: f64,

    culler: TimelineViewCuller,

    scale_factor: f64,
    view_width_pixels: f32,
    view_height_pixels: f32,

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
            drag_start_scroll_x: 0.0,
            drag_start_pixel_x_offset: 0.0,
            drag_start_horizontal_zoom_normalized: 0.0,
            culler: TimelineViewCuller::new(),
            scale_factor: 1.0,
            view_width_pixels: 0.0,
            view_height_pixels: 0.0,
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
            TimelineViewEvent::ClipUpdated { track_index, clip_id } => {
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
            TimelineViewEvent::ClipInserted { track_index, clip_id } => {
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
            TimelineViewEvent::ClipRemoved { track_index, clip_id } => {
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
            TimelineViewEvent::LoopStateUpdated => {
                self.culler.cull_markers(&*self.shared_state.borrow());

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

                if self.view_width_pixels != width
                    || self.view_height_pixels != height && self.scale_factor != scale_factor
                {
                    self.view_width_pixels = width;
                    self.view_height_pixels = height;
                    self.scale_factor = scale_factor;

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

                        cx.emit(AppAction::Timeline(TimelineAction::Navigate {
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
    ClipUpdated { track_index: usize, clip_id: u64 },
    ClipInserted { track_index: usize, clip_id: u64 },
    ClipRemoved { track_index: usize, clip_id: u64 },
    LoopStateUpdated,
}

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
