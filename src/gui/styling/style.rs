use std::rc::Rc;
use yarrow::{
    prelude::*,
    vg::{color, text::Metrics},
};

use super::theme::AppTheme;

pub struct AppStyle {
    pub clear_color: RGBA8,

    pub top_panel_bg: Rc<QuadStyle>,
    pub top_panel_height: f32,
    pub top_panel_icon_btn: Rc<IconButtonStyle>,
    pub top_panel_toggle_btn: Rc<IconToggleButtonStyle>,
    pub top_panel_section_title_label: Rc<LabelStyle>,
    pub top_panel_seperator: Rc<SeparatorStyle>,
    pub top_panel_seperator_width: f32,
    pub top_panel_section_seperator_width: f32,
    pub top_panel_record_btn: Rc<IconToggleButtonStyle>,
    pub top_panel_dropdown_btn: Rc<IconLabelButtonStyle>,
    pub top_panel_label: Rc<LabelStyle>,
    pub top_panel_play_pause_btn: Rc<IconToggleButtonStyle>,
    pub top_panel_transport_box: Rc<QuadStyle>,
    pub top_panel_box_separator: Rc<SeparatorStyle>,
    pub top_panel_numeric_text_input: Rc<TextInputStyle>,
    pub top_panel_icon: Rc<IconStyle>,

    pub tooltip: Rc<LabelStyle>,

    #[cfg(debug_assertions)]
    pub dev_icon: Rc<IconStyle>,
}

impl AppStyle {
    pub fn new(theme: AppTheme) -> Self {
        let icon_btn_size = 24.0;
        let icon_btn_padding = Padding::new(2.0, 2.0, 2.0, 2.0);
        let text_btn_padding = Padding::new(8.0, 6.0, 8.0, 6.0);

        let btn_text_properties = TextProperties {
            metrics: Metrics {
                font_size: 13.0,
                line_height: 13.0,
            },
            ..Default::default()
        };

        let top_panel_numeric_input_props = TextProperties {
            metrics: Metrics {
                font_size: 13.0,
                line_height: 13.0,
            },
            attrs: Attrs::new().family(Family::Monospace),
            ..Default::default()
        };

        Self {
            clear_color: theme.clear_color,

            top_panel_bg: Rc::new(QuadStyle {
                bg: theme.top_panel_background,
                border: theme.top_panel_border.into(),
            }),
            top_panel_height: 50.0,

            top_panel_icon_btn: Rc::new(
                theme
                    .top_panel_button
                    .into_icon_button_style(icon_btn_size, icon_btn_padding),
            ),
            top_panel_toggle_btn: Rc::new(
                theme
                    .top_panel_toggle_btn
                    .into_icon_toggle_button_style(icon_btn_size, icon_btn_padding),
            ),
            top_panel_record_btn: Rc::new(
                theme
                    .top_panel_record_btn
                    .into_icon_toggle_button_style(icon_btn_size, icon_btn_padding),
            ),
            top_panel_play_pause_btn: Rc::new(
                theme
                    .top_panel_play_pause_btn
                    .into_icon_toggle_button_style(icon_btn_size, icon_btn_padding),
            ),
            top_panel_section_title_label: Rc::new(LabelStyle {
                properties: TextProperties {
                    metrics: Metrics {
                        font_size: 10.0,
                        line_height: 10.0,
                    },
                    ..Default::default()
                },
                font_color: theme.top_panel_title_color,
                padding: Padding::new(4.0, 16.0, 4.0, 16.0),
                ..Default::default()
            }),
            top_panel_label: Rc::new(LabelStyle {
                properties: TextProperties {
                    metrics: Metrics {
                        font_size: 12.0,
                        line_height: 12.0,
                    },
                    align: Some(yarrow::vg::text::Align::End),
                    ..Default::default()
                },
                font_color: theme.top_panel_label_color,
                padding: Padding::new(7.5, 0.0, 7.5, 6.0),
                ..Default::default()
            }),
            top_panel_seperator: Rc::new(SeparatorStyle {
                quad_style: QuadStyle {
                    bg: Background::Solid(theme.top_panel_seperator_color),
                    border: BorderStyle::TRANSPARENT,
                },
                size: SeparatorSizeType::Scale(0.75),
                align: Align::Center,
            }),
            top_panel_dropdown_btn: Rc::new(theme.top_panel_dropdown_btn.into_dropdown_style(
                btn_text_properties.clone(),
                icon_btn_size,
                text_btn_padding,
                Padding::new(0.0, 4.0, 0.0, 0.0),
            )),
            top_panel_seperator_width: 1.0,
            top_panel_section_seperator_width: 1.0,
            top_panel_transport_box: Rc::new(QuadStyle {
                bg: theme.top_panel_transport_box_bg,
                border: theme.top_panel_transport_box_border.into(),
            }),
            top_panel_numeric_text_input: Rc::new(TextInputStyle {
                properties: top_panel_numeric_input_props,
                placeholder_text_attrs: Attrs::new().family(Family::Monospace),
                font_color: theme.top_panel_numeric_input_font_color,
                font_color_placeholder: color::TRANSPARENT,
                font_color_disabled: theme.top_panel_numeric_input_font_color,
                font_color_highlighted: theme.top_panel_numeric_input_font_highlight_color,
                highlight_bg_color: theme.top_panel_numeric_input_highlight_bg_color,
                cursor_color: theme.top_panel_numeric_input_cursor_color,
                padding: Padding::new(0.0, 3.0, 0.0, 3.0),
                back_quad_disabled: QuadStyle::TRANSPARENT,
                back_quad_focused: QuadStyle {
                    bg: Background::TRANSPARENT,
                    border: theme.top_panel_numeric_input_active_border.into(),
                },
                back_quad_unfocused: QuadStyle::TRANSPARENT,
                ..Default::default()
            }),
            top_panel_box_separator: Rc::new(SeparatorStyle {
                quad_style: QuadStyle {
                    bg: Background::Solid(theme.top_panel_box_seperator_color),
                    border: BorderStyle::TRANSPARENT,
                },
                size: SeparatorSizeType::Scale(0.75),
                align: Align::Center,
            }),
            top_panel_icon: Rc::new(IconStyle {
                size: icon_btn_size,
                color: theme.top_panel_label_color,
                back_quad: QuadStyle::TRANSPARENT,
                padding: icon_btn_padding,
            }),

            tooltip: Rc::new(LabelStyle {
                properties: btn_text_properties.clone(),
                padding: text_btn_padding,
                font_color: theme.tootlip_font_color,
                back_quad: QuadStyle {
                    bg: theme.tooltip_background,
                    border: theme.tooltip_border.into(),
                },
                ..Default::default()
            }),

            #[cfg(debug_assertions)]
            dev_icon: Rc::new(IconStyle {
                size: icon_btn_size,
                color: theme.dev_icon_color,
                back_quad: QuadStyle::TRANSPARENT,
                padding: icon_btn_padding,
            }),
        }
    }
}
