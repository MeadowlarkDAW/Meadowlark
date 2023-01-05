use vizia::prelude::Data;

/// This struct contains all of the non-project-related state such as
/// panel sizes, which panels are open, etc.
///
/// This app state is also what gets turned into a config file.
///
/// This is only allowed to be mutated within the `state_system::handle_action` method..
pub struct AppState {
    pub browser_panel: BrowserPanelState,

    pub selected_timeline_tool: TimelineTool,
    pub timeline_snap_active: bool,
    pub timeline_snap_mode: SnapMode,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            browser_panel: BrowserPanelState {
                panel_shown: true,
                current_tab: BrowserPanelTab::Samples,
                panel_width: 200.0,
                volume_normalized: 1.0,
                volume_default_normalized: 1.0,
                playback_on_select: true,
            },
            selected_timeline_tool: TimelineTool::Pointer,
            timeline_snap_active: true,
            timeline_snap_mode: SnapMode::Line,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Data)]
pub enum BrowserPanelTab {
    Samples,
    Multisamples,
    Synths,
    Effects,
    PianoRollClips,
    AutomationClips,
    Projects,
    Files,
}

#[derive(Debug, Clone)]
pub struct BrowserPanelState {
    pub panel_shown: bool,
    pub current_tab: BrowserPanelTab,
    pub panel_width: f32,
    pub volume_normalized: f32,
    pub volume_default_normalized: f32,
    pub playback_on_select: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Data)]
pub enum TimelineTool {
    Pointer,
    Pencil,
    Slicer,
    Eraser,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Data)]
pub enum SnapMode {
    Line,
    Beat,
    HalfBeat,
    ThirdBeat,
    QuarterBeat,
    EigthBeat,
    SixteenthBeat,
    _32ndBeat,
    // TODO: More
}

impl SnapMode {
    pub fn to_text(&self) -> &'static str {
        match self {
            SnapMode::Line => "Line",
            SnapMode::Beat => "Beat",
            SnapMode::HalfBeat => "1/2 Beat",
            SnapMode::ThirdBeat => "1/3 Beat",
            SnapMode::QuarterBeat => "1/4 Beat",
            SnapMode::EigthBeat => "1/8 Beat",
            SnapMode::SixteenthBeat => "1/16 Beat",
            SnapMode::_32ndBeat => "1/32 Beat",
        }
    }
}
