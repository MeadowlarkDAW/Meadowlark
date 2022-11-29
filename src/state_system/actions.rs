use std::path::PathBuf;

use crate::state_system::app_state::BrowserPanelTab;

#[derive(Debug, Clone)]
pub enum AppAction {
    PollEngine,
    BrowserPanel(BrowserPanelAction),
    Track(TrackAction),
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
    PlayFile(PathBuf),
    StopPlayback,
    Refresh,
}

#[derive(Debug, Clone)]
pub enum TrackAction {
    SelectMasterTrack,
    SelectTrackByIndex { index: usize },
    ResizeMasterTrackLane { height: f32 },
    ResizeTrackLaneByIndex { index: usize, height: f32 },
}
