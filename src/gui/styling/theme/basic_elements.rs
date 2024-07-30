use yarrow::{
    elements::{button::ButtonStylePart, icon_label_button::IconLabelButtonStylePart},
    prelude::*,
};

use super::AppBorderStyle;

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct AppButtonStyle {
    pub font_color_idle: RGBA8,
    pub font_color_hover: RGBA8,
    pub font_color_down: RGBA8,
    pub font_color_disabled: RGBA8,

    pub bg_color_idle: RGBA8,
    pub bg_color_hover: RGBA8,
    pub bg_color_down: RGBA8,
    pub bg_color_disabled: RGBA8,

    pub border_idle: AppBorderStyle,
    pub border_hover: AppBorderStyle,
    pub border_down: AppBorderStyle,
    pub border_disabled: AppBorderStyle,
}

impl AppButtonStyle {
    pub fn as_button_style(&self, properties: TextProperties, padding: Padding) -> ButtonStyle {
        ButtonStyle {
            properties,
            padding,
            idle: ButtonStylePart {
                back_quad: QuadStyle {
                    bg: Background::Solid(self.bg_color_idle),
                    border: self.border_idle.into(),
                },
                font_color: self.font_color_idle,
            },
            hovered: ButtonStylePart {
                back_quad: QuadStyle {
                    bg: Background::Solid(self.bg_color_hover),
                    border: self.border_hover.into(),
                },
                font_color: self.font_color_hover,
            },
            down: ButtonStylePart {
                back_quad: QuadStyle {
                    bg: Background::Solid(self.bg_color_down),
                    border: self.border_down.into(),
                },
                font_color: self.font_color_down,
            },
            disabled: ButtonStylePart {
                back_quad: QuadStyle {
                    bg: Background::Solid(self.bg_color_disabled),
                    border: self.border_disabled.into(),
                },
                font_color: self.font_color_disabled,
            },
            ..Default::default()
        }
    }

    pub fn as_icon_button_style(&self, size: f32, padding: Padding) -> IconButtonStyle {
        IconButtonStyle {
            size,
            padding,
            idle: ButtonStylePart {
                back_quad: QuadStyle {
                    bg: Background::Solid(self.bg_color_idle),
                    border: self.border_idle.into(),
                },
                font_color: self.font_color_idle,
            },
            hovered: ButtonStylePart {
                back_quad: QuadStyle {
                    bg: Background::Solid(self.bg_color_hover),
                    border: self.border_hover.into(),
                },
                font_color: self.font_color_hover,
            },
            down: ButtonStylePart {
                back_quad: QuadStyle {
                    bg: Background::Solid(self.bg_color_down),
                    border: self.border_down.into(),
                },
                font_color: self.font_color_down,
            },
            disabled: ButtonStylePart {
                back_quad: QuadStyle {
                    bg: Background::Solid(self.bg_color_disabled),
                    border: self.border_disabled.into(),
                },
                font_color: self.font_color_disabled,
            },
            ..Default::default()
        }
    }

    pub fn as_icon_label_button_style(
        &self,
        icon_size: f32,
        text_properties: TextProperties,
        icon_padding: Padding,
        text_padding: Padding,
        layout: IconLabelLayout,
    ) -> IconLabelButtonStyle {
        IconLabelButtonStyle {
            icon_size,
            text_properties,
            icon_padding,
            text_padding,
            layout,
            idle: IconLabelButtonStylePart {
                back_quad: QuadStyle {
                    bg: Background::Solid(self.bg_color_idle),
                    border: self.border_idle.into(),
                },
                text_color: self.font_color_idle,
                icon_color: self.font_color_idle,
            },
            hovered: IconLabelButtonStylePart {
                back_quad: QuadStyle {
                    bg: Background::Solid(self.bg_color_hover),
                    border: self.border_hover.into(),
                },
                text_color: self.font_color_hover,
                icon_color: self.font_color_hover,
            },
            down: IconLabelButtonStylePart {
                back_quad: QuadStyle {
                    bg: Background::Solid(self.bg_color_down),
                    border: self.border_down.into(),
                },
                text_color: self.font_color_down,
                icon_color: self.font_color_down,
            },
            disabled: IconLabelButtonStylePart {
                back_quad: QuadStyle {
                    bg: Background::Solid(self.bg_color_disabled),
                    border: self.border_disabled.into(),
                },
                text_color: self.font_color_disabled,
                icon_color: self.font_color_disabled,
            },
            ..Default::default()
        }
    }
}

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct AppToggleButtonStyle {
    pub font_color_idle_off: RGBA8,
    pub font_color_hover_off: RGBA8,
    pub font_color_down_off: RGBA8,
    pub font_color_disabled_off: RGBA8,

    pub font_color_idle_on: RGBA8,
    pub font_color_hover_on: RGBA8,
    pub font_color_down_on: RGBA8,
    pub font_color_disabled_on: RGBA8,

    pub bg_color_idle_off: RGBA8,
    pub bg_color_hover_off: RGBA8,
    pub bg_color_down_off: RGBA8,
    pub bg_color_disabled_off: RGBA8,

    pub bg_color_idle_on: RGBA8,
    pub bg_color_hover_on: RGBA8,
    pub bg_color_down_on: RGBA8,
    pub bg_color_disabled_on: RGBA8,

    pub border_idle_off: AppBorderStyle,
    pub border_hover_off: AppBorderStyle,
    pub border_down_off: AppBorderStyle,
    pub border_disabled_off: AppBorderStyle,

    pub border_idle_on: AppBorderStyle,
    pub border_hover_on: AppBorderStyle,
    pub border_down_on: AppBorderStyle,
    pub border_disabled_on: AppBorderStyle,
}

