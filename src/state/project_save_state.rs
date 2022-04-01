use rusty_daw_core::{MusicalTime, Seconds, SuperFrames};

use crate::backend::timeline::{
    AudioClipState, LoopState, TempoMap, TimelineTrackState, TimelineTransportState,
};
use crate::backend::BackendCoreState;

/// This struct should contain all information needed to create a "save file"
/// for a project. This includes the state of the backend.
///
/// TODO: Project file format. This will need to be future-proof.
#[derive(Debug, Clone)]
pub struct ProjectSaveState {
    pub backend_core: BackendCoreState,
    pub timeline_tracks: Vec<TimelineTrackState>,
}

impl ProjectSaveState {
    pub fn timeline_tracks(&self) -> (&TempoMap, &Vec<TimelineTrackState>) {
        (&self.backend_core.tempo_map, &self.timeline_tracks)
    }

    pub fn timeline_tracks_mut(&mut self) -> (&TempoMap, &mut Vec<TimelineTrackState>) {
        (&self.backend_core.tempo_map, &mut self.timeline_tracks)
    }
}

impl ProjectSaveState {
    pub fn new_empty() -> Self {
        Self { backend_core: BackendCoreState::default(), timeline_tracks: Vec::new() }
    }

    pub fn test() -> Self {
        let timeline_transport = TimelineTransportState {
            seek_to: MusicalTime::default(),
            loop_state: LoopState::Active {
                loop_start: MusicalTime::new(0, 0),
                loop_end: MusicalTime::new(12, 0),
            },
        };

        let backend_core = BackendCoreState::new(timeline_transport, TempoMap::default());

        let mut timeline_tracks: Vec<TimelineTrackState> = Vec::new();

        timeline_tracks.push(TimelineTrackState {
            name: String::from("Track 1"),
            audio_clips: vec![AudioClipState {
                name: String::from("Kick"),
                pcm_path: "./assets/test_files/drums/kick.wav".into(),
                timeline_start: MusicalTime::new(0, 0),
                duration: Seconds::new(0.5).to_nearest_super_frame_round(),
                clip_start_offset: SuperFrames::new(0),
                clip_start_offset_is_negative: false,
                clip_gain_db: -3.0,
                fades: Default::default(),
            }],
        });

        timeline_tracks.push(TimelineTrackState {
            name: String::from("Track 2"),
            audio_clips: vec![AudioClipState {
                name: String::from("Snare"),
                pcm_path: "./assets/test_files/drums/snare.wav".into(),
                timeline_start: MusicalTime::new(1, 0),
                duration: Seconds::new(0.5).to_nearest_super_frame_round(),
                clip_start_offset: SuperFrames::new(0),
                clip_start_offset_is_negative: false,
                clip_gain_db: -3.0,
                fades: Default::default(),
            }],
        });

        Self { backend_core, timeline_tracks }
    }
}
