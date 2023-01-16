use crate::resource::PcmKey;
use crate::state_system::time::{SuperclockTime, Timestamp};

use super::PaletteColor;

static MAX_CROSSFADE_SECONDS: u32 = 1_000;

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
    pub volume_normalized: f32,
    pub pan_normalized: f32,

    pub routed_to: TrackRouteType,
    //pub parent_track_index: Option<usize>, // TODO
    pub type_: TrackType,
}

#[derive(Debug, Clone)]
pub enum TrackType {
    Audio(ProjectAudioTrackState),
    Synth,
}

#[derive(Debug, Clone)]
pub struct ProjectAudioTrackState {
    pub clips: Vec<AudioClipState>,
}

#[derive(Debug, Clone)]
pub struct AudioClipState {
    pub name: String,
    pub pcm_key: PcmKey,

    pub copyable: AudioClipCopyableState,
}

#[derive(Debug, Clone, Copy)]
pub struct AudioClipCopyableState {
    pub timeline_start: Timestamp,

    pub clip_length: SuperclockTime,

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
