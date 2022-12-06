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
use vizia::resource::FontOrId;
use vizia::{prelude::*, vg::Color};

use crate::state_system::actions::{AppAction, ScrollUnits, TimelineAction};
use crate::state_system::app_state::timeline_state::{TimelineMode, TimelineState};

static PIXELS_PER_BEAT: f64 = 70.0;
static MARKER_REGION_HEIGHT: f32 = 28.0;
static DRAG_ZOOM_SCALAR: f64 = 0.001;

pub static MIN_ZOOM: f64 = 0.2; // TODO: Find a good value for this.
pub static MAX_ZOOM: f64 = 8.0; // TODO: Find a good value for this.

/// The zoom threshold at which major lines represent measures and minor lines
/// represent bars.
static ZOOM_THRESHOLD_BARS: f64 = 0.125;
/// The zoom threshold at which major lines represent bars and minor lines represent
/// beats.
static ZOOM_THRESHOLD_BEATS: f64 = 0.25;
/// The zoom threshold at which major lines represent beats and minor lines represent
/// quarter-notes.
static ZOOM_THRESHOLD_QUARTER_BEATS: f64 = 0.5;
/// The zoom threshold at which major lines represent beats and minor lines represent
/// eight-notes.
static ZOOM_THRESHOLD_EIGTH_BEATS: f64 = 1.5;
/// The zoom threshold at which major lines represent beats and minor lines represent
/// sixteenth-notes.
static ZOOM_THRESHOLD_SIXTEENTH_BEATS: f64 = 1.5;

pub enum TimelineViewEvent {
    Navigate {
        /// The horizontal zoom level. 1.0 = default zoom
        horizontal_zoom: f64,

        /// The x position of the left side of the timeline view.
        scroll_units_x: ScrollUnits,
    },
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

            major_line_color: Color::rgb(0x0f, 0x10, 0x10),
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
        (zoom_normal * zoom_normal * (MAX_ZOOM - MIN_ZOOM)) + MIN_ZOOM
    }
}

fn zoom_value_to_normal(zoom: f64) -> f64 {
    if zoom >= MAX_ZOOM {
        1.0
    } else if zoom <= MIN_ZOOM {
        0.0
    } else {
        ((zoom - MIN_ZOOM) / (MAX_ZOOM - MIN_ZOOM)).sqrt()
    }
}

pub struct TimelineView {
    /// The horizontal zoom level. 1.0 = default zoom
    horizontal_zoom: f64,
    horizontal_zoom_normalized: f64,

    /// The x position of the left side of the view. When the timeline is in
    /// musical mode, this is in units of beats. When the timeline is in
    /// H:M:S mode, this is in units of seconds.
    scroll_units_x: f64,

    /// The mode in which the timeline displays its contents.
    mode: TimelineMode,

    style: TimelineViewStyle,

