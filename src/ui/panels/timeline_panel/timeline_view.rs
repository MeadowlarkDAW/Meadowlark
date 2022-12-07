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
use vizia::resource::FontOrId;
use vizia::{prelude::*, vg::Color};

use crate::state_system::actions::{AppAction, ScrollUnits, TimelineAction};
use crate::state_system::app_state::timeline_state::{TimelineMode, TimelineState};

static PIXELS_PER_BEAT: f64 = 100.0;
static MARKER_REGION_HEIGHT: f32 = 28.0;
static DRAG_ZOOM_SCALAR: f64 = 0.0005;

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

pub enum TimelineViewEvent {
    Navigate,
    LaneHeightSet { lane_index: usize },
}

#[derive(Debug, Clone)]
pub struct TimelineViewStyle {
    pub background_color_1: Color,
    pub background_color_2: Color,

    pub major_line_color: Color,
    pub minor_line_color_1: Color,
    pub minor_line_color_2: Color,

    pub major_line_width: f32,
    pub minor_line_width: f32,

    pub line_marker_label_color: Color,
    pub line_marker_bg_color: Color,
    pub line_marker_label_size: f32,
}

impl Default for TimelineViewStyle {
    fn default() -> Self {
        Self {
            background_color_1: Color::rgb(0x2a, 0x2b, 0x2a),
            background_color_2: Color::rgb(0x28, 0x28, 0x28),

            major_line_color: Color::rgb(0x18, 0x1a, 0x1a),
            minor_line_color_1: Color::rgb(0x1f, 0x1f, 0x1f),
            minor_line_color_2: Color::rgb(0x1e, 0x1f, 0x1e),

            major_line_width: 2.0,
            minor_line_width: 1.0,

            line_marker_label_color: Color::rgb(0x7d, 0x7e, 0x81),
            line_marker_bg_color: Color::rgb(0x22, 0x22, 0x22),
            line_marker_label_size: 12.0,
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
        (zoom_normal * zoom_normal * zoom_normal * (MAX_ZOOM - MIN_ZOOM)) + MIN_ZOOM
    }
}

fn zoom_value_to_normal(zoom: f64) -> f64 {
    if zoom >= MAX_ZOOM {
        1.0
    } else if zoom <= MIN_ZOOM {
        0.0
    } else {
        ((zoom - MIN_ZOOM) / (MAX_ZOOM - MIN_ZOOM)).powf(1.0 / 3.0)
    }
}

pub struct TimelineView {
    /// Only the `StateSystem` struct is allowed to mutate this.
    state: Rc<RefCell<TimelineState>>,

    horizontal_zoom_normalized: f64,

    style: TimelineViewStyle,

    is_dragging_marker_region: bool,
    is_dragging_with_middle_click: bool,
    drag_start_scroll_x: f64,
    drag_start_pixel_x_offset: f64,
    drag_start_horizontal_zoom_normalized: f64,

    bounds_width: f32,
    bounds_height: f32,
    custom_draw_cache: RefCell<CustomDrawCache>,
}

impl TimelineView {
    pub fn new<'a>(
        cx: &'a mut Context,
        state: &Rc<RefCell<TimelineState>>,
        style: TimelineViewStyle,
    ) -> Handle<'a, Self> {
        let horizontal_zoom = { state.borrow().horizontal_zoom };
        assert_ne!(horizontal_zoom, 0.0);

        Self {
            state: Rc::clone(state),
            horizontal_zoom_normalized: zoom_value_to_normal(horizontal_zoom),
            style,
            is_dragging_marker_region: false,
            is_dragging_with_middle_click: false,
            drag_start_scroll_x: 0.0,
            drag_start_pixel_x_offset: 0.0,
            drag_start_horizontal_zoom_normalized: 0.0,
            bounds_width: 0.0,
            bounds_height: 0.0,
            custom_draw_cache: RefCell::new(CustomDrawCache::new()),
        }
        .build(cx, move |cx| {})
    }
}

