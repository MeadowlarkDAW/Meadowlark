// Some modified code from baseplug:
//
// https://github.com/wrl/baseplug/blob/trunk/src/parameter.rs
// https://github.com/wrl/baseplug/blob/trunk/LICENSE-APACHE
// https://github.com/wrl/baseplug/blob/trunk/LICENSE-MIT
//
//  Thanks wrl! :)

use std::sync::{
    atomic::{AtomicBool, AtomicI32, Ordering},
    Arc,
};

use crate::atomic_float::{AtomicF32, AtomicF64};
use crate::decibel::{
    coeff_to_db_clamped_neg_90_db_f32, coeff_to_db_clamped_neg_90_db_f64,
    db_to_coeff_clamped_neg_90_db_f32, db_to_coeff_clamped_neg_90_db_f64,
};

mod declick;
mod smooth;

pub use declick::{Declick, DeclickOutput};
pub use smooth::{SmoothF32, SmoothF64, SmoothOutputF32, SmoothOutputF64};

/// A good default value to use as `smooth_secs` parameter when creating a [`ParamF32`]/[`ParamF64`].
///
/// This specifies that the low-pass parameter smoothing filter should use a period of `5 ms`.
///
/// [`ParamF32`]: struct.ParamF32.html
/// [`ParamF64`]: struct.ParamF64.html
pub const DEFAULT_SMOOTH_SECS: f64 = 5.0 / 1_000.0;

/// A good default value to use as `gradient` parameter when creating a [`ParamF32`]/[`ParamF64`] that
/// deals with decibels.
pub const DEFAULT_DB_GRADIENT: Gradient = Gradient::Power(0.15);

/// The gradient used when mapping the normalized value in the range `[0.0, 1.0]` to the
/// desired value.
///
/// For example, it is useful for parameters dealing with decibels to have a mapping
/// gradient around `Power(0.15)`. This is so one tick near the top of the slider/knob
/// controlling this parameter causes a small change in dB around `0.0 dB` and one tick
/// on the other end causes a large change in dB around `-90.0 dB`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Gradient {
    /// Linear mapping
    Linear,
    /// Power mapping
    ///
    /// For example, it is useful for parameters dealing with decibels to have a mapping
    /// gradient around `Power(0.15)`. This is so one tick near the top of the slider/knob
    /// controlling this parameter causes a small change in dB around `0.0 dB` and one tick
    /// on the other end causes a large change in dB around `-90.0 dB`.
    Power(f32),
    /// Exponential (logarithmic) mapping
    ///
    /// This is useful for parameters dealing with frequency in Hz.
    Exponential,
}

/// The unit of this parameter. This signifies how the value displayed to the end user should
/// differ from the actual value used in DSP.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Unit {
    /// Any kind of unit where the value displayed to the end user is the same value used
    /// in the DSP.
    Generic,
    /// Signifies that the value displayed to the end user should be in decibels and the
    /// value used in the DSP should be in raw amplitude.
    ///
    /// In addition, whenever the dB value is less than or equal to `-90.0 dB`, then the
    /// resulting raw DSP ampilitude value will be clamped to `0.0` (essentially equaling
    /// `-infinity dB`).
    Decibels,
}

impl Unit {
    /// Convert the given unit value to the corresponding raw value used in DSP.
    ///
    /// This is only effective when this unit is not of type `Unit::Generic`.
    pub fn unit_to_dsp_f32(&self, value: f32) -> f32 {
        match self {
            Unit::Decibels => db_to_coeff_clamped_neg_90_db_f32(value),
            _ => value,
        }
    }

    /// Convert the given raw DSP value to the corresponding unit value.
    ///
    /// This is only effective when this unit is not of type `Unit::Generic`.
    pub fn dsp_to_unit_f32(&self, dsp_value: f32) -> f32 {
        match self {
            Unit::Decibels => coeff_to_db_clamped_neg_90_db_f32(dsp_value),
            _ => dsp_value,
        }
    }

    /// Convert the given unit value to the corresponding raw value used in DSP.
    ///
    /// This is only effective when this unit is not of type `Unit::Generic`.
    pub fn unit_to_dsp_f64(&self, value: f64) -> f64 {
        match self {
            Unit::Decibels => db_to_coeff_clamped_neg_90_db_f64(value),
            _ => value,
        }
    }

    /// Convert the given raw DSP value to the corresponding unit value.
    ///
    /// This is only effective when this unit is not of type `Unit::Generic`.
    pub fn dsp_to_unit_f64(&self, dsp_value: f64) -> f64 {
        match self {
            Unit::Decibels => coeff_to_db_clamped_neg_90_db_f64(dsp_value),
            _ => dsp_value,
        }
    }
}

/// An auto-smoothed parameter with an `f32` value.
pub struct ParamF32 {
    min_value: f32,
    max_value: f32,
    gradient: Gradient,
    unit: Unit,

    shared_normalized: Arc<AtomicF32>,
    normalized: f32,

    value: f32,
    default_value: f32,

    smoothed: SmoothF32,
    smooth_secs: f64,
}

