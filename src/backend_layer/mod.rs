//! # Backend (Engine) Layer
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
//! [`Rusty DAW Engine`]: https://github.com/RustyDAW/rusty-daw-engine
//! [`CLAP`]: https://github.com/free-audio/clap
