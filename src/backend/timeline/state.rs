use rusty_daw_core::{MusicalTime, SuperFrames};
use std::path::PathBuf;

use super::{AudioClipFades, LoopState};

#[derive(Debug, Clone, Copy)]
pub struct TimelineTransportState {
    /// The place where the playhead will seek to on project load/transport stop.
    pub seek_to: MusicalTime,
    pub loop_state: LoopState,
}

impl Default for TimelineTransportState {
    fn default() -> Self {
        Self { seek_to: MusicalTime::default(), loop_state: LoopState::Inactive }
    }
}

#[derive(Debug, Clone)]
pub struct TimelineTrackState {
    /// The name displayed on this timeline track.
    pub name: String,

    /// The audio clips on this timeline track. These may not be
    /// in any particular order.
    pub audio_clips: Vec<AudioClipState>,
}

#[derive(Debug, Clone)]
pub struct AudioClipState {
    /// The name displayed on the audio clip.
    pub name: String,

    /// The path to the audio file containing the PCM data.
    pub pcm_path: PathBuf,

    /// Where the clip starts on the timeline.
    pub timeline_start: MusicalTime,

    /// The duration of the clip on the timeline.
    pub duration: SuperFrames,

    /// The offset in the pcm resource where the "start" of the clip should start playing from.
    pub clip_start_offset: SuperFrames,

    /// Whether or not the `clip_start_offset` value should be positive (false) or negative (true)
    pub clip_start_offset_is_negative: bool,

    /// The gain of the audio clip in decibels.
    pub clip_gain_db: f32,

    /// The fades on this audio clip.
    pub fades: AudioClipFades,
}
