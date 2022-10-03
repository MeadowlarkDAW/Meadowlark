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
                    .font("meadowlark")
                    .class("icon");
            })
            .size(Auto)
    }
}

impl View for Icon {}

pub enum IconCode {
    ArrowDown,
    ArrowDownFilled,
    Automation,
    Cursor,
    Dropdown,
    DrumSequencer,
    Eraser,
    FileHierarchy,
    Folder,
    Menu,
    HatMinus,
    HatPlus,
    Hierarchy,
    Home,
    Loop,
    Mixer,
    Pencil,
    Piano,
    MarkerLeft,
    MarkerRight,
    Play,
    Plug,
    Plus,
    Quantize,
    QuantizeBolt,
    Record,
    Sample,
    Cut,
    Search,
    Stop,
    Grid,
    Stack,
    Terminal,
    Tools,
    ZoomFrame,
    ZoomFit,
    ZoomIn,
    ZoomOut,
}

impl From<IconCode> for &'static str {
    fn from(icon: IconCode) -> Self {
        match icon {
            IconCode::ArrowDown => "\u{0041}",
            IconCode::ArrowDownFilled => "\u{0042}",
            IconCode::Automation => "\u{0043}",
            IconCode::Cursor => "\u{0044}",
            IconCode::Dropdown => "\u{0045}",
            IconCode::DrumSequencer => "\u{0046}",
            IconCode::Eraser => "\u{0047}",
            IconCode::Folder => "\u{0048}",
            IconCode::Menu => "\u{0049}",
            IconCode::HatMinus => "\u{004A}",
            IconCode::HatPlus => "\u{004B}",
            IconCode::Hierarchy => "\u{004C}",
            IconCode::FileHierarchy => "\u{004D}",
            IconCode::Home => "\u{004E}",
            IconCode::Loop => "\u{004F}",
            IconCode::Mixer => "\u{0050}",
            IconCode::Pencil => "\u{0051}",
            IconCode::Piano => "\u{0052}",
            IconCode::MarkerLeft => "\u{0053}",
            IconCode::MarkerRight => "\u{0054}",
            IconCode::Play => "\u{0055}",
            IconCode::Plug => "\u{0056}",
            IconCode::Plus => "\u{0057}",
            IconCode::Quantize => "\u{0058}",
            IconCode::QuantizeBolt => "\u{0059}",
            IconCode::Record => "\u{005A}",
            IconCode::Sample => "\u{0061}",
            IconCode::Cut => "\u{0062}",
            IconCode::Search => "\u{0063}",
            IconCode::Stop => "\u{0064}",
            IconCode::Grid => "\u{0065}",
            IconCode::Stack => "\u{0066}",
            IconCode::Terminal => "\u{0067}",
            IconCode::Tools => "\u{0068}",
            IconCode::ZoomFrame => "\u{0069}",
            IconCode::ZoomFit => "\u{006A}",
            IconCode::ZoomIn => "\u{006B}",
            IconCode::ZoomOut => "\u{006C}",
        }
    }
}
