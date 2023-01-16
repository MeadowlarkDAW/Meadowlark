use vizia::resource::FontOrId;
use vizia::vg::Paint;
use vizia::{prelude::*, vg::Color};

use crate::ui::panels::timeline_panel::timeline_view::state::TimelineLaneType;

use super::culler::TimelineViewCuller;
use super::{
    TimelineViewStyle, TimelineViewWorkingState, MARKER_REGION_HEIGHT, POINTS_PER_BEAT,
    ZOOM_THRESHOLD_BARS, ZOOM_THRESHOLD_BEATS, ZOOM_THRESHOLD_EIGTH_BEATS,
    ZOOM_THRESHOLD_QUARTER_BEATS,
};

pub(super) struct RendererCache {
    pub do_full_redraw: bool,
    clip_label_paint: Option<Paint>,
}

impl RendererCache {
    pub fn new() -> Self {
        Self { do_full_redraw: true, clip_label_paint: None }
    }
}

pub(super) fn render_timeline_view(
    cx: &mut DrawContext,
    canvas: &mut Canvas,
    cache: &mut RendererCache,
    state: &TimelineViewWorkingState,
    culler: &TimelineViewCuller,
    style: &TimelineViewStyle,
) {
    use vizia::vg::{Baseline, Path};

    // TODO: Make this work at different DPI scale factors.

    static MAJOR_LINE_TOP_PADDING: f32 = 14.0; // TODO: Make this part of the style?
    static LINE_MARKER_LABEL_TOP_OFFSET: f32 = 19.0; // TODO: Make this part of the style?
    static LINE_MARKER_LABEL_LEFT_OFFSET: f32 = 7.0; // TODO: Make this part of the style?

    let bounds = cx.bounds();
    let view_width_pixels = bounds.width();
    let scale_factor = cx.style.dpi_factor as f32;

    // TODO: Actually cache drawing into a texture.
    let do_full_redraw = {
        let mut res = cache.do_full_redraw;
        cache.do_full_redraw = false;

        // Vizia doesn't always send a `GeometryChanged` event before drawing. (Might be
        // a bug?)
        if culler.view_width_pixels != bounds.width()
            || culler.view_height_pixels != bounds.height()
        {
            res = true;
        }

        res
    };

    if cache.clip_label_paint.is_none() {
        let clip_label_font = cx.resource_manager.fonts.get("inter-medium").unwrap();
        let clip_label_font = if let FontOrId::Id(id) = clip_label_font {
            id
        } else {
            panic!("inter-medium font was not loaded");
        };

        let mut clip_label_vg_paint = Paint::color(style.clip_label_color);
        clip_label_vg_paint.set_font(&[*clip_label_font]);
        clip_label_vg_paint.set_font_size(style.clip_label_size * scale_factor);

        cache.clip_label_paint = Some(clip_label_vg_paint);
    }
    let clip_label_paint = cache.clip_label_paint.as_ref().unwrap();

    // Make sure content doesn't render outside of the view bounds.
    canvas.scissor(bounds.x, bounds.y, bounds.width(), bounds.height());

    let mut bg_path = Path::new();
    bg_path.rect(bounds.x, bounds.y, bounds.width(), bounds.height());
    canvas.fill_path(&mut bg_path, &Paint::color(style.background_color));

    // -- Draw the line markers on the top ----------------------------------------

    let mut bg_path = Path::new();
    bg_path.rect(bounds.x, bounds.y, bounds.width(), MARKER_REGION_HEIGHT * scale_factor);
    canvas.fill_path(&mut bg_path, &Paint::color(style.line_marker_bg_color));

    // -- Draw the vertical gridlines ---------------------------------------------

    let major_line_start_y = bounds.y + (MAJOR_LINE_TOP_PADDING * scale_factor);
    let major_line_height = bounds.height() - (MAJOR_LINE_TOP_PADDING * scale_factor);

    let minor_line_start_y = bounds.y + (MARKER_REGION_HEIGHT * scale_factor);
    let minor_line_height = bounds.height() - (MARKER_REGION_HEIGHT * scale_factor);

    let major_line_width = style.major_line_width * scale_factor;
    let major_line_width_offset = (major_line_width / 2.0).floor();
    let major_line_width_2 = style.major_line_width_2 * scale_factor;
    let major_line_width_2_offset = (major_line_width_2 / 2.0).floor();
    let major_line_paint = Paint::color(style.major_line_color);
    let major_line_paint_2 = Paint::color(style.major_line_color_2);

    let minor_line_width = style.minor_line_width * scale_factor;
    let minor_line_width_offset = (minor_line_width / 2.0).floor();
    let minor_line_paint = Paint::color(style.minor_line_color);

    let mut line_marker_label_paint = Paint::color(style.line_marker_label_color);
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
    line_marker_label_paint.set_font_size(style.line_marker_label_size * scale_factor);
    line_marker_label_paint.set_text_baseline(Baseline::Middle);
    let line_marker_label_y = (bounds.y + (LINE_MARKER_LABEL_TOP_OFFSET * scale_factor)).round();

    let beat_delta_x = (POINTS_PER_BEAT * state.horizontal_zoom) as f32 * scale_factor;
    let first_beat_x = bounds.x
        - ((state.scroll_beats_x.fract() * POINTS_PER_BEAT * state.horizontal_zoom) as f32
            * scale_factor);
    let first_beat = state.scroll_beats_x.floor() as i64;

    enum MajorValueDeltaType {
        WholeUnits(i64),
        Fractional(i64),
    }

    let draw_vertical_gridlines = |canvas: &mut Canvas,
                                   first_major_value: i64,
                                   first_major_x: f32,
                                   major_value_delta: MajorValueDeltaType,
                                   mut major_value_fraction_count: i64,
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
        }
    };

    // TODO: Account for different time signatures.
    // TODO: Account for time signature changes.
    let beats_per_bar: i64 = 4;
    let bars_per_measure: i64 = 4;
    let beats_per_measure: i64 = beats_per_bar * bars_per_measure;

    if state.horizontal_zoom < ZOOM_THRESHOLD_BARS {
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

        draw_vertical_gridlines(
            canvas,
            (num_measures * bars_per_measure) + 1,
            first_measure_beat_x,
            MajorValueDeltaType::WholeUnits(bars_per_measure),
            0,
            measure_delta_x,
            (beats_per_measure / beats_per_bar) as usize,
            view_end_x,
        );
    } else if state.horizontal_zoom < ZOOM_THRESHOLD_BEATS {
        // The zoom threshold at which major lines represent bars and minor lines represent
        // beats.

        let bar_delta_x = beat_delta_x * beats_per_bar as f32;

        let num_bars = first_beat / beats_per_bar;
        let first_bar_beat = num_bars * beats_per_bar;
        let first_bar_beat_x =
            first_beat_x - (((first_beat - first_bar_beat) as f32) * beat_delta_x);

        // Draw one extra to make sure that the text of the last marker is rendered.
        let view_end_x = bounds.x + bounds.width() + bar_delta_x;

        draw_vertical_gridlines(
            canvas,
            num_bars + 1,
            first_bar_beat_x,
            MajorValueDeltaType::WholeUnits(1),
            0,
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

        let num_bars = first_beat / beats_per_bar;
        let first_bar_beat = num_bars * beats_per_bar;
        let first_bar_beat_x =
            first_beat_x - (((first_beat - first_bar_beat) as f32) * beat_delta_x);
        let bar_fraction_count = first_beat - first_bar_beat;

        draw_vertical_gridlines(
            canvas,
            num_bars + 1,
            first_beat_x,
            MajorValueDeltaType::Fractional(beats_per_bar),
            bar_fraction_count,
            beat_delta_x,
            num_subbeat_divisions,
            view_end_x,
        );
    }

    // -- Draw the loop markers ---------------------------------------------------

    if culler.loop_start_pixels_x.is_some() || culler.loop_end_pixels_x.is_some() {
        let loop_marker_width = style.loop_marker_width * scale_factor;
        let loop_marker_width_offset = (loop_marker_width / 2.0).floor();

        let loop_marker_color = if state.loop_active {
            style.loop_marker_active_color
        } else {
            style.loop_marker_inactive_color
        };
        let loop_marker_paint = Paint::color(loop_marker_color);

        let flag_size = style.loop_marker_flag_size * scale_factor;

        if let Some(x) = culler.loop_start_pixels_x {
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
        if let Some(x) = culler.loop_end_pixels_x {
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

    // -- Draw the playhead seek marker -------------------------------------------

    if let Some(playhead_seek_pixels_x) = culler.playhead_seek_pixels_x {
        // The ratio of an equilateral triangle's height to half its width.
        const HEIGHT_TO_HALF_WIDTH: f32 = 0.577350269;

        let playhead_paint = Paint::color(style.playhead_color);

        let flag_size = style.playhead_flag_size * scale_factor;

        let mut flag_path = Path::new();
        let flag_x = (bounds.x + playhead_seek_pixels_x).round();
        flag_path.move_to(flag_x, bounds.y);
        flag_path.line_to(flag_x, bounds.y + flag_size);
        flag_path
            .line_to(flag_x + (flag_size * HEIGHT_TO_HALF_WIDTH), bounds.y + (flag_size / 2.0));
        flag_path.close();
        canvas.fill_path(&mut flag_path, &playhead_paint);
    }

    // -- Draw lanes --------------------------------------------------------------

    // Draw the first line above the first track.
    //
    // We draw rectangles instead of lines because those are more
    // efficient to draw.
    let y = (bounds.y + (MARKER_REGION_HEIGHT * scale_factor)).round() - major_line_width_offset;
    let mut first_line_path = Path::new();
    first_line_path.rect(bounds.x, y, view_width_pixels, major_line_width);
    canvas.fill_path(&mut first_line_path, &major_line_paint);

    let clip_top_height = (style.clip_top_height * scale_factor).round();
    let clip_threshold_height = (style.clip_threshold_height * scale_factor).round();

    let clip_border_width = style.clip_border_width * scale_factor;
    let clip_selected_border_width = style.clip_selected_border_width * scale_factor;
    let clip_border_width_offset = clip_border_width / 2.0;

    let mut clip_border_paint = Paint::color(style.clip_border_color);
    clip_border_paint.set_line_width(clip_border_width);
    let mut clip_selected_border_paint = Paint::color(style.clip_selected_border_color);
    clip_selected_border_paint.set_line_width(clip_selected_border_width);

    let clip_border_radius = style.clip_border_radius * scale_factor;

    let clip_label_lr_padding = style.clip_label_lr_padding * scale_factor;
    let clip_label_y_offset = (style.clip_label_y_offset * scale_factor).round();

    let start_y: f32 = bounds.y + (MARKER_REGION_HEIGHT * scale_factor);
    if !culler.visible_lanes.is_empty() {
        let mut current_lane_y: f32 = start_y + culler.visible_lanes[0].view_start_pixels_y;

        for visible_lane in culler.visible_lanes.iter() {
            let lane_state = &state.lane_states[visible_lane.lane_index];

            let lane_end_y = current_lane_y + (lane_state.height * scale_factor);

            // We draw rectangles instead of lines because those are more
            // efficient to draw.
            let horizontal_line_y = lane_end_y.round() - major_line_width_offset;
            let mut line_path = Path::new();
            line_path.rect(bounds.x, horizontal_line_y, view_width_pixels, major_line_width);
            canvas.fill_path(&mut line_path, &major_line_paint);

            // Draw clips

            let clip_start_y =
                (current_lane_y - major_line_width_offset + major_line_width).round();
            let clip_height =
                (lane_end_y - major_line_width_offset - clip_start_y).round() - clip_border_width;
            let clip_start_y = clip_start_y + clip_border_width_offset;

            for visible_clip in visible_lane.visible_clips.iter() {
                let x = (bounds.x + visible_clip.view_start_pixels_x).round()
                    + clip_border_width_offset;
                let end_x =
                    (bounds.x + visible_clip.view_end_pixels_x).round() - clip_border_width_offset;
                let (width, mut label_clip_width) = if end_x <= bounds.right() {
                    (end_x - x, (end_x - x - clip_label_lr_padding))
                } else {
                    (bounds.right() - x, bounds.right() - x)
                };

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

                    if visible_clip.selected {
                        canvas.stroke_path(&mut top_path, &clip_selected_border_paint);
                    } else {
                        canvas.stroke_path(&mut top_path, &clip_border_paint);
                    }
                } else {
                    let clip_body_color = Color::rgbaf(
                        clip_top_color.r,
                        clip_top_color.g,
                        clip_top_color.b,
                        style.clip_body_alpha,
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

                    if visible_clip.selected {
                        canvas.stroke_path(&mut top_path, &clip_selected_border_paint);
                    } else {
                        canvas.stroke_path(&mut body_path, &clip_border_paint);
                    }
                }

                match &lane_state.type_ {
                    TimelineLaneType::Audio(audio_lane_state) => {
                        let name = &audio_lane_state.clips[visible_clip.clip_index].clip_state.name;

                        // TODO: Clip text with ellipses.
                        // TODO: Don't render text at all if it lies completely out of view.
                        let label_clip_x = if x < bounds.x {
                            label_clip_width -= bounds.x - x;
                            bounds.x
                        } else {
                            x
                        };
                        if label_clip_width > 1.0 {
                            canvas.scissor(
                                label_clip_x,
                                clip_start_y,
                                label_clip_width,
                                clip_height,
                            );
                            canvas
                                .fill_text(
                                    x + clip_label_lr_padding,
                                    clip_start_y + clip_label_y_offset,
                                    name,
                                    clip_label_paint,
                                )
                                .unwrap();
                            canvas.scissor(bounds.x, bounds.y, bounds.width(), bounds.height());
                        }
                    }
                }
            }

            current_lane_y = lane_end_y;
        }
    }

    // -- Draw the playhead -------------------------------------------------------

    if let Some(x) = culler.playhead_pixels_x {
        let playhead_width = style.playhead_width * scale_factor;
        let playhead_width_offset = (playhead_width / 2.0).floor();

        let playhead_paint = Paint::color(style.playhead_color);

        let mut line_path = Path::new();
        let line_x = (bounds.x + x - playhead_width_offset).round();
        line_path.rect(line_x, minor_line_start_y, playhead_width, minor_line_height);
        canvas.fill_path(&mut line_path, &playhead_paint);
    }

    canvas.reset_scissor();
}
