use rusty_daw_core::{Frames, MusicalTime, SampleRate, Seconds, SuperFrames};

// TODO: Make tempo map work like series of automation lines/curves between points in time.

/// --- TODO: Update this documentation ------------------------------------
///
/// A map of all tempo changes in the current project.
///
/// Here is the intended workflow for keeping time:
/// 1. The GUI/non-realtime thread stores all events in [`MusicalTime`] (unit of beats).
/// 2. The GUI/non-realtime thread creates a new [`TempoMap`] on project startup and whenever anything about
/// the tempo changes. It this sends this new [`TempoMap`] to the realtime-thread.
/// 3. When the realtime thread detects a new [`TempoMap`], all processors with events stored in [`MusicalTime`] use
/// the [`TempoMap`] plus the [`SampleRate`] to convert each [`MusicalTime`] into the corresponding discrete [`Frames`] (or sub-frames).
/// It keeps this new [`Frames`] for all future use (until a new [`TempoMap`] is recieved).
/// 4. When playback occurs, the realtime-thread keeps tracks of the number of discrete frames that have elapsed. It
/// sends this "playhead" to each of the processors, which in turn compares it to it's own previously calculated [`Frames`] to know when
/// events should be played.
/// 5. Once the realtime thread is done processing a buffer, it uses the [`TempoMap`] plus this [`SampleRate`] to convert this
/// playhead into the corresponding [`MusicalTime`]. It then sends this to the GUI/non-realtime thread for visual
/// feedback of the playhead.
/// 6. When the GUI/non-realtime thread wants to manually change the position of the playhead, it sends the [`MusicalTime`] that
/// should be seeked to the realtime thread. The realtime thread then uses it, the [`TempoMap`], and the [`SampleRate`] to find
/// the nearest (floored) frame to set as the new playhead.
///
/// [`TempoMap`]: struct.TempoMap.html
/// [`Frames`]: ../struct.Frames.html
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
    pub fn musical_to_seconds(&self, musical_time: MusicalTime) -> Seconds {
        musical_time.to_seconds(self.bpm())
    }

    /// Convert the given [`Seconds`] into the corresponding [`MusicalTime`].
    ///
    /// Note that this must be re-calculated after recieving a new [`TempoMap`].
    ///
    /// [`Seconds`]: ../struct.Seconds.html
    /// [`MusicalTime`]: ../struct.MusicalTime.html
    pub fn seconds_to_musical(&self, seconds: Seconds) -> MusicalTime {
        seconds.to_musical(self.bpm())
    }

    /// Convert the given [`Frames`] into the corresponding [`MusicalTime`].
    ///
    /// Note that this must be re-calculated after recieving a new [`TempoMap`].
    ///
    /// [`Frames`]: ../struct.Frames.html
    /// [`MusicalTime`]: ../struct.MusicalTime.html
    /// [`TempoMap`]: struct.TempoMap.html
    pub fn frame_to_musical(&self, frames: Frames) -> MusicalTime {
        frames.to_musical(self.bpm(), self.sample_rate)
        //MusicalTime(frames.to_seconds(self.sample_rate).0 * self.beats_per_second)
    }

    /// Convert the given [`SuperFrames`] into the corresponding [`MusicalTime`].
    ///
    /// Note that this must be re-calculated after recieving a new [`TempoMap`].
    ///
    /// [`SuperFrames`]: ../struct.SuperFrames.html
    /// [`MusicalTime`]: ../struct.MusicalTime.html
    /// [`TempoMap`]: struct.TempoMap.html
    pub fn super_frame_to_musical(&self, super_frames: SuperFrames) -> MusicalTime {
        super_frames.to_musical(self.bpm())
    }

    /// Convert the given [`SuperFrames`] into the corresponding [`Seconds`].
    ///
    /// [`SuperFrames`]: ../struct.SuperFrames.html
    /// [`Seconds`]: ../struct.Seconds.html
    pub fn super_frame_to_seconds(&self, super_frames: SuperFrames) -> Seconds {
        super_frames.to_seconds()
    }

    /// Convert the given [`MusicalTime`] into the corresponding discrete [`Frames`].
    /// This will be rounded to the nearest frame.
    ///
    /// Note that this must be re-calculated after recieving a new [`TempoMap`].
    ///
    /// [`MusicalTime`]: ../struct.MusicalTime.html
    /// [`Frames`]: ../struct.Frames.html
    /// [`TempoMap`]: struct.TempoMap.html
    pub fn musical_to_nearest_frame_round(&self, musical_time: MusicalTime) -> Frames {
        musical_time.to_nearest_frame_round(self.bpm(), self.sample_rate)
    }

    /// Convert the given [`MusicalTime`] into the corresponding discrete [`Frames`].
    /// This will be floored to the nearest frame.
    ///
    /// Note that this must be re-calculated after recieving a new [`TempoMap`].
    ///
    /// [`MusicalTime`]: ../struct.MusicalTime.html
    /// [`Frames`]: ../struct.Frames.html
    /// [`TempoMap`]: struct.TempoMap.html
    pub fn musical_to_nearest_frame_floor(&self, musical_time: MusicalTime) -> Frames {
        musical_time.to_nearest_frame_floor(self.bpm(), self.sample_rate)
    }

    /// Convert the given [`MusicalTime`] into the corresponding discrete [`Frames`].
    /// This will be ceil-ed to the nearest frame.
    ///
    /// Note that this must be re-calculated after recieving a new [`TempoMap`].
    ///
    /// [`MusicalTime`]: ../struct.MusicalTime.html
    /// [`Frames`]: ../struct.Frames.html
    /// [`TempoMap`]: struct.TempoMap.html
    pub fn musical_to_nearest_frame_ceil(&self, musical_time: MusicalTime) -> Frames {
        musical_time.to_nearest_frame_ceil(self.bpm(), self.sample_rate)
    }

    /// Convert the given [`MusicalTime`] into the corresponding discrete [`Frames`]
    /// floored to the nearest frame, while also returning the fractional sub-frame part.
    ///
    /// Note that this must be re-calculated after recieving a new [`TempoMap`].
    ///
    /// [`MusicalTime`]: ../struct.MusicalTime.html
    /// [`Frames`]: ../struct.Frames.html
    /// [`TempoMap`]: struct.TempoMap.html
    pub fn musical_to_sub_frame(&self, musical_time: MusicalTime) -> (Frames, f64) {
        musical_time.to_sub_frames(self.bpm(), self.sample_rate)
    }

    /// Convert the given [`Seconds`] into the corresponding discrete [`Frames`].
    /// This will be rounded to the nearest frame.
    ///
    /// Note that this must be re-calculated after recieving a new [`TempoMap`].
    ///
    /// [`Seconds`]: ../struct.Seconds.html
    /// [`Frames`]: ../struct.Frames.html
    /// [`TempoMap`]: struct.TempoMap.html
    pub fn seconds_to_nearest_frame_round(&self, seconds: Seconds) -> Frames {
        seconds.to_nearest_frame_round(self.sample_rate)
    }

    /// Convert the given [`Seconds`] into the corresponding discrete [`Frames`].
    /// This will be floored to the nearest frame.
    ///
    /// Note that this must be re-calculated after recieving a new [`TempoMap`].
    ///
    /// [`Seconds`]: ../struct.Seconds.html
    /// [`Frames`]: ../struct.Frames.html
    /// [`TempoMap`]: struct.TempoMap.html
    pub fn seconds_to_nearest_frame_floor(&self, seconds: Seconds) -> Frames {
        seconds.to_nearest_frame_floor(self.sample_rate)
    }

    /// Convert the given [`Seconds`] into the corresponding discrete [`Frames`].
    /// This will be ceil-ed to the nearest frame.
    ///
    /// Note that this must be re-calculated after recieving a new [`TempoMap`].
    ///
    /// [`Seconds`]: ../struct.Seconds.html
    /// [`Frames`]: ../struct.Frames.html
    /// [`TempoMap`]: struct.TempoMap.html
    pub fn seconds_to_nearest_frame_ceil(&self, seconds: Seconds) -> Frames {
        seconds.to_nearest_frame_ceil(self.sample_rate)
    }

    /// Convert the given [`Seconds`] into the corresponding discrete [`Frames`]
    /// floored to the nearest frame, while also returning the fractional sub-frame part.
    ///
    /// Note that this must be re-calculated after recieving a new [`TempoMap`].
    ///
    /// [`Seconds`]: ../struct.Seconds.html
    /// [`Frames`]: ../struct.Frames.html
    /// [`TempoMap`]: struct.TempoMap.html
    pub fn seconds_to_sub_frame(&self, seconds: Seconds) -> (Frames, f64) {
        seconds.to_sub_frames(self.sample_rate)
    }

    /// Convert the given [`MusicalTime`] into the corresponding discrete [`SuperFrames`].
    /// This will be rounded to the nearest super-frame.
    ///
    /// Note that this must be re-calculated after recieving a new [`TempoMap`].
    ///
    /// [`MusicalTime`]: ../struct.MusicalTime.html
    /// [`SuperFrames`]: ../struct.SuperFrames.html
    /// [`TempoMap`]: struct.TempoMap.html
    pub fn musical_to_nearest_super_frame_round(&self, musical_time: MusicalTime) -> SuperFrames {
        musical_time.to_nearest_super_frame_round(self.bpm())
    }

    /// Convert the given [`MusicalTime`] into the corresponding discrete [`SuperFrames`].
    /// This will be floored to the nearest super-frame.
    ///
    /// Note that this must be re-calculated after recieving a new [`TempoMap`].
    ///
    /// [`MusicalTime`]: ../struct.MusicalTime.html
    /// [`SuperFrames`]: ../struct.SuperFrames.html
    /// [`TempoMap`]: struct.TempoMap.html
    pub fn musical_to_nearest_super_frame_floor(&self, musical_time: MusicalTime) -> SuperFrames {
        musical_time.to_nearest_super_frame_floor(self.bpm())
    }

    /// Convert the given [`MusicalTime`] into the corresponding discrete [`SuperFrames`].
    /// This will be ceil-ed to the nearest super-frame.
    ///
    /// Note that this must be re-calculated after recieving a new [`TempoMap`].
    ///
    /// [`MusicalTime`]: ../struct.MusicalTime.html
    /// [`SuperFrames`]: ../struct.SuperFrames.html
    /// [`TempoMap`]: struct.TempoMap.html
    pub fn musical_to_nearest_super_frame_ceil(&self, musical_time: MusicalTime) -> SuperFrames {
        musical_time.to_nearest_super_frame_ceil(self.bpm())
    }

    /// Convert the given [`MusicalTime`] into the corresponding discrete [`SuperFrames`]
    /// floored to the nearest frame, while also returning the fractional sub-super-frame part.
    ///
    /// Note that this must be re-calculated after recieving a new [`TempoMap`].
    ///
    /// [`MusicalTime`]: ../struct.MusicalTime.html
    /// [`SuperFrames`]: ../struct.SuperFrames.html
    /// [`TempoMap`]: struct.TempoMap.html
    pub fn musical_to_sub_super_frame(&self, musical_time: MusicalTime) -> (SuperFrames, f64) {
        musical_time.to_sub_super_frames(self.bpm())
    }

    /// Convert the given [`Seconds`] into the corresponding discrete [`SuperFrames`].
    /// This will be rounded to the nearest super-frame.
    ///
    /// Note that this must be re-calculated after recieving a new [`TempoMap`].
    ///
    /// [`Seconds`]: ../struct.Seconds.html
    /// [`SuperFrames`]: ../struct.SuperFrames.html
    /// [`TempoMap`]: struct.TempoMap.html
    pub fn seconds_to_nearest_super_frame_round(&self, seconds: Seconds) -> SuperFrames {
        seconds.to_nearest_super_frame_round()
    }

    /// Convert the given [`Seconds`] into the corresponding discrete [`SuperFrames`].
    /// This will be floored to the nearest super-frame.
    ///
    /// Note that this must be re-calculated after recieving a new [`TempoMap`].
    ///
    /// [`Seconds`]: ../struct.Seconds.html
    /// [`SuperFrames`]: ../struct.SuperFrames.html
    /// [`TempoMap`]: struct.TempoMap.html
    pub fn seconds_to_nearest_super_frame_floor(&self, seconds: Seconds) -> SuperFrames {
        seconds.to_nearest_super_frame_floor()
    }

    /// Convert the given [`Seconds`] into the corresponding discrete [`SuperFrames`].
    /// This will be ceil-ed to the nearest super-frame.
    ///
    /// Note that this must be re-calculated after recieving a new [`TempoMap`].
    ///
    /// [`Seconds`]: ../struct.Seconds.html
    /// [`SuperFrames`]: ../struct.SuperFrames.html
    /// [`TempoMap`]: struct.TempoMap.html
    pub fn seconds_to_nearest_super_frame_ceil(&self, seconds: Seconds) -> SuperFrames {
        seconds.to_nearest_super_frame_ceil()
    }

    /// Convert the given [`Seconds`] into the corresponding discrete [`SuperFrames`]
    /// floored to the nearest frame, while also returning the fractional sub-super-frame part.
    ///
    /// Note that this must be re-calculated after recieving a new [`TempoMap`].
    ///
    /// [`Seconds`]: ../struct.Seconds.html
    /// [`SuperFrames`]: ../struct.SuperFrames.html
    /// [`TempoMap`]: struct.TempoMap.html
    pub fn seconds_to_sub_super_frame(&self, seconds: Seconds) -> (SuperFrames, f64) {
        seconds.to_sub_super_frames()
    }
}

impl Default for TempoMap {
    fn default() -> Self {
        TempoMap::new(120.0, SampleRate::default())
    }
}
