use std::ops::{Add, AddAssign, Mul, MulAssign};

use super::{FrameTime, SecondsF64, SuperclockTime};

/// (`1,241,856,000`) This number was chosen because it is nicely divisible by a whole slew of factors
/// including `2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 18, 20, 24, 32, 64, 128, 256, 512,
/// 1,024, 1,920, 2,048, 4,096, 8,192, 16,384, and 32,768`. This ensures that all these subdivisions of
/// musical beats can be stored and operated on with *exact* precision. This number is also much larger
/// than all of the common sampling rates, allowing for sample-accurate precision even at very high
/// sampling rates and very low BPMs.
pub static SUPER_BEAT_TICKS_PER_BEAT: u32 = 1_241_856_000;

/// Musical time in units of beats + ticks.
///
/// A "tick" is a unit of time equal to `1 / 1,241,856,000` of a beat. This number was chosen because
/// it is nicely divisible by a whole slew of factors including `2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12,
/// 13, 14, 15, 16, 18, 20, 24, 32, 64, 128, 256, 512, 1,024, 1,920, 2,048, 4,096, 8,192, 16,384, and
/// 32,768`. This ensures that all these subdivisions of musical beats can be stored and operated on
/// with *exact* precision. This number is also much larger than all of the common sampling rates,
/// allowing for sample-accurate precision even at very high sampling rates and very low BPMs.
#[derive(Default, Debug, Clone, Copy, Hash)]
pub struct MusicalTime {
    beats: u32,
    ticks: u32,
}

impl MusicalTime {
    /// * `beats` - The time in musical beats.
    /// * `ticks` - The number of ticks (after the time in `beats`) (Note this value
    /// will be constrained to the range `[0, 1,241,856,000)`).
    ///
    /// A "tick" is a unit of time equal to `1 / 1,241,856,000` of a beat. This number was chosen
    /// because it is nicely divisible by a whole slew of factors including `2, 3, 4, 5, 6, 7, 8, 9,
    /// 10, 11, 12, 13, 14, 15, 16, 18, 20, 24, 32, 64, 128, 256, 512, 1,024, 1,920, 2,048, 4,096,
    /// 8,192, 16,384, and 32,768`. This ensures that all these subdivisions of musical beats can be
    /// stored and operated on with *exact* precision. This number is also much larger than all of
    /// the common sampling rates, allowing for sample-accurate precision even at very high sampling
    /// rates and very low BPMs.
    pub fn new(beats: u32, ticks: u32) -> Self {
        Self { beats, ticks: ticks.min(SUPER_BEAT_TICKS_PER_BEAT - 1) }
    }

    /// The time in musical beats (floored to the nearest beat).
    pub fn beats(&self) -> u32 {
        self.beats
    }

    /// The fractional number of ticks (after the time in `self.beats()`).
    ///
    /// A "tick" is a unit of time equal to `1 / 1,241,856,000` of a beat. This number was chosen
    /// because it is nicely divisible by a whole slew of factors including `2, 3, 4, 5, 6, 7, 8, 9,
    /// 10, 11, 12, 13, 14, 15, 16, 18, 20, 24, 32, 64, 128, 256, 512, 1,024, 1,920, 2,048, 4,096,
    /// 8,192, 16,384, and 32,768`. This ensures that all these subdivisions of musical beats can be
    /// stored and operated on with *exact* precision. This number is also much larger than all of
    /// the common sampling rates, allowing for sample-accurate precision even at very high sampling
    /// rates and very low BPMs.
    ///
    /// This value will always be in the range `[0, 1,241,856,000)`.
    pub fn ticks(&self) -> u32 {
        self.ticks
    }

    /// The total number of ticks.
    ///
    /// A "tick" is a unit of time equal to `1 / 1,241,856,000` of a beat. This number was chosen
    /// because it is nicely divisible by a whole slew of factors including `2, 3, 4, 5, 6, 7, 8, 9,
    /// 10, 11, 12, 13, 14, 15, 16, 18, 20, 24, 32, 64, 128, 256, 512, 1,024, 1,920, 2,048, 4,096,
    /// 8,192, 16,384, and 32,768`. This ensures that all these subdivisions of musical beats can be
    /// stored and operated on with *exact* precision. This number is also much larger than all of
    /// the common sampling rates, allowing for sample-accurate precision even at very high sampling
    /// rates and very low BPMs.
    pub fn total_ticks(&self) -> u64 {
        (u64::from(self.beats) * u64::from(SUPER_BEAT_TICKS_PER_BEAT)) + u64::from(self.ticks)
    }

