pub mod gain;
pub mod monitor;
pub mod sine_gen;

use crate::frontend_state::Gradient;

pub const SMOOTH_MS: f32 = 5.0;
pub const DB_GRADIENT: Gradient = Gradient::Power(0.15);
