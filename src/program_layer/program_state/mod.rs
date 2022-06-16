mod clip;
mod hrack_effect;
mod timeline_grid;
mod track;

pub use clip::*;
pub use hrack_effect::*;
pub use timeline_grid::*;
pub use track::*;

use vizia::prelude::*;

/// The state of the whole program.
///
/// Unless explicitely stated, the UI may NOT directly mutate the state of any
/// of these variables. It is intended for the UI to call the methods on this
/// struct in order to mutate state.
#[derive(Debug, Lens, Clone)]
pub struct ProgramState {
    /// True if a backend engine is currently running, false if not.
    ///
    /// Nothing except the settings menu can be accessed when this is false.
    pub engine_running: bool,

    /// This contains all of the text for any notifications (errors or otherwise)
    /// that are being displayed to the user.
    ///
    /// The UI may mutate this directly without an event.
    pub notification_log: Vec<NotificationLogType>,

    /// A "track" refers to a mixer track.
    ///
    /// This also contains the state of all clips.
    pub tracks: Vec<TrackState>,

    /// The state of the timeline grid.
    ///
    /// (This does not contain the state of the clips.)
    pub timeline_grid: TimelineGridState,
}

#[derive(Debug, Lens, Clone)]
pub enum NotificationLogType {
    Error(String),
    Info(String),
}
