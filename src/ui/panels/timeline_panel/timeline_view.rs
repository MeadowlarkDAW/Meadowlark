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
use vizia::{cache::BoundingBox, prelude::*, vg::Color};

use crate::state_system::app_state::timeline_state::{TimelineMode, TimelineState};

static PIXELS_PER_BEAT: f64 = 70.0;

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
    SetHorizontalZoom(f64),
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
    prev_bounds: BoundingBox,
    do_full_redraw: bool,
}

impl CustomDrawCache {
    fn new() -> Self {
        Self { prev_bounds: BoundingBox::default(), do_full_redraw: true }
    }
}

pub struct TimelineView {
    /// The horizontal zoom level. 1.0 = default zoom
    horizontal_zoom: f64,

    /// The x position of the left side of the view. When the timeline is in
    /// musical mode, this is in units of beats. When the timeline is in
    /// H:M:S mode, this is in units of seconds.
    scroll_units_x: f64,

    /// The mode in which the timeline displays its contents.
    mode: TimelineMode,

    style: TimelineViewStyle,

    custom_draw_cache: RefCell<CustomDrawCache>,
}

impl TimelineView {
    pub fn new<'a>(
        cx: &'a mut Context,
        timeline_state: &TimelineState,
        style: TimelineViewStyle,
    ) -> Handle<'a, Self> {
        Self {
            horizontal_zoom: timeline_state.horizontal_zoom,
            scroll_units_x: timeline_state.scroll_units_x,
            mode: timeline_state.mode,
            style,
            custom_draw_cache: RefCell::new(CustomDrawCache::new()),
        }
        .build(cx, move |cx| {})
    }
}

impl View for TimelineView {
    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        event.map(|timeline_view_event, _| match timeline_view_event {
            TimelineViewEvent::SetHorizontalZoom(zoom) => {
                if self.horizontal_zoom != *zoom {
                    self.horizontal_zoom = *zoom;
                    cx.needs_redraw();
                }
            }
        });

        event.map(|window_event, _| match window_event {
            WindowEvent::MouseDown(button) if *button == MouseButton::Left => {}
            _ => {}
        });
    }

    fn draw(&self, cx: &mut DrawContext, canvas: &mut Canvas) {
        use vizia::vg::{Paint, Path};

        // TODO: Make this work at different scale factors.
        // TODO: Add support for different time signatures (once we have a proper tempo map).

        static LINE_MARKERS_HEIGHT: f32 = 26.0;
        static MAJOR_LINE_TOP_PADDING: f32 = 14.0; // TODO: Make this part of the style?

        let bounds = cx.bounds();
        let mut custom_draw_cache = self.custom_draw_cache.borrow_mut();

        // TODO: Actually cache drawing into a texture.
        let do_full_redraw = {
            let mut res = custom_draw_cache.do_full_redraw;
            custom_draw_cache.do_full_redraw = false;

            if custom_draw_cache.prev_bounds != bounds {
                custom_draw_cache.prev_bounds = bounds;
                res = true;
            }

            res
        };

        // Make sure content doesn't render outside of the view bounds.
        canvas.scissor(bounds.x, bounds.y, bounds.width(), bounds.height());

        // -- Draw the line markers on the top ----------------------------------------

        let mut bg_path = Path::new();
        bg_path.rect(bounds.x, bounds.y, bounds.width(), LINE_MARKERS_HEIGHT);
        canvas.fill_path(&mut bg_path, &Paint::color(self.style.line_marker_bg_color));

        // -- Draw the vertical sections and gridlines --------------------------------

        let major_line_start_y = bounds.y + MAJOR_LINE_TOP_PADDING;
        let major_line_height = bounds.height() - MAJOR_LINE_TOP_PADDING;

        let minor_line_start_y = bounds.y + LINE_MARKERS_HEIGHT;
        let minor_line_height = bounds.height() - LINE_MARKERS_HEIGHT;

        let major_line_width = self.style.major_line_width;
        let major_line_width_offset = (major_line_width / 2.0).floor();
        let major_line_paint = Paint::color(self.style.line_marker_label_color);

        let minor_line_width = self.style.minor_line_width;
        let minor_line_width_offset = (minor_line_width / 2.0).floor();
        let minor_line_paint = Paint::color(self.style.line_marker_label_color);

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
