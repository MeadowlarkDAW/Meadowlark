use std::ops::{Add, AddAssign, Mul, MulAssign};

use super::{FrameTime, MusicalTime, SecondsF64};

/// (`282,240,000`) This number was chosen because it is nicely divisible by all the common sample
/// rates: `22,050, 24,000, 44,100, 48,000, 88,200, 96,000, 176,400, 192,000, 352,800, and
/// 384,000`. This ensures that no information is lost when switching between sample rates.
pub static SUPER_SAMPLE_TICKS_PER_SECOND: u32 = 282_240_000;

/// Unit of time length in seconds + ticks.
///
/// A "tick" is a unit of time that is exactly 1 / 282,240,000 of a second. This number
/// happens to be nicely divisible by all common sampling rates: `22,050, 24,000, 44,100, 48,000,
/// 88,200, 96,000, 176,400, 192,000, 352,800, and 384,000`. This ensures that no information is
/// lost when switching between sample rates.
#[derive(Default, Debug, Clone, Copy, Hash)]
pub struct SuperclockTime {
    seconds: u32,
    ticks: u32,
}

impl SuperclockTime {
    /// * `seconds` - The time in seconds.
    /// * `ticks` - The number of ticks (after the time in `seconds`) (Note this value
    /// will be constrained to the range `[0, 282,240,000)`).
    ///
    /// A "tick" is a unit of time that is exactly 1 / 282,240,000 of a second. This number
    /// happens to be nicely divisible by all common sampling rates: `22,050, 24,000, 44,100,
    /// 48,000, 88,200, 96,000, 176,400, 192,000, 352,800, and 384,000`. This ensures that no
    /// information is lost when switching between sample rates.
    pub fn new(seconds: u32, ticks: u32) -> Self {
        Self { seconds, ticks: ticks.min(SUPER_SAMPLE_TICKS_PER_SECOND - 1) }
    }

    /// The time in seconds (floored to the nearest second).
    pub fn seconds(&self) -> u32 {
        self.seconds
    }

    /// The fractional number of ticks (after the time in `self.seconds()`).
    ///
    /// A "tick" is a unit of time that is exactly 1 / 282,240,000 of a second. This number
    /// happens to be nicely divisible by all common sampling rates: `22,050, 24,000, 44,100,
    /// 48,000, 88,200, 96,000, 176,400, 192,000, 352,800, and 384,000`. This ensures that no
    /// information is lost when switching between sample rates.
    ///
    /// This value will always be in the range `[0, 282,240,000)`.
    pub fn ticks(&self) -> u32 {
        self.ticks
    }

    /// The total number of ticks.
    ///
    /// A "tick" is a unit of time that is exactly 1 / 282,240,000 of a second. This number
    /// happens to be nicely divisible by all common sampling rates: `22,050, 24,000, 44,100,
    /// 48,000, 88,200, 96,000, 176,400, 192,000, 352,800, and 384,000`. This ensures that no
    /// information is lost when switching between sample rates.
    pub fn total_ticks(&self) -> u64 {
        (u64::from(self.seconds) * u64::from(SUPER_SAMPLE_TICKS_PER_SECOND)) + u64::from(self.ticks)
    }

    /// * `seconds` - The time in seconds.
    pub fn from_seconds(seconds: u32) -> Self {
        Self { seconds, ticks: 0 }
    }

    /// Get the time in [`SuperclockTime`] from the time in [`SecondsF64`]
    ///
    /// Note that this conversion is *NOT* lossless.
    ///
    /// If the seconds value is negative, then the `SuperclockTime`'s values will be 0.
    ///
    /// [`SuperclockTime`]: struct.SuperclockTime.html
    /// [`SecondsF64`]: struct.SecondsF64.html
    pub fn from_seconds_f64(seconds: SecondsF64) -> Self {
        if seconds.0 > 0.0 {
            let mut secs = seconds.0.trunc() as u32;
            let mut ticks =
                (seconds.0.fract() * f64::from(SUPER_SAMPLE_TICKS_PER_SECOND)).round() as u32;

            if ticks >= SUPER_SAMPLE_TICKS_PER_SECOND {
                ticks = 0;
                secs += 1;
            }

            Self { seconds: secs, ticks }
        } else {
            Self { seconds: 0, ticks: 0 }
        }
    }

