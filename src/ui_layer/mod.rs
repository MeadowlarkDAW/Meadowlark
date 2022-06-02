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

use crate::program_layer::ProgramLayer;

pub fn run_ui(program_layer: ProgramLayer) {
    // Go nuts @geom3trik ;)
}
