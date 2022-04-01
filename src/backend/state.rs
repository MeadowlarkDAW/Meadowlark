use rusty_daw_core::{SampleRate, Seconds};

use crate::backend::timeline::TempoMap;

use crate::backend::timeline::{
    audio_clip::DEFAULT_AUDIO_CLIP_DECLICK_TIME, TimelineTransportState,
};

/// This struct should contain all information needed to create a "save file"
/// for the backend core.
#[derive(Debug, Clone)]
pub struct BackendCoreState {
    pub timeline_transport: TimelineTransportState,
    pub tempo_map: TempoMap,
    pub audio_clip_declick_time: Seconds,
}

impl Default for BackendCoreState {
    fn default() -> Self {
        Self {
            timeline_transport: TimelineTransportState::default(),
            tempo_map: TempoMap::default(),
            audio_clip_declick_time: DEFAULT_AUDIO_CLIP_DECLICK_TIME,
        }
    }
}

impl BackendCoreState {
    pub fn new(timeline_transport: TimelineTransportState, tempo_map: TempoMap) -> Self {
        Self {
            timeline_transport,
            tempo_map,
            audio_clip_declick_time: DEFAULT_AUDIO_CLIP_DECLICK_TIME,
        }
    }

    pub fn clone_with_sample_rate(&self, sample_rate: SampleRate) -> Self {
        let mut tempo_map = self.tempo_map.clone();
        tempo_map.sample_rate = sample_rate;
        Self {
            timeline_transport: self.timeline_transport,
            tempo_map,
            audio_clip_declick_time: self.audio_clip_declick_time,
        }
    }
}
