use super::{FrameTime, MusicalTime, SecondsF64, Timestamp};
use dropseed::engine::{DSTempoMap, TransportInfoAtFrame};
use dropseed::plugin_api::{BeatTime, SecondsTime};

// TODO: Make tempo map work like series of automation lines/curves between points in time.

/// A map of all tempo changes in the current project. Used to convert timestamps
/// to/from time in frames.
#[derive(Debug, Clone)]
pub struct TempoMap {
    /// Temporary static tempo
    bpm: f64,
    beats_per_second: f64,
    seconds_per_beat: f64,

    /// Temporary static time signature
    tsig_num: u16,
    tsig_denom: u16,

    sample_rate: u32,
    sample_rate_u64: u64,
    sample_rate_recip: f64,
}

impl TempoMap {
    /// Temporary static tempo and time signature
    pub fn new(bpm: f64, tsig_num: u16, tsig_denom: u16, sample_rate: u32) -> Self {
        assert_ne!(sample_rate, 0);
        assert_ne!(bpm, 0.0);
        assert_ne!(tsig_num, 0);
        assert_ne!(tsig_denom, 0);

        TempoMap {
            bpm,
            beats_per_second: bpm / 60.0,
            seconds_per_beat: 60.0 / bpm,
            tsig_num,
            tsig_denom,
            sample_rate: sample_rate.into(),
            sample_rate_u64: u64::from(sample_rate),
            sample_rate_recip: 1.0 / f64::from(sample_rate),
        }
    }

    pub fn bpm(&self) -> f64 {
        self.bpm
    }

    pub fn beats_per_second(&self) -> f64 {
        self.beats_per_second
    }

    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    pub fn sample_rate_recip(&self) -> f64 {
        self.sample_rate_recip
    }

    pub fn set_bpm(&mut self, bpm: f64) {
        assert_ne!(bpm, 0.0);

        self.bpm = bpm;
        self.beats_per_second = bpm / 60.0;
        self.seconds_per_beat = 60.0 / bpm;
    }

    pub fn set_sample_rate(&mut self, sample_rate: u32) {
        assert_ne!(sample_rate, 0);

        self.sample_rate = sample_rate.into();
        self.sample_rate_u64 = u64::from(sample_rate);
        self.sample_rate_recip = 1.0 / f64::from(sample_rate);
    }

    pub fn set_tsig(&mut self, tsig_num: u16, tsig_denom: u16) {
        assert_ne!(tsig_num, 0);
        assert_ne!(tsig_denom, 0);

        self.tsig_num = tsig_num;
        self.tsig_denom = tsig_denom;
    }

    pub fn timestamp_to_nearest_frame_round(&self, timestamp: Timestamp) -> FrameTime {
        match timestamp {
            Timestamp::Musical(t) => self.musical_to_nearest_frame_round(t),
            Timestamp::Superclock(t) => t.to_nearest_frame_round(self.sample_rate),
        }
    }

    /// Convert the given `MusicalTime` into the corresponding time in `SecondsF64`.
    ///
    /// Note that this must be re-calculated after recieving a new `TempoMap`.
    #[inline]
    pub fn musical_to_seconds(&self, musical_time: MusicalTime) -> SecondsF64 {
        // temporary static tempo
        SecondsF64(musical_time.as_beats_f64() * self.seconds_per_beat)
    }

    /// Convert the given `SecondsF64` into the corresponding `MusicalTime`.
    ///
    /// Note that this must be re-calculated after recieving a new `TempoMap`.
    #[inline]
    pub fn seconds_to_musical(&self, seconds: SecondsF64) -> MusicalTime {
        // temporary static tempo
        MusicalTime::from_beats_f64(seconds.0 * self.beats_per_second)
    }

    /*
    /// Convert the given `FrameTime` time into the corresponding `MusicalTime`.
    ///
    /// Note that this must be re-calculated after recieving a new `TempoMap`.
    #[inline]
    pub fn frame_to_musical(&self, frame: FrameTime) -> MusicalTime {
        // temporary static tempo
        MusicalTime::from_beats_f64(
            frame.to_seconds_f64(self.sample_rate).0 * self.beats_per_second,
        )
    }

    /// Convert the given `FrameTime` time into the corresponding time in `SecondsF64`.
    ///
    /// Note that this must be re-calculated after recieving a new `TempoMap`.
    #[inline]
    pub fn frame_to_seconds(&self, frame: FrameTime) -> SecondsF64 {
        // temporary static tempo
        SecondsF64::from_frame(frame, self.sample_rate)
    }
    */

    /// Convert the given `MusicalTime` into the corresponding discrete `FrameTime` time.
    /// This will be rounded to the nearest frame.
    ///
    /// Note that this must be re-calculated after recieving a new `TempoMap`.
    #[inline]
    pub fn musical_to_nearest_frame_round(&self, musical_time: MusicalTime) -> FrameTime {
        // temporary static tempo
        self.musical_to_seconds(musical_time).to_nearest_frame_round(self.sample_rate)
    }

