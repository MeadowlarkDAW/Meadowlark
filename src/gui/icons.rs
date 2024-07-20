use yarrow::{
    prelude::ResourceCtx,
    vg::text::{ContentType, CustomGlyphID},
};

/// The icons used by the application.
///
/// Icons are sorted in alphabetical order.
#[repr(u16)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, enum_iterator::Sequence)]
pub enum AppIcon {
    Automation,
    AutoReturn,
    Browser,
    ClipsPanel,
    CommandPalette,
    CPU,
    DropdownArrow,
    FXRack,
    Loop,
    Menu,
    Metronome,
    Mic,
    Mixer,
    Open,
    Pause,
    PianoKeys,
    Play,
    Properties,
    Record,
    Redo,
    SaveAs,
    Save,
    SkipBack,
    SkipForward,
    Stop,
    Tracks,
    Undo,
}

impl AppIcon {
    fn is_symbolic(&self) -> bool {
        match self {
            _ => true,
        }
    }

    #[rustfmt::skip]
    fn source(&self) -> &'static [u8] {
        match self {
            Self::Automation => include_bytes!("../../assets/icons/automation-24.svg"),
            Self::AutoReturn => include_bytes!("../../assets/icons/auto-return-24.svg"),
            Self::Browser => include_bytes!("../../assets/icons/browser-24.svg"),
            Self::ClipsPanel => include_bytes!("../../assets/icons/clips-panel-24.svg"),
            Self::CommandPalette => include_bytes!("../../assets/icons/command-palette-24.svg"),
            Self::CPU => include_bytes!("../../assets/icons/cpu-24.svg"),
            Self::DropdownArrow => include_bytes!("../../assets/icons/dropdown-arrow-24.svg"),
            Self::FXRack => include_bytes!("../../assets/icons/fx-rack-24.svg"),
            Self::Loop => include_bytes!("../../assets/icons/loop-24.svg"),
            Self::Menu => include_bytes!("../../assets/icons/menu-24.svg"),
            Self::Metronome => include_bytes!("../../assets/icons/metronome-24.svg"),
            Self::Mic => include_bytes!("../../assets/icons/mic-24.svg"),
            Self::Mixer => include_bytes!("../../assets/icons/mixer-24.svg"),
            Self::Open => include_bytes!("../../assets/icons/open-24.svg"),
            Self::Pause => include_bytes!("../../assets/icons/pause-24.svg"),
            Self::PianoKeys => include_bytes!("../../assets/icons/piano-keys-24.svg"),
            Self::Play => include_bytes!("../../assets/icons/play-24.svg"),
            Self::Properties => include_bytes!("../../assets/icons/properties-24.svg"),
            Self::Record => include_bytes!("../../assets/icons/record-24.svg"),
            Self::Redo => include_bytes!("../../assets/icons/redo-24.svg"),
            Self::SaveAs => include_bytes!("../../assets/icons/save-as-24.svg"),
            Self::Save => include_bytes!("../../assets/icons/save-24.svg"),
            Self::SkipBack => include_bytes!("../../assets/icons/skip-back-24.svg"),
            Self::SkipForward => include_bytes!("../../assets/icons/skip-forward-24.svg"),
            Self::Stop => include_bytes!("../../assets/icons/stop-24.svg"),
            Self::Tracks => include_bytes!("../../assets/icons/tracks-24.svg"),
            Self::Undo => include_bytes!("../../assets/icons/undo-24.svg"),
        }
    }
}

impl Into<CustomGlyphID> for AppIcon {
    fn into(self) -> CustomGlyphID {
        self as CustomGlyphID
    }
}

pub fn load_icons(res: &mut ResourceCtx) {
    let opt = yarrow::vg::text::svg::resvg::usvg::Options {
        default_size: yarrow::vg::text::svg::resvg::tiny_skia::Size::from_wh(24.0, 24.0).unwrap(),
        ..Default::default()
    };

    // TODO: Load icons from theme folder.
    for icon in enum_iterator::all::<AppIcon>() {
        let data = icon.source();
        let content_type = if icon.is_symbolic() {
            ContentType::Mask
        } else {
            ContentType::Color
        };

        res.svg_icon_system
            .add_from_bytes(icon, data, &opt, content_type)
            .unwrap();
    }
}
