use std::ops::{Add, AddAssign, Mul, MulAssign, Sub, SubAssign};

use super::{SecondsF64, SuperclockTime};

/// Unit of time length in frames (samples in a single audio channel).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Hash)]
pub struct FrameTime(pub u64);

impl FrameTime {
    pub fn new(frame: u64) -> Self {
        Self(frame)
    }

    /// Convert to the corresponding time in [`SecondsF64`] with the given sample rate.
    ///
    /// Note that this conversion is *NOT* lossless.
    ///
    /// [`SecondsF64`]: struct.SecondsF64.html
    pub fn to_seconds_f64(&self, sample_rate: u32, sample_rate_recip: f64) -> SecondsF64 {
        let whole_seconds = self.0 / u64::from(sample_rate);
        let fract_frames = self.0 % u64::from(sample_rate);

        let fract_seconds = fract_frames as f64 * sample_rate_recip;

        SecondsF64(whole_seconds as f64 + fract_seconds)
    }

    /// Convert to the corresponding time length in [`SuperclockTime`] from the given sample rate.
    ///
    /// This conversion **IS** lossless if the sample rate happens to be equal to one of the common
    /// sample rates: `22050, 24000, 44100, 48000, 88200, 96000, 176400, or 192000`. This
    /// conversion is *NOT* lossless otherwise.
    ///
    /// [`SuperclockTime`]: struct.SuperclockTime.html
    pub fn to_super_frame(&self, sample_rate: u32) -> SuperclockTime {
        SuperclockTime::from_frame(*self, sample_rate)
    }
}

impl Default for FrameTime {
    fn default() -> Self {
        FrameTime(0)
    }
}

impl From<u8> for FrameTime {
    fn from(s: u8) -> Self {
        FrameTime(u64::from(s))
    }
}
impl From<u16> for FrameTime {
    fn from(s: u16) -> Self {
        FrameTime(u64::from(s))
    }
}
impl From<u32> for FrameTime {
    fn from(s: u32) -> Self {
        FrameTime(u64::from(s))
    }
}
impl From<u64> for FrameTime {
    fn from(s: u64) -> Self {
        FrameTime(s)
    }
}
impl From<usize> for FrameTime {
    fn from(s: usize) -> Self {
        FrameTime(s as u64)
    }
}

impl Add<FrameTime> for FrameTime {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}
impl Sub<FrameTime> for FrameTime {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}
impl Mul<u64> for FrameTime {
    type Output = Self;
    fn mul(self, rhs: u64) -> Self::Output {
        Self(self.0 * rhs)
    }
}

impl AddAssign<FrameTime> for FrameTime {
    fn add_assign(&mut self, other: Self) {
        self.0 += other.0;
    }
}
impl SubAssign<FrameTime> for FrameTime {
    fn sub_assign(&mut self, other: Self) {
        self.0 -= other.0;
    }
}
impl MulAssign<u64> for FrameTime {
    fn mul_assign(&mut self, other: u64) {
        *self = *self * other
    }
}
