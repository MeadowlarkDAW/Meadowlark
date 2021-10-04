use rusty_daw_core::{MusicalTime, Seconds};
use std::path::PathBuf;
use tuix::Lens;

use super::{AudioClipFades, LoopState};

#[derive(Debug, Clone, Copy, Lens)]
pub struct TimelineTransportSaveState {
    pub seek_to: MusicalTime,
    pub loop_state: LoopState,
}

impl Default for TimelineTransportSaveState {
    fn default() -> Self {
        Self { seek_to: MusicalTime::new(0.0), loop_state: LoopState::Inactive }
    }
}

#[derive(Debug, Clone, Lens)]
pub struct TimelineTrackSaveState {
    /// The name displayed on this timeline track.
    pub name: String,

    /// The audio clips on this timeline track. These may not be
    /// in any particular order.
    pub audio_clips: Vec<AudioClipSaveState>,
}

#[derive(Debug, Clone, Lens)]
pub struct AudioClipSaveState {
    /// The name displayed on the audio clip.
    pub name: String,

    /// The path to the audio file containing the PCM data.
    pub pcm_path: PathBuf,

    /// Where the clip starts on the timeline.
    pub timeline_start: MusicalTime,

    /// The duration of the clip on the timeline.
    pub duration: Seconds,

    /// The offset in the pcm resource where the "start" of the clip should start playing from.
    pub clip_start_offset: Seconds,

    /// The gain of the audio clip in decibels.
    pub clip_gain_db: f32,

    /// The fades on this audio clip.
    pub fades: AudioClipFades,
}
