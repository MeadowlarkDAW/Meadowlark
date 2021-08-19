use rusty_daw_time::{MusicalTime, SampleRate, Seconds, TempoMap};
use tuix::Lens;

use crate::backend::timeline::{
    audio_clip::DEFAULT_AUDIO_CLIP_DECLICK_TIME, AudioClipSaveState, LoopState,
    TimelineTrackSaveState, TimelineTransportSaveState,
};

/// This struct should contain all information needed to create a "save file"
/// for the project.
///
/// TODO: Project file format. This will need to be future-proof.
#[derive(Lens)]
pub struct ProjectSaveState {
    pub timeline_tracks: Vec<TimelineTrackSaveState>,
    pub timeline_transport: TimelineTransportSaveState,
    pub tempo_map: TempoMap,
    pub audio_clip_declick_time: Seconds,
}

impl ProjectSaveState {
    pub fn new_empty(sample_rate: SampleRate) -> Self {
        Self {
            timeline_tracks: Vec::new(),
            timeline_transport: Default::default(),
            tempo_map: TempoMap::new(110.0, sample_rate.into()),
            audio_clip_declick_time: DEFAULT_AUDIO_CLIP_DECLICK_TIME,
        }
    }

    pub fn test(sample_rate: SampleRate) -> Self {
        let mut new_self = ProjectSaveState::new_empty(sample_rate);

        new_self.timeline_transport.loop_state = LoopState::Active {
            loop_start: MusicalTime::new(0.0),
            loop_end: MusicalTime::new(4.0),
        };

        new_self.timeline_tracks.push(TimelineTrackSaveState::new(
            String::from("Track 1"),
            vec![AudioClipSaveState::new(
                String::from("Audio Clip 1"),
                "./test_files/synth_keys/synth_keys_48000_16bit.wav".into(),
                MusicalTime::new(0.0),
                Seconds::new(3.0),
                Seconds::new(0.0),
                -3.0,
                Default::default(),
            )],
        ));

        new_self.timeline_tracks.push(TimelineTrackSaveState::new(
            String::from("Track 2"),
            vec![AudioClipSaveState::new(
                String::from("Audio Clip 1"),
                "./test_files/synth_keys/synth_keys_48000_16bit.wav".into(),
                MusicalTime::new(0.0),
                Seconds::new(3.0),
                Seconds::new(0.0),
                -3.0,
                Default::default(),
            )],
        ));

        new_self
    }
}
