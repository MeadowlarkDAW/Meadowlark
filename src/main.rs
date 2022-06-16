//! Architecture Overview
//!
//! Meadowlark is mainly divided into three layers: The backend layer, the
//! program layer, and the ui layer.
//!
//! ------------------------------------------------------------------
//!
//! ## Backend (Engine) Layer
//!
//! This layer owns the bulk of the "engine", which owns the audio graph and the
//! plugins it hosts. It also automatically recompiles the audio-graph
//! behind-the-scenes when necessary.
//!
//! The bulk of this engine lives in a separate crate called [`Rusty DAW Engine`].
//! Having this live in its own crate will make it easier for developers to create
//! their own frontend for their own open-source DAW if they wish.
//!
//! The engine takes messages from the program layer to spawn plugins, remove
//! plugins, and to connect plugins together. The engine then sends events back to
//! the program layer describing which operations were successful and which were
//! not. This message-passing model also allows the engine to run fully
//! asynchronously from the rest of the program.
//!
//! The events that the engine sends back may contain `PluginHandle`'s, which the
//! program layer can use to interface with the plugin such as controlling its
//! parameters.
//!
//! Everything in the audio graph is treated as if it were a "plugin", including
//! the timeline, the metronome, and the sample browser. This internal plugin
//! format is very closely modelled after the [`CLAP`] plugin format.
//!
//! ------------------------------------------------------------------
//!
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
//!
//! ------------------------------------------------------------------
//!
//! # UI (Frontend) Layer
//!
//! This layer is in charge of displaying a UI to the user. It is also
//! responsible for running scripts.
//!
//! The UI is implemented with the [`VIZIA`] GUI library.
//!
//! [`Rusty DAW Engine`]: https://github.com/RustyDAW/rusty-daw-engine
//! [`CLAP`]: https://github.com/free-audio/clap
//! [`VIZIA`]: https://github.com/vizia/vizia

mod backend_layer;
mod program_layer;
mod ui_layer;

fn main() -> Result<(), String> {
    // We use the fast_log crate for logging.
    //
    // TODO: Ability to log to a file.
    #[cfg(debug_assertions)]
    fast_log::init(fast_log::Config::new().console().level(log::LevelFilter::Debug)).unwrap();
    #[cfg(not(debug_assertions))]
    fast_log::init(fast_log::Config::new().console().level(log::LevelFilter::Info)).unwrap();

    //let program_layer = program_layer::ProgramLayer::new().unwrap();

    ui_layer::run_ui()
}
