use vizia::prelude::*;

pub struct Icon {}

impl Icon {
    // Creates an Icon with a set size for the outer frame and the icon.
    pub fn new<'a>(
        cx: &'a mut Context,
        icon: IconCode,
        frame_size: f32,
        icon_size: f32,
    ) -> Handle<'a, Self> {
        Self {}
            .build(cx, |cx| {
                let icon_str: &str = icon.into();

                let mut icon_sz = icon_size;

                // Icon can't be bigger than the frame it's held in.
                if icon_size > frame_size {
                    icon_sz = frame_size;
                }

                Label::new(cx, icon_str)
                    .width(Pixels(frame_size))
                    .height(Pixels(frame_size))
                    .font_size(icon_sz)
                    .child_space(Stretch(1.0))
                    .font("meadowlark-icons");
            })
            .size(Auto)
    }
}

impl View for Icon {}

pub enum IconCode {
    Save,
    Undo,
    Redo,
    Loop,
    ReturnPlayhead,
    Play,
    Pause,
    Stop,
    Record,
    Menu,
    MenuSmall,
    Folder,
    PianoRoll,
    Properties,
    Audio,
    Instrument,
    Synth,
    FX,
    Midi,
    Automation,
    Music,
    SoundHigh,
    SoundMed,
    SoundLow,
    SoundOff,
    Home,
    Terminal,
    Search,
}

impl From<IconCode> for &'static str {
    fn from(icon: IconCode) -> Self {
        match icon {
            IconCode::Save => "\u{0001}",
            IconCode::Undo => "\u{0002}",
            IconCode::Redo => "\u{0003}",
            IconCode::Loop => "\u{0004}",
            IconCode::ReturnPlayhead => "\u{0005}",
            IconCode::Play => "\u{0006}",
            IconCode::Pause => "\u{0007}",
            IconCode::Stop => "\u{0008}",
            IconCode::Record => "\u{0009}",
            IconCode::Menu => "\u{000A}",
            IconCode::MenuSmall => "\u{000B}",
            IconCode::Folder => "\u{000C}",
            IconCode::PianoRoll => "\u{0026}",
            IconCode::Properties => "\u{000E}",
            IconCode::Audio => "\u{000F}",
            IconCode::Instrument => "\u{0010}",
            IconCode::Synth => "\u{0011}",
            IconCode::FX => "\u{0012}",
            IconCode::Midi => "\u{0013}",
            IconCode::Automation => "\u{0014}",
            IconCode::Music => "\u{0015}",
            IconCode::SoundHigh => "\u{0016}",
            IconCode::SoundMed => "\u{0017}",
            IconCode::SoundLow => "\u{0018}",
            IconCode::SoundOff => "\u{0019}",
            IconCode::Home => "\u{001A}",
            IconCode::Terminal => "\u{001B}",
            IconCode::Search => "\u{001C}",
        }
    }
}
