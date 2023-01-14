// Some modified code from baseplug:
//
// https://github.com/wrl/baseplug/blob/trunk/src/smooth.rs
// https://github.com/wrl/baseplug/blob/trunk/LICENSE-APACHE
// https://github.com/wrl/baseplug/blob/trunk/LICENSE-MIT
//
//  Thanks wrl! :)

use std::fmt;
use std::ops;
use std::slice;

const SETTLE: f32 = 0.00001f32;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum SmoothStatus {
    Inactive,
    Active,
    Deactivating,
}

impl SmoothStatus {
    fn is_active(&self) -> bool {
        self != &SmoothStatus::Inactive
    }
}

pub struct SmoothOutputF32<'a> {
    pub values: &'a [f32],
    pub status: SmoothStatus,
}

impl<'a> SmoothOutputF32<'a> {
    pub fn is_smoothing(&self) -> bool {
        self.status.is_active()
    }
}

impl<'a, I> ops::Index<I> for SmoothOutputF32<'a>
where
    I: slice::SliceIndex<[f32]>,
{
    type Output = I::Output;

    #[inline]
    fn index(&self, idx: I) -> &I::Output {
        &self.values[idx]
    }
}

pub struct SmoothF32 {
    output: Vec<f32>,
    input: f32,

    status: SmoothStatus,

    a: f32,
    b: f32,
    last_output: f32,
}

impl SmoothF32 {
    pub fn new(input: f32, max_blocksize: usize) -> Self {
        Self {
            status: SmoothStatus::Inactive,
            input,
            output: vec![input; max_blocksize],

            a: 1.0,
            b: 0.0,
            last_output: input,
        }
    }

    pub fn reset(&mut self, val: f32) {
        self.status = SmoothStatus::Inactive;
        self.input = val;
        self.last_output = val;

        let max_blocksize = self.output.len();

        self.output.clear();
        self.output.resize(max_blocksize, val);
    }

    pub fn set(&mut self, val: f32) {
        self.input = val;
        self.status = SmoothStatus::Active;
    }

    pub fn dest(&self) -> f32 {
        self.input
    }

    pub fn output(&self) -> SmoothOutputF32 {
        SmoothOutputF32 { values: &self.output, status: self.status }
    }

    pub fn current_value(&self) -> (f32, SmoothStatus) {
        (self.last_output, self.status)
    }

    pub fn update_status_with_epsilon(&mut self, epsilon: f32) -> SmoothStatus {
        let status = self.status;

        match status {
            SmoothStatus::Active => {
                if (self.input - self.output[0]).abs() < epsilon {
                    self.reset(self.input);
                    self.status = SmoothStatus::Deactivating;
                }
            }

            SmoothStatus::Deactivating => self.status = SmoothStatus::Inactive,

            _ => (),
        };

        self.status
    }

    pub fn process(&mut self, frames: usize) {
        if self.status != SmoothStatus::Active || frames == 0 {
            return;
        }

        let frames = frames.min(self.output.len());
        let input = self.input * self.a;

        self.output[0] = input + (self.last_output * self.b);

        for i in 1..frames {
            self.output[i] = input + (self.output[i - 1] * self.b);
        }

        self.last_output = self.output[frames - 1];
    }

    pub fn is_active(&self) -> bool {
        self.status.is_active()
    }

    pub fn set_speed(&mut self, sample_rate: u32, seconds: f64) {
        self.b = (-1.0f32 / (seconds as f32 * sample_rate as f32)).exp();
        self.a = 1.0f32 - self.b;
    }

    pub fn update_status(&mut self) -> SmoothStatus {
        self.update_status_with_epsilon(SETTLE)
    }

    pub fn max_blocksize(&self) -> usize {
        self.output.len()
    }
}

impl fmt::Debug for SmoothF32 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct(concat!("SmoothF32"))
            .field("output[0]", &self.output[0])
            .field("max_blocksize", &self.output.len())
            .field("input", &self.input)
            .field("status", &self.status)
            .field("last_output", &self.last_output)
            .finish()
    }
}

// ------  F64  -------------------------------------------------------------------------

pub struct SmoothOutputF64<'a> {
    pub values: &'a [f64],
    pub status: SmoothStatus,
}

impl<'a> SmoothOutputF64<'a> {
    pub fn is_smoothing(&self) -> bool {
        self.status.is_active()
    }
}

impl<'a, I> ops::Index<I> for SmoothOutputF64<'a>
where
    I: slice::SliceIndex<[f64]>,
{
    type Output = I::Output;

    #[inline]
    fn index(&self, idx: I) -> &I::Output {
        &self.values[idx]
    }
}

pub struct SmoothF64 {
    output: Vec<f64>,
    input: f64,

    status: SmoothStatus,

    a: f64,
    b: f64,
    last_output: f64,
}

impl SmoothF64 {
    pub fn new(input: f64, max_blocksize: usize) -> Self {
        Self {
            status: SmoothStatus::Inactive,
            input,
            output: vec![input; max_blocksize],

            a: 1.0,
            b: 0.0,
            last_output: input,
        }
    }

    pub fn reset(&mut self, val: f64) {
        self.status = SmoothStatus::Inactive;
        self.input = val;
        self.last_output = val;

        let max_blocksize = self.output.len();

        self.output.clear();
        self.output.resize(max_blocksize, val);
    }

    pub fn set(&mut self, val: f64) {
        self.input = val;
        self.status = SmoothStatus::Active;
    }

    pub fn dest(&self) -> f64 {
        self.input
    }

    pub fn output(&self) -> SmoothOutputF64 {
        SmoothOutputF64 { values: &self.output, status: self.status }
    }

    pub fn current_value(&self) -> (f64, SmoothStatus) {
        (self.last_output, self.status)
    }

    pub fn update_status_with_epsilon(&mut self, epsilon: f64) -> SmoothStatus {
        let status = self.status;

        match status {
            SmoothStatus::Active => {
                if (self.input - self.output[0]).abs() < epsilon {
                    self.reset(self.input);
                    self.status = SmoothStatus::Deactivating;
                }
            }

            SmoothStatus::Deactivating => self.status = SmoothStatus::Inactive,

            _ => (),
        };

        self.status
    }

    pub fn process(&mut self, frames: usize) {
        if self.status != SmoothStatus::Active || frames == 0 {
            return;
        }

        let frames = frames.min(self.output.len());
        let input = self.input * self.a;

        self.output[0] = input + (self.last_output * self.b);

        for i in 1..frames {
            self.output[i] = input + (self.output[i - 1] * self.b);
        }

        self.last_output = self.output[frames - 1];
    }

    pub fn is_active(&self) -> bool {
        self.status.is_active()
    }

    pub fn set_speed(&mut self, sample_rate: u32, seconds: f64) {
        self.b = (-1.0f64 / (seconds * f64::from(sample_rate))).exp();
        self.a = 1.0f64 - self.b;
    }

    pub fn update_status(&mut self) -> SmoothStatus {
        self.update_status_with_epsilon(SETTLE as f64)
    }

    pub fn max_blocksize(&self) -> usize {
        self.output.len()
    }
}

impl fmt::Debug for SmoothF64 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct(concat!("SmoothF64"))
            .field("output[0]", &self.output[0])
            .field("max_blocksize", &self.output.len())
            .field("input", &self.input)
            .field("status", &self.status)
            .field("last_output", &self.last_output)
            .finish()
    }
}
