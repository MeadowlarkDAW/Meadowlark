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
