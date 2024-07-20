use yarrow::{
    elements::{button::ButtonStylePart, icon_label_button::IconLabelButtonStylePart},
    prelude::*,
};

use super::AppBorderStyle;

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct AppButtonStyle {
    pub bg_idle: Background,
    pub bg_hover: Background,
    pub bg_down: Background,
    pub bg_disabled: Background,

    pub border_idle: AppBorderStyle,
    pub border_hover: AppBorderStyle,
    pub border_down: AppBorderStyle,
    pub border_disabled: AppBorderStyle,

    pub font_color_idle: RGBA8,
    pub font_color_hover: RGBA8,
    pub font_color_down: RGBA8,
    pub font_color_disabled: RGBA8,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct AppToggleButtonStyle {
    pub bg_idle_off: Background,
    pub bg_hover_off: Background,
    pub bg_disabled_off: Background,

    pub bg_idle_on: Background,
    pub bg_hover_on: Background,
    pub bg_disabled_on: Background,

    pub border_idle_off: AppBorderStyle,
    pub border_hover_off: AppBorderStyle,
    pub border_disabled_off: AppBorderStyle,

    pub border_idle_on: AppBorderStyle,
    pub border_hover_on: AppBorderStyle,
    pub border_disabled_on: AppBorderStyle,

    pub font_color_idle_off: RGBA8,
    pub font_color_hover_off: RGBA8,
    pub font_color_disabled_off: RGBA8,

    pub font_color_idle_on: RGBA8,
    pub font_color_hover_on: RGBA8,
    pub font_color_disabled_on: RGBA8,
}

impl AppButtonStyle {
    pub fn into_button_style(&self, properties: TextProperties, padding: Padding) -> ButtonStyle {
        ButtonStyle {
            properties,
            padding,
            idle: ButtonStylePart {
                font_color: self.font_color_idle,
                back_quad: QuadStyle {
                    bg: self.bg_idle.clone(),
                    border: self.border_idle.into(),
                },
            },
            hovered: ButtonStylePart {
                font_color: self.font_color_hover,
                back_quad: QuadStyle {
                    bg: self.bg_hover.clone(),
                    border: self.border_hover.into(),
                },
            },
            down: ButtonStylePart {
                font_color: self.font_color_down,
                back_quad: QuadStyle {
                    bg: self.bg_down.clone(),
                    border: self.border_down.into(),
                },
            },
            disabled: ButtonStylePart {
                font_color: self.font_color_disabled,
                back_quad: QuadStyle {
                    bg: self.bg_disabled.clone(),
                    border: self.border_disabled.into(),
                },
            },
            ..Default::default()
        }
    }

    pub fn into_icon_button_style(&self, size: f32, padding: Padding) -> IconButtonStyle {
        IconButtonStyle {
            size,
            padding,
            idle: ButtonStylePart {
                font_color: self.font_color_idle,
                back_quad: QuadStyle {
                    bg: self.bg_idle.clone(),
                    border: self.border_idle.into(),
                },
            },
            hovered: ButtonStylePart {
                font_color: self.font_color_hover,
                back_quad: QuadStyle {
                    bg: self.bg_hover.clone(),
                    border: self.border_hover.into(),
                },
            },
            down: ButtonStylePart {
                font_color: self.font_color_down,
                back_quad: QuadStyle {
                    bg: self.bg_down.clone(),
                    border: self.border_down.into(),
                },
            },
            disabled: ButtonStylePart {
                font_color: self.font_color_disabled,
                back_quad: QuadStyle {
                    bg: self.bg_disabled.clone(),
                    border: self.border_disabled.into(),
                },
            },
        }
    }

    pub fn into_dropdown_style(
        &self,
        text_properties: TextProperties,
        icon_size: f32,
        text_padding: Padding,
        icon_padding: Padding,
    ) -> IconLabelButtonStyle {
        IconLabelButtonStyle {
            text_properties,
            icon_size,
            text_padding,
            icon_padding,
            idle: IconLabelButtonStylePart {
                text_color: self.font_color_idle,
                icon_color: self.font_color_idle,
                back_quad: QuadStyle {
                    bg: self.bg_idle.clone(),
                    border: self.border_idle.into(),
                },
            },
            hovered: IconLabelButtonStylePart {
                text_color: self.font_color_hover,
                icon_color: self.font_color_hover,
                back_quad: QuadStyle {
                    bg: self.bg_hover.clone(),
                    border: self.border_hover.into(),
                },
            },
            down: IconLabelButtonStylePart {
                text_color: self.font_color_down,
                icon_color: self.font_color_down,
                back_quad: QuadStyle {
                    bg: self.bg_down.clone(),
                    border: self.border_down.into(),
                },
            },
            disabled: IconLabelButtonStylePart {
                text_color: self.font_color_disabled,
                icon_color: self.font_color_disabled,
                back_quad: QuadStyle {
                    bg: self.bg_disabled.clone(),
                    border: self.border_disabled.into(),
                },
            },
            layout: IconLabelLayout::LeftAlignTextRightAlignIcon,
            ..Default::default()
        }
    }
}

impl AppToggleButtonStyle {
    pub fn into_icon_toggle_button_style(
        &self,
        size: f32,
        padding: Padding,
    ) -> IconToggleButtonStyle {
        IconToggleButtonStyle {
            size,
            padding,
            idle_off: ButtonStylePart {
                font_color: self.font_color_idle_off,
                back_quad: QuadStyle {
                    bg: self.bg_idle_off.clone(),
                    border: self.border_idle_off.into(),
                },
            },
            hovered_off: ButtonStylePart {
                font_color: self.font_color_hover_off,
                back_quad: QuadStyle {
                    bg: self.bg_hover_off.clone(),
                    border: self.border_hover_off.into(),
                },
            },
            disabled_off: ButtonStylePart {
                font_color: self.font_color_disabled_off,
                back_quad: QuadStyle {
                    bg: self.bg_disabled_off.clone(),
                    border: self.border_disabled_off.into(),
                },
            },
            idle_on: ButtonStylePart {
                font_color: self.font_color_idle_on,
                back_quad: QuadStyle {
                    bg: self.bg_idle_on.clone(),
                    border: self.border_idle_on.into(),
                },
            },
            hovered_on: ButtonStylePart {
                font_color: self.font_color_hover_on,
                back_quad: QuadStyle {
                    bg: self.bg_hover_on.clone(),
                    border: self.border_hover_on.into(),
                },
            },
            disabled_on: ButtonStylePart {
                font_color: self.font_color_disabled_on,
                back_quad: QuadStyle {
                    bg: self.bg_disabled_on.clone(),
                    border: self.border_disabled_on.into(),
                },
            },
            ..Default::default()
        }
    }
}
