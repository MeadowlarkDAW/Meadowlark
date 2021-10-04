use rusty_daw_core::{MusicalTime, SampleRate, SampleTime, Seconds};

// TODO: Make tempo map work like series of automation lines/curves between points in time.

/// A map of all tempo changes in the current project.
///
/// Here is the intended workflow for keeping time:
/// 1. The GUI/non-realtime thread stores all events in [`MusicalTime`] (unit of beats).
/// 2. The GUI/non-realtime thread creates a new [`TempoMap`] on project startup and whenever anything about
/// the tempo changes. It this sends this new [`TempoMap`] to the realtime-thread.
/// 3. When the realtime thread detects a new [`TempoMap`], all processors with events stored in [`MusicalTime`] use
/// the [`TempoMap`] plus the [`SampleRate`] to convert each [`MusicalTime`] into the corresponding discrete [`SampleTime`] (or sub-samples).
/// It keeps this new [`SampleTime`] for all future use (until a new [`TempoMap`] is recieved).
/// 4. When playback occurs, the realtime-thread keeps tracks of the number of discrete samples that have elapsed. It
/// sends this "playhead" to each of the processors, which in turn compares it to it's own previously calculated [`SampleTime`] to know when
/// events should be played.
/// 5. Once the realtime thread is done processing a buffer, it uses the [`TempoMap`] plus this [`SampleRate`] to convert this
/// playhead into the corresponding [`MusicalTime`]. It then sends this to the GUI/non-realtime thread for visual
/// feedback of the playhead.
/// 6. When the GUI/non-realtime thread wants to manually change the position of the playhead, it sends the [`MusicalTime`] that
/// should be seeked to the realtime thread. The realtime thread then uses it, the [`TempoMap`], and the [`SampleRate`] to find
/// the nearest (floored) sample to set as the new playhead.
///
/// [`TempoMap`]: struct.TempoMap.html
/// [`SampleTime`]: ../struct.Sampletime.html
/// [`SampleRate`]: ../struct.SampleRate.html
#[derive(Debug, Clone)]
pub struct TempoMap {
    pub sample_rate: SampleRate,

    /// Temporary static tempo
    beats_per_second: f64,
    seconds_per_beat: f64,
}

impl TempoMap {
    /// Temporary static tempo
    pub fn new(bpm: f64, sample_rate: SampleRate) -> Self {
        TempoMap { beats_per_second: bpm / 60.0, seconds_per_beat: 60.0 / bpm, sample_rate }
    }

    /// Temporary static tempo
    #[inline]
    pub fn bpm(&self) -> f64 {
        self.beats_per_second * 60.0
    }

    /// Temporary static tempo
    pub fn set_bpm(&mut self, bpm: f64) {
        self.beats_per_second = bpm / 60.0;
        self.seconds_per_beat = 60.0 / bpm;
    }

    /// Convert the given [`MusicalTime`] into the corresponding time in [`Seconds`].
    ///
    /// Note that this must be re-calculated after recieving a new [`TempoMap`].
    ///
    /// [`MusicalTime`]: ../struct.MusicalTime.html
    /// [`Seconds`]: ../struct.Seconds.html
    /// [`TempoMap`]: struct.TempoMap.html
    #[inline]
    pub fn musical_to_seconds(&self, musical_time: MusicalTime) -> Seconds {
        Seconds(musical_time.0 * self.seconds_per_beat)
    }

    /// Convert the given [`Seconds`] into the corresponding [`MusicalTime`].
    ///
    /// Note that this must be re-calculated after recieving a new [`TempoMap`].
    ///
    /// [`Seconds`]: ../struct.Seconds.html
    /// [`MusicalTime`]: ../struct.MusicalTime.html
    #[inline]
    pub fn seconds_to_musical(&self, seconds: Seconds) -> MusicalTime {
        MusicalTime(seconds.0 * self.beats_per_second)
    }

    /// Convert the given [`SampleTime`] into the corresponding [`MusicalTime`].
    ///
    /// Note that this must be re-calculated after recieving a new [`TempoMap`].
    ///
    /// [`SampleTime`]: ../struct.Sampletime.html
    /// [`MusicalTime`]: ../struct.MusicalTime.html
    /// [`TempoMap`]: struct.TempoMap.html
    #[inline]
    pub fn sample_to_musical(&self, sample_time: SampleTime) -> MusicalTime {
        MusicalTime(sample_time.to_seconds(self.sample_rate).0 * self.beats_per_second)
    }

    /// Convert the given [`MusicalTime`] into the corresponding discrete [`SampleTime`].
    /// This will be rounded to the nearest sample.
    ///
    /// Note that this must be re-calculated after recieving a new [`TempoMap`].
    ///
    /// [`MusicalTime`]: ../struct.MusicalTime.html
    /// [`SampleTime`]: ../struct.SampleTime.html
    /// [`TempoMap`]: struct.TempoMap.html
    #[inline]
    pub fn musical_to_nearest_sample_round(&self, musical_time: MusicalTime) -> SampleTime {
        self.musical_to_seconds(musical_time).to_nearest_sample_round(self.sample_rate)
    }

    /// Convert the given [`Seconds`] into the corresponding discrete [`SampleTime`].
    /// This will be rounded to the nearest sample.
    ///
    /// Note that this must be re-calculated after recieving a new [`TempoMap`].
    ///
    /// [`Seconds`]: ../struct.Seconds.html
    /// [`SampleTime`]: ../struct.SampleTime.html
    /// [`TempoMap`]: struct.TempoMap.html
    #[inline]
    pub fn seconds_to_nearest_sample_round(&self, seconds: Seconds) -> SampleTime {
        seconds.to_nearest_sample_round(self.sample_rate)
    }

    /// Convert the given [`MusicalTime`] into the corresponding discrete [`SampleTime`].
    /// This will be floored to the nearest sample.
    ///
    /// Note that this must be re-calculated after recieving a new [`TempoMap`].
    ///
    /// [`MusicalTime`]: ../struct.MusicalTime.html
    /// [`SampleTime`]: ../struct.SampleTime.html
    /// [`TempoMap`]: struct.TempoMap.html
    #[inline]
    pub fn musical_to_nearest_sample_floor(&self, musical_time: MusicalTime) -> SampleTime {
        self.musical_to_seconds(musical_time).to_nearest_sample_floor(self.sample_rate)
    }

    /// Convert the given [`Seconds`] into the corresponding discrete [`SampleTime`].
    /// This will be floored to the nearest sample.
    ///
    /// Note that this must be re-calculated after recieving a new [`TempoMap`].
    ///
    /// [`Seconds`]: ../struct.Seconds.html
    /// [`SampleTime`]: ../struct.SampleTime.html
    /// [`TempoMap`]: struct.TempoMap.html
    #[inline]
    pub fn seconds_to_nearest_sample_floor(&self, seconds: Seconds) -> SampleTime {
        seconds.to_nearest_sample_floor(self.sample_rate)
    }

    /// Convert the given [`MusicalTime`] into the corresponding discrete [`SampleTime`].
    /// This will be ceil-ed to the nearest sample.
    ///
    /// Note that this must be re-calculated after recieving a new [`TempoMap`].
    ///
    /// [`MusicalTime`]: ../struct.MusicalTime.html
    /// [`SampleTime`]: ../struct.SampleTime.html
    /// [`TempoMap`]: struct.TempoMap.html
    #[inline]
    pub fn musical_to_nearest_sample_ceil(&self, musical_time: MusicalTime) -> SampleTime {
        self.musical_to_seconds(musical_time).to_nearest_sample_ceil(self.sample_rate)
    }

    /// Convert the given [`Seconds`] into the corresponding discrete [`SampleTime`].
    /// This will be ceil-ed to the nearest sample.
    ///
    /// Note that this must be re-calculated after recieving a new [`TempoMap`].
    ///
    /// [`Seconds`]: ../struct.Seconds.html
    /// [`SampleTime`]: ../struct.SampleTime.html
    /// [`TempoMap`]: struct.TempoMap.html
    #[inline]
    pub fn seconds_to_nearest_sample_ceil(&self, seconds: Seconds) -> SampleTime {
        seconds.to_nearest_sample_ceil(self.sample_rate)
    }

    /// Convert the given [`MusicalTime`] into the corresponding discrete [`SampleTime`]
    /// floored to the nearest sample, while also returning the fractional sub-sample part.
    ///
    /// Note that this must be re-calculated after recieving a new [`TempoMap`].
    ///
    /// [`MusicalTime`]: ../struct.MusicalTime.html
    /// [`SampleTime`]: ../struct.SampleTime.html
    /// [`TempoMap`]: struct.TempoMap.html
    #[inline]
    pub fn musical_to_sub_sample(&self, musical_time: MusicalTime) -> (SampleTime, f64) {
        self.musical_to_seconds(musical_time).to_sub_sample(self.sample_rate)
    }

    /// Convert the given [`Seconds`] into the corresponding discrete [`SampleTime`]
    /// floored to the nearest sample, while also returning the fractional sub-sample part.
    ///
    /// Note that this must be re-calculated after recieving a new [`TempoMap`].
    ///
    /// [`Seconds`]: ../struct.Seconds.html
    /// [`SampleTime`]: ../struct.SampleTime.html
    /// [`TempoMap`]: struct.TempoMap.html
    #[inline]
    pub fn seconds_to_sub_sample(&self, seconds: Seconds) -> (SampleTime, f64) {
        seconds.to_sub_sample(self.sample_rate)
    }
}

impl Default for TempoMap {
    fn default() -> Self {
        TempoMap::new(110.0, SampleRate::default())
    }
}
