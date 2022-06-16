use super::clip::{AudioClipState, AutomationClipState, PianoRollClipState};
use super::hrack_effect::HRackEffectState;
use vizia::prelude::*;

#[derive(Debug, Lens, Clone, Data, Serialize, Deserialize)]
pub enum TrackBaseColor {
    /// This is an index into a bunch of preset colors that are defined
    /// by the current theme.
    Preset(u16),
    RGB {
        r: u8,
        g: u8,
        b: u8,
    },
}

/// A "track" refers to a mixer track.
#[derive(Debug, Lens, Clone, Serialize, Deserialize)]
pub struct TrackState {
    pub name: String,
    pub color: TrackBaseColor,

    /// The audio clips assigned to this track.
    pub audio_clips: Vec<AudioClipState>,
    /// The audio clips assigned to this track.
    pub piano_roll_clips: Vec<PianoRollClipState>,
    /// The audio clips assigned to this track.
    pub automation_clips: Vec<AutomationClipState>,

    // TODO: Use some kind of tree structure instead of a Vec once we
    // implement container effects.
    pub effects: Vec<HRackEffectState>,

    /// The index to the track that this track is routed to.
    ///
    /// The master track is always at index 0.
    pub routed_to: usize,

    /// The normalized value of the track's output gain in the range [0.0, 1.0].
    pub out_gain_normalized: f64,

    /// The normalized value of the track's output pan in the range [0.0, 1.0].
    pub out_pan_normalized: f64,

    /// The currently displayed value for the track's output gain (i.e. "-12.0dB").
    pub out_gain_display: String,

    /// The currently displayed value for the track's output pan (i.e. "75R").
    pub out_pan_display: String,

    /// True if this track is currently being "soloed".
    pub soloed: bool,

    /// True if this track is currently being muted.
    pub muted: bool,
    // TODO: Sends
}
