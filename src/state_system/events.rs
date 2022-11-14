use super::BrowserPanelTab;

#[derive(Debug, Clone)]
pub enum AppEvent {
    PollEngine,
    ToggleBrowserPanelShown,
    SelectBrowserPanelTab(BrowserPanelTab),
    SetBrowserPanelWidth(f32),
    SetBrowserPanelSearchText(String),
}
