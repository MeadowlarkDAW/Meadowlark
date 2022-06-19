use meadowlark_core_types::{MusicalTime, Seconds, SuperFrames};
use vizia::prelude::*;

#[derive(Debug, Lens, Clone, Data, Serialize, Deserialize)]
pub struct AudioClipState {
    pub name: String,

    pub length: MusicalTime,

    pub clip_start: ClipStart,

    pub fade_in_secs: Seconds,

    pub fade_out_secs: Seconds,

    /// The amount of time between the start of the raw waveform data
    /// and the start of the clip.
    ///
    /// TODO
    pub clip_start_offset: SuperFrames,
    // TODO: pointer to waveform data
}

#[derive(Debug, Lens, Clone, Data, Serialize, Deserialize)]
pub struct PianoRollClipState {
    pub name: String,
    pub timeline_start: MusicalTime,
    pub length: MusicalTime,
    // TODO
}

#[derive(Debug, Lens, Clone, Data, Serialize, Deserialize)]
pub struct AutomationClipState {
    pub name: String,
    pub timeline_start: MusicalTime,
    pub length: MusicalTime,
    // TODO
}

#[derive(Debug, Lens, Clone, Data, Serialize, Deserialize)]
pub enum ClipStart {
    OnLane(OnLane),
    /// This means that the clip is not currently on the timeline,
    /// and instead just lives in the clips panel.
    NotInTimeline,
}

#[derive(Debug, Lens, Clone, Data, Serialize, Deserialize)]
pub struct OnLane {
    lane_index: u32,
    timeline_start: MusicalTime,
}
