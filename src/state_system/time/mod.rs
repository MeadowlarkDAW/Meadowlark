mod frame_time;
mod musical_time;
mod seconds;
mod superclock_time;
mod tempo_map;
//mod video_timecode;

pub use frame_time::FrameTime;
pub use musical_time::{MusicalTime, SUPER_BEAT_TICKS_PER_BEAT};
pub use seconds::SecondsF64;
pub use superclock_time::{SuperclockTime, SUPER_SAMPLE_TICKS_PER_SECOND};
pub use tempo_map::TempoMap;
//pub use video_timecode::{VideoFpsFormat, VideoTimecode};

/// A reliable timestamp for events on the timeline.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Timestamp {
    /// Musical time in units of beats + ticks.
    ///
    /// A "tick" is a unit of time equal to `1 / 1,241,856,000` of a beat. This number was chosen
    /// because it is nicely divisible by a whole slew of factors including `2, 3, 4, 5, 6, 7, 8, 9,
    /// 10, 11, 12, 13, 14, 15, 16, 18, 20, 24, 32, 64, 128, 256, 512, 1,024, 1,920, 2,048, 4,096,
    /// 8,192, 16,384, and 32,768`. This ensures that all these subdivisions of musical beats can be
    /// stored and operated on with *exact* precision. This number is also much larger than all of
    /// the common sampling rates, allowing for sample-accurate precision even at very high sampling
    /// rates and very low BPMs.
    Musical(MusicalTime),
    /// Unit of time length in seconds + ticks.
    ///
    /// A "tick" is a unit of time that is exactly 1 / 282,240,000 of a second. This number
    /// happens to be nicely divisible by all common sampling rates: `22,050, 24,000, 44,100,
    /// 48,000, 88,200, 96,000, 176,400, 192,000, 352,800, and 384,000`. This ensures that no
    /// information is lost when switching between sample rates.
    Superclock(SuperclockTime),
    // TODO: Flesh this out once I have a better idea how this should work.
    // Video(VideoTimecode),
}
