use fnv::FnvHashMap;
use meadowlark_core_types::time::{SuperclockTime, Timestamp};

use super::PaletteColor;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrackType {
    Audio,
    Synth,
    //Folder, // TODO
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrackRouteType {
    ToMaster,
    ToTrackAtIndex(usize),
    None,
}

#[derive(Debug, Clone)]
pub struct ProjectTrackState {
    pub name: String,
    pub color: PaletteColor,
    pub lane_height: f32,
    pub type_: TrackType,
    pub volume_normalized: f32,
    pub pan_normalized: f32,

    pub routed_to: TrackRouteType,
    //pub parent_track_index: Option<usize>, // TODO
    pub clips: FnvHashMap<u64, ClipState>,
}

#[derive(Debug, Clone)]
pub struct ClipState {
    pub timeline_start: Timestamp,
    pub type_: ClipType,
}

#[derive(Debug, Clone)]
pub enum ClipType {
    Audio(AudioClipState),
}

#[derive(Debug, Clone)]
pub struct AudioClipState {
    pub length: SuperclockTime,
}
