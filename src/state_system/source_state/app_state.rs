use vizia::prelude::Data;

/// This struct contains all of the non-project-related state such as
/// panel sizes, which panels are open, etc.
///
/// This app state is also what gets turned into a config file.
///
/// Only the `StateSystem` struct is allowed to mutate this.
pub struct AppState {
    pub browser_panel: BrowserPanelState,
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
