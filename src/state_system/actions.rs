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
    SelectTrack { index: usize },
    SetMasterTrackVolumeNormalized(f32),
    SetMasterTrackPanNormalized(f32),
    ResizeMasterTrackLane { height: f32 },
    ResizeTrackLane { index: usize, height: f32 },
    SetTrackVolumeNormalized { index: usize, volume_normalized: f32 },
    SetTrackPanNormalized { index: usize, pan_normalized: f32 },
}
