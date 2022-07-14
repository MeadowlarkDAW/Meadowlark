use std::path::PathBuf;

use super::clip::{AudioClipState, AutomationClipState, PianoRollClipState};
use super::hrack_effect::HRackEffectState;
use vizia::prelude::*;

#[derive(Debug, Lens, Clone, Data)]
pub enum ChannelBaseColor {
    /// This is an index into a bunch of preset colors that are defined
    /// by the current theme.
    Preset(u16),
    Color(Color),
}

impl From<ChannelBaseColor> for Color {
    fn from(col: ChannelBaseColor) -> Self {
        match col {
            ChannelBaseColor::Preset(_) => Color::red(),
            ChannelBaseColor::Color(col) => col,
        }
    }
}

impl From<Color> for ChannelBaseColor {
    fn from(col: Color) -> Self {
        ChannelBaseColor::Color(col)
    }
}

/// A "channel" refers to a mixer channel.
#[derive(Debug, Lens, Clone, Data)]
pub struct ChannelState {
    /// The channel name
    pub name: String,

    pub path: PathBuf,

    /// The channel color
    pub color: ChannelBaseColor,

    pub parent_channel: Option<usize>,

    /// Subchannels of this Channel
    pub subchannels: Vec<usize>,

    /// Flag indicating whether the channel is currently selected in UI
    pub selected: bool,

    /// The audio clips assigned to this channel.
    pub audio_clips: Vec<AudioClipState>,
    /// The audio clips assigned to this channel.
    pub piano_roll_clips: Vec<PianoRollClipState>,
    /// The audio clips assigned to this channel.
    pub automation_clips: Vec<AutomationClipState>,

    // TODO: Use some kind of tree structure instead of a Vec once we
    // implement container effects.
    pub effects: Vec<HRackEffectState>,

    /// The index to the channel that this channel is routed to.
    ///
    /// The master channel is always at index 0.
    pub routed_to: usize,

    /// The normalized value of the channel's output gain in the range [0.0, 1.0].
    pub out_gain_normalized: f64,

    /// The normalized value of the channel's output pan in the range [0.0, 1.0].
    pub out_pan_normalized: f64,

    /// The currently displayed value for the channel's output gain (i.e. "-12.0dB").
    pub out_gain_display: String,

    /// The currently displayed value for the channel's output pan (i.e. "75R").
    pub out_pan_display: String,

    /// True if this channel is currently being "soloed".
    pub soloed: bool,

    /// True if this channel is currently being muted.
    pub muted: bool,
    // TODO: Sends
}

impl Default for ChannelState {
    fn default() -> Self {
        ChannelState {
            name: String::from("Channel"),
            path: PathBuf::from("Channel"),
            color: ChannelBaseColor::Color(Color::red()),
            parent_channel: Some(0),
            subchannels: vec![],
            selected: false,
            audio_clips: vec![],
            piano_roll_clips: vec![],
            automation_clips: vec![],
            effects: vec![],
            routed_to: 0,
            out_gain_normalized: 1.0,
            out_pan_normalized: 0.5,
            out_gain_display: String::from("0dB"),
            out_pan_display: String::from("0"),
            soloed: false,
            muted: false,
        }
    }
}

#[derive(PartialEq, Clone)]
pub enum ChannelEvent {
    SelectChannel(usize),
    SelectChannelGroup(usize),
    AddChannel,
    RemoveChannel,
    // DragChannel(usize),
    // DropChannel(usize),
}
