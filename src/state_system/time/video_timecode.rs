
// TODO: Flesh this out once I have a better idea how this should work.

/// The different framerate formats used with video encoding.
///
/// Useful when editing the sound of video with the timeline.
#[cfg_attr(feature = "serde-derive", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Hash)]
pub enum VideoFpsFormat {
    Fps23_976,
    Fps24,
    Fps24_976,
    Fps25,
    Fps29_97,
    Fps29_97drop,
    Fps29_97000,
    Fps29_97000drop,
    Fps30,
    Fps30drop,
    Fps59_94,
    Fps60,
}

#[cfg_attr(feature = "serde-derive", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Hash)]
pub struct VideoTimecode {
    pub sample: u32,
    pub sub_frames: u32,
    pub format: VideoFpsFormat,
}

// TODO: Methods to help convert a `SampleTimestamp` to its actual time in seconds.