use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};

use super::{FrameTime, MusicalTime, SuperclockTime};

/// Unit of time in "Seconds"
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct SecondsF64(pub f64);

impl SecondsF64 {
    pub fn new(seconds: f64) -> Self {
        SecondsF64(seconds)
    }

    pub fn as_f32(&self) -> f32 {
        self.0 as f32
    }

    /// Creates a new time in `Seconds` from [`FrameTime`] and a sample rate.
    ///
    /// Note that this conversion is *NOT* lossless.
    ///
    /// [`FrameTime`]: struct.FrameTime.html
    pub fn from_frame(frame: FrameTime, sample_rate: u32, sample_rate_recip: f64) -> Self {
        frame.to_seconds_f64(sample_rate, sample_rate_recip)
    }

    /// Creates a new time in `Seconds` from [`SuperclockTime`].
    ///
    /// Note that this conversion is *NOT* lossless.
    ///
    /// [`SuperclockTime`]: struct.SuperclockTime.html
    pub fn from_superclock_time(superclock_time: SuperclockTime) -> Self {
        superclock_time.to_seconds_f64()
    }

    /// Convert to discrete [`FrameTime`] with the given sample rate. This will
    /// be rounded to the nearest frame.
    ///
    /// Note that this conversion is *NOT* lossless.
    ///
    /// If the seconds value is negative, then `FrameTime(0)` will be returned instead.
    ///
    /// [`FrameTime`]: struct.FrameTime.html
    pub fn to_nearest_frame_round(&self, sample_rate: u32) -> FrameTime {
        if self.0 > 0.0 {
            let whole_second_frames = self.0.floor() as u64 * u64::from(sample_rate);
            let fract_second_frames = (self.0.fract() * f64::from(sample_rate)).round() as u64;

            FrameTime(whole_second_frames + fract_second_frames)
        } else {
            FrameTime(0)
        }
    }

    /// Convert to discrete [`FrameTime`] with the given sample rate. This will
    /// be floored to the nearest frame.
    ///
    /// Note that this conversion is *NOT* lossless.
    ///
    /// If the seconds value is negative, then `FrameTime(0)` will be returned instead.
    ///
    /// [`FrameTime`]: struct.FrameTime.html
    pub fn to_nearest_frame_floor(&self, sample_rate: u32) -> FrameTime {
        if self.0 > 0.0 {
            let whole_second_frames = self.0.floor() as u64 * u64::from(sample_rate);
            let fract_second_frames = (self.0.fract() * f64::from(sample_rate)).floor() as u64;

            FrameTime(whole_second_frames + fract_second_frames)
        } else {
            FrameTime(0)
        }
    }

    /// Convert to discrete [`FrameTime`] with the given sample rate. This will
    /// be ceil-ed to the nearest frame.
    ///
    /// Note that this conversion is *NOT* lossless.
    ///
    /// If the seconds value is negative, then `FrameTime(0)` will be returned instead.
    ///
    /// [`FrameTime`]: struct.FrameTime.html
    pub fn to_nearest_frame_ceil(&self, sample_rate: u32) -> FrameTime {
        if self.0 > 0.0 {
            let whole_second_frames = self.0.floor() as u64 * u64::from(sample_rate);
            let fract_second_frames = (self.0.fract() * f64::from(sample_rate)).ceil() as u64;

            FrameTime(whole_second_frames + fract_second_frames)
        } else {
            FrameTime(0)
        }
    }

    /// Convert to discrete [`FrameTime`] given the sample rate floored to the nearest
    /// sample, while also return the fractional sub-sample part.
    ///
    /// Note that this conversion is *NOT* lossless.
    ///
    /// If the seconds value is negative, then `(FrameTime(0), 0.0)` will be returned instead.
    ///
    /// [`FrameTime`]: struct.FrameTime.html
    pub fn to_sub_frame(&self, sample_rate: u32) -> (FrameTime, f64) {
        if self.0 > 0.0 {
            let whole_second_frames = self.0.floor() as u64 * u64::from(sample_rate);
            let fract_second_frames = self.0.fract() * f64::from(sample_rate);

            let fract_second_frames_floor = fract_second_frames.floor() as u64;
            let fract_frames = fract_second_frames.fract();

            (FrameTime(whole_second_frames + fract_second_frames_floor), fract_frames)
        } else {
            (FrameTime(0), 0.0)
        }
    }