    /// Get the time in [`SuperclockTime`] from the time in [`SecondsF64`], floored to the
    /// nearest tick.
    ///
    /// Note that this conversion is *NOT* lossless.
    ///
    /// If the seconds value is negative, then the `SuperclockTime`'s values will be 0.
    ///
    /// [`SuperclockTime`]: struct.SuperclockTime.html
    /// [`SecondsF64`]: struct.SecondsF64.html
    pub fn from_seconds_f64_floor(seconds: SecondsF64) -> Self {
        if seconds.0 > 0.0 {
            let mut secs = seconds.0.trunc() as u32;
            let mut ticks =
                (seconds.0.fract() * f64::from(SUPER_SAMPLE_TICKS_PER_SECOND)).floor() as u32;

            if ticks >= SUPER_SAMPLE_TICKS_PER_SECOND {
                ticks = 0;
                secs += 1;
            }

            Self { seconds: secs, ticks }
        } else {
            Self { seconds: 0, ticks: 0 }
        }
    }

    /// Get the time in [`SuperclockTime`] from the time in [`SecondsF64`], ceiled to the
    /// nearest tick.
    ///
    /// Note that this conversion is *NOT* lossless.
    ///
    /// If the seconds value is negative, then the `SuperclockTime`'s values will be 0.
    ///
    /// [`SuperclockTime`]: struct.SuperclockTime.html
    /// [`SecondsF64`]: struct.SecondsF64.html
    pub fn from_seconds_f64_ceil(seconds: SecondsF64) -> Self {
        if seconds.0 > 0.0 {
            let mut secs = seconds.0.trunc() as u32;
            let mut ticks =
                (seconds.0.fract() * f64::from(SUPER_SAMPLE_TICKS_PER_SECOND)).ceil() as u32;

            if ticks >= SUPER_SAMPLE_TICKS_PER_SECOND {
                ticks = 0;
                secs += 1;
            }

            Self { seconds: secs, ticks }
        } else {
            Self { seconds: 0, ticks: 0 }
        }
    }

    /// Get the time in [`SuperclockTime`] from the time in [`SecondsF64`], floored to the
    /// nearest tick, while also return the fractional sub-tick part.
    ///
    /// Note that this conversion is *NOT* lossless.
    ///
    /// If the seconds value is negative, then the `SuperclockTime`'s values and the
    /// fractional value will both be 0.
    ///
    /// [`SuperclockTime`]: struct.FrameTime.html
    pub fn from_seconds_f64_with_sub_tick(seconds: SecondsF64) -> (Self, f64) {
        if seconds.0 > 0.0 {
            let mut secs = seconds.0.trunc() as u32;
            let ticks_f64 = seconds.0.fract() * f64::from(SUPER_SAMPLE_TICKS_PER_SECOND);

            let mut ticks = ticks_f64.trunc() as u32;
            let sub_ticks = ticks_f64.fract();

            if ticks >= SUPER_SAMPLE_TICKS_PER_SECOND {
                ticks = 0;
                secs += 1;
            }

            (Self { seconds: secs, ticks }, sub_ticks)
        } else {
            (Self { seconds: 0, ticks: 0 }, 0.0)
        }
    }

    /// Get the time in [`SuperclockTime`] from the time in [`FrameTime`] when the samplerate
    /// is one of the common sample rates.
    ///
    /// This conversion is *ONLY* correct if the `SAMPLE_RATE` constant is one of the following
    /// common sample rates: `22,050, 24,000, 44,100, 48,000, 88,200, 96,000, 176,400, 192,000,
    /// 352,800, or 384,000`. Otherwise, please use `Self::from_frame()`.
    ///
    /// [`SuperclockTime`]: struct.SuperclockTime.html
    /// [`FrameTime`]: struct.FrameTime.html
    pub fn from_frame_with_common_samplerate<const SAMPLE_RATE: u32>(sample: FrameTime) -> Self {
        // Make sure that the compiler optimizes these two operations into a single operation.
        let seconds = sample.0 / u64::from(SAMPLE_RATE);
        let samples_after = sample.0 % u64::from(SAMPLE_RATE);

        Self {
            seconds: seconds as u32,
            ticks: (samples_after as u32) * (SUPER_SAMPLE_TICKS_PER_SECOND / SAMPLE_RATE),
        }
    }

