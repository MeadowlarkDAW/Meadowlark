use vizia::prelude::*;

pub fn icon<'a>(
    cx: &'a mut Context,
    icon: impl Res<IconCode>,
    frame_size: f32,
    icon_size: f32,
) -> Handle<'a, Label> {
    // Icon can't be bigger than the frame it's held in.
    let icon_size = icon_size.min(frame_size);

    Label::new(cx, icon)
        .width(Pixels(frame_size))
        .height(Pixels(frame_size))
        .font_size(icon_size)
        .child_space(Stretch(1.0))
        .class("icon-label")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Data)]
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
    Refresh,
    Pencil,
    Slice,
    Eraser,
    Magnet,
    ZoomIn,
    ZoomOut,
    ZoomReset,
    DropdownArrow,
    MasterTrack,
    GripVertical,
    GripHorizontal,
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
            IconCode::Refresh => "\u{0061}",
            IconCode::Pencil => "\u{0062}",
            IconCode::Slice => "\u{0063}",
            IconCode::Eraser => "\u{0064}",
            IconCode::Magnet => "\u{0065}",
            IconCode::ZoomIn => "\u{0066}",
            IconCode::ZoomOut => "\u{0067}",
            IconCode::ZoomReset => "\u{0068}",
            IconCode::DropdownArrow => "\u{0069}",
            IconCode::MasterTrack => "\u{006a}",
            IconCode::GripVertical => "\u{006b}",
            IconCode::GripHorizontal => "\u{006c}",
        }
    }
}

impl std::fmt::Display for IconCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s: &str = (*self).into();

        write!(f, "{}", s)
    }
}

impl<'s> Res<IconCode> for IconCode {
    fn get_val(&self, _: &Context) -> IconCode {
        *self
    }

    fn set_or_bind<F>(&self, cx: &mut Context, entity: Entity, closure: F)
    where
        F: 'static + Fn(&mut Context, Entity, Self),
    {
        (closure)(cx, entity, *self);
    }
}
