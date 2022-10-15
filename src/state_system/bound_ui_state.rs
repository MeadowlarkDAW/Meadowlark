use vizia::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

#[derive(Debug, Lens, Clone)]
pub struct BoundUiState {
    pub browser_panel_shown: bool,
    pub browser_panel_tab: BrowserPanelTab,
}

impl BoundUiState {
    pub fn new() -> Self {
        Self { browser_panel_shown: true, browser_panel_tab: BrowserPanelTab::Samples }
    }
}

impl Model for BoundUiState {
    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {}
}