    /// Get the time in [`SuperclockTime`] from the time in [`FrameTime`].
    ///
    /// This conversion **IS** lossless if the sample rate happens to be equal to one of the
    /// common sample rates: `22,050, 24,000, 44,100, 48,000, 88,200, 96,000, 176,400, 192,000,
    /// 352,800, or 384,000`. This conversion is *NOT* lossless otherwise (especially if the
    /// given `sample` value is very large).
    ///
    /// [`SuperclockTime`]: struct.SuperclockTime.html
    /// [`FrameTime`]: struct.FrameTime.html
    pub fn from_frame(sample: FrameTime, sample_rate: u32) -> Self {
        match sample_rate {
            44_100 => Self::from_frame_with_common_samplerate::<44_100>(sample),
            48_000 => Self::from_frame_with_common_samplerate::<48_000>(sample),
            88_200 => Self::from_frame_with_common_samplerate::<88_200>(sample),
            96_000 => Self::from_frame_with_common_samplerate::<96_000>(sample),
            176_400 => Self::from_frame_with_common_samplerate::<176_400>(sample),
            192_000 => Self::from_frame_with_common_samplerate::<192_000>(sample),
            352_800 => Self::from_frame_with_common_samplerate::<352_800>(sample),
            384_000 => Self::from_frame_with_common_samplerate::<384_000>(sample),
            22_050 => Self::from_frame_with_common_samplerate::<22_050>(sample),
            24_000 => Self::from_frame_with_common_samplerate::<24_000>(sample),
            _ => {
                let seconds = sample.0 / u64::from(sample_rate);
                let samples_after = sample.0 % u64::from(sample_rate);

                Self {
                    seconds: seconds as u32,
                    ticks: ((samples_after as f64)
                        * (f64::from(SUPER_SAMPLE_TICKS_PER_SECOND) / f64::from(sample_rate)))
                    .round() as u32,
                }
            }
        }
    }

    /// Convert to the corresponding time in [`SecondsF64`].
    ///
    /// Note that this conversion is *NOT* lossless.
    ///
    /// [`SecondsF64`]: struct.SecondsF64.html
    pub fn to_seconds_f64(&self) -> SecondsF64 {
        SecondsF64(
            f64::from(self.seconds)
                + (f64::from(self.ticks) / f64::from(SUPER_SAMPLE_TICKS_PER_SECOND)),
        )
    }

    /// Convert to the corresponding [`MusicalTime`].
    ///
    /// Note that this conversion is *NOT* lossless.
    ///
    /// [`MusicalTime`]: struct.MusicalTime.html
    pub fn to_musical(&self, bpm: f64) -> MusicalTime {
        self.to_seconds_f64().to_musical(bpm)
    }

    /// Convert to the corresponding time length in [`FrameTime`] from the given [`u32`],
    /// rounded to the nearest frame.
    ///
    /// Note that this conversion is *NOT* lossless.
    ///
    /// [`FrameTime`]: struct.FrameTime.html
    /// [`u32`]: struct.u32.html
    pub fn to_nearest_frame_round(&self, sample_rate: u32) -> FrameTime {
        let whole_second_frames = u64::from(self.seconds) * u64::from(sample_rate);
        let fract_second_frames = (f64::from(self.ticks) / f64::from(SUPER_SAMPLE_TICKS_PER_SECOND)
            * f64::from(sample_rate))
        .round() as u64;

        FrameTime(whole_second_frames + fract_second_frames)
    }