    is_dragging_marker_region: bool,
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
        timeline_state: &TimelineState,
        style: TimelineViewStyle,
    ) -> Handle<'a, Self> {
        assert_ne!(timeline_state.horizontal_zoom, 0.0);

        Self {
            horizontal_zoom: timeline_state.horizontal_zoom,
            horizontal_zoom_normalized: zoom_value_to_normal(1.0),
            scroll_units_x: timeline_state.scroll_units_x,
            mode: timeline_state.mode,
            style,
            is_dragging_marker_region: false,
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
            TimelineViewEvent::Navigate {
                /// The horizontal zoom level. 1.0 = default zoom
                horizontal_zoom,
                /// The x position of the left side of the timeline view.
                scroll_units_x,
            } => {
                let scroll_units_x = match scroll_units_x {
                    ScrollUnits::Musical(beats_x) => {
                        if self.mode == TimelineMode::Musical {
                            *beats_x
                        } else {
                            // TODO
                            0.0
                        }
                    }
                    ScrollUnits::HMS(seconds_x) => {
                        // TODO
                        0.0
                    }
                };

                if self.horizontal_zoom != *horizontal_zoom || self.scroll_units_x != scroll_units_x
                {
                    self.horizontal_zoom = *horizontal_zoom;
                    self.horizontal_zoom_normalized = zoom_value_to_normal(*horizontal_zoom);
                    self.scroll_units_x = scroll_units_x;

                    cx.needs_redraw();
                }
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
                if *button == MouseButton::Left {
                    let current = cx.current();
                    let bounds = cx.cache.get_bounds(current);

                    if cx.mouse.left.pos_down.1 >= bounds.y
                        && cx.mouse.left.pos_down.1 <= bounds.y + MARKER_REGION_HEIGHT
                        && bounds.width() != 0.0
                    {
                        dbg!(self.scroll_units_x);

                        self.is_dragging_marker_region = true;
                        self.drag_start_horizontal_zoom_normalized =
                            zoom_value_to_normal(self.horizontal_zoom);
                        self.drag_start_scroll_x = cursor_x_to_beats(
                            cx.mouse.left.pos_down.0,
                            bounds.x,
                            self.scroll_units_x,
                            self.horizontal_zoom,
                        );
                        self.drag_start_pixel_x_offset =
                            f64::from(cx.mouse.left.pos_down.0 - bounds.x);

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
                    cx.release();
                }
            }
            WindowEvent::MouseMove(x, y) => {
                if self.is_dragging_marker_region {
                    let (offset_x_pixels, offset_y_pixels) = cx.mouse.delta(MouseButton::Left);

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

                    if self.mode == TimelineMode::Musical {
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
        use vizia::vg::{Paint, Path, Baseline};

        // TODO: Make this work at different scale factors.
        // TODO: Add support for different time signatures (once we have a proper tempo map).

        static MAJOR_LINE_TOP_PADDING: f32 = 14.0; // TODO: Make this part of the style?

        let bounds = cx.bounds();
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

        // -- Draw the line markers on the top ----------------------------------------

        let mut bg_path = Path::new();
        bg_path.rect(bounds.x, bounds.y, bounds.width(), MARKER_REGION_HEIGHT);
        canvas.fill_path(&mut bg_path, &Paint::color(self.style.line_marker_bg_color));

        // -- Draw the vertical sections and gridlines --------------------------------

        let major_line_start_y = bounds.y + MAJOR_LINE_TOP_PADDING;
        let major_line_height = bounds.height() - MAJOR_LINE_TOP_PADDING;

        let minor_line_start_y = bounds.y + MARKER_REGION_HEIGHT;
        let minor_line_height = bounds.height() - MARKER_REGION_HEIGHT;

        let major_line_width = self.style.major_line_width;
        let major_line_width_offset = (major_line_width / 2.0).floor();
        let major_line_paint = Paint::color(self.style.line_marker_label_color);

        let minor_line_width = self.style.minor_line_width;
        let minor_line_width_offset = (minor_line_width / 2.0).floor();
        let minor_line_paint = Paint::color(self.style.line_marker_label_color);

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

        let beat_delta_x = (PIXELS_PER_BEAT * self.horizontal_zoom) as f32;
        let first_beat_x = bounds.x
            - (self.scroll_units_x.fract() * PIXELS_PER_BEAT * self.horizontal_zoom) as f32;
        // Draw one extra to make sure that the text of the last marker is rendered.
        let view_end_x = bounds.x + bounds.width() + beat_delta_x;
        let mut current_beat_x = first_beat_x;

        let num_subbeat_divisions = 4;
        let subbeat_delta_x = beat_delta_x / num_subbeat_divisions as f32;

        while current_beat_x <= view_end_x {
            // Draw the sub-beat markers.
            for i in 1..num_subbeat_divisions {
                let line_x = (current_beat_x + (subbeat_delta_x * i as f32)).round();

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
            let line_x = current_beat_x.round();

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

            current_beat_x += beat_delta_x;
        }

        canvas.reset_scissor();
    }
}

pub enum InternalTimelineViewEvent {}

fn cursor_x_to_beats(cursor_x: f32, view_x: f32, scroll_units_x: f64, horizontal_zoom: f64) -> f64 {
    assert_ne!(horizontal_zoom, 0.0);
    scroll_units_x + (f64::from(cursor_x - view_x) / (horizontal_zoom * PIXELS_PER_BEAT))
}