    /// * `beats` - The time in musical beats.
    pub fn from_beats(beats: u32) -> Self {
        Self { beats, ticks: 0 }
    }

    pub fn from_fractional_beats<const DIVISOR: u32>(beats: u32, fract_beats: u32) -> Self {
        Self { beats, ticks: fract_beats.min(DIVISOR - 1) * (SUPER_BEAT_TICKS_PER_BEAT / DIVISOR) }
    }

    /// * `beats` - The time in musical beats.
    /// * `half_beats` - The number of half-beats (after the time `beats`). This will be
    /// constrained to the range `[0, 1]`.
    pub fn from_half_beats(beats: u32, half_beats: u32) -> Self {
        Self::from_fractional_beats::<2>(beats, half_beats)
    }

    /// * `beats` - The time in musical beats.
    /// * `quarter_beats` - The number of quarter-beats (after the time `beats`). This will be
    /// constrained to the range `[0, 3]`.
    pub fn from_quarter_beats(beats: u32, quarter_beats: u32) -> Self {
        Self::from_fractional_beats::<4>(beats, quarter_beats)
    }

    /// * `beats` - The time in musical beats.
    /// * `eigth_beats` - The number of eigth-beats (after the time `beats`). This will be
    /// constrained to the range `[0, 7]`.
    pub fn from_eighth_beats(beats: u32, eigth_beats: u32) -> Self {
        Self::from_fractional_beats::<8>(beats, eigth_beats)
    }

    /// * `beats` - The time in musical beats.
    /// * `sixteenth_beats` - The number of sixteenth-beats (after the time `beats`). This will be
    /// constrained to the range `[0, 15]`.
    pub fn from_sixteenth_beats(beats: u32, sixteenth_beats: u32) -> Self {
        Self::from_fractional_beats::<16>(beats, sixteenth_beats)
    }

    /// * `beats` - The time in musical beats.
    /// * `_32nd_beats` - The number of 32nd-beats (after the time `beats`). This will be
    /// constrained to the range `[0, 31]`.
    pub fn from_32nd_beats(beats: u32, _32nd_beats: u32) -> Self {
        Self::from_fractional_beats::<32>(beats, _32nd_beats)
    }

    /// * `beats` - The time in musical beats.
    /// * `_64th_beats` - The number of 64th-beats (after the time `beats`). This will be
    /// constrained to the range `[0, 63]`.
    pub fn from_64th_beats(beats: u32, _64th_beats: u32) -> Self {
        Self::from_fractional_beats::<64>(beats, _64th_beats)
    }

    /// * `beats` - The time in musical beats.
    /// * `_128th_beats` - The number of 128th-beats (after the time `beats`). This will be
    /// constrained to the range `[0, 127]`.
    pub fn from_128th_beats(beats: u32, _128th_beats: u32) -> Self {
        Self::from_fractional_beats::<128>(beats, _128th_beats)
    }

    /// * `beats` - The time in musical beats.
    /// * `_256th_beats` - The number of 256th-beats (after the time `beats`). This will be
    /// constrained to the range `[0, 255]`.
    pub fn from_256th_beats(beats: u32, _256th_beats: u32) -> Self {
        Self::from_fractional_beats::<256>(beats, _256th_beats)
    }

    /// * `beats` - The time in musical beats.
    /// * `_512th_beats` - The number of 512th-beats (after the time `beats`). This will be
    /// constrained to the range `[0, 511]`.
    pub fn from_512th_beats(beats: u32, _512th_beats: u32) -> Self {
        Self::from_fractional_beats::<512>(beats, _512th_beats)
    }

    /// * `beats` - The time in musical beats.
    /// * `_1024th_beats` - The number of 1024th-beats (after the time `beats`). This will be
    /// constrained to the range `[0, 1023]`.
    pub fn from_1024th_beats(beats: u32, _1024th_beats: u32) -> Self {
        Self::from_fractional_beats::<1024>(beats, _1024th_beats)
    }

    /// * `beats` - The time in musical beats.
    /// * `_2048th_beats` - The number of 2048th-beats (after the time `beats`). This will be
    /// constrained to the range `[0, 2047]`.
    pub fn from_2048th_beats(beats: u32, _2048th_beats: u32) -> Self {
        Self::from_fractional_beats::<2048>(beats, _2048th_beats)
    }

