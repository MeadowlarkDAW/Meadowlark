mod basic_elements;

use std::io::BufRead;

pub use basic_elements::*;

use yarrow::{
    style::{Background, BorderStyle},
    vg::color::{self, RGBA8},
};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AppTheme {
    pub clear_color: RGBA8,
    pub button_border_radius: f32,

    pub top_panel_background: Background,
    pub top_panel_border: AppBorderStyle,

    pub top_panel_button: AppButtonStyle,
    pub top_panel_toggle_btn: AppToggleButtonStyle,
    pub top_panel_title_color: RGBA8,
    pub top_panel_seperator_color: RGBA8,
    pub top_panel_record_btn: AppToggleButtonStyle,
    pub top_panel_dropdown_btn: AppButtonStyle,
    pub top_panel_label_color: RGBA8,
    pub top_panel_play_pause_btn: AppToggleButtonStyle,

    pub top_panel_transport_box_bg: Background,
    pub top_panel_transport_box_border: AppBorderStyle,
    pub top_panel_box_seperator_color: RGBA8,
    pub top_panel_numeric_input_font_color: RGBA8,
    pub top_panel_numeric_input_font_highlight_color: RGBA8,
    pub top_panel_numeric_input_cursor_color: RGBA8,
    pub top_panel_numeric_input_highlight_bg_color: RGBA8,
    pub top_panel_numeric_input_active_border: AppBorderStyle,
}

