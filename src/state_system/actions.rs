use super::BrowserPanelTab;

#[derive(Debug, Clone)]
pub enum AppAction {
    PollEngine,
    BrowserPanel(BrowserPanelAction),
    TrackHeadersPanel(TrackHeadersPanelAction),
}

#[derive(Debug, Clone)]
pub enum BrowserPanelAction {
    SetPanelShown(bool),
    SelectTab(BrowserPanelTab),
    SetPanelWidth(f32),
    SetSearchText(String),
    SetVolumeNormalized(f32),
    SelectEntryByIndex { index: usize, invoked_by_play_btn: bool },
    EnterParentDirectory,
    EnterRootDirectory,
    SetPlaybackOnSelect(bool),
    StopPlayback,
    Refresh,
}

#[derive(Debug, Clone)]
pub enum TrackHeadersPanelAction {
    ResizeTrackByIndex { index: usize, height: f32 },
}