    /// * `beats` - The time in musical beats.
    /// * `third_beats` - The number of third-beats (after the time `beats`). This will be
    /// constrained to the range `[0, 2]`.
    pub fn from_third_beats(beats: u32, third_beats: u32) -> Self {
        Self::from_fractional_beats::<3>(beats, third_beats)
    }

    /// * `beats` - The time in musical beats.
    /// * `fifth_beats` - The number of fifth-beats (after the time `beats`). This will be
    /// constrained to the range `[0, 4]`.
    pub fn from_fifth_beats(beats: u32, fifth_beats: u32) -> Self {
        Self::from_fractional_beats::<5>(beats, fifth_beats)
    }

    /// * `beats` - The time in musical beats.
    /// * `sixth_beats` - The number of sixth-beats (after the time `beats`). This will be
    /// constrained to the range `[0, 5]`.
    pub fn from_sixth_beats(beats: u32, sixth_beats: u32) -> Self {
        Self::from_fractional_beats::<6>(beats, sixth_beats)
    }

    /// * `beats` - The time in musical beats.
    /// * `seventh_beats` - The number of seventh-beats (after the time `beats`). This will be
    /// constrained to the range `[0, 6]`.
    pub fn from_seventh_beats(beats: u32, seventh_beats: u32) -> Self {
        Self::from_fractional_beats::<7>(beats, seventh_beats)
    }

    /// * `beats` - The time in musical beats.
    /// * `ninth_beats` - The number of ninth-beats (after the time `beats`). This will be
    /// constrained to the range `[0, 8]`.
    pub fn from_ninth_beats(beats: u32, ninth_beats: u32) -> Self {
        Self::from_fractional_beats::<9>(beats, ninth_beats)
    }

    /// * `beats` - The time in musical beats.
    /// * `tenth_beats` - The number of tenth-beats (after the time `beats`). This will be
    /// constrained to the range `[0, 9]`.
    pub fn from_tenth_beats(beats: u32, tenth_beats: u32) -> Self {
        Self::from_fractional_beats::<10>(beats, tenth_beats)
    }

    /// * `beats` - The time in musical beats.
    /// * `eleventh_beats` - The number of eleventh-beats (after the time `beats`). This will be
    /// constrained to the range `[0, 10]`.
    pub fn from_eleventh_beats(beats: u32, eleventh_beats: u32) -> Self {
        Self::from_fractional_beats::<11>(beats, eleventh_beats)
    }

    /// * `beats` - The time in musical beats.
    /// * `twelfth_beats` - The number of twelfth-beats (after the time `beats`). This will be
    /// constrained to the range `[0, 11]`.
    pub fn from_twelth_beats(beats: u32, twelfth_beats: u32) -> Self {
        Self::from_fractional_beats::<12>(beats, twelfth_beats)
    }

    /// * `beats` - The time in musical beats.
    /// * `_24th_beats` - The number of 24th-beats (after the time `beats`). This will be
    /// constrained to the range `[0, 23]`.
    pub fn from_24th_beats(beats: u32, _24th_beats: u32) -> Self {
        Self::from_fractional_beats::<24>(beats, _24th_beats)
    }

    /// Get the corresponding musical time from the number of beats (as an `f64`).
    ///
    /// Note that this conversion is *NOT* lossless.
    ///
    /// If `beats` is less than 0.0, then a musical time of `0` will be returned instead.
    pub fn from_beats_f64(beats: f64) -> Self {
        if beats >= 0.0 {
            let mut beats_u32 = beats.trunc() as u32;
            let mut ticks = (beats.fract() * f64::from(SUPER_BEAT_TICKS_PER_BEAT)).round() as u32;

            if ticks >= SUPER_BEAT_TICKS_PER_BEAT {
                ticks = 0;
                beats_u32 += 1;
            }

            Self { beats: beats_u32, ticks }
        } else {
            Self { beats: 0, ticks: 0 }
        }
    }

    /// Convert the corresponding musical time in units of beats (as an `f64` value).
    ///
    /// Note that this conversion is *NOT* lossless.
    ///
    /// This is useful for displaying notes in UI.
    pub fn as_beats_f64(&self) -> f64 {
        f64::from(self.beats) + (f64::from(self.ticks) / f64::from(SUPER_BEAT_TICKS_PER_BEAT))
    }

