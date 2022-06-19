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

use meadowlark_core_types::MusicalTime;
pub use program_state::ProgramState;

use program_state::{
    ChannelRackOrientation, ChannelState, PanelState, PatternState, TimelineGridState,
};
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
                channels: vec![
                    ChannelState {
                        name: String::from("Master"),
                        selected: false,
                        color: Color::from("#D4D5D5").into(),
                        subchannels: vec![1, 5],
                        ..Default::default()
                    },
                    ChannelState {
                        name: String::from("Drum Group"),
                        selected: false,
                        color: Color::from("#EDE171").into(),
                        subchannels: vec![2, 3, 4],
                        ..Default::default()
                    },
                    ChannelState {
                        name: String::from("Kick"),
                        selected: false,
                        color: Color::from("#EDE171").into(),
                        subchannels: vec![],
                        ..Default::default()
                    },
                    ChannelState {
                        name: String::from("Snare"),
                        selected: true,
                        color: Color::from("#EDE171").into(),
                        subchannels: vec![],
                        ..Default::default()
                    },
                    ChannelState {
                        name: String::from("Hat"),
                        selected: false,
                        color: Color::from("#EDE171").into(),
                        subchannels: vec![],
                        ..Default::default()
                    },
                    ChannelState {
                        name: String::from("Spicy Synth"),
                        selected: false,
                        color: Color::from("#EA716C").into(),
                        subchannels: vec![],
                        ..Default::default()
                    },
                ],

                patterns: vec![PatternState { name: String::from("Drum Group 1"), channel: 1 }],
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
                panels: PanelState {
                    channel_rack_orientation: ChannelRackOrientation::Horizontal,
                    hide_patterns: false,
                    hide_piano_roll: false,
                    browser_width: 100.0,
                    show_browser: true,
                },
            },
        })
    }

    // TODO
    // pub fn poll(&mut self) {
    //     // TODO
    // }
}

pub enum ProgramEvent {
    SaveProject,
    LoadProject,
}

impl Model for ProgramLayer {
    // Update the program layer here
    fn event(&mut self, cx: &mut Context, event: &mut Event) {
        event.map(|program_event, _| match program_event {
            ProgramEvent::SaveProject => {
                let save_state = serde_json::to_string(&self.state).unwrap();
                std::fs::write("project.json", save_state).unwrap();
            }

            ProgramEvent::LoadProject => {
                let save_state = std::fs::read_to_string("project.json").unwrap();
                let project_state = serde_json::from_str(&save_state).unwrap();
                self.state = project_state;
            }
        });

        self.state.event(cx, event);
    }
}
