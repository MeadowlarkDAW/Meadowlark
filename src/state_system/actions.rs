use std::path::PathBuf;
use vizia::prelude::Entity;

use super::source_state::{AudioClipCopyableState, BrowserPanelTab, SnapMode, TimelineTool};

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

    SetRecordActive(bool),
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

    /// Sent when the user is in the process of dragging/modifying audio clips
    /// on the timeline.
    ///
    /// This is to avoid filling the undo stack with actions sent every frame.
    /// Once the user is done gesturing (mouse up), then a `SetAudioClipStates`
    /// action will be sent. That action will be the one that gets pushed onto
    /// the undo stack.
    ///
    /// Also because syncing the state to the backend engine requires cloning
    /// the vec of all audio clips on a given track, this action is used
    /// to avoid that happening every frame (because it is slow and it can
    /// potentially create a lot of garbage for the garbage collector.
    GestureAudioClipCopyableStates {
        track_index: usize,
        /// (index, new state)
        changed_clips: Vec<(usize, AudioClipCopyableState)>,
    },
    SetAudioClipCopyableStates {
        track_index: usize,
        /// (index, new state)
        changed_clips: Vec<(usize, AudioClipCopyableState)>,
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
