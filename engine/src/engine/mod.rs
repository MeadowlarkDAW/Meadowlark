pub(crate) mod audio_thread;
pub(crate) mod timer_wheel;

mod main_thread;
mod process_thread;
mod settings;
mod tempo_map;

pub mod error;
pub mod modify_request;

pub use audio_thread::EngineAudioThread;
pub use main_thread::*;
pub use settings::{
    ActivateEngineSettings, EngineSettings, DEFAULT_GARBAGE_COLLECT_INTERVAL_MS,
    DEFAULT_IDLE_INTERVAL_MS,
};
pub use tempo_map::{DefaultTempoMap, EngineTempoMap, TransportInfoAtFrame};
