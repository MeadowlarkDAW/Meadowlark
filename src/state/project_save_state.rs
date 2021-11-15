use rusty_daw_core::{MusicalTime, Seconds};

use crate::backend::timeline::{
    AudioClipSaveState, LoopState, TempoMap, TimelineTrackSaveState, TimelineTransportSaveState,
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

        timeline_tracks.push(TimelineTrackSaveState {
            name: String::from("Track 1"),
            audio_clips: vec![AudioClipSaveState {
                name: String::from("Audio Clip 1"),
                pcm_path: "./assets/test_files/synth_keys/synth_keys_48000_16bit.wav".into(),
                timeline_start: MusicalTime::new(0.0),
                duration: Seconds::new(3.0),
                clip_start_offset: Seconds::new(0.0),
                clip_gain_db: -3.0,
                fades: Default::default(),
            }],
        });

        timeline_tracks.push(TimelineTrackSaveState {
            name: String::from("Track 2"),
            audio_clips: vec![AudioClipSaveState {
                name: String::from("Audio Clip 1"),
                pcm_path: "./assets/test_files/synth_keys/synth_keys_48000_16bit.wav".into(),
                timeline_start: MusicalTime::new(1.0),
                duration: Seconds::new(3.0),
                clip_start_offset: Seconds::new(0.0),
                clip_gain_db: -3.0,
                fades: Default::default(),
            }],
        });

        Self { backend, timeline_tracks }
    }
}
