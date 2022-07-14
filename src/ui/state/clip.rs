use super::core_types::{WMusicalTime, WSeconds, WSuperFrames};
use vizia::prelude::*;

#[derive(Debug, Lens, Clone, Data)]
pub struct ClipState {
    pub name: String,
    pub timeline_start: ClipStart,
    pub length: WMusicalTime,

    pub channel: usize,

    pub type_: ClipType,
}

#[derive(Debug, Lens, Clone, Data)]
pub enum ClipType {
    Audio(AudioClipState),
    PianoRoll(PianoRollClipState),
    Automation(AutomationClipState),
}

#[derive(Debug, Lens, Clone, Data)]
pub struct AudioClipState {
    pub fade_in_secs: WSeconds,

    pub fade_out_secs: WSeconds,

    /// The amount of time between the start of the raw waveform data
    /// and the start of the clip.
    ///
    /// TODO
    pub clip_start_offset: WSuperFrames,
    // TODO: pointer to waveform data
}

#[derive(Debug, Lens, Clone, Data)]
pub struct PianoRollClipState {
    // TODO
}

#[derive(Debug, Lens, Clone, Data)]
pub struct AutomationClipState {
    // TODO
}

#[derive(Debug, Lens, Clone, Data)]
pub enum ClipStart {
    OnLane(OnLane),
    /// This means that the clip is not currently on the timeline,
    /// and instead just lives in the clips panel.
    NotInTimeline,
}

#[derive(Debug, Lens, Clone, Data)]
pub struct OnLane {
    lane_index: u32,
    timeline_start: WMusicalTime,
}
