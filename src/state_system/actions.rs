use super::BrowserPanelTab;

#[derive(Debug, Clone)]
pub enum AppAction {
    PollEngine,
    ToggleBrowserPanelShown,
    SelectBrowserPanelTab(BrowserPanelTab),
    SetBrowserPanelWidth(f32),
    SetBrowserPanelSearchText(String),
    SetBrowserVolumeNormalized(f32),
    BrowserItemSelected(usize),
}
