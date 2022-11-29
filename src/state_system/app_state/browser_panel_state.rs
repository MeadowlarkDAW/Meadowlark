use vizia::prelude::*;

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
