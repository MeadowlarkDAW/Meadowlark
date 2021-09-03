pub mod gain;
pub mod monitor;
pub mod pan;
pub mod sine_gen;
pub mod sum;

use rusty_daw_time::Seconds;

use crate::backend::parameter::Gradient;

pub const SMOOTH_SECS: Seconds = Seconds(5.0 / 1_000.0);
pub const DB_GRADIENT: Gradient = Gradient::Power(0.15);
