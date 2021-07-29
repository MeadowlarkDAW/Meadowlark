static U24_TO_F32_RATIO: f32 = 2.0 / 0x00FFFFFF as f32;
static I16_TO_F32_RATIO: f32 = 1.0 / std::i16::MAX as f32;
static U8_TO_F32_RATIO: f32 = 2.0 / std::u8::MAX as f32;

pub mod loader;

pub use loader::{PcmLoadError, PcmLoader};
use rusty_daw_time::{SampleRate, SampleTime, Seconds};

#[non_exhaustive]
#[derive(Debug)]
pub enum AnyPcm {
    Mono(MonoPcm),
    Stereo(StereoPcm),
}

#[derive(Debug)]
pub struct MonoPcm {
    data: Vec<f32>,
    sample_rate: SampleRate,
    len_secs: Seconds,
}

impl MonoPcm {
    pub fn new(data: Vec<f32>, sample_rate: SampleRate) -> Self {
        let len_secs = SampleTime(data.len() as i64).to_seconds(sample_rate);

        Self {
            data,
            sample_rate,
            len_secs,
        }
    }

    #[inline]
    pub fn data(&self) -> &[f32] {
        &self.data
    }

    #[inline]
    pub fn sample_rate(&self) -> SampleRate {
        self.sample_rate
    }

    #[inline]
    pub fn len_seconds(&self) -> Seconds {
        self.len_secs
    }
}

#[derive(Debug)]
pub struct StereoPcm {
    left: Vec<f32>,
    right: Vec<f32>,

    sample_rate: SampleRate,
    len_secs: Seconds,
}

impl StereoPcm {
    pub fn new(left: Vec<f32>, right: Vec<f32>, sample_rate: SampleRate) -> Self {
        assert_eq!(left.len(), right.len());

        let len_secs = SampleTime(left.len() as i64).to_seconds(sample_rate);

        Self {
            left,
            right,
            sample_rate,
            len_secs,
        }
    }

    #[inline]
    pub fn left(&self) -> &[f32] {
        &self.left
    }

    #[inline]
    pub fn right(&self) -> &[f32] {
        &self.right
    }

    #[inline]
    pub fn left_right(&self) -> (&[f32], &[f32]) {
        (&self.left, &self.right)
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.left.len()
    }

    #[inline]
    pub fn sample_rate(&self) -> SampleRate {
        self.sample_rate
    }

    #[inline]
    pub fn len_seconds(&self) -> Seconds {
        self.len_secs
    }
}