    /// Convert to discrete [`SuperclockTime`]. This will
    /// be rounded to the nearest super-frame.
    ///
    /// Note that this conversion is *NOT* lossless.
    ///
    /// If the seconds value is negative, then the `SuperclockTime`'s value will be 0.
    ///
    /// [`SuperclockTime`]: struct.FrameTime.html
    pub fn to_nearest_super_frame_round(&self) -> SuperclockTime {
        SuperclockTime::from_seconds_f64(*self)
    }

    /// Convert to discrete [`SuperclockTime`]. This will
    /// be floored to the nearest super-frame.
    ///
    /// Note that this conversion is *NOT* lossless.
    ///
    /// If the seconds value is negative, then the `SuperclockTime`'s values will be 0.
    ///
    /// [`SuperclockTime`]: struct.FrameTime.html
    pub fn to_nearest_super_frame_floor(&self) -> SuperclockTime {
        SuperclockTime::from_seconds_f64_floor(*self)
    }

    /// Convert to discrete [`SuperclockTime`]. This will
    /// be ceil-ed to the nearest super-frame.
    ///
    /// Note that this conversion is *NOT* lossless.
    ///
    /// If the seconds value is negative, then the `SuperclockTime`'s values will be 0.
    ///
    /// [`SuperclockTime`]: struct.FrameTime.html
    pub fn to_nearest_super_frame_ceil(&self) -> SuperclockTime {
        SuperclockTime::from_seconds_f64_ceil(*self)
    }

    /// Convert to discrete [`FrameTime`] floored to the nearest
    /// super-frame, while also return the fractional sub-super-frame part.
    ///
    /// Note that this conversion is *NOT* lossless.
    ///
    /// If the seconds value is negative, then the `SuperclockTime`'s values and the
    /// fractional value will both be 0.
    ///
    /// [`SuperclockTime`]: struct.FrameTime.html
    pub fn to_sub_super_frame(&self) -> (SuperclockTime, f64) {
        SuperclockTime::from_seconds_f64_with_sub_tick(*self)
    }

    /// Convert to the corresponding [`MusicalTime`].
    ///
    /// Note that this conversion is *NOT* lossless.
    ///
    /// [`MusicalTime`]: ../time/struct.MusicalTime.html
    pub fn to_musical(&self, bpm: f64) -> MusicalTime {
        MusicalTime::from_beats_f64(self.0 * (bpm / 60.0))
    }
}

impl Default for SecondsF64 {
    fn default() -> Self {
        SecondsF64(0.0)
    }
}

impl From<i8> for SecondsF64 {
    fn from(s: i8) -> Self {
        SecondsF64(f64::from(s))
    }
}
impl From<u8> for SecondsF64 {
    fn from(s: u8) -> Self {
        SecondsF64(f64::from(s))
    }
}
impl From<i16> for SecondsF64 {
    fn from(s: i16) -> Self {
        SecondsF64(f64::from(s))
    }
}
impl From<u16> for SecondsF64 {
    fn from(s: u16) -> Self {
        SecondsF64(f64::from(s))
    }
}
impl From<i32> for SecondsF64 {
    fn from(s: i32) -> Self {
        SecondsF64(f64::from(s))
    }
}
impl From<u32> for SecondsF64 {
    fn from(s: u32) -> Self {
        SecondsF64(f64::from(s))
    }
}
impl From<f32> for SecondsF64 {
    fn from(s: f32) -> Self {
        SecondsF64(f64::from(s))
    }
}

impl Add<SecondsF64> for SecondsF64 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}
impl Sub<SecondsF64> for SecondsF64 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}
impl Mul<SecondsF64> for SecondsF64 {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        Self(self.0 * rhs.0)
    }
}
impl Div<SecondsF64> for SecondsF64 {
    type Output = Self;
    fn div(self, rhs: Self) -> Self::Output {
        Self(self.0 / rhs.0)
    }
}

impl AddAssign<SecondsF64> for SecondsF64 {
    fn add_assign(&mut self, other: Self) {
        self.0 += other.0;
    }
}
impl SubAssign<SecondsF64> for SecondsF64 {
    fn sub_assign(&mut self, other: Self) {
        self.0 -= other.0;
    }
}
impl MulAssign<SecondsF64> for SecondsF64 {
    fn mul_assign(&mut self, other: Self) {
        self.0 *= other.0;
    }
}
impl DivAssign<SecondsF64> for SecondsF64 {
    fn div_assign(&mut self, other: Self) {
        self.0 /= other.0;
    }
}