    /// Convert to the corresponding time length in [`FrameTime`] from the given [`u32`],
    /// floored to the nearest frame.
    ///
    /// Note that this conversion is *NOT* lossless.
    ///
    /// [`FrameTime`]: struct.FrameTime.html
    /// [`u32`]: struct.u32.html
    pub fn to_nearest_frame_floor(&self, sample_rate: u32) -> FrameTime {
        let whole_second_frames = u64::from(self.seconds) * u64::from(sample_rate);
        let fract_second_frames = (f64::from(self.ticks) / f64::from(SUPER_SAMPLE_TICKS_PER_SECOND)
            * f64::from(sample_rate))
        .floor() as u64;

        FrameTime(whole_second_frames + fract_second_frames)
    }

    /// Convert to the corresponding time length in [`FrameTime`] from the given [`u32`],
    /// ceil-ed to the nearest frame.
    ///
    /// Note that this conversion is *NOT* lossless.
    ///
    /// [`FrameTime`]: struct.FrameTime.html
    /// [`u32`]: struct.u32.html
    pub fn to_nearest_frame_ceil(&self, sample_rate: u32) -> FrameTime {
        let whole_second_frames = u64::from(self.seconds) * u64::from(sample_rate);
        let fract_second_frames = (f64::from(self.ticks) / f64::from(SUPER_SAMPLE_TICKS_PER_SECOND)
            * f64::from(sample_rate))
        .ceil() as u64;

        FrameTime(whole_second_frames + fract_second_frames)
    }

    /// Try subtracting `rhs` from self. This will return `None` if the resulting value
    /// is negative due to `rhs` being larger than self (overflow).
    pub fn checked_sub(self, rhs: SuperclockTime) -> Option<SuperclockTime> {
        if rhs.seconds > self.seconds {
            None
        } else if rhs.seconds == self.seconds {
            if rhs.ticks > self.ticks {
                None
            } else {
                Some(SuperclockTime { seconds: 0, ticks: self.ticks - rhs.ticks })
            }
        } else {
            if rhs.ticks > self.ticks {
                Some(SuperclockTime {
                    seconds: self.seconds - rhs.seconds - 1,
                    ticks: SUPER_SAMPLE_TICKS_PER_SECOND - (rhs.ticks - self.ticks),
                })
            } else {
                Some(SuperclockTime {
                    seconds: self.seconds - rhs.seconds,
                    ticks: self.ticks - rhs.ticks,
                })
            }
        }
    }
}

impl PartialEq for SuperclockTime {
    fn eq(&self, other: &Self) -> bool {
        self.seconds == other.seconds && self.ticks == other.ticks
    }
}

impl Eq for SuperclockTime {}

impl PartialOrd for SuperclockTime {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if self.seconds != other.seconds {
            self.seconds.partial_cmp(&other.seconds)
        } else {
            self.ticks.partial_cmp(&other.ticks)
        }
    }
}

impl Ord for SuperclockTime {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if self.seconds != other.seconds {
            self.seconds.cmp(&other.seconds)
        } else {
            self.ticks.cmp(&other.ticks)
        }
    }
}

impl Add<SuperclockTime> for SuperclockTime {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        let mut seconds = self.seconds + rhs.seconds;
        let mut ticks = self.ticks + rhs.ticks;

        if ticks >= SUPER_SAMPLE_TICKS_PER_SECOND {
            ticks -= SUPER_SAMPLE_TICKS_PER_SECOND;
            seconds += 1;
        }

        Self { seconds, ticks }
    }
}
impl Mul<u32> for SuperclockTime {
    type Output = Self;
    fn mul(self, rhs: u32) -> Self::Output {
        let mut seconds = self.seconds * rhs;
        let mut ticks = u64::from(self.ticks) * u64::from(rhs);

        if ticks >= u64::from(SUPER_SAMPLE_TICKS_PER_SECOND) {
            seconds += (ticks / u64::from(SUPER_SAMPLE_TICKS_PER_SECOND)) as u32;
            ticks = ticks % u64::from(SUPER_SAMPLE_TICKS_PER_SECOND);
        }

        Self { seconds, ticks: ticks as u32 }
    }
}

impl AddAssign<SuperclockTime> for SuperclockTime {
    fn add_assign(&mut self, other: Self) {
        *self = *self + other
    }
}
impl MulAssign<u32> for SuperclockTime {
    fn mul_assign(&mut self, other: u32) {
        *self = *self * other
    }
}