    pub fn snap_to_nearest_beat(&self) -> MusicalTime {
        if self.ticks >= SUPER_BEAT_TICKS_PER_BEAT / 2 {
            Self { beats: self.beats + 1, ticks: 0 }
        } else {
            Self { beats: self.beats, ticks: 0 }
        }
    }

    pub fn snap_to_nearest_fractional_beat<const DIVISOR: u32>(&self) -> MusicalTime {
        let nearest_floored_tick = (self.ticks / (SUPER_BEAT_TICKS_PER_BEAT / DIVISOR))
            * (SUPER_BEAT_TICKS_PER_BEAT / DIVISOR);

        let mut beats = self.beats;
        let mut nearest_tick =
            if self.ticks - nearest_floored_tick >= (SUPER_BEAT_TICKS_PER_BEAT / DIVISOR) / 2 {
                nearest_floored_tick + (SUPER_BEAT_TICKS_PER_BEAT / DIVISOR)
            } else {
                nearest_floored_tick
            };

        if nearest_tick >= SUPER_BEAT_TICKS_PER_BEAT {
            nearest_tick -= SUPER_BEAT_TICKS_PER_BEAT;
            beats += 1;
        }

        Self { beats, ticks: nearest_tick }
    }

    pub fn snap_to_nearest_half_beat(&self) -> MusicalTime {
        self.snap_to_nearest_fractional_beat::<2>()
    }

    pub fn snap_to_nearest_quarter_beat(&self) -> MusicalTime {
        self.snap_to_nearest_fractional_beat::<4>()
    }

    pub fn snap_to_nearest_eigth_beat(&self) -> MusicalTime {
        self.snap_to_nearest_fractional_beat::<8>()
    }

    pub fn snap_to_nearest_sixteenth_beat(&self) -> MusicalTime {
        self.snap_to_nearest_fractional_beat::<16>()
    }

    pub fn snap_to_nearest_32nd_beat(&self) -> MusicalTime {
        self.snap_to_nearest_fractional_beat::<32>()
    }

    pub fn snap_to_nearest_64th_beat(&self) -> MusicalTime {
        self.snap_to_nearest_fractional_beat::<64>()
    }

    pub fn snap_to_nearest_128th_beat(&self) -> MusicalTime {
        self.snap_to_nearest_fractional_beat::<128>()
    }

    pub fn snap_to_nearest_256th_beat(&self) -> MusicalTime {
        self.snap_to_nearest_fractional_beat::<256>()
    }

    pub fn snap_to_nearest_512th_beat(&self) -> MusicalTime {
        self.snap_to_nearest_fractional_beat::<512>()
    }

    pub fn snap_to_nearest_1024th_beat(&self) -> MusicalTime {
        self.snap_to_nearest_fractional_beat::<1024>()
    }

    pub fn snap_to_nearest_2048th_beat(&self) -> MusicalTime {
        self.snap_to_nearest_fractional_beat::<2048>()
    }

    pub fn snap_to_nearest_third_beat(&self) -> MusicalTime {
        self.snap_to_nearest_fractional_beat::<3>()
    }

    pub fn snap_to_nearest_fifth_beat(&self) -> MusicalTime {
        self.snap_to_nearest_fractional_beat::<5>()
    }

    pub fn snap_to_nearest_sixth_beat(&self) -> MusicalTime {
        self.snap_to_nearest_fractional_beat::<6>()
    }

    pub fn snap_to_nearest_seventh_beat(&self) -> MusicalTime {
        self.snap_to_nearest_fractional_beat::<7>()
    }

    pub fn snap_to_nearest_ninth_beat(&self) -> MusicalTime {
        self.snap_to_nearest_fractional_beat::<9>()
    }

    pub fn snap_to_nearest_tenth_beat(&self) -> MusicalTime {
        self.snap_to_nearest_fractional_beat::<10>()
    }

    pub fn snap_to_nearest_eleventh_beat(&self) -> MusicalTime {
        self.snap_to_nearest_fractional_beat::<11>()
    }

    pub fn snap_to_nearest_twelfth_beat(&self) -> MusicalTime {
        self.snap_to_nearest_fractional_beat::<12>()
    }

