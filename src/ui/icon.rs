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
    Undo,
    Redo,
    Save,
    Loop,
    Stop,
    Play,
    Pause,
    Record,
    RecordActive,
    Menu,
    Folder,
    File,
    FileAudio,
    Search,
    Filter,
    Soundwave,
    Piano,
    Properties,
    Knob,
    FX,
    Midi,
    Automation,
    VolumeMute,
    VolumeMin,
    VolumeMed,
    VolumeMax,
    Home,
    Terminal,
    DoubleArrowRight,
    DoubleArrowDown,
    DoubleArrowUp,
    ChevronUp,
    Cursor,
}

impl From<IconCode> for &'static str {
    fn from(icon: IconCode) -> Self {
        match icon {
            IconCode::Undo => "\u{0040}",
            IconCode::Redo => "\u{0041}",
            IconCode::Save => "\u{0042}",
            IconCode::Loop => "\u{0043}",
            IconCode::Stop => "\u{0044}",
            IconCode::Play => "\u{0045}",
            IconCode::Pause => "\u{0046}",
            IconCode::Record => "\u{0047}",
            IconCode::RecordActive => "\u{0048}",
            IconCode::Menu => "\u{0049}",
            IconCode::Folder => "\u{004a}",
            IconCode::File => "\u{004b}",
            IconCode::FileAudio => "\u{004c}",
            IconCode::Search => "\u{004d}",
            IconCode::Filter => "\u{004e}",
            IconCode::Soundwave => "\u{004f}",
            IconCode::Piano => "\u{0050}",
            IconCode::Properties => "\u{0051}",
            IconCode::Knob => "\u{0052}",
            IconCode::FX => "\u{0053}",
            IconCode::Midi => "\u{0054}",
            IconCode::Automation => "\u{0055}",
            IconCode::VolumeMute => "\u{0056}",
            IconCode::VolumeMin => "\u{0057}",
            IconCode::VolumeMed => "\u{0058}",
            IconCode::VolumeMax => "\u{0059}",
            IconCode::Home => "\u{005a}",
            IconCode::Terminal => "\u{005b}",
            IconCode::DoubleArrowRight => "\u{005c}",
            IconCode::DoubleArrowDown => "\u{005d}",
            IconCode::DoubleArrowUp => "\u{005e}",
            IconCode::ChevronUp => "\u{005f}",
            IconCode::Cursor => "\u{0060}",
        }
    }
}
