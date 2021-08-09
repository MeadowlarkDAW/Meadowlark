// Some modified code from baseplug:
//
// https://github.com/wrl/baseplug/blob/trunk/src/parameter.rs
// https://github.com/wrl/baseplug/blob/trunk/LICENSE-APACHE
// https://github.com/wrl/baseplug/blob/trunk/LICENSE-MIT
//
//  Thanks wrl! :)

use std::sync::Arc;

use rusty_daw_time::{SampleRate, Seconds};

use crate::util::decibel::db_to_coeff_clamped_neg_90_db_f32;
use crate::util::AtomicF32;

pub mod declick;
pub mod smooth;

pub use declick::{Declick, DeclickOutput};
pub use smooth::{Smooth, SmoothOutput, SmoothStatus};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Gradient {
    Linear,
    Power(f32),
    Exponential,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Unit {
    Generic,
    Decibels,
}

pub struct ParamF32 {
    min: f32,
    max: f32,
    gradient: Gradient,
    unit: Unit,

    shared_normalized: Arc<AtomicF32>,
    normalized: f32,

    value: f32,

    smoothed: Smooth<f32>,
}

impl ParamF32 {
    pub fn from_value(
        value: f32,
        min: f32,
        max: f32,
        gradient: Gradient,
        unit: Unit,
        smooth_secs: Seconds,
        sample_rate: SampleRate,
    ) -> (Self, ParamF32Handle) {
        let normalized = value_to_normalized(value, min, max, gradient);

        let handle_value = normalized_to_value(normalized, min, max, gradient);
        let rt_value = match unit {
            Unit::Decibels => db_to_coeff_clamped_neg_90_db_f32(handle_value),
            _ => handle_value,
        };

        let shared_normalized = Arc::new(AtomicF32::new(normalized));

        let mut smoothed = Smooth::new(rt_value);
        smoothed.set_speed(sample_rate, smooth_secs);

        (
            Self {
                min,
                max,
                gradient,
                unit,
                shared_normalized: Arc::clone(&shared_normalized),
                normalized,
                value: rt_value,
                smoothed,
            },
            ParamF32Handle {
                min,
                max,
                gradient,
                unit,
                shared_normalized,
                normalized,
                value: handle_value,
            },
        )
    }

    pub fn from_normalized(
        normalized: f32,
        min_value: f32,
        max_value: f32,
        gradient: Gradient,
        unit: Unit,
        smooth_secs: Seconds,
        sample_rate: SampleRate,
    ) -> (Self, ParamF32Handle) {
        let normalized = normalized.clamp(0.0, 1.0);

        let shared_normalized = Arc::new(AtomicF32::new(normalized));

        let handle_value = normalized_to_value(normalized, min_value, max_value, gradient);
        let rt_value = match unit {
            Unit::Decibels => db_to_coeff_clamped_neg_90_db_f32(handle_value),
            _ => handle_value,
        };

        let mut smoothed = Smooth::new(rt_value);
        smoothed.set_speed(sample_rate, smooth_secs);

        (
            Self {
                min: min_value,
                max: max_value,
                gradient,
                unit,
                shared_normalized: Arc::clone(&shared_normalized),
                normalized,
                value: rt_value,
                smoothed,
            },
            ParamF32Handle {
                min: min_value,
                max: max_value,
                gradient,
                unit,
                shared_normalized,
                normalized,
                value: handle_value,
            },
        )
    }

    pub fn smoothed(&mut self, frames: usize) -> SmoothOutput<f32> {
        let new_normalized = self.shared_normalized.get();
        if self.normalized != new_normalized {
            self.normalized = new_normalized;

            let v = normalized_to_value(self.normalized, self.min, self.max, self.gradient);
            self.value = match self.unit {
                Unit::Decibels => db_to_coeff_clamped_neg_90_db_f32(v),
                _ => v,
            };

            self.smoothed.set(self.value);
        }

        self.smoothed.process(frames);
        self.smoothed.update_status();

        self.smoothed.output()
    }

    pub fn min(&self) -> f32 {
        self.min
    }

    pub fn max(&self) -> f32 {
        self.max
    }

    pub fn gradient(&self) -> Gradient {
        self.gradient
    }

    pub fn unit(&self) -> Unit {
        self.unit
    }
}

pub struct ParamF32Handle {
    min: f32,
    max: f32,
    gradient: Gradient,
    unit: Unit,

    shared_normalized: Arc<AtomicF32>,
    normalized: f32,

    value: f32,
}

impl ParamF32Handle {
    pub fn normalized(&self) -> f32 {
        self.normalized
    }

    pub fn set_normalized(&mut self, normalized: f32) {
        if self.normalized != normalized {
            self.normalized = normalized.clamp(0.0, 1.0);

            self.shared_normalized.set(self.normalized);

            self.value = normalized_to_value(self.normalized, self.min, self.max, self.gradient);
        }
    }

    pub fn value(&self) -> f32 {
        self.value
    }

    pub fn set_value(&mut self, value: f32) {
        if self.value != value {
            self.normalized = value_to_normalized(value, self.min, self.max, self.gradient);
            self.value = normalized_to_value(self.normalized, self.min, self.max, self.gradient);

            self.shared_normalized.set(self.normalized);
        }
    }

    pub fn min(&self) -> f32 {
        self.min
    }

    pub fn max(&self) -> f32 {
        self.max
    }

    pub fn gradient(&self) -> Gradient {
        self.gradient
    }

    pub fn unit(&self) -> Unit {
        self.unit
    }
}

fn normalized_to_value(normalized: f32, min: f32, max: f32, gradient: Gradient) -> f32 {
    let normalized = normalized.min(1.0).max(0.0);

    let map = |x: f32| -> f32 {
        let range = max - min;
        (x * range) + min
    };

    match gradient {
        Gradient::Linear => map(normalized),

        Gradient::Power(exponent) => map(normalized.powf(exponent)),

        Gradient::Exponential => {
            if normalized == 0.0 {
                return min;
            }

            if normalized == 1.0 {
                return max;
            }

            let minl = min.log2();
            let range = max.log2() - minl;
            2.0f32.powf((normalized * range) + minl)
        }
    }
}

fn value_to_normalized(value: f32, min: f32, max: f32, gradient: Gradient) -> f32 {
    if value <= min {
        return 0.0;
    }

    if value >= max {
        return 1.0;
    }

    let unmap = |x: f32| -> f32 {
        let range = max - min;
        (x - min) / range
    };

    match gradient {
        Gradient::Linear => unmap(value),

        Gradient::Power(exponent) => unmap(value).powf(1.0 / exponent),

        Gradient::Exponential => {
            let minl = min.log2();
            let range = max.log2() - minl;
            (value.log2() - minl) / range
        }
    }
}
