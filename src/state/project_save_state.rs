use rusty_daw_core::{MusicalTime, Seconds};

use crate::backend::timeline::audio_clip::AudioClipSaveState;
use crate::backend::timeline::{
    LoopState, TempoMap, TimelineTrackSaveState, TimelineTransportSaveState,
};
use crate::backend::BackendSaveState;

/// This struct should contain all information needed to create a "save file"
/// for a project.
///
/// TODO: Project file format. This will need to be future-proof.
#[derive(Debug, Clone)]
pub struct ProjectSaveState {
    pub backend: BackendSaveState,
    pub timeline_tracks: Vec<TimelineTrackSaveState>,
}

impl ProjectSaveState {
    pub fn new_empty() -> Self {
        Self { backend: BackendSaveState::default(), timeline_tracks: Vec::new() }
    }

    pub fn test() -> Self {
        let timeline_transport = TimelineTransportSaveState {
            seek_to: MusicalTime(0.0),
            loop_state: LoopState::Active {
                loop_start: MusicalTime::new(0.0),
                loop_end: MusicalTime::new(4.0),
            },
        };

        let backend = BackendSaveState::new(timeline_transport, TempoMap::default());

        let mut timeline_tracks: Vec<TimelineTrackSaveState> = Vec::new();

        timeline_tracks.push(TimelineTrackSaveState::new(
            String::from("Track 1"),
            vec![AudioClipSaveState::new(
                String::from("Audio Clip 1"),
                "./assets/test_files/synth_keys/synth_keys_48000_16bit.wav".into(),
                MusicalTime::new(0.0),
                Seconds::new(3.0),
                Seconds::new(0.0),
                -3.0,
                Default::default(),
            )],
        ));

        timeline_tracks.push(TimelineTrackSaveState::new(
            String::from("Track 2"),
            vec![AudioClipSaveState::new(
                String::from("Audio Clip 1"),
                "./assets/test_files/synth_keys/synth_keys_48000_16bit.wav".into(),
                MusicalTime::new(1.0),
                Seconds::new(3.0),
                Seconds::new(0.0),
                -3.0,
                Default::default(),
            )],
        ));

        Self { backend, timeline_tracks }
    }
}
