use vizia::style::Color;

// TODO: Make this themeable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaletteColor {
    Unassigned,
    Color0,
    Color1,
    Color2,
    // TODO: More colors (perhaps 16 or more colors?)
}

impl PaletteColor {
    pub fn into_color(&self) -> Color {
        match self {
            PaletteColor::Unassigned => Color::rgb(0xba, 0xb9, 0xba),
            PaletteColor::Color0 => Color::rgb(0xeb, 0x70, 0x71),
            PaletteColor::Color1 => Color::rgb(0xeb, 0xe3, 0x71),
            PaletteColor::Color2 => Color::rgb(0x00, 0x8e, 0xaa),
        }
    }
}
