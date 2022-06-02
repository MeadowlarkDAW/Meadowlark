//! Architecture Overview
//!
//! Meadowlark is mainly divided into three layers: The backend layer, the
//! program layer, and the ui layer.
//!
//! ------------------------------------------------------------------
//!
//! # Backend Layer - (The bottom-most layer)
//!
//! This layer owns the actual audio graph engine and plugins. It handles
//! hosting plugins and compiling the audio graph into a schedule. It also
//! executes the schedule in a separate realtime thread.
//!
//! Note that the backend layer does not actually own the audio thread.
//! Instead, it runs in its own isolated (sandboxed) process (TODO). The
//! program layer interfaces with the backend layer in a similar manner
//! to a server-client model.
//!
//! The bulk of the backend code lives in a separate crate:
//! https://github.com/RustyDAW/rusty-daw-engine
//!
//! ------------------------------------------------------------------
//!
//! # Program Layer - (The middle layer)
//!
//! This layer owns the actual state of the program.
//!
//! It is solely in charge of mutating this state. The backend layer and
//! the UI layer cannot mutate this state directly (with the exception of
//! some UI-specific state that does not need to be undo-able such as
//! panel or window size). The backend layer indirectly mutates this state
//! by sending events, and the ui layer indirectly mutates this state by
//! calling methods on the ProgramState struct. The program layer is in
//! charge of handling these events and properly mutating the state accordingly.
//!
//! The program layer owns the audio thread and is in charge of
//! connecting to the system's audio and MIDI devices. It also owns the
//! handle to the BackendLayerHandle struct.
//!
//! The program layer is also in charge of some offline DSP such as
//! resampling audio clips.
//!
//! ------------------------------------------------------------------
//!
//! # UI Layer - (The top-most layer)
//!
//! This layer is in charge of displaying a UI to the user.
//!
//! The UI layer cannot access the backend layer directly. It must go
//! through the program layer. The UI layer owns the ProgramLayer
//! struct.
//!
//! This layer is also responsible for running scripts (once we
//! implement that).
//!
//! The UI is implemented with VIZIA GUI library:
//! https://github.com/vizia/vizia

mod backend_layer;
mod program_layer;
mod ui_layer;

fn main() {
    // We use the fast_log crate for logging.
    //
    // TODO: Ability to log to a file.
    #[cfg(debug_assertions)]
    fast_log::init(fast_log::Config::new().console().level(log::LevelFilter::Debug)).unwrap();
    #[cfg(not(debug_assertions))]
    fast_log::init(fast_log::Config::new().console().level(log::LevelFilter::Info)).unwrap();

    let program_layer = program_layer::ProgramLayer::new().unwrap();

    ui_layer::run_ui(program_layer)
}
