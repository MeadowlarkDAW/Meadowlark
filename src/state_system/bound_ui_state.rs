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
    pub browser_panel_width: f32,
    pub browser_panel_search_text: String,
}

impl BoundUiState {
    pub fn new() -> Self {
        Self {
            browser_panel_shown: true,
            browser_panel_tab: BrowserPanelTab::Samples,
            browser_panel_width: 200.0,
            browser_panel_search_text: String::new(),
        }
    }
}

impl Model for BoundUiState {
    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {}
}
