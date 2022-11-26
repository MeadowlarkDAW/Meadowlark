use vizia::prelude::*;
use vizia::style::Color;

use crate::ui::views::{Icon, IconCode};

// TODO: Make this themeable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Data)]
pub enum TrackColor {
    MasterTrack,
    Unassigned,
    Color0,
    Color1,
    Color2,
    // TODO: More colors (perhaps 16 or more colors?)
}

impl TrackColor {
    pub fn into_color(&self) -> Color {
        match self {
            TrackColor::MasterTrack => Color::rgb(0xba, 0xb9, 0xba),
            TrackColor::Unassigned => Color::rgb(0xba, 0xb9, 0xba),
            TrackColor::Color0 => Color::rgb(0xeb, 0x70, 0x71),
            TrackColor::Color1 => Color::rgb(0xeb, 0xe3, 0x71),
            TrackColor::Color2 => Color::rgb(0x00, 0x8e, 0xaa),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Data)]
pub enum TrackType {
    Master,
    Audio,
    Synth,
}

pub struct TrackHeader {}

impl TrackHeader {
    pub fn new<'a>(
        cx: &'a mut Context,
        color: TrackColor,
        height: f32,
        name: String,
        track_type: TrackType,
    ) -> Handle<'a, Self> {
        Self {}
            .build(cx, |cx| {
                HStack::new(cx, |cx| {
                    HStack::new(cx, |cx| {
                        Label::new(cx, IconCode::GripVertical)
                            .font_size(19.0)
                            .top(Stretch(1.0))
                            .bottom(Stretch(1.0))
                            .width(Pixels(20.0))
                            .height(Pixels(20.0))
                            .position_type(PositionType::SelfDirected)
                            .class("grip")
                            .font("meadowlark-icons");
                    })
                    .width(Pixels(20.0))
                    .background_color(color.into_color());

                    VStack::new(cx, |cx| {
                        Label::new(cx, &name).class("name");

                        // TODO: Fix icon sizes,
                        let (icon, icon_size) = match track_type {
                            TrackType::Master => (IconCode::MasterTrack, 20.0),
                            TrackType::Audio => (IconCode::Soundwave, 20.0),
                            TrackType::Synth => (IconCode::Piano, 16.0),
                        };

                        Icon::new(cx, icon, 21.0, icon_size).top(Stretch(1.0)).bottom(Stretch(1.0));
                    })
                    .left(Pixels(2.0))
                    .child_space(Pixels(4.0));

                    VStack::new(cx, |cx| {
                        HStack::new(cx, |cx| {
                            HStack::new(cx, |cx| {
                                Button::new(
                                    cx,
                                    |_| {},
                                    |cx| Icon::new(cx, IconCode::Record, 18.0, 16.0),
                                )
                                .class("icon_btn");
                            })
                            .class("button_group")
                            .height(Pixels(20.0))
                            .width(Auto);

                            HStack::new(cx, |cx| {
                                Button::new(
                                    cx,
                                    |_| {},
                                    |cx| Label::new(cx, "S").bottom(Pixels(3.0)),
                                )
                                .child_space(Stretch(1.0))
                                .width(Pixels(20.0))
                                .height(Pixels(20.0))
                                .class("solo_btn");

                                Element::new(cx).class("button_group_separator");

                                Button::new(
                                    cx,
                                    |_| {},
                                    |cx| Label::new(cx, "M").bottom(Pixels(3.0)),
                                )
                                .child_space(Stretch(1.0))
                                .width(Pixels(20.0))
                                .height(Pixels(20.0))
                                .class("mute_btn");
                            })
                            .class("button_group")
                            .left(Pixels(5.0))
                            .height(Pixels(20.0))
                            .width(Auto);
                        })
                        .bottom(Stretch(1.0));
                    })
                    .width(Auto)
                    .left(Stretch(1.0))
                    .right(Pixels(3.0));

                    // TODO: Make decibel meter widget.
                    Element::new(cx).width(Pixels(12.0)).class("db_meter").space(Pixels(4.0));
                });
            })
            .height(Pixels(height))
    }
}

impl View for TrackHeader {
    fn element(&self) -> Option<&'static str> {
        Some("trackheader")
    }
}
