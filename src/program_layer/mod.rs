//! # Program (State) Layer
//!
//! This layer owns the state of the program.
//!
//! It is solely in charge of mutating this state. The backend layer and the UI
//! layer cannot mutate this state directly (with the exception of some
//! UI-specific state that does not need to be undo-able such as panel or window
//! size). The backend layer indirectly mutates this state by sending events to
//! the program layer, and the ui layer indirectly mutates this state by calling
//! methods on the ProgramState struct which the UI layer owns.
//!
//! The program layer also owns the handle to the audio thread and is in charge
//! of connecting to the system's audio and MIDI devices. It is also in charge
//! of some offline DSP such as resampling audio clips.

pub mod program_state;

pub use program_state::ProgramState;
use rusty_daw_core::MusicalTime;

use self::program_state::TimelineGridState;
use vizia::prelude::*;

/// This is in charge of keeping track of state for the whole program.
///
/// The UI must continually call `ProgramLayer::poll()` (on every frame or an
/// equivalent timer).
#[derive(Debug, Lens, Clone)]
pub struct ProgramLayer {
    /// The state of the whole program.
    ///
    /// Unless explicitely stated, the UI may NOT directly mutate the state of any
    /// of these variables. It is intended for the UI to call the methods on this
    /// struct in order to mutate state.
    pub state: ProgramState,
}

impl ProgramLayer {
    // Create some dummy state for now
    pub fn new() -> Result<Self, ()> {
        Ok(ProgramLayer {
            state: ProgramState {
                engine_running: false,
                notification_log: Vec::new(),
                tracks: Vec::new(),
                timeline_grid: TimelineGridState {
                    horizontal_zoom_level: 0.0,
                    vertical_zoom_level: 0.0,
                    left_start: MusicalTime::from_beats(0),
                    top_start: 0.0,
                    lane_height: 1.0,
                    lanes: Vec::new(),
                    project_length: MusicalTime::from_beats(4),
                    used_lanes: 0,
                },
            },
        })
    }

    pub fn poll(&mut self) {
        // TODO
    }
}

impl Model for ProgramLayer {
    // Update the program layer here
}
