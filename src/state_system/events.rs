use super::BrowserPanelTab;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AppEvent {
    PollEngine,
    ToggleBrowserPanelShown,
    SelectBrowserPanelTab(BrowserPanelTab),
}
