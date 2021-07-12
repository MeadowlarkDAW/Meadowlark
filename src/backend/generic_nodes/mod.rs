pub mod gain;
pub mod monitor;
pub mod pan;
pub mod sine_gen;
pub mod sum;

use crate::backend::parameter::Gradient;

pub const SMOOTH_MS: f32 = 5.0;
pub const DB_GRADIENT: Gradient = Gradient::Power(0.15);