    pub fn snap_to_nearest_24th_beat(&self) -> MusicalTime {
        self.snap_to_nearest_fractional_beat::<24>()
    }

    /// The number of fractional-beats *after* `self.beats()` (floored to
    /// the nearest fractional-beat).
    ///
    /// This will always be in the range `[0, DIVISOR - 1]`.
    pub fn num_fractional_beats<const DIVISOR: u32>(&self) -> u32 {
        self.ticks / (SUPER_BEAT_TICKS_PER_BEAT / DIVISOR)
    }

    /// The number of half-beats *after* `self.beats()` (floored to
    /// the nearest half-beat).
    ///
    /// This will always be in the range `[0, 1]`.
    pub fn num_half_beats(&self) -> u32 {
        self.num_fractional_beats::<2>()
    }

    /// The number of quarter-beats *after* `self.beats()` (floored to
    /// the nearest quarter-beat).
    ///
    /// This will always be in the range `[0, 3]`.
    pub fn num_quarter_beats(&self) -> u32 {
        self.num_fractional_beats::<4>()
    }

    /// The number of eigth-beats *after* `self.beats()` (floored to
    /// the nearest eigth-beat).
    ///
    /// This will always be in the range `[0, 7]`.
    pub fn num_eigth_beats(&self) -> u32 {
        self.num_fractional_beats::<8>()
    }

    /// The number of sixteenth-beats *after* `self.beats()` (floored to
    /// the nearest sixteenth-beat).
    ///
    /// This will always be in the range `[0, 15]`.
    pub fn num_sixteenth_beats(&self) -> u32 {
        self.num_fractional_beats::<16>()
    }

    /// The number of 32nd-beats *after* `self.beats()` (floored to
    /// the nearest 32nd-beat).
    ///
    /// This will always be in the range `[0, 31]`.
    pub fn num_32nd_beats(&self) -> u32 {
        self.num_fractional_beats::<32>()
    }

    /// The number of 64th-beats *after* `self.beats()` (floored to
    /// the nearest 64th-beat).
    ///
    /// This will always be in the range `[0, 63]`.
    pub fn num_64th_beats(&self) -> u32 {
        self.num_fractional_beats::<64>()
    }

    /// The number of 128th-beats *after* `self.beats()` (floored to
    /// the nearest 128th-beat).
    ///
    /// This will always be in the range `[0, 127]`.
    pub fn num_128th_beats(&self) -> u32 {
        self.num_fractional_beats::<128>()
    }

    /// The number of 256th-beats *after* `self.beats()` (floored to
    /// the nearest 256th-beat).
    ///
    /// This will always be in the range `[0, 255]`.
    pub fn num_256th_beats(&self) -> u32 {
        self.num_fractional_beats::<256>()
    }

    /// The number of 512th-beats *after* `self.beats()` (floored to
    /// the nearest 512th-beat).
    ///
    /// This will always be in the range `[0, 511]`.
    pub fn num_512th_beats(&self) -> u32 {
        self.num_fractional_beats::<512>()
    }

    /// The number of 1024th-beats *after* `self.beats()` (floored to
    /// the nearest 1024th-beat).
    ///
    /// This will always be in the range `[0, 1023]`.
    pub fn num_1024th_beats(&self) -> u32 {
        self.num_fractional_beats::<1024>()
    }

    /// The number of 2048th-beats *after* `self.beats()` (floored to
    /// the nearest 2048th-beat).
    ///
    /// This will always be in the range `[0, 2047]`.
    pub fn num_2048th_beats(&self) -> u32 {
        self.num_fractional_beats::<2048>()
    }

    /// The number of third-beats *after* `self.beats()` (floored to
    /// the nearest third-beat).
    ///
    /// This will always be in the range `[0, 2]`.
    pub fn num_third_beats(&self) -> u32 {
        self.num_fractional_beats::<3>()
    }

    /// The number of fifth-beats *after* `self.beats()` (floored to
    /// the nearest fifth-beat).
    ///
    /// This will always be in the range `[0, 4]`.
    pub fn num_fifth_beats(&self) -> u32 {
        self.num_fractional_beats::<5>()
    }

    /// The number of sixth-beats *after* `self.beats()` (floored to
    /// the nearest sixth-beat).
    ///
    /// This will always be in the range `[0, 5]`.
    pub fn num_sixth_beats(&self) -> u32 {
        self.num_fractional_beats::<6>()
    }

