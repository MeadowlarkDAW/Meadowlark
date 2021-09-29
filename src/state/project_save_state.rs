use rusty_daw_time::{MusicalTime, SampleRate, Seconds};

use crate::backend::timeline::audio_clip::AudioClipSaveState;
use crate::backend::timeline::{LoopState, TimelineTrackSaveState};
use crate::backend::BackendSaveState;

/// This struct should contain all information needed to create a "save file"
/// for a project.
///
/// TODO: Project file format. This will need to be future-proof.
#[derive(Debug, Clone)]
pub struct ProjectSaveState {
    pub backend: BackendSaveState,
}

impl ProjectSaveState {
    pub fn new_empty(sample_rate: SampleRate) -> Self {
        Self { backend: BackendSaveState::new(sample_rate) }
    }

    pub fn test() -> Self {
        let mut backend_save_state = BackendSaveState::new(SampleRate(48_000.0));

        backend_save_state.timeline_transport.loop_state = LoopState::Active {
            loop_start: MusicalTime::new(0.0),
            loop_end: MusicalTime::new(4.0),
        };

        backend_save_state.timeline_tracks.push(TimelineTrackSaveState::new(
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

        backend_save_state.timeline_tracks.push(TimelineTrackSaveState::new(
            String::from("Track 2"),
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

        Self { backend: backend_save_state }
    }
}
