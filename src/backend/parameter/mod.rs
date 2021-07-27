// Some code used from baseplug:
//
// https://github.com/wrl/baseplug/blob/trunk/src/parameter.rs
// https://github.com/wrl/baseplug/blob/trunk/LICENSE-APACHE
//
//  Thanks wrl! :)
use basedrop::{Handle, Shared, SharedCell};

pub mod declick;
pub mod smooth;

pub use declick::{Declick, DeclickOutput};
use rusty_daw_time::{SampleRate, Seconds};
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

    shared_normalized: Shared<SharedCell<f32>>,
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
        coll_handle: Handle,
    ) -> (Self, ParamF32Handle) {
        let normalized = value_to_normalized(value, min, max, gradient);

        let handle_value = normalized_to_value(normalized, min, max, gradient);
        let rt_value = match unit {
            Unit::Decibels => db_to_coeff(handle_value),
            _ => handle_value,
        };

        let shared_normalized = Shared::new(
            &coll_handle,
            SharedCell::new(Shared::new(&coll_handle, normalized)),
        );

        let mut smoothed = Smooth::new(rt_value);
        smoothed.set_speed(sample_rate, smooth_secs);

        (
            Self {
                min,
                max,
                gradient,
                unit,
                shared_normalized: Shared::clone(&shared_normalized),
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
                coll_handle,
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
        coll_handle: Handle,
    ) -> (Self, ParamF32Handle) {
        let normalized = normalized.min(1.0).max(0.0);

        let shared_normalized = Shared::new(
            &coll_handle,
            SharedCell::new(Shared::new(&coll_handle, normalized)),
        );

        let handle_value = normalized_to_value(normalized, min_value, max_value, gradient);
        let rt_value = match unit {
            Unit::Decibels => db_to_coeff(handle_value),
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
                shared_normalized: Shared::clone(&shared_normalized),
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
                coll_handle,
            },
        )
    }

    pub fn smoothed(&mut self, frames: usize) -> SmoothOutput<f32> {
        let new_normalized = *self.shared_normalized.get();
        if self.normalized != new_normalized {
            self.normalized = new_normalized;

            let v = normalized_to_value(self.normalized, self.min, self.max, self.gradient);
            self.value = match self.unit {
                Unit::Decibels => db_to_coeff(v),
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

    shared_normalized: Shared<SharedCell<f32>>,
    normalized: f32,

    value: f32,

    coll_handle: Handle,
}

impl ParamF32Handle {
    pub fn normalized(&self) -> f32 {
        self.normalized
    }

    pub fn set_normalized(&mut self, normalized: f32) {
        if self.normalized != normalized {
            let normalized = normalized.min(1.0).max(0.0);

            self.normalized = normalized;
            self.shared_normalized
                .set(Shared::new(&self.coll_handle, normalized));

            self.value = normalized_to_value(normalized, self.min, self.max, self.gradient);
        }
    }

    pub fn value(&self) -> f32 {
        self.value
    }

    pub fn set_value(&mut self, value: f32) {
        if self.value != value {
            self.normalized = value_to_normalized(value, self.min, self.max, self.gradient);
            self.value = normalized_to_value(self.normalized, self.min, self.max, self.gradient);

            self.shared_normalized
                .set(Shared::new(&self.coll_handle, self.normalized));
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

#[inline]
pub fn db_to_coeff(db: f32) -> f32 {
    if db < -90.0 {
        0.0
    } else {
        10.0f32.powf(0.05 * db)
    }
}

#[inline]
pub fn coeff_to_db(coeff: f32) -> f32 {
    if coeff <= 0.00003162277 {
        -90.0
    } else {
        20.0 * coeff.log(10.0)
    }
}
