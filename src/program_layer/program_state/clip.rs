use rusty_daw_core::{MusicalTime, Seconds, SuperFrames};

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

pub struct PianoRollClipState {
    pub name: String,

    pub timeline_start: MusicalTime,
    pub length: MusicalTime,
    // TODO
}

pub struct AutomationClipState {
    pub name: String,

    pub timeline_start: MusicalTime,
    pub length: MusicalTime,
    // TODO
}

pub enum ClipStart {
    OnLane {
        /// The index of the lane that this clip is on.
        lane_index: u32,
        timeline_start: MusicalTime,
    },
    /// This means that the clip is not currently on the timeline,
    /// and instead just lives in the clips panel.
    NotInTimeline,
}
