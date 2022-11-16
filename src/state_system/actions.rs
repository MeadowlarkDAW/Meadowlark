use super::BrowserPanelTab;

#[derive(Debug, Clone)]
pub enum AppAction {
    PollEngine,
    BrowserPanel(BrowserPanelAction),
}

#[derive(Debug, Clone)]
pub enum BrowserPanelAction {
    SetPanelShown(bool),
    SelectTab(BrowserPanelTab),
    SetPanelWidth(f32),
    SetSearchText(String),
    SetVolumeNormalized(f32),
    SelectEntryByIndex(usize),
    EnterParentDirectory,
    EnterRootDirectory,
}