impl View for TimelineView {
    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        event.map(|timeline_view_event, _| match timeline_view_event {
            TimelineViewEvent::Navigate => {
                let state = self.state.borrow();

                self.horizontal_zoom_normalized = zoom_value_to_normal(state.horizontal_zoom);
                cx.needs_redraw();
            }
            TimelineViewEvent::LaneHeightSet { lane_index } => {
                cx.needs_redraw();
            }
        });

        event.map(|window_event, meta| match window_event {
            WindowEvent::GeometryChanged(_) => {
                let current = cx.current();
                let width = cx.cache.get_width(current);
                let height = cx.cache.get_height(current);

                if self.bounds_width != width || self.bounds_height != height {
                    self.bounds_width = width;
                    self.bounds_height = height;

                    self.custom_draw_cache.borrow_mut().do_full_redraw = true;

                    cx.needs_redraw();
                }
            }
            WindowEvent::MouseDown(button) => {
                let state = self.state.borrow();

                if *button == MouseButton::Left {
                    let current = cx.current();
                    let bounds = cx.cache.get_bounds(current);

                    if cx.mouse.left.pos_down.1 >= bounds.y
                        && cx.mouse.left.pos_down.1 <= bounds.y + MARKER_REGION_HEIGHT
                        && bounds.width() != 0.0
                        && !self.is_dragging_with_middle_click
                    {
                        self.is_dragging_marker_region = true;
                        self.drag_start_horizontal_zoom_normalized =
                            zoom_value_to_normal(state.horizontal_zoom);
                        self.drag_start_scroll_x = cursor_x_to_beats(
                            cx.mouse.left.pos_down.0,
                            bounds.x,
                            state.scroll_units_x,
                            state.horizontal_zoom,
                        );
                        self.drag_start_pixel_x_offset =
                            f64::from(cx.mouse.left.pos_down.0 - bounds.x);

                        meta.consume();
                        cx.capture();
                        cx.focus_with_visibility(false);

                        // TODO: Lock the pointer in place once Vizia gets that ability.
                    }
                } else if *button == MouseButton::Middle {
                    let current = cx.current();
                    let bounds = cx.cache.get_bounds(current);

                    if bounds.width() != 0.0 && !self.is_dragging_marker_region {
                        self.is_dragging_with_middle_click = true;
                        self.drag_start_horizontal_zoom_normalized =
                            zoom_value_to_normal(state.horizontal_zoom);
                        self.drag_start_scroll_x = cursor_x_to_beats(
                            cx.mouse.middle.pos_down.0,
                            bounds.x,
                            state.scroll_units_x,
                            state.horizontal_zoom,
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
                    let state = self.state.borrow();

                    let (offset_x_pixels, offset_y_pixels) = if self.is_dragging_marker_region {
                        cx.mouse.delta(MouseButton::Left)
                    } else {
                        cx.mouse.delta(MouseButton::Middle)
                    };

                    let delta_zoom_normal = -f64::from(offset_y_pixels) * DRAG_ZOOM_SCALAR;
                    let new_zoom_normal = (self.drag_start_horizontal_zoom_normalized
                        + delta_zoom_normal)
                        .clamp(0.0, 1.0);
                    let horizontal_zoom = zoom_normal_to_value(new_zoom_normal);

                    // Calculate the new scroll position offset for the left side of the view so
                    // that zooming is centered around the point where the mouse button last
                    // pressed down.
                    let zoom_x_offset =
                        self.drag_start_pixel_x_offset / (PIXELS_PER_BEAT * horizontal_zoom);

                    if state.mode == TimelineMode::Musical {
                        let pan_offset_x_beats =
                            f64::from(offset_x_pixels) / (PIXELS_PER_BEAT * horizontal_zoom);

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
        use vizia::vg::{Baseline, Paint, Path};

        // TODO: Make this work at different DPI scale factors.

        static MAJOR_LINE_TOP_PADDING: f32 = 14.0; // TODO: Make this part of the style?
        static LINE_MARKER_LABEL_TOP_OFFSET: f32 = 19.0; // TODO: Make this part of the style?
        static LINE_MARKER_LABEL_LEFT_OFFSET: f32 = 7.0; // TODO: Make this part of the style?

        let state = self.state.borrow();

        let bounds = cx.bounds();
        let bounds_width = bounds.width();
        let mut custom_draw_cache = self.custom_draw_cache.borrow_mut();

        // TODO: Actually cache drawing into a texture.
        let do_full_redraw = {
            let mut res = custom_draw_cache.do_full_redraw;
            custom_draw_cache.do_full_redraw = false;

            // Vizia doesn't always send a `GeometryChanged` event before drawing. (Might be
            // a bug?)
            if self.bounds_width != bounds.width() || self.bounds_height != bounds.height() {
                res = true;
            }

            res
        };

        // Make sure content doesn't render outside of the view bounds.
        canvas.scissor(bounds.x, bounds.y, bounds.width(), bounds.height());

        let mut bg_path = Path::new();
        bg_path.rect(bounds.x, bounds.y, bounds.width(), bounds.height());
        canvas.fill_path(&mut bg_path, &Paint::color(self.style.background_color_2));

        // -- Draw the line markers on the top ----------------------------------------

        let mut bg_path = Path::new();
        bg_path.rect(bounds.x, bounds.y, bounds.width(), MARKER_REGION_HEIGHT + 3.0);
        canvas.fill_path(&mut bg_path, &Paint::color(self.style.line_marker_bg_color));

        // -- Draw the vertical gridlines ---------------------------------------------

        let major_line_start_y = bounds.y + MAJOR_LINE_TOP_PADDING;
        let major_line_height = bounds.height() - MAJOR_LINE_TOP_PADDING;

        let minor_line_start_y = bounds.y + MARKER_REGION_HEIGHT;
        let minor_line_height = bounds.height() - MARKER_REGION_HEIGHT;

        let major_line_width = self.style.major_line_width;
        let major_line_width_offset = (major_line_width / 2.0).floor();
        let major_line_paint = Paint::color(self.style.major_line_color);

        let minor_line_width = self.style.minor_line_width;
        let minor_line_width_offset = (minor_line_width / 2.0).floor();
        let minor_line_paint = Paint::color(self.style.minor_line_color_2);

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
        line_marker_label_paint.set_font_size(self.style.line_marker_label_size);
        line_marker_label_paint.set_text_baseline(Baseline::Middle);
        let line_marker_label_y = (bounds.y + LINE_MARKER_LABEL_TOP_OFFSET).round();

        let beat_delta_x = (PIXELS_PER_BEAT * state.horizontal_zoom) as f32;
        let first_beat_x = bounds.x
            - (state.scroll_units_x.fract() * PIXELS_PER_BEAT * state.horizontal_zoom) as f32;
        let first_beat = state.scroll_units_x.floor() as i64;

        let draw_vertical_gridlines = |canvas: &mut Canvas,
                                       first_major_value: i64,
                                       first_major_x: f32,
                                       major_value_delta: i64,
                                       major_delta_x: f32,
                                       num_minor_subdivisions: usize,
                                       view_end_x: f32| {
            let minor_delta_x = major_delta_x / num_minor_subdivisions as f32;

            let mut current_major_value = first_major_value;
            let mut current_major_x = first_major_x;
            while current_major_x <= view_end_x {
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

                    canvas.fill_path(&mut minor_line_path, &minor_line_paint);
                }

                // Round to the nearest pixel so lines are sharp.
                let line_x = current_major_x.round();

                // We draw rectangles instead of lines because those are more
                // efficient to draw.
                let mut major_line_path = Path::new();
                major_line_path.rect(
                    line_x - major_line_width_offset,
                    major_line_start_y,
                    major_line_width,
                    major_line_height,
                );

                canvas.fill_path(&mut major_line_path, &major_line_paint);

                canvas
                    .fill_text(
                        current_major_x + LINE_MARKER_LABEL_LEFT_OFFSET,
                        line_marker_label_y,
                        format!("{}", current_major_value),
                        &line_marker_label_paint,
                    )
                    .unwrap();

                current_major_value += major_value_delta;
                current_major_x += major_delta_x;
            }
        };

        if state.horizontal_zoom < ZOOM_THRESHOLD_BARS {
            // The zoom threshold at which major lines represent measures and minor lines
            // represent bars.

            // TODO: Account for different time signatures.
            // TODO: Account for time signature changes.
            let beats_per_bar: i64 = 4;
            let beats_per_measure: i64 = beats_per_bar * 4;
            let measure_delta_x = beat_delta_x * beats_per_measure as f32;

            let first_measure_beat = (first_beat / beats_per_measure) * beats_per_measure;
            let first_measure_beat_x =
                first_beat_x - (((first_beat - first_measure_beat) as f32) * beat_delta_x);

            // Draw one extra to make sure that the text of the last marker is rendered.
            let view_end_x = bounds.x + bounds.width() + measure_delta_x;

            draw_vertical_gridlines(
                canvas,
                first_measure_beat,
                first_measure_beat_x,
                beats_per_measure,
                measure_delta_x,
                (beats_per_measure / beats_per_bar) as usize,
                view_end_x,
            );
        } else if state.horizontal_zoom < ZOOM_THRESHOLD_BEATS {
            // The zoom threshold at which major lines represent bars and minor lines represent
            // beats.

            // TODO: Account for different time signatures.
            // TODO: Account for time signature changes.
            let beats_per_bar: i64 = 4;
            let bar_delta_x = beat_delta_x * beats_per_bar as f32;

            let first_bar_beat = (first_beat / beats_per_bar) * beats_per_bar;
            let first_bar_beat_x =
                first_beat_x - (((first_beat - first_bar_beat) as f32) * beat_delta_x);

            // Draw one extra to make sure that the text of the last marker is rendered.
            let view_end_x = bounds.x + bounds.width() + bar_delta_x;

            draw_vertical_gridlines(
                canvas,
                first_bar_beat,
                first_bar_beat_x,
                beats_per_bar,
                bar_delta_x,
                beats_per_bar as usize,
                view_end_x,
            );
        } else {
            // The zoom threshold at which major lines represent beats and minor lines represent
            // beat subdivisions.

            let num_subbeat_divisions = if state.horizontal_zoom < ZOOM_THRESHOLD_QUARTER_BEATS {
                4
            } else if state.horizontal_zoom < ZOOM_THRESHOLD_EIGTH_BEATS {
                8
            } else {
                16
            }; // TODO: More subdivisions?

            // Draw one extra to make sure that the text of the last marker is rendered.
            let view_end_x = bounds.x + bounds.width() + beat_delta_x;

            draw_vertical_gridlines(
                canvas,
                first_beat,
                first_beat_x,
                1,
                beat_delta_x,
                num_subbeat_divisions,
                view_end_x,
            );
        }

        // -- Draw the horizontal gridlines -------------------------------------------

        // Draw the first line above the first track.
        //
        // We draw rectangles instead of lines because those are more
        // efficient to draw.
        let y = (bounds.y + MARKER_REGION_HEIGHT + 3.0).round() - major_line_width_offset;
        let mut first_line_path = Path::new();
        first_line_path.rect(bounds.x, y, bounds_width, major_line_width);
        canvas.fill_path(&mut first_line_path, &major_line_paint);

        let mut current_line_y: f32 = bounds.y + MARKER_REGION_HEIGHT + 3.0;
        for lane_state in state.lane_states.iter() {
            let y = (current_line_y + lane_state.height).round() - major_line_width_offset;

            // We draw rectangles instead of lines because those are more
            // efficient to draw.
            let mut line_path = Path::new();
            line_path.rect(bounds.x, y, bounds_width, major_line_width);
            canvas.fill_path(&mut line_path, &major_line_paint);

            current_line_y += lane_state.height;
        }

        canvas.reset_scissor();
    }
}

pub enum InternalTimelineViewEvent {}

fn cursor_x_to_beats(cursor_x: f32, view_x: f32, scroll_units_x: f64, horizontal_zoom: f64) -> f64 {
    assert_ne!(horizontal_zoom, 0.0);
    scroll_units_x + (f64::from(cursor_x - view_x) / (horizontal_zoom * PIXELS_PER_BEAT))
}