    /// The number of seventh-beats *after* `self.beats()` (floored to
    /// the nearest seventh-beat).
    ///
    /// This will always be in the range `[0, 6]`.
    pub fn num_seventh_beats(&self) -> u32 {
        self.num_fractional_beats::<7>()
    }

    /// The number of ninth-beats *after* `self.beats()` (floored to
    /// the nearest ninth-beat).
    ///
    /// This will always be in the range `[0, 8]`.
    pub fn num_ninth_beats(&self) -> u32 {
        self.num_fractional_beats::<9>()
    }

    /// The number of tenth-beats *after* `self.beats()` (floored to
    /// the nearest tenth-beat).
    ///
    /// This will always be in the range `[0, 9]`.
    pub fn num_tenth_beats(&self) -> u32 {
        self.num_fractional_beats::<10>()
    }

    /// The number of eleventh-beats *after* `self.beats()` (floored to
    /// the nearest eleventh-beat).
    ///
    /// This will always be in the range `[0, 10]`.
    pub fn num_eleventh_beats(&self) -> u32 {
        self.num_fractional_beats::<11>()
    }

    /// The number of twelfth-beats *after* `self.beats()` (floored to
    /// the nearest twelfth-beat).
    ///
    /// This will always be in the range `[0, 11]`.
    pub fn num_twelfth_beats(&self) -> u32 {
        self.num_fractional_beats::<12>()
    }

    /// The number of 24th-beats *after* `self.beats()` (floored to
    /// the nearest 24th-beat).
    ///
    /// This will always be in the range `[0, 23]`.
    pub fn num_24th_beats(&self) -> u32 {
        self.num_fractional_beats::<24>()
    }

    /// Convert to the corresponding time in [`SecondsF64`].
    ///
    /// Note that this conversion is *NOT* lossless.
    ///
    /// [`SecondsF64`]: struct.SecondsF64.html
    pub fn to_seconds_f64(&self, bpm: f64) -> SecondsF64 {
        SecondsF64(self.as_beats_f64() * 60.0 / bpm)
    }

    /// Convert to the corresponding discrete [`FrameTime`]. This will be rounded to the nearest frame.
    ///
    /// Note that this conversion is *NOT* lossless.
    ///
    /// Note that this must be re-calculated after recieving a new sample rate.
    ///
    /// [`FrameTime`]: struct.FrameTime.html
    pub fn to_nearest_frame_round(&self, bpm: f64, sample_rate: u32) -> FrameTime {
        self.to_seconds_f64(bpm).to_nearest_frame_round(sample_rate)
    }

    /// Convert to the corresponding discrete [`FrameTime`]. This will be floored to the nearest frame.
    ///
    /// Note that this conversion is *NOT* lossless.
    ///
    /// Note that this must be re-calculated after recieving a new sample rate.
    ///
    /// [`FrameTime`]: struct.FrameTime.html
    pub fn to_nearest_frame_floor(&self, bpm: f64, sample_rate: u32) -> FrameTime {
        self.to_seconds_f64(bpm).to_nearest_frame_floor(sample_rate)
    }

    /// Convert to the corresponding discrete [`FrameTime`]. This will be ceil-ed to the nearest frame.
    ///
    /// Note that this conversion is *NOT* lossless.
    ///
    /// Note that this must be re-calculated after recieving a new sample rate.
    ///
    /// [`FrameTime`]: struct.FrameTime.html
    pub fn to_nearest_frame_ceil(&self, bpm: f64, sample_rate: u32) -> FrameTime {
        self.to_seconds_f64(bpm).to_nearest_frame_ceil(sample_rate)
    }

    /// Convert to the corresponding discrete [`FrameTime`] floored to the nearest frame,
    /// while also returning the fractional sub-sample part.
    ///
    /// Note that this conversion is *NOT* lossless.
    ///
    /// Note that this must be re-calculated after recieving a new sample rate.
    ///
    /// [`FrameTime`]: struct.FrameTime.html
    pub fn to_sub_frame(&self, bpm: f64, sample_rate: u32) -> (FrameTime, f64) {
        self.to_seconds_f64(bpm).to_sub_frame(sample_rate)
    }

    /// Convert to the corresponding discrete [`SuperclockTime`]. This will be rounded to the nearest super-frame.
    ///
    /// Note that this conversion is *NOT* lossless.
    ///
    /// [`SuperclockTime`]: struct.SuperclockTime.html
    pub fn to_nearest_super_frame_round(&self, bpm: f64) -> SuperclockTime {
        self.to_seconds_f64(bpm).to_nearest_super_frame_round()
    }

