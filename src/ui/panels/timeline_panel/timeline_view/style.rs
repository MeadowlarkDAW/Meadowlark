use vizia::vg::Color;

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
    pub clip_label_size: f32,
    pub clip_label_color: Color,
    pub clip_label_lr_padding: f32,
    pub clip_label_y_offset: f32,

    pub loop_marker_width: f32,
    pub loop_marker_active_color: Color,
    pub loop_marker_inactive_color: Color,
    pub loop_marker_flag_size: f32,

    pub playhead_width: f32,
    pub playhead_color: Color,
    pub playhead_flag_size: f32,
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
            clip_label_size: 11.0,
            clip_label_color: Color::rgba(0x33, 0x33, 0x33, 0xee),
            clip_label_lr_padding: 5.0,
            clip_label_y_offset: 10.0,

            loop_marker_width: 1.0,
            loop_marker_active_color: Color::rgb(0x8b, 0x8b, 0x8b),
            loop_marker_inactive_color: Color::rgb(0x44, 0x44, 0x44),
            loop_marker_flag_size: 10.0,

            playhead_width: 1.0,
            playhead_color: Color::rgb(0xeb, 0x70, 0x71),
            playhead_flag_size: 12.0,
        }
    }
}