impl ParamF32 {
    /// Create a Parameter/Handle pair from its (de-normalized) value.
    ///
    /// * value - The initial (de-normalized) value of the parameter.
    /// * default_value - The default (de-normalized) value of the parameter.
    /// * min_value - The minimum (de-normalized) value of the parameter.
    /// * max_value - The maximum (de-normalized) value of the parameter.
    /// * gradient - The [`Gradient`] mapping used when converting from the normalized value
    /// in the range `[0.0, 1.0]` to the desired value. If this parameter deals with decibels,
    /// you may use `ParamF32::DEFAULT_SMOOTH_SECS` as a good default.
    /// * unit - The [`Unit`] that signifies how the value displayed to the end user should
    /// differ from the actual value used in DSP.
    /// * smooth_secs: The period of the low-pass parameter smoothing filter (for declicking). You
    /// may use `ParamF32::DEFAULT_SMOOTH_SECS` as a good default.
    /// * sample_rate: The sample rate of this process. This is used for the low-pass parameter
    /// smoothing filter.
    ///
    /// [`Gradient`]: enum.Gradient.html
    /// [`Unit`]: enum.Unit.html
    #[allow(clippy::too_many_arguments)] // Fix this?
    pub fn from_value(
        value: f32,
        default_value: f32,
        min_value: f32,
        max_value: f32,
        gradient: Gradient,
        unit: Unit,
        smooth_secs: f64,
        sample_rate: u32,
        max_blocksize: usize,
    ) -> (Self, ParamF32Handle) {
        let normalized = value_to_normalized_f32(value, min_value, max_value, gradient);

        let handle_value = normalized_to_value_f32(normalized, min_value, max_value, gradient);
        let rt_value = match unit {
            Unit::Decibels => db_to_coeff_clamped_neg_90_db_f32(handle_value),
            _ => handle_value,
        };

        let shared_normalized = Arc::new(AtomicF32::new(normalized));

        let mut smoothed = SmoothF32::new(rt_value, max_blocksize);
        smoothed.set_speed(sample_rate, smooth_secs);

        (
            Self {
                min_value,
                max_value,
                gradient,
                unit,
                shared_normalized: Arc::clone(&shared_normalized),
                normalized,
                value: rt_value,
                default_value,
                smoothed,
                smooth_secs,
            },
            ParamF32Handle {
                min_value,
                max_value,
                gradient,
                unit,
                default_value,
                shared_normalized,
            },
        )
    }

    /// Create a Parameter/Handle pair from its normalized value in the range `[0.0, 1.0]`.
    ///
    /// * normalized - The initial normalized value of the parameter in the range `[0.0, 1.0]`.
    /// * default_value - The default (de-normalized) value of the parameter.
    /// * min_value - The minimum (de-normalized) value of the parameter.
    /// * max_value - The maximum (de-normalized) value of the parameter.
    /// * gradient - The [`Gradient`] mapping used when converting from the normalized value
    /// in the range `[0.0, 1.0]` to the desired value. If this parameter deals with decibels,
    /// you may use `ParamF32::DEFAULT_SMOOTH_SECS` as a good default.
    /// * unit - The [`Unit`] that signifies how the value displayed to the end user should
    /// differ from the actual value used in DSP.
    /// * smooth_secs: The period of the low-pass parameter smoothing filter (for declicking). You
    /// may use `ParamF32::DEFAULT_SMOOTH_SECS` as a good default.
    /// * sample_rate: The sample rate of this process. This is used for the low-pass parameter
    /// smoothing filter.
    ///
    /// [`Gradient`]: enum.Gradient.html
    /// [`Unit`]: enum.Unit.html
    #[allow(clippy::too_many_arguments)] // Fix this?
    pub fn from_normalized(
        normalized: f32,
        default_value: f32,
        min_value: f32,
        max_value: f32,
        gradient: Gradient,
        unit: Unit,
        smooth_secs: f64,
        sample_rate: u32,
        max_blocksize: usize,
    ) -> (Self, ParamF32Handle) {
        let normalized = normalized.clamp(0.0, 1.0);

        let shared_normalized = Arc::new(AtomicF32::new(normalized));

        let handle_value = normalized_to_value_f32(normalized, min_value, max_value, gradient);
        let rt_value = match unit {
            Unit::Decibels => db_to_coeff_clamped_neg_90_db_f32(handle_value),
            _ => handle_value,
        };

        let mut smoothed = SmoothF32::new(rt_value, max_blocksize);
        smoothed.set_speed(sample_rate, smooth_secs);

        (
            Self {
                min_value,
                max_value,
                gradient,
                unit,
                shared_normalized: Arc::clone(&shared_normalized),
                normalized,
                value: rt_value,
                default_value,
                smoothed,
                smooth_secs,
            },
            ParamF32Handle {
                min_value,
                max_value,
                gradient,
                unit,
                default_value,
                shared_normalized,
            },
        )
    }

    /// Set the (de-normalized) value of this parameter.
    pub fn set_value(&mut self, value: f32) {
        if self.value != value {
            self.normalized =
                value_to_normalized_f32(value, self.min_value, self.max_value, self.gradient);
            self.shared_normalized.set(self.normalized);

            let v = normalized_to_value_f32(
                self.normalized,
                self.min_value,
                self.max_value,
                self.gradient,
            );
            self.value = match self.unit {
                Unit::Decibels => db_to_coeff_clamped_neg_90_db_f32(v),
                _ => v,
            };

            self.smoothed.set(self.value);
        }
    }

    /// Set the normalized value of this parameter in the range `[0.0, 1.0]`.
    pub fn set_normalized(&mut self, normalized: f32) {
        if self.normalized != normalized {
            self.normalized = normalized.clamp(0.0, 1.0);
            self.shared_normalized.set(self.normalized);

            let v = normalized_to_value_f32(
                self.normalized,
                self.min_value,
                self.max_value,
                self.gradient,
            );
            self.value = match self.unit {
                Unit::Decibels => db_to_coeff_clamped_neg_90_db_f32(v),
                _ => v,
            };

            self.smoothed.set(self.value);
        }
    }

    /// Reset this parameter (without any smoothing) to the given (de-normalized) value.
    pub fn reset_from_value(&mut self, value: f32) {
        self.normalized =
            value_to_normalized_f32(value, self.min_value, self.max_value, self.gradient);
        self.shared_normalized.set(self.normalized);

        let v =
            normalized_to_value_f32(self.normalized, self.min_value, self.max_value, self.gradient);
        self.value = match self.unit {
            Unit::Decibels => db_to_coeff_clamped_neg_90_db_f32(v),
            _ => v,
        };

        self.smoothed.reset(self.value);
    }

    /// Reset this parameter (without any smoothing) to the given normalized value in the range `[0.0, 1.0]`.
    pub fn reset_from_normalized(&mut self, normalized: f32) {
        self.normalized = normalized.clamp(0.0, 1.0);
        self.shared_normalized.set(self.normalized);

        let v =
            normalized_to_value_f32(self.normalized, self.min_value, self.max_value, self.gradient);
        self.value = match self.unit {
            Unit::Decibels => db_to_coeff_clamped_neg_90_db_f32(v),
            _ => v,
        };

        self.smoothed.reset(self.value);
    }

