use vizia::prelude::*;
use vizia::style::Color;

// TODO: Make this themeable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Data)]
pub enum TrackColor {
    Unassigned,
    Color0,
    Color1,
    Color2,
    // TODO: More colors (perhaps 16 or more colors?)
}

impl TrackColor {
    pub fn into_color(&self) -> Color {
        match self {
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

#[derive(Debug, Lens, Clone, Data)]
pub struct TrackHeaderState {
    pub name: String,
    pub color: TrackColor,
    pub height: f32,
    pub type_: TrackType,
}