impl AppToggleButtonStyle {
    pub fn as_icon_toggle_button_style(
        &self,
        size: f32,
        padding: Padding,
    ) -> IconToggleButtonStyle {
        IconToggleButtonStyle {
            size,
            padding,
            idle_on: ButtonStylePart {
                font_color: self.font_color_idle_on,
                back_quad: QuadStyle {
                    bg: Background::Solid(self.bg_color_idle_on),
                    border: self.border_idle_on.into(),
                },
            },
            hovered_on: ButtonStylePart {
                font_color: self.font_color_hover_on,
                back_quad: QuadStyle {
                    bg: Background::Solid(self.bg_color_hover_on),
                    border: self.border_hover_on.into(),
                },
            },
            down_on: ButtonStylePart {
                font_color: self.font_color_down_on,
                back_quad: QuadStyle {
                    bg: Background::Solid(self.bg_color_down_on),
                    border: self.border_down_on.into(),
                },
            },
            disabled_on: ButtonStylePart {
                font_color: self.font_color_disabled_on,
                back_quad: QuadStyle {
                    bg: Background::Solid(self.bg_color_disabled_on),
                    border: self.border_disabled_on.into(),
                },
            },
            idle_off: ButtonStylePart {
                font_color: self.font_color_idle_off,
                back_quad: QuadStyle {
                    bg: Background::Solid(self.bg_color_idle_off),
                    border: self.border_idle_off.into(),
                },
            },
            hovered_off: ButtonStylePart {
                font_color: self.font_color_hover_off,
                back_quad: QuadStyle {
                    bg: Background::Solid(self.bg_color_hover_off),
                    border: self.border_hover_off.into(),
                },
            },
            down_off: ButtonStylePart {
                font_color: self.font_color_down_off,
                back_quad: QuadStyle {
                    bg: Background::Solid(self.bg_color_down_off),
                    border: self.border_down_off.into(),
                },
            },
            disabled_off: ButtonStylePart {
                font_color: self.font_color_disabled_off,
                back_quad: QuadStyle {
                    bg: Background::Solid(self.bg_color_disabled_off),
                    border: self.border_disabled_off.into(),
                },
            },
        }
    }
}

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct AppTextInputStyle {
    pub bg_idle: RGBA8,
    pub bg_hover: RGBA8,
    pub bg_focused: RGBA8,
    pub bg_disabled: RGBA8,

    pub font_color_idle: RGBA8,
    pub font_color_hover: RGBA8,
    pub font_color_focused: RGBA8,
    pub font_color_disabled: RGBA8,
    pub font_color_placeholder: RGBA8,
    pub font_color_highlighted: RGBA8,

    pub border_idle: AppBorderStyle,
    pub border_hover: AppBorderStyle,
    pub border_focused: AppBorderStyle,
    pub border_disabled: AppBorderStyle,

    pub highlight_bg_color: RGBA8,
    pub cusor_color: RGBA8,
}

impl AppTextInputStyle {
    pub fn as_text_input_style(
        &self,
        properties: TextProperties,
        placeholder_text_attrs: Attrs<'static>,
        padding: Padding,
        highlight_padding: Padding,
    ) -> TextInputStyle {
        TextInputStyle {
            properties,
            placeholder_text_attrs,
            font_color: self.font_color_idle,
            font_color_placeholder: self.font_color_placeholder,
            font_color_disabled: self.font_color_disabled,
            font_color_highlighted: self.font_color_highlighted,
            highlight_bg_color: self.highlight_bg_color,
            cursor_width: 1.0,
            cursor_color: self.cusor_color,
            padding,
            highlight_padding,
            back_quad_unfocused: QuadStyle {
                bg: Background::Solid(self.bg_idle),
                border: self.border_idle.into(),
            },
            back_quad_focused: QuadStyle {
                bg: Background::Solid(self.bg_focused),
                border: self.border_focused.into(),
            },
            back_quad_disabled: QuadStyle {
                bg: Background::Solid(self.bg_disabled),
                border: self.border_disabled.into(),
            },
            ..Default::default()
        }
    }
}
