use basedrop::Shared;
use rusty_daw_time::{MusicalTime, SampleRate, SampleTime, Seconds, TempoMap};

use crate::backend::pcm::{MonoPcm, StereoPcm};

pub struct MonoAudioClip {
    pub name: String,
    pub pcm_id: String,
    pub clip_start_offset: Seconds,
    pub timeline_start: MusicalTime,
    pub timeline_duration: MusicalTime,
    pub clip_gain_db: f32,
}

pub struct MonoAudioClipProcInfo {
    // PcmResources are always immutable. This reflects the non-destructive nature
    // of this sampler engine.
    pub pcm: Shared<MonoPcm>,

    pub clip_start_offset: Seconds,

    pub timeline_start_smp: SampleTime,
    pub timeline_end_smp: SampleTime,

    pub clip_gain_amp: f32,
}

pub struct StereoAudioClip {
    pub name: String,
    pub pcm_id: String,
    pub clip_start_offset: Seconds,
    pub timeline_start: MusicalTime,
    pub timeline_duration: MusicalTime,
    pub clip_gain_db: f32,
}

pub struct StereoAudioClipProcInfo {
    // PcmResources are always immutable. This reflects the non-destructive nature
    // of this sampler engine.
    pub pcm: Shared<StereoPcm>,

    pub clip_start_offset: Seconds,

    pub timeline_start_smp: SampleTime,
    pub timeline_end_smp: SampleTime,

    pub clip_gain_amp: f32,
}
