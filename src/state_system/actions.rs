use std::path::PathBuf;
use vizia::prelude::Entity;

use super::{
    source_state::{BrowserPanelTab, SnapMode, TimelineTool},
    time::Timestamp,
};

#[derive(Debug, Clone)]
pub enum AppAction {
    _PollEngine,
    BrowserPanel(BrowserPanelAction),
    Track(TrackAction),
    Timeline(TimelineAction),
    _Internal(InternalAction),
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
    SetMasterTrackHeight { height: f32 },
    SetTrackHeight { index: usize, height: f32 },
    SetTrackVolumeNormalized { index: usize, volume_normalized: f32 },
    SetTrackPanNormalized { index: usize, pan_normalized: f32 },
}

#[derive(Debug, Clone)]
pub enum TimelineAction {
    Navigate {
        /// The horizontal zoom level. 1.0 = default zoom
        horizontal_zoom: f64,

        /// The x position of the left side of the timeline view.
        scroll_beats_x: f64,
    },
    TransportPlay,
    TransportPause,
    TransportStop,
    SetLoopActive(bool),
    SelectTool(TimelineTool),
    SetSnapActive(bool),
    SetSnapMode(SnapMode),
    ZoomIn,
    ZoomOut,
    ZoomReset,
    SelectSingleClip {
        track_index: usize,
        clip_index: usize,
    },
    DeselectAllClips,
    SetClipStartPosition {
        track_index: usize,
        clip_index: usize,
        timeline_start: Timestamp,
    },
}

#[derive(Debug, Clone)]
pub enum InternalAction {
    /// The ID for the timeline view. This will only be sent once
    /// on creation.
    ///
    /// TODO: Find a cleaner way to do this that doesn't involve
    /// actions?
    TimelineViewID(Entity),
}