impl Default for AppTheme {
    fn default() -> Self {
        const GRAY_LEVEL_0: RGBA8 = RGBA8::new(5, 5, 5, 255);
        const GRAY_LEVEL_1: RGBA8 = RGBA8::new(18, 18, 18, 255);
        const GRAY_LEVEL_2: RGBA8 = RGBA8::new(37, 37, 37, 255);
        const GRAY_LEVEL_3: RGBA8 = RGBA8::new(51, 51, 51, 255);
        const GRAY_LEVEL_4: RGBA8 = RGBA8::new(64, 64, 64, 255);
        const GRAY_LEVEL_5: RGBA8 = RGBA8::new(95, 95, 95, 255);
        const GRAY_LEVEL_6: RGBA8 = RGBA8::new(170, 170, 170, 255);

        const MAIN_FONT_COLOR: RGBA8 = RGBA8::new(255, 255, 255, 195);
        const BRIGHT_FONT_COLOR: RGBA8 = RGBA8::new(255, 255, 255, 255);
        const DIMMED_FONT_COLOR: RGBA8 = RGBA8::new(255, 255, 255, 100);

        const ACCENT_COLOR_1: RGBA8 = RGBA8::new(241, 83, 74, 255);
        const ACCENT_COLOR_2: RGBA8 = RGBA8::new(74, 129, 241, 255);
        const ACCENT_COLOR_1_BG: RGBA8 =
            RGBA8::new(ACCENT_COLOR_1.r, ACCENT_COLOR_1.g, ACCENT_COLOR_1.b, 50);
        const ACCENT_COLOR_2_BG: RGBA8 =
            RGBA8::new(ACCENT_COLOR_2.r, ACCENT_COLOR_2.g, ACCENT_COLOR_2.b, 50);
        const ACCENT_COLOR_2_BG_HOVER: RGBA8 =
            RGBA8::new(ACCENT_COLOR_2.r, ACCENT_COLOR_2.g, ACCENT_COLOR_2.b, 150);

        const RECORD_COLOR: RGBA8 = RGBA8::new(241, 83, 74, 255);
        const RECORD_COLOR_BG: RGBA8 =
            RGBA8::new(RECORD_COLOR.r, RECORD_COLOR.g, RECORD_COLOR.b, 50);
        const RECORD_COLOR_BG_HOVER: RGBA8 =
            RGBA8::new(RECORD_COLOR.r, RECORD_COLOR.g, RECORD_COLOR.b, 150);

        const BUTTON_BORDER: AppBorderStyle = AppBorderStyle {
            radius: 4.0,
            width: 1.0,
            color: color::TRANSPARENT,
        };

        Self {
            clear_color: GRAY_LEVEL_0,
            button_border_radius: 4.0,

            top_panel_background: Background::Solid(GRAY_LEVEL_2),
            top_panel_border: AppBorderStyle::default(),

            top_panel_button: AppButtonStyle {
                bg_idle: Background::TRANSPARENT,
                bg_hover: Background::Solid(GRAY_LEVEL_5),
                bg_down: Background::Solid(GRAY_LEVEL_1),
                bg_disabled: Background::TRANSPARENT,

                border_idle: BUTTON_BORDER,
                border_hover: AppBorderStyle {
                    color: GRAY_LEVEL_0,
                    ..BUTTON_BORDER
                },
                border_down: AppBorderStyle {
                    color: GRAY_LEVEL_0,
                    ..BUTTON_BORDER
                },
                border_disabled: BUTTON_BORDER,

                font_color_idle: MAIN_FONT_COLOR,
                font_color_hover: BRIGHT_FONT_COLOR,
                font_color_down: BRIGHT_FONT_COLOR,
                font_color_disabled: DIMMED_FONT_COLOR,
            },

            top_panel_toggle_btn: AppToggleButtonStyle {
                bg_idle_off: Background::Solid(GRAY_LEVEL_1),
                bg_hover_off: Background::Solid(GRAY_LEVEL_5),
                bg_down_off: Background::Solid(GRAY_LEVEL_1),
                bg_disabled_off: Background::TRANSPARENT,

                bg_idle_on: Background::Solid(ACCENT_COLOR_2_BG),
                bg_hover_on: Background::Solid(ACCENT_COLOR_2_BG_HOVER),
                bg_down_on: Background::Solid(ACCENT_COLOR_2_BG),
                bg_disabled_on: Background::TRANSPARENT,

                border_idle_off: AppBorderStyle {
                    color: GRAY_LEVEL_0,
                    ..BUTTON_BORDER
                },
                border_hover_off: AppBorderStyle {
                    color: GRAY_LEVEL_0,
                    ..BUTTON_BORDER
                },
                border_down_off: AppBorderStyle {
                    color: GRAY_LEVEL_0,
                    ..BUTTON_BORDER
                },
                border_disabled_off: BUTTON_BORDER,

                border_idle_on: AppBorderStyle {
                    color: GRAY_LEVEL_0,
                    ..BUTTON_BORDER
                },
                border_hover_on: AppBorderStyle {
                    color: GRAY_LEVEL_0,
                    ..BUTTON_BORDER
                },
                border_down_on: AppBorderStyle {
                    color: GRAY_LEVEL_0,
                    ..BUTTON_BORDER
                },
                border_disabled_on: BUTTON_BORDER,

                font_color_idle_off: DIMMED_FONT_COLOR,
                font_color_hover_off: DIMMED_FONT_COLOR,
                font_color_down_off: DIMMED_FONT_COLOR,
                font_color_disabled_off: DIMMED_FONT_COLOR,

                font_color_idle_on: MAIN_FONT_COLOR,
                font_color_hover_on: MAIN_FONT_COLOR,
                font_color_down_on: MAIN_FONT_COLOR,
                font_color_disabled_on: DIMMED_FONT_COLOR,
            },

            top_panel_record_btn: AppToggleButtonStyle {
                bg_idle_off: Background::TRANSPARENT,
                bg_hover_off: Background::Solid(GRAY_LEVEL_5),
                bg_down_off: Background::TRANSPARENT,
                bg_disabled_off: Background::TRANSPARENT,

                bg_idle_on: Background::Solid(RECORD_COLOR_BG),
                bg_hover_on: Background::Solid(RECORD_COLOR_BG_HOVER),
                bg_down_on: Background::Solid(RECORD_COLOR_BG),
                bg_disabled_on: Background::TRANSPARENT,

                border_idle_off: BUTTON_BORDER,
                border_hover_off: AppBorderStyle {
                    color: GRAY_LEVEL_0,
                    ..BUTTON_BORDER
                },
                border_down_off: AppBorderStyle {
                    color: GRAY_LEVEL_0,
                    ..BUTTON_BORDER
                },
                border_disabled_off: BUTTON_BORDER,

                border_idle_on: AppBorderStyle {
                    color: RECORD_COLOR,
                    ..BUTTON_BORDER
                },
                border_hover_on: AppBorderStyle {
                    color: RECORD_COLOR,
                    ..BUTTON_BORDER
                },
                border_down_on: AppBorderStyle {
                    color: GRAY_LEVEL_0,
                    ..BUTTON_BORDER
                },
                border_disabled_on: BUTTON_BORDER,

                font_color_idle_off: RECORD_COLOR,
                font_color_hover_off: RECORD_COLOR,
                font_color_down_off: RECORD_COLOR,
                font_color_disabled_off: DIMMED_FONT_COLOR,

                font_color_idle_on: MAIN_FONT_COLOR,
                font_color_hover_on: MAIN_FONT_COLOR,
                font_color_down_on: MAIN_FONT_COLOR,
                font_color_disabled_on: DIMMED_FONT_COLOR,
            },

            top_panel_dropdown_btn: AppButtonStyle {
                bg_idle: Background::TRANSPARENT,
                bg_hover: Background::Solid(GRAY_LEVEL_5),
                bg_down: Background::Solid(GRAY_LEVEL_1),
                bg_disabled: Background::TRANSPARENT,

                border_idle: BUTTON_BORDER,
                border_hover: AppBorderStyle {
                    color: GRAY_LEVEL_0,
                    ..BUTTON_BORDER
                },
                border_down: AppBorderStyle {
                    color: GRAY_LEVEL_0,
                    ..BUTTON_BORDER
                },
                border_disabled: BUTTON_BORDER,

                font_color_idle: MAIN_FONT_COLOR,
                font_color_hover: BRIGHT_FONT_COLOR,
                font_color_down: BRIGHT_FONT_COLOR,
                font_color_disabled: DIMMED_FONT_COLOR,
            },

            top_panel_play_pause_btn: AppToggleButtonStyle {
                bg_idle_off: Background::TRANSPARENT,
                bg_hover_off: Background::Solid(GRAY_LEVEL_5),
                bg_down_off: Background::Solid(GRAY_LEVEL_1),
                bg_disabled_off: Background::TRANSPARENT,

                bg_idle_on: Background::TRANSPARENT,
                bg_hover_on: Background::Solid(GRAY_LEVEL_5),
                bg_down_on: Background::Solid(GRAY_LEVEL_1),
                bg_disabled_on: Background::TRANSPARENT,

                border_idle_off: BUTTON_BORDER,
                border_hover_off: AppBorderStyle {
                    color: GRAY_LEVEL_0,
                    ..BUTTON_BORDER
                },
                border_down_off: AppBorderStyle {
                    color: GRAY_LEVEL_0,
                    ..BUTTON_BORDER
                },
                border_disabled_off: BUTTON_BORDER,

                border_idle_on: BUTTON_BORDER,
                border_hover_on: AppBorderStyle {
                    color: GRAY_LEVEL_0,
                    ..BUTTON_BORDER
                },
                border_down_on: AppBorderStyle {
                    color: GRAY_LEVEL_0,
                    ..BUTTON_BORDER
                },
                border_disabled_on: BUTTON_BORDER,

                font_color_idle_off: MAIN_FONT_COLOR,
                font_color_hover_off: MAIN_FONT_COLOR,
                font_color_down_off: MAIN_FONT_COLOR,
                font_color_disabled_off: DIMMED_FONT_COLOR,

                font_color_idle_on: MAIN_FONT_COLOR,
                font_color_hover_on: MAIN_FONT_COLOR,
                font_color_down_on: MAIN_FONT_COLOR,
                font_color_disabled_on: DIMMED_FONT_COLOR,
            },

            top_panel_title_color: DIMMED_FONT_COLOR,
            top_panel_label_color: DIMMED_FONT_COLOR,
            top_panel_seperator_color: GRAY_LEVEL_1,

            top_panel_transport_box_bg: Background::Solid(GRAY_LEVEL_1),
            top_panel_transport_box_border: AppBorderStyle {
                radius: 4.0,
                width: 1.0,
                color: GRAY_LEVEL_0,
                ..Default::default()
            },
            top_panel_box_seperator_color: GRAY_LEVEL_3,
            top_panel_numeric_input_font_color: MAIN_FONT_COLOR,
            top_panel_numeric_input_font_highlight_color: MAIN_FONT_COLOR,
            top_panel_numeric_input_cursor_color: MAIN_FONT_COLOR,
            top_panel_numeric_input_highlight_bg_color: ACCENT_COLOR_1_BG,
            top_panel_numeric_input_active_border: AppBorderStyle {
                radius: 4.0,
                width: 1.0,
                color: DIMMED_FONT_COLOR,
                ..Default::default()
            },
        }
    }
}

const fn disabled_color(c: RGBA8, a: u8) -> RGBA8 {
    RGBA8::new(c.r, c.g, c.b, a)
}

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct AppBorderStyle {
    pub radius: f32,
    pub width: f32,
    pub color: RGBA8,
}

impl AppBorderStyle {
    pub const fn from_radius(radius: f32) -> Self {
        Self {
            radius,
            width: 0.0,
            color: color::TRANSPARENT,
        }
    }
}

impl Default for AppBorderStyle {
    fn default() -> Self {
        Self {
            radius: 0.0,
            width: 0.0,
            color: color::TRANSPARENT,
        }
    }
}

impl Into<BorderStyle> for AppBorderStyle {
    fn into(self) -> BorderStyle {
        BorderStyle {
            color: self.color,
            width: self.width,
            radius: self.radius.into(),
        }
    }
}