    /// Convert the given `SecondsF64` into the corresponding discrete `FrameTime` time.
    /// This will be rounded to the nearest frame.
    ///
    /// Note that this must be re-calculated after recieving a new `TempoMap`.
    #[inline]
    pub fn seconds_to_nearest_frame_round(&self, seconds: SecondsF64) -> FrameTime {
        // temporary static tempo
        seconds.to_nearest_frame_round(self.sample_rate)
    }

    /// Convert the given `MusicalTime` into the corresponding discrete `FrameTime` time.
    /// This will be floored to the nearest frame.
    ///
    /// Note that this must be re-calculated after recieving a new `TempoMap`.
    #[inline]
    pub fn musical_to_nearest_frame_floor(&self, musical_time: MusicalTime) -> FrameTime {
        // temporary static tempo
        self.musical_to_seconds(musical_time).to_nearest_frame_floor(self.sample_rate)
    }

    /// Convert the given `SecondsF64` into the corresponding discrete `FrameTime` time.
    /// This will be floored to the nearest frame.
    ///
    /// Note that this must be re-calculated after recieving a new `TempoMap`.
    #[inline]
    pub fn seconds_to_nearest_frame_floor(&self, seconds: SecondsF64) -> FrameTime {
        // temporary static tempo
        seconds.to_nearest_frame_floor(self.sample_rate)
    }

    /// Convert the given `MusicalTime` into the corresponding discrete `FrameTime` time.
    /// This will be ceil-ed to the nearest frame.
    ///
    /// Note that this must be re-calculated after recieving a new `TempoMap`.
    #[inline]
    pub fn musical_to_nearest_frame_ceil(&self, musical_time: MusicalTime) -> FrameTime {
        // temporary static tempo
        self.musical_to_seconds(musical_time).to_nearest_frame_ceil(self.sample_rate)
    }

    /// Convert the given `SecondsF64` into the corresponding discrete `FrameTime` time.
    /// This will be ceil-ed to the nearest frame.
    ///
    /// Note that this must be re-calculated after recieving a new `TempoMap`.
    #[inline]
    pub fn seconds_to_nearest_frame_ceil(&self, seconds: SecondsF64) -> FrameTime {
        // temporary static tempo
        seconds.to_nearest_frame_ceil(self.sample_rate)
    }

    /// Convert the given `MusicalTime` into the corresponding discrete `FrameTime` time
    /// floored to the nearest frame, while also returning the fractional sub-frame part.
    ///
    /// Note that this must be re-calculated after recieving a new `TempoMap`.
    #[inline]
    pub fn musical_to_sub_frame(&self, musical_time: MusicalTime) -> (FrameTime, f64) {
        // temporary static tempo
        self.musical_to_seconds(musical_time).to_sub_frame(self.sample_rate)
    }

    /// Convert the given `SecondsF64` into the corresponding discrete `FrameTime` time
    /// floored to the nearest frame, while also returning the fractional sub-frame part.
    ///
    /// Note that this must be re-calculated after recieving a new `TempoMap`.
    #[inline]
    pub fn seconds_to_sub_frame(&self, seconds: SecondsF64) -> (FrameTime, f64) {
        // temporary static tempo
        seconds.to_sub_frame(self.sample_rate)
    }
}

impl DSTempoMap for TempoMap {
    fn frame_to_beat(&self, frame: u64) -> BeatTime {
        let whole_seconds = frame / self.sample_rate_u64;
        let fract_frames = frame % self.sample_rate_u64;

        let whole_beats = whole_seconds as f64 * self.beats_per_second;

        let fract_seconds = fract_frames as f64 * self.sample_rate_recip;
        let fract_beats = fract_seconds * self.beats_per_second;

        BeatTime::from_float(whole_beats + fract_beats)
    }

    fn frame_to_seconds(&self, frame: u64) -> SecondsTime {
        let whole_seconds = frame / self.sample_rate_u64;
        let fract_frames = frame % self.sample_rate_u64;

        let fract_seconds = fract_frames as f64 * self.sample_rate_recip;

        SecondsTime::from_float(whole_seconds as f64 + fract_seconds)
    }

    fn transport_info_at_frame(&self, frame: u64) -> TransportInfoAtFrame {
        let current_beat = self.frame_to_beat(frame).to_int() as u64;

        let current_bar_number = current_beat / u64::from(self.tsig_num);

        let current_bar_start =
            BeatTime::from_int((current_bar_number * u64::from(self.tsig_num)) as i64);

        TransportInfoAtFrame {
            tempo: self.bpm,
            tempo_inc: 0.0,
            tsig_num: self.tsig_num,
            tsig_denom: self.tsig_denom,
            current_bar_number: current_bar_number as i32,
            current_bar_start,
        }
    }
}

impl Default for TempoMap {
    fn default() -> Self {
        TempoMap::new(110.0, 4, 4, 44_100)
    }
}
