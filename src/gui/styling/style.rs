use std::rc::Rc;
use yarrow::{prelude::*, vg::text::Metrics};

use super::theme::AppTheme;

pub struct AppStyle {
    pub clear_color: RGBA8,

    pub top_panel_height: f32,
    pub top_panel_section_padding: f32,
    pub top_panel_element_spacing: f32,
    pub top_panel_padding_bottom: f32,
    pub top_panel_bg: Rc<QuadStyle>,
    pub top_panel_icon_btn: Rc<IconButtonStyle>,
    pub top_panel_toggle_btn: Rc<IconToggleButtonStyle>,
    pub top_panel_section_title_label: Rc<LabelStyle>,
    pub top_panel_seperator: Rc<SeparatorStyle>,
    pub top_panel_seperator_width: f32,
    pub top_panel_section_seperator_width: f32,
    pub top_panel_record_btn: Rc<IconToggleButtonStyle>,
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
        let icon_btn_size = 22.0;
        let icon_btn_padding = Padding::new(4.0, 4.0, 4.0, 4.0);
        let text_btn_padding = Padding::new(8.0, 6.0, 8.0, 6.0);

        let text_properties = TextProperties {
            metrics: Metrics {
                font_size: 13.0,
                line_height: 13.0,
            },
            attrs: Attrs::new().family(Family::SansSerif),
            ..Default::default()
        };

        Self {
            clear_color: theme.clear_color,

            top_panel_height: 52.0,
            top_panel_section_padding: 12.0,
            top_panel_element_spacing: 3.0,
            top_panel_padding_bottom: 4.0,
            top_panel_bg: Rc::new(QuadStyle {
                bg: Background::Solid(theme.top_panel_bg_color),
                border: theme.top_panel_border.into(),
            }),
            top_panel_icon_btn: Rc::new(
                theme
                    .top_panel_button
                    .as_icon_button_style(icon_btn_size, icon_btn_padding),
            ),
            top_panel_toggle_btn: Rc::new(
                theme
                    .top_panel_toggle_btn
                    .as_icon_toggle_button_style(icon_btn_size, icon_btn_padding),
            ),
            top_panel_record_btn: Rc::new(
                theme
                    .top_panel_record_btn
                    .as_icon_toggle_button_style(icon_btn_size, icon_btn_padding),
            ),
            top_panel_play_pause_btn: Rc::new(
                theme
                    .top_panel_play_pause_btn
                    .as_icon_toggle_button_style(icon_btn_size, icon_btn_padding),
            ),
            top_panel_section_title_label: Rc::new(LabelStyle {
                properties: TextProperties {
                    metrics: Metrics {
                        font_size: 11.0,
                        line_height: 11.0,
                    },
                    ..Default::default()
                },
                font_color: theme.top_panel_title_color,
                padding: Padding::new(3.0, 16.0, 3.0, 16.0),
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
            top_panel_seperator_width: 1.0,
            top_panel_section_seperator_width: 1.0,
            top_panel_transport_box: Rc::new(QuadStyle {
                bg: Background::Solid(theme.top_panel_transport_box_bg_color),
                border: theme.top_panel_transport_box_border.into(),
            }),
            top_panel_numeric_text_input: Rc::new(
                theme.top_panel_numeric_input.as_text_input_style(
                    text_properties,
                    Attrs::new().family(Family::SansSerif),
                    Padding::new(0.0, 3.0, 0.0, 3.0),
                    Padding::new(1.0, 0.0, 1.0, 0.0),
                ),
            ),
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
                properties: text_properties.clone(),
                padding: text_btn_padding,
                font_color: theme.tootlip_font_color,
                back_quad: QuadStyle {
                    bg: Background::Solid(theme.tooltip_bg_color),
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
