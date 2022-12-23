use fnv::FnvHashMap;
use meadowlark_core_types::time::{SuperclockTime, Timestamp};

use crate::backend::resource_loader::PcmKey;

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
    pub name: String,
    pub type_: ClipType,
}

#[derive(Debug, Clone)]
pub enum ClipType {
    Audio(AudioClipState),
}

#[derive(Debug, Clone)]
pub struct AudioClipState {
    pub clip_length: SuperclockTime,

    pub pcm_key: PcmKey,

    // TODO: Automated gain.
    pub gain_db: f32,

    pub clip_to_pcm_offset: SuperclockTime,
    pub clip_to_pcm_offset_is_negative: bool,

    pub incrossfade_type: CrossfadeType,
    pub incrossfade_time: SuperclockTime,

    pub outcrossfade_type: CrossfadeType,
    pub outcrossfade_time: SuperclockTime,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CrossfadeType {
    ConstantPower,
    Linear,
    //Symmetric, // TODO
    //Fast, // TODO
    //Slow, // TODO
}

impl Default for CrossfadeType {
    fn default() -> Self {
        CrossfadeType::ConstantPower
    }
}
