use rusty_daw_time::{SampleRate, Seconds, TempoMap};

use crate::backend::timeline::{
    audio_clip::DEFAULT_AUDIO_CLIP_DECLICK_TIME, TimelineTrackSaveState, TimelineTransportSaveState,
};

/// This struct should contain all information needed to create a "save file"
/// for the backend.
///
/// TODO: Project file format. This will need to be future-proof.
#[derive(Debug, Clone)]
pub struct BackendSaveState {
    pub timeline_tracks: Vec<TimelineTrackSaveState>,
    pub timeline_transport: TimelineTransportSaveState,
    pub tempo_map: TempoMap,
    pub audio_clip_declick_time: Seconds,
}

impl BackendSaveState {
    pub fn new(sample_rate: SampleRate) -> Self {
        Self {
            timeline_tracks: Vec::new(),
            timeline_transport: Default::default(),
            tempo_map: TempoMap::new(110.0, sample_rate.into()),
            audio_clip_declick_time: DEFAULT_AUDIO_CLIP_DECLICK_TIME,
        }
    }
}