    /// Reset the internal smoothing buffer.
    pub fn reset(&mut self) {
        self.smoothed.reset(self.value);
    }

    /// Get the smoothed buffer of values for use in DSP.
    pub fn smoothed(&mut self, frames: usize) -> SmoothOutputF32 {
        let new_normalized = self.shared_normalized.get();
        if self.normalized != new_normalized {
            self.normalized = new_normalized;

            let v = normalized_to_value_f32(
                self.normalized,
                self.min_value,
                self.max_value,
                self.gradient,
            );
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

    /// Update the sample rate (used for the parameter smoothing LPF).
    pub fn set_sample_rate(&mut self, sample_rate: u32) {
        self.smoothed.set_speed(sample_rate, self.smooth_secs);
    }

    /// The minimum value of this parameter.
    pub fn min_value(&self) -> f32 {
        self.min_value
    }

    /// The maximum value of this parameter.
    pub fn max_value(&self) -> f32 {
        self.max_value
    }

    /// The [`Gradient`] mapping used when converting from the normalized value
    /// in the range `[0.0, 1.0]` to the desired value.
    ///
    /// [`Gradient`]: enum.Gradient.html
    pub fn gradient(&self) -> Gradient {
        self.gradient
    }

    /// The [`Unit`] that signifies how the value displayed to the end user should
    /// differ from the actual value used in DSP.
    ///
    /// [`Unit`]: enum.Unit.html
    pub fn unit(&self) -> Unit {
        self.unit
    }

    /// Convert the given value to the corresponding normalized range `[0.0, 1.0]`
    /// of this parameter.
    pub fn value_to_normalized(&self, value: f32) -> f32 {
        value_to_normalized_f32(value, self.min_value, self.max_value, self.gradient)
    }

    /// Convert the given normalized value in the range `[0.0, 1.0]` into the
    /// corresponding value of this parameter.
    pub fn normalized_to_value(&self, normalized: f32) -> f32 {
        normalized_to_value_f32(normalized, self.min_value, self.max_value, self.gradient)
    }

    /// The current normalized value in the range `[0.0, 1.0]`. This is only meant for
    /// communicating with the host. This is not meant to be used to retrieve the latest
    /// value for DSP. To get the latest value for DSP please use `ParamF32::smoothed()`
    /// instead.
    ///
    /// Please note that this should be called *after* calling `ParamF32::smoothed()`
    /// if you need the latest value from the corresponding [`ParamF32Handle`],
    /// otherwise this may not return the latest value.
    ///
    /// [`ParamF32Handle`]: struct.ParamF32Handle.html
    pub fn host_get_normalized(&self) -> f32 {
        self.normalized
    }

    /// The (un-normalized) default value of the parameter.
    pub fn default_value(&self) -> f32 {
        self.default_value
    }

    /// The normalized default value of the parameter in the range `[0.0, 1.0]`.
    pub fn default_normalized(&self) -> f32 {
        self.value_to_normalized(self.default_value)
    }

    /// The current (un-normalized) value of the parameter. This is only meant for
    /// communicating with the host. This is not meant to be used to retrieve the latest
    /// value for DSP. To get the latest value for DSP please use `ParamF32::smoothed()`
    /// instead.
    ///
    /// Please note that this should be called *after* calling `ParamF32::smoothed()`
    /// if you need the latest value from the corresponding [`ParamF32Handle`],
    /// otherwise this may not return the latest value.
    ///
    /// [`ParamF32Handle`]: struct.ParamF32Handle.html
    pub fn host_get_value(&self) -> f32 {
        self.value
    }

    /// Get the shared normalized float value.
    ///
    /// This can be useful to integrate with various plugin APIs.
    pub fn shared_normalized(&self) -> Arc<AtomicF32> {
        Arc::clone(&self.shared_normalized)
    }
}

/// A handle to get and update the value of an auto-smoothed [`ParamF32`] from a UI.
///
/// [`ParamF32`]: struct.ParamF32.html
pub struct ParamF32Handle {
    min_value: f32,
    max_value: f32,
    gradient: Gradient,
    unit: Unit,
    default_value: f32,

    shared_normalized: Arc<AtomicF32>,
}

impl ParamF32Handle {
    /// The normalized value in the range `[0.0, 1.0]`.
    pub fn normalized(&self) -> f32 {
        self.shared_normalized.get()
    }

    /// The (un-normalized) value of this parameter.
    ///
    /// Please note that this is calculated from the shared normalized value every time, so
    /// avoid calling this every frame if you can.
    pub fn value(&self) -> f32 {
        normalized_to_value_f32(
            self.shared_normalized.get(),
            self.min_value,
            self.max_value,
            self.gradient,
        )
    }

    /// The (un-normalized) default value of the parameter.
    pub fn default_value(&self) -> f32 {
        self.default_value
    }

    /// The normalized default value of the parameter in the range `[0.0, 1.0]`.
    pub fn default_normalized(&self) -> f32 {
        self.value_to_normalized(self.default_value)
    }

    /// Set the normalized value of this parameter in the range `[0.0, 1.0]`.
    ///
    /// Please note that this will ***NOT*** automatically notify the host of the value change
    /// if you are using this inside a plugin spec such as VST. It is intended for you use your
    /// own method for achieving this.
    pub fn set_normalized(&self, normalized: f32) {
        self.shared_normalized.set(normalized.clamp(0.0, 1.0));
    }

    /// Set the (un-normalized) value of this parameter.
    ///
    /// Please note that this will ***NOT*** automatically notify the host of the value change
    /// if you are using this inside a plugin spec such as VST. It is intended for you use your
    /// own method for achieving this.
    pub fn set_value(&self, value: f32) {
        let normalized =
            value_to_normalized_f32(value, self.min_value, self.max_value, self.gradient);
        self.set_normalized(normalized);
    }

    /// The minimum value of this parameter.
    pub fn min_value(&self) -> f32 {
        self.min_value
    }

    /// The maximum value of this parameter.
    pub fn max_value(&self) -> f32 {
        self.max_value
    }

    /// The [`Gradient`] mapping used when converting from the normalized value
    /// in the range `[0.0, 1.0]` to the desired value.
    ///
    /// [`Gradient`]: enum.Gradient.html
    pub fn gradient(&self) -> Gradient {
        self.gradient
    }

    /// The [`Unit`] that signifies how the value displayed to the end user should
    /// differ from the actual value used in DSP.
    ///
    /// [`Unit`]: enum.Unit.html
    pub fn unit(&self) -> Unit {
        self.unit
    }

    /// Convert the given value to the corresponding normalized range `[0.0, 1.0]`
    /// of this parameter.
    pub fn value_to_normalized(&self, value: f32) -> f32 {
        value_to_normalized_f32(value, self.min_value, self.max_value, self.gradient)
    }

    /// Convert the given normalized value in the range `[0.0, 1.0]` into the
    /// corresponding value of this parameter.
    pub fn normalized_to_value(&self, normalized: f32) -> f32 {
        normalized_to_value_f32(normalized, self.min_value, self.max_value, self.gradient)
    }

    /// Get the shared normalized float value.
    ///
    /// This can be useful to integrate with various plugin APIs.
    pub fn shared_normalized(&self) -> Arc<AtomicF32> {
        Arc::clone(&self.shared_normalized)
    }
}

impl Clone for ParamF32Handle {
    fn clone(&self) -> Self {
        Self {
            min_value: self.min_value,
            max_value: self.max_value,
            gradient: self.gradient,
            unit: self.unit,
            default_value: self.default_value,

            shared_normalized: Arc::clone(&self.shared_normalized),
        }
    }
}

pub fn normalized_to_value_f32(
    normalized: f32,
    min_value: f32,
    max_value: f32,
    gradient: Gradient,
) -> f32 {
    let normalized = normalized.clamp(0.0, 1.0);

    let map = |x: f32| -> f32 {
        let range = max_value - min_value;
        (x * range) + min_value
    };

    match gradient {
        Gradient::Linear => map(normalized),

        Gradient::Power(exponent) => map(normalized.powf(exponent)),

        Gradient::Exponential => {
            if normalized == 0.0 {
                return min_value;
            }

            if normalized == 1.0 {
                return max_value;
            }

            let minl = min_value.log2();
            let range = max_value.log2() - minl;
            2.0f32.powf((normalized * range) + minl)
        }
    }
}

pub fn value_to_normalized_f32(
    value: f32,
    min_value: f32,
    max_value: f32,
    gradient: Gradient,
) -> f32 {
    if value <= min_value {
        return 0.0;
    }

    if value >= max_value {
        return 1.0;
    }

    let unmap = |x: f32| -> f32 {
        let range = max_value - min_value;
        (x - min_value) / range
    };

    match gradient {
        Gradient::Linear => unmap(value),

        Gradient::Power(exponent) => unmap(value).powf(1.0 / exponent),

        Gradient::Exponential => {
            let minl = min_value.log2();
            let range = max_value.log2() - minl;
            (value.log2() - minl) / range
        }
    }
}

// ------  F64  -------------------------------------------------------------------------

/// An auto-smoothed parameter with an `f64` value.
pub struct ParamF64 {
    min_value: f64,
    max_value: f64,
    gradient: Gradient,
    unit: Unit,

    shared_normalized: Arc<AtomicF64>,
    normalized: f64,

    value: f64,
    default_value: f64,

    smoothed: SmoothF64,
    smooth_secs: f64,
}

impl ParamF64 {
    /// Create a Parameter/Handle pair from its (de-normalized) value.
    ///
    /// * value - The initial (de-normalized) value of the parameter.
    /// * default_value - The default (de-normalized) value of the parameter.
    /// * min_value - The minimum (de-normalized) value of the parameter.
    /// * max_value - The maximum (de-normalized) value of the parameter.
    /// * gradient - The [`Gradient`] mapping used when converting from the normalized value
    /// in the range `[0.0, 1.0]` to the desired value. If this parameter deals with decibels,
    /// you may use `ParamF64::DEFAULT_SMOOTH_SECS` as a good default.
    /// * unit - The [`Unit`] that signifies how the value displayed to the end user should
    /// differ from the actual value used in DSP.
    /// * smooth_secs: The period of the low-pass parameter smoothing filter (for declicking). You
    /// may use `ParamF64::DEFAULT_SMOOTH_SECS` as a good default.
    /// * sample_rate: The sample rate of this process. This is used for the low-pass parameter
    /// smoothing filter.
    ///
    /// [`Gradient`]: enum.Gradient.html
    /// [`Unit`]: enum.Unit.html
    #[allow(clippy::too_many_arguments)] // Fix this?
    pub fn from_value(
        value: f64,
        default_value: f64,
        min_value: f64,
        max_value: f64,
        gradient: Gradient,
        unit: Unit,
        smooth_secs: f64,
        sample_rate: u32,
        max_blocksize: usize,
    ) -> (Self, ParamF64Handle) {
        let normalized = value_to_normalized_f64(value, min_value, max_value, gradient);

        let handle_value = normalized_to_value_f64(normalized, min_value, max_value, gradient);
        let rt_value = match unit {
            Unit::Decibels => db_to_coeff_clamped_neg_90_db_f64(handle_value),
            _ => handle_value,
        };

        let shared_normalized = Arc::new(AtomicF64::new(normalized));

        let mut smoothed = SmoothF64::new(rt_value, max_blocksize);
        smoothed.set_speed(sample_rate, smooth_secs);

        (
            Self {
                min_value,
                max_value,
                gradient,
                unit,
                shared_normalized: Arc::clone(&shared_normalized),
                normalized,
                value: rt_value,
                default_value,
                smoothed,
                smooth_secs,
            },
            ParamF64Handle {
                min_value,
                max_value,
                gradient,
                unit,
                default_value,
                shared_normalized,
            },
        )
    }

    /// Create a Parameter/Handle pair from its normalized value in the range `[0.0, 1.0]`.
    ///
    /// * normalized - The initial normalized value of the parameter in the range `[0.0, 1.0]`.
    /// * default_value - The default (de-normalized) value of the parameter.
    /// * min_value - The minimum (de-normalized) value of the parameter.
    /// * max_value - The maximum (de-normalized) value of the parameter.
    /// * gradient - The [`Gradient`] mapping used when converting from the normalized value
    /// in the range `[0.0, 1.0]` to the desired value. If this parameter deals with decibels,
    /// you may use `ParamF64::DEFAULT_SMOOTH_SECS` as a good default.
    /// * unit - The [`Unit`] that signifies how the value displayed to the end user should
    /// differ from the actual value used in DSP.
    /// * smooth_secs: The period of the low-pass parameter smoothing filter (for declicking). You
    /// may use `ParamF64::DEFAULT_SMOOTH_SECS` as a good default.
    /// * sample_rate: The sample rate of this process. This is used for the low-pass parameter
    /// smoothing filter.
    ///
    /// [`Gradient`]: enum.Gradient.html
    /// [`Unit`]: enum.Unit.html
    #[allow(clippy::too_many_arguments)] // Fix this?
    pub fn from_normalized(
        normalized: f64,
        default_value: f64,
        min_value: f64,
        max_value: f64,
        gradient: Gradient,
        unit: Unit,
        smooth_secs: f64,
        sample_rate: u32,
        max_blocksize: usize,
    ) -> (Self, ParamF64Handle) {
        let normalized = normalized.clamp(0.0, 1.0);

        let shared_normalized = Arc::new(AtomicF64::new(normalized));

        let handle_value = normalized_to_value_f64(normalized, min_value, max_value, gradient);
        let rt_value = match unit {
            Unit::Decibels => db_to_coeff_clamped_neg_90_db_f64(handle_value),
            _ => handle_value,
        };

        let mut smoothed = SmoothF64::new(rt_value, max_blocksize);
        smoothed.set_speed(sample_rate, smooth_secs);

        (
            Self {
                min_value,
                max_value,
                gradient,
                unit,
                shared_normalized: Arc::clone(&shared_normalized),
                normalized,
                value: rt_value,
                default_value,
                smoothed,
                smooth_secs,
            },
            ParamF64Handle {
                min_value,
                max_value,
                gradient,
                unit,
                default_value,
                shared_normalized,
            },
        )
    }

    /// Set the (de-normalized) value of this parameter.
    pub fn set_value(&mut self, value: f64) {
        if self.value != value {
            self.normalized =
                value_to_normalized_f64(value, self.min_value, self.max_value, self.gradient);
            self.shared_normalized.set(self.normalized);

            let v = normalized_to_value_f64(
                self.normalized,
                self.min_value,
                self.max_value,
                self.gradient,
            );
            self.value = match self.unit {
                Unit::Decibels => db_to_coeff_clamped_neg_90_db_f64(v),
                _ => v,
            };

            self.smoothed.set(self.value);
        }
    }

    /// Set the normalized value of this parameter in the range `[0.0, 1.0]`.
    pub fn set_normalized(&mut self, normalized: f64) {
        if self.normalized != normalized {
            self.normalized = normalized.clamp(0.0, 1.0);
            self.shared_normalized.set(self.normalized);

            let v = normalized_to_value_f64(
                self.normalized,
                self.min_value,
                self.max_value,
                self.gradient,
            );
            self.value = match self.unit {
                Unit::Decibels => db_to_coeff_clamped_neg_90_db_f64(v),
                _ => v,
            };

            self.smoothed.set(self.value);
        }
    }

    /// Reset this parameter (without any smoothing) to the given (de-normalized) value.
    pub fn reset_from_value(&mut self, value: f64) {
        self.normalized =
            value_to_normalized_f64(value, self.min_value, self.max_value, self.gradient);
        self.shared_normalized.set(self.normalized);

        let v =
            normalized_to_value_f64(self.normalized, self.min_value, self.max_value, self.gradient);
        self.value = match self.unit {
            Unit::Decibels => db_to_coeff_clamped_neg_90_db_f64(v),
            _ => v,
        };

        self.smoothed.reset(self.value);
    }

    /// Reset this parameter (without any smoothing) to the given normalized value in the range `[0.0, 1.0]`.
    pub fn reset_from_normalized(&mut self, normalized: f64) {
        self.normalized = normalized.clamp(0.0, 1.0);
        self.shared_normalized.set(self.normalized);

        let v =
            normalized_to_value_f64(self.normalized, self.min_value, self.max_value, self.gradient);
        self.value = match self.unit {
            Unit::Decibels => db_to_coeff_clamped_neg_90_db_f64(v),
            _ => v,
        };

        self.smoothed.reset(self.value);
    }

    /// Reset the internal smoothing buffer.
    pub fn reset(&mut self) {
        self.smoothed.reset(self.value);
    }

    /// Get the smoothed buffer of values for use in DSP.
    pub fn smoothed(&mut self, frames: usize) -> SmoothOutputF64 {
        let new_normalized = self.shared_normalized.get();
        if self.normalized != new_normalized {
            self.normalized = new_normalized;

            let v = normalized_to_value_f64(
                self.normalized,
                self.min_value,
                self.max_value,
                self.gradient,
            );
            self.value = match self.unit {
                Unit::Decibels => db_to_coeff_clamped_neg_90_db_f64(v),
                _ => v,
            };

            self.smoothed.set(self.value);
        }

        self.smoothed.process(frames);
        self.smoothed.update_status();

        self.smoothed.output()
    }

    /// Update the sample rate (used for the parameter smoothing LPF).
    pub fn set_sample_rate(&mut self, sample_rate: u32) {
        self.smoothed.set_speed(sample_rate, self.smooth_secs);
    }

    /// The minimum value of this parameter.
    pub fn min_value(&self) -> f64 {
        self.min_value
    }

    /// The maximum value of this parameter.
    pub fn max_value(&self) -> f64 {
        self.max_value
    }

    /// The [`Gradient`] mapping used when converting from the normalized value
    /// in the range `[0.0, 1.0]` to the desired value.
    ///
    /// [`Gradient`]: enum.Gradient.html
    pub fn gradient(&self) -> Gradient {
        self.gradient
    }

    /// The [`Unit`] that signifies how the value displayed to the end user should
    /// differ from the actual value used in DSP.
    ///
    /// [`Unit`]: enum.Unit.html
    pub fn unit(&self) -> Unit {
        self.unit
    }

    /// Convert the given value to the corresponding normalized range `[0.0, 1.0]`
    /// of this parameter.
    pub fn value_to_normalized(&self, value: f64) -> f64 {
        value_to_normalized_f64(value, self.min_value, self.max_value, self.gradient)
    }

    /// Convert the given normalized value in the range `[0.0, 1.0]` into the
    /// corresponding value of this parameter.
    pub fn normalized_to_value(&self, normalized: f64) -> f64 {
        normalized_to_value_f64(normalized, self.min_value, self.max_value, self.gradient)
    }

    /// The current normalized value in the range `[0.0, 1.0]`. This is only meant for
    /// communicating with the host. This is not meant to be used to retrieve the latest
    /// value for DSP. To get the latest value for DSP please use `ParamF32::smoothed()`
    /// instead.
    ///
    /// Please note that this should be called *after* calling `ParamF32::smoothed()`
    /// if you need the latest value from the corresponding [`ParamF32Handle`],
    /// otherwise this may not return the latest value.
    ///
    /// [`ParamF32Handle`]: struct.ParamF32Handle.html
    pub fn host_get_normalized(&self) -> f64 {
        self.normalized
    }

    /// The (un-normalized) default value of the parameter.
    pub fn default_value(&self) -> f64 {
        self.default_value
    }

    /// The normalized default value of the parameter in the range `[0.0, 1.0]`.
    pub fn default_normalized(&self) -> f64 {
        self.value_to_normalized(self.default_value)
    }

    /// The current (un-normalized) value of the parameter. This is only meant for
    /// communicating with the host. This is not meant to be used to retrieve the latest
    /// value for DSP. To get the latest value for DSP please use `ParamF64::smoothed()`
    /// instead.
    ///
    /// Please note that this should be called *after* calling `ParamF64::smoothed()`
    /// if you need the latest value from the corresponding [`ParamF64Handle`],
    /// otherwise this may not return the latest value.
    ///
    /// [`ParamF64Handle`]: struct.ParamF64Handle.html
    pub fn host_get_value(&self) -> f64 {
        self.value
    }

    /// Get the shared normalized float value.
    ///
    /// This can be useful to integrate with various plugin APIs.
    pub fn shared_normalized(&self) -> Arc<AtomicF64> {
        Arc::clone(&self.shared_normalized)
    }
}

/// A handle to get and update the value of an auto-smoothed [`ParamF64`] from a UI.
///
/// [`ParamF64`]: struct.ParamF64.html
pub struct ParamF64Handle {
    min_value: f64,
    max_value: f64,
    gradient: Gradient,
    unit: Unit,
    default_value: f64,

    shared_normalized: Arc<AtomicF64>,
}

impl ParamF64Handle {
    /// The normalized value in the range `[0.0, 1.0]`.
    pub fn normalized(&self) -> f64 {
        self.shared_normalized.get()
    }

    /// The (un-normalized) value of this parameter.
    ///
    /// Please note that this is calculated from the shared normalized value every time, so
    /// avoid calling this every frame if you can.
    pub fn value(&self) -> f64 {
        normalized_to_value_f64(
            self.shared_normalized.get(),
            self.min_value,
            self.max_value,
            self.gradient,
        )
    }

    /// The (un-normalized) default value of the parameter.
    pub fn default_value(&self) -> f64 {
        self.default_value
    }

    /// The normalized default value of the parameter in the range `[0.0, 1.0]`.
    pub fn default_normalized(&self) -> f64 {
        self.value_to_normalized(self.default_value)
    }

    /// Set the normalized value of this parameter in the range `[0.0, 1.0]`.
    ///
    /// Please note that this will ***NOT*** automatically notify the host of the value change
    /// if you are using this inside a plugin spec such as VST. It is intended for you use your
    /// own method for achieving this.
    pub fn set_normalized(&self, normalized: f64) {
        self.shared_normalized.set(normalized.clamp(0.0, 1.0));
    }

    /// Set the (un-normalized) value of this parameter.
    ///
    /// Please note that this will ***NOT*** automatically notify the host of the value change
    /// if you are using this inside a plugin spec such as VST. It is intended for you use your
    /// own method for achieving this.
    pub fn set_value(&self, value: f64) {
        let normalized =
            value_to_normalized_f64(value, self.min_value, self.max_value, self.gradient);
        self.set_normalized(normalized);
    }

    /// The minimum value of this parameter.
    pub fn min_value(&self) -> f64 {
        self.min_value
    }

    /// The maximum value of this parameter.
    pub fn max_value(&self) -> f64 {
        self.max_value
    }

    /// The [`Gradient`] mapping used when converting from the normalized value
    /// in the range `[0.0, 1.0]` to the desired value.
    ///
    /// [`Gradient`]: enum.Gradient.html
    pub fn gradient(&self) -> Gradient {
        self.gradient
    }

    /// The [`Unit`] that signifies how the value displayed to the end user should
    /// differ from the actual value used in DSP.
    ///
    /// [`Unit`]: enum.Unit.html
    pub fn unit(&self) -> Unit {
        self.unit
    }

    /// Convert the given value to the corresponding normalized range `[0.0, 1.0]`
    /// of this parameter.
    pub fn value_to_normalized(&self, value: f64) -> f64 {
        value_to_normalized_f64(value, self.min_value, self.max_value, self.gradient)
    }

    /// Convert the given normalized value in the range `[0.0, 1.0]` into the
    /// corresponding value of this parameter.
    pub fn normalized_to_value(&self, normalized: f64) -> f64 {
        normalized_to_value_f64(normalized, self.min_value, self.max_value, self.gradient)
    }

    /// Get the shared normalized float value.
    ///
    /// This can be useful to integrate with various plugin APIs.
    pub fn shared_normalized(&self) -> Arc<AtomicF64> {
        Arc::clone(&self.shared_normalized)
    }
}

impl Clone for ParamF64Handle {
    fn clone(&self) -> Self {
        Self {
            min_value: self.min_value,
            max_value: self.max_value,
            gradient: self.gradient,
            unit: self.unit,
            default_value: self.default_value,

            shared_normalized: Arc::clone(&self.shared_normalized),
        }
    }
}

pub fn normalized_to_value_f64(
    normalized: f64,
    min_value: f64,
    max_value: f64,
    gradient: Gradient,
) -> f64 {
    let normalized = normalized.clamp(0.0, 1.0);

    let map = |x: f64| -> f64 {
        let range = max_value - min_value;
        (x * range) + min_value
    };

    match gradient {
        Gradient::Linear => map(normalized),

        Gradient::Power(exponent) => map(normalized.powf(f64::from(exponent))),

        Gradient::Exponential => {
            if normalized == 0.0 {
                return min_value;
            }

            if normalized == 1.0 {
                return max_value;
            }

            let minl = min_value.log2();
            let range = max_value.log2() - minl;
            2.0f64.powf((normalized * range) + minl)
        }
    }
}

pub fn value_to_normalized_f64(
    value: f64,
    min_value: f64,
    max_value: f64,
    gradient: Gradient,
) -> f64 {
    if value <= min_value {
        return 0.0;
    }

    if value >= max_value {
        return 1.0;
    }

    let unmap = |x: f64| -> f64 {
        let range = max_value - min_value;
        (x - min_value) / range
    };

    match gradient {
        Gradient::Linear => unmap(value),

        Gradient::Power(exponent) => unmap(value).powf(1.0 / f64::from(exponent)),

        Gradient::Exponential => {
            let minl = min_value.log2();
            let range = max_value.log2() - minl;
            (value.log2() - minl) / range
        }
    }
}

/// A parameter with an `i32` value.
pub struct ParamI32 {
    min_value: i32,
    max_value: i32,
    default_value: i32,

    shared: Arc<AtomicI32>,
}

impl ParamI32 {
    /// Create a Parameter/Handle pair from its (de-normalized) value.
    ///
    /// * value - The initial (de-normalized) value of the parameter.
    /// * default_value - The (de-normalized) default value of the parameter.
    /// * min_value - The minimum (de-normalized) value of the parameter.
    /// * max_value - The maximum (de-normalized) value of the parameter.
    pub fn from_value(
        value: i32,
        default_value: i32,
        min_value: i32,
        max_value: i32,
    ) -> (Self, ParamI32Handle) {
        let value = value.clamp(min_value, max_value);

        let shared = Arc::new(AtomicI32::new(value));

        (
            Self { min_value, max_value, default_value, shared: Arc::clone(&shared) },
            ParamI32Handle { min_value, max_value, default_value, shared },
        )
    }

    /// Create a Parameter/Handle pair from its normalized value in the range `[0.0, 1.0]`.
    ///
    /// * normalized - The initial normalized value of the parameter in the range `[0.0, 1.0]`.
    /// * default_value - The (de-normalized) default value of the parameter.
    /// * min - The minimum (de-normalized) value of the parameter.
    /// * max - The maximum (de-normalized) value of the parameter.
    pub fn from_normalized(
        normalized: f64,
        default_value: i32,
        min_value: i32,
        max_value: i32,
    ) -> (Self, ParamI32Handle) {
        let normalized = normalized.clamp(0.0, 1.0);
        let value =
            ((normalized * f64::from(max_value - min_value)) + f64::from(min_value)).round() as i32;

        Self::from_value(value, default_value, min_value, max_value)
    }

    /// Set the (de-normalized) value of this parameter.
    pub fn set_value(&mut self, value: i32) {
        self.shared.store(value.clamp(self.min_value, self.max_value), Ordering::Relaxed);
    }

    /// Set the normalized value of this parameter in the range `[0.0, 1.0]`.
    pub fn set_normalized(&mut self, normalized: f64) {
        let normalized = normalized.clamp(0.0, 1.0);
        let value = ((normalized * f64::from(self.max_value - self.min_value))
            + f64::from(self.max_value))
        .round() as i32;

        self.set_value(value);
    }

    /// The (un-normalized) value of this parameter.
    pub fn value(&self) -> i32 {
        self.shared.load(Ordering::Relaxed)
    }

    /// The (un-normalized) default value of the parameter.
    pub fn default_value(&self) -> i32 {
        self.default_value
    }

    /// The normalized default value of the parameter in the range `[0.0, 1.0]`.
    pub fn default_normalized(&self) -> f64 {
        self.value_to_normalized(self.default_value)
    }

    /// The minimum value of this parameter.
    pub fn min_value(&self) -> i32 {
        self.min_value
    }

    /// The maximum value of this parameter.
    pub fn max_value(&self) -> i32 {
        self.max_value
    }

    /// Convert the given value to the corresponding normalized range `[0.0, 1.0]`
    /// of this parameter.
    pub fn value_to_normalized(&self, value: i32) -> f64 {
        let value = value.clamp(self.min_value, self.max_value);
        f64::from(value - self.min_value) / f64::from(self.max_value - self.min_value)
    }

    /// Convert the given normalized value in the range `[0.0, 1.0]` into the
    /// corresponding value of this parameter.
    pub fn normalized_to_value(&self, normalized: f64) -> i32 {
        let normalized = normalized.clamp(0.0, 1.0);
        ((normalized * f64::from(self.max_value - self.min_value)) + f64::from(self.max_value))
            .round() as i32
    }
}

/// A handle to get and update the value of a [`ParamI32`] from a UI.
///
/// [`ParamI32`]: struct.ParamI32.html
pub struct ParamI32Handle {
    min_value: i32,
    max_value: i32,
    default_value: i32,

    shared: Arc<AtomicI32>,
}

impl ParamI32Handle {
    /// The (un-normalized) value of this parameter.
    pub fn value(&self) -> i32 {
        self.shared.load(Ordering::Relaxed)
    }

    /// The (un-normalized) default value of the parameter.
    pub fn default_value(&self) -> i32 {
        self.default_value
    }

    /// The normalized default value of the parameter in the range `[0.0, 1.0]`.
    pub fn default_normalized(&self) -> f64 {
        self.value_to_normalized(self.default_value)
    }

    /// Set the (de-normalized) value of this parameter.
    pub fn set_value(&mut self, value: i32) {
        self.shared.store(value.clamp(self.min_value, self.max_value), Ordering::Relaxed);
    }

    /// Set the normalized value of this parameter in the range `[0.0, 1.0]`.
    pub fn set_normalized(&mut self, normalized: f64) {
        self.set_value(self.normalized_to_value(normalized));
    }

    /// The minimum value of this parameter.
    pub fn min_value(&self) -> i32 {
        self.min_value
    }

    /// The maximum value of this parameter.
    pub fn max_value(&self) -> i32 {
        self.max_value
    }

    /// Convert the given value to the corresponding normalized range `[0.0, 1.0]`
    /// of this parameter.
    pub fn value_to_normalized(&self, value: i32) -> f64 {
        let value = value.clamp(self.min_value, self.max_value);
        f64::from(value - self.min_value) / f64::from(self.max_value - self.min_value)
    }

    /// Convert the given normalized value in the range `[0.0, 1.0]` into the
    /// corresponding value of this parameter.
    pub fn normalized_to_value(&self, normalized: f64) -> i32 {
        let normalized = normalized.clamp(0.0, 1.0);
        ((normalized * f64::from(self.max_value - self.min_value)) + f64::from(self.max_value))
            .round() as i32
    }
}

impl Clone for ParamI32Handle {
    fn clone(&self) -> Self {
        Self {
            min_value: self.min_value,
            max_value: self.max_value,
            default_value: self.default_value,
            shared: Arc::clone(&self.shared),
        }
    }
}

/// A parameter with an `bool` value.
pub struct ParamBool {
    shared: Arc<AtomicBool>,
    default_value: bool,
}

impl ParamBool {
    /// Create a Parameter/Handle pair from its (de-normalized) boolean value.
    ///
    /// * value - The initial boolean value of the parameter.
    /// * default_value - The default boolean value of the parameter.
    pub fn from_value(value: bool, default_value: bool) -> (Self, ParamBoolHandle) {
        let shared = Arc::new(AtomicBool::new(value));

        (
            Self { shared: Arc::clone(&shared), default_value },
            ParamBoolHandle { shared, default_value },
        )
    }

    /// Create a Parameter/Handle pair from its normalized value in the range `[0.0, 1.0]`.
    ///
    /// * normalized - The initial normalized value of the parameter in the range `[0.0, 1.0]`.
    /// * default_value - The (un-normalized) default boolean value of the parameter.
    pub fn from_normalized(normalized: f32, default_value: bool) -> (Self, ParamBoolHandle) {
        let normalized = normalized.clamp(0.0, 1.0);
        let value = normalized >= 0.5;

        Self::from_value(value, default_value)
    }

    /// Set the (de-normalized) boolean value of this parameter.
    pub fn set_value(&mut self, value: bool) {
        self.shared.store(value, Ordering::Relaxed);
    }

    /// Set the normalized value of this parameter in the range `[0.0, 1.0]`.
    pub fn set_normalized(&mut self, normalized: f32) {
        let normalized = normalized.clamp(0.0, 1.0);
        let value = normalized >= 0.5;

        self.set_value(value);
    }

    /// The (un-normalized) boolean value of this parameter.
    pub fn value(&self) -> bool {
        self.shared.load(Ordering::Relaxed)
    }

    /// The (un-normalized) default boolean value of the parameter.
    pub fn default_value(&self) -> bool {
        self.default_value
    }

    /// The normalized default value of the parameter in the range `[0.0, 1.0]`.
    pub fn default_normalized(&self) -> f32 {
        self.value_to_normalized(self.default_value)
    }

    /// Convert the given value to the corresponding normalized range `[0.0, 1.0]`
    /// of this parameter.
    pub fn value_to_normalized(&self, value: bool) -> f32 {
        if value {
            1.0
        } else {
            0.0
        }
    }

    /// Convert the given normalized value in the range `[0.0, 1.0]` into the
    /// corresponding boolean value of this parameter.
    pub fn normalized_to_value(&self, normalized: f32) -> bool {
        let normalized = normalized.clamp(0.0, 1.0);
        normalized >= 0.5
    }
}

/// A handle to get and update the value of a [`ParamBool`] from a UI.
///
/// [`ParamBool`]: struct.ParamBool.html
pub struct ParamBoolHandle {
    shared: Arc<AtomicBool>,
    default_value: bool,
}

impl ParamBoolHandle {
    /// The (un-normalized) boolean value of this parameter.
    pub fn value(&self) -> bool {
        self.shared.load(Ordering::Relaxed)
    }

    /// The (un-normalized) default boolean value of the parameter.
    pub fn default_value(&self) -> bool {
        self.default_value
    }

    /// The normalized default value of the parameter in the range `[0.0, 1.0]`.
    pub fn default_normalized(&self) -> f32 {
        self.value_to_normalized(self.default_value)
    }

    /// The normalized value of this parameter in the range `[0.0, `1.0]`.
    pub fn normalized(&self) -> f32 {
        self.value_to_normalized(self.value())
    }

    /// Set the (de-normalized) boolean value of this parameter.
    pub fn set_value(&mut self, value: bool) {
        self.shared.store(value, Ordering::Relaxed);
    }

    /// Set the normalized value of this parameter in the range `[0.0, 1.0]`.
    pub fn set_normalized(&mut self, normalized: f32) {
        let normalized = normalized.clamp(0.0, 1.0);
        let value = normalized >= 0.5;

        self.set_value(value);
    }

    /// Convert the given value to the corresponding normalized range `[0.0, 1.0]`
    /// of this parameter.
    pub fn value_to_normalized(&self, value: bool) -> f32 {
        if value {
            1.0
        } else {
            0.0
        }
    }

    /// Convert the given normalized value in the range `[0.0, 1.0]` into the
    /// corresponding boolean value of this parameter.
    pub fn normalized_to_value(&self, normalized: f32) -> bool {
        let normalized = normalized.clamp(0.0, 1.0);
        normalized >= 0.5
    }
}

impl Clone for ParamBoolHandle {
    fn clone(&self) -> Self {
        Self { shared: Arc::clone(&self.shared), default_value: self.default_value }
    }
}