    /// Convert to the corresponding discrete [`SuperclockTime`]. This will be floored to the nearest super-frame.
    ///
    /// Note that this conversion is *NOT* lossless.
    ///
    /// [`SuperclockTime`]: struct.SuperclockTime.html
    pub fn to_nearest_super_frame_floor(&self, bpm: f64) -> SuperclockTime {
        self.to_seconds_f64(bpm).to_nearest_super_frame_floor()
    }

    /// Convert to the corresponding discrete [`SuperclockTime`]. This will be ceil-ed to the nearest super-frame.
    ///
    /// Note that this conversion is *NOT* lossless.
    ///
    /// [`SuperclockTime`]: struct.SuperclockTime.html
    pub fn to_nearest_super_frame_ceil(&self, bpm: f64) -> SuperclockTime {
        self.to_seconds_f64(bpm).to_nearest_super_frame_ceil()
    }

    /// Convert to the corresponding discrete [`SuperclockTime`] floored to the nearest super-frame,
    /// while also returning the fractional sub-super-frame part.
    ///
    /// Note that this conversion is *NOT* lossless.
    ///
    /// [`SuperclockTime`]: struct.SuperclockTime.html
    pub fn to_sub_super_frame(&self, bpm: f64) -> (SuperclockTime, f64) {
        self.to_seconds_f64(bpm).to_sub_super_frame()
    }

    /// Try subtracting `rhs` from self. This will return `None` if the resulting value
    /// is negative due to `rhs` being larger than self (overflow).
    pub fn checked_sub(self, rhs: MusicalTime) -> Option<MusicalTime> {
        if rhs.beats > self.beats {
            None
        } else if rhs.beats == self.beats {
            if rhs.ticks > self.ticks {
                None
            } else {
                Some(MusicalTime { beats: 0, ticks: self.ticks - rhs.ticks })
            }
        } else {
            if rhs.ticks > self.ticks {
                Some(MusicalTime {
                    beats: self.beats - rhs.beats - 1,
                    ticks: SUPER_BEAT_TICKS_PER_BEAT - (rhs.ticks - self.ticks),
                })
            } else {
                Some(MusicalTime { beats: self.beats - rhs.beats, ticks: self.ticks - rhs.ticks })
            }
        }
    }
}

impl PartialEq for MusicalTime {
    fn eq(&self, other: &Self) -> bool {
        self.beats == other.beats && self.ticks == other.ticks
    }
}

impl Eq for MusicalTime {}

impl PartialOrd for MusicalTime {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if self.beats != other.beats {
            self.beats.partial_cmp(&other.beats)
        } else {
            self.ticks.partial_cmp(&other.ticks)
        }
    }
}

impl Ord for MusicalTime {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if self.beats != other.beats {
            self.beats.cmp(&other.beats)
        } else {
            self.ticks.cmp(&other.ticks)
        }
    }
}

impl Add<MusicalTime> for MusicalTime {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        let mut beats = self.beats + rhs.beats;
        let mut ticks = self.ticks + rhs.ticks;

        if ticks >= SUPER_BEAT_TICKS_PER_BEAT {
            ticks -= SUPER_BEAT_TICKS_PER_BEAT;
            beats += 1;
        }

        Self { beats, ticks }
    }
}
impl Mul<u32> for MusicalTime {
    type Output = Self;
    fn mul(self, rhs: u32) -> Self::Output {
        let mut beats = self.beats * rhs;
        let mut ticks = u64::from(self.ticks) * u64::from(rhs);

        if ticks >= u64::from(SUPER_BEAT_TICKS_PER_BEAT) {
            beats += (ticks / u64::from(SUPER_BEAT_TICKS_PER_BEAT)) as u32;
            ticks = ticks % u64::from(SUPER_BEAT_TICKS_PER_BEAT);
        }

        Self { beats, ticks: ticks as u32 }
    }
}

impl AddAssign<MusicalTime> for MusicalTime {
    fn add_assign(&mut self, other: Self) {
        *self = *self + other
    }
}
impl MulAssign<u32> for MusicalTime {
    fn mul_assign(&mut self, other: u32) {
        *self = *self * other
    }
}
