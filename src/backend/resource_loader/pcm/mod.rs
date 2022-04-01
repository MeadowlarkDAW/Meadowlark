static U24_TO_F32_RATIO: f32 = 2.0 / 0x00FFFFFF as f32;
static I16_TO_F32_RATIO: f32 = 1.0 / std::i16::MAX as f32;
static U8_TO_F32_RATIO: f32 = 2.0 / std::u8::MAX as f32;

pub mod loader;

pub use loader::{PcmLoadError, PcmLoader};
use rusty_daw_core::{Frames, SampleRate, Seconds, SuperFrames};

#[non_exhaustive]
#[derive(Debug)]
pub enum AnyPcm {
    Mono(MonoPcm),
    Stereo(StereoPcm),
}

impl AnyPcm {
    pub fn sample_rate(&self) -> SampleRate {
        match self {
            AnyPcm::Mono(pcm) => pcm.sample_rate(),
            AnyPcm::Stereo(pcm) => pcm.sample_rate(),
        }
    }

    pub fn frames(&self) -> Frames {
        match self {
            AnyPcm::Mono(pcm) => pcm.frames(),
            AnyPcm::Stereo(pcm) => pcm.frames(),
        }
    }

    pub fn super_frames(&self) -> SuperFrames {
        match self {
            AnyPcm::Mono(pcm) => pcm.super_frames(),
            AnyPcm::Stereo(pcm) => pcm.super_frames(),
        }
    }

    pub fn seconds(&self) -> Seconds {
        match self {
            AnyPcm::Mono(pcm) => pcm.seconds(),
            AnyPcm::Stereo(pcm) => pcm.seconds(),
        }
    }
}

#[derive(Debug)]
pub struct MonoPcm {
    data: Vec<f32>,
    sample_rate: SampleRate,
    seconds: Seconds,
    super_frames: SuperFrames,
}

impl MonoPcm {
    pub fn new(data: Vec<f32>, sample_rate: SampleRate) -> Self {
        let seconds = Frames(data.len() as u64).to_seconds(sample_rate);
        let super_frames = Frames(data.len() as u64).to_super_frames(sample_rate);

        Self { data, sample_rate, seconds, super_frames }
    }

    pub fn data(&self) -> &[f32] {
        &self.data
    }

    pub fn sample_rate(&self) -> SampleRate {
        self.sample_rate
    }

    pub fn frames(&self) -> Frames {
        Frames(self.data.len() as u64)
    }

    pub fn super_frames(&self) -> SuperFrames {
        self.super_frames
    }

    pub fn seconds(&self) -> Seconds {
        self.seconds
    }
}

#[derive(Debug)]
pub struct StereoPcm {
    left: Vec<f32>,
    right: Vec<f32>,

    sample_rate: SampleRate,
    seconds: Seconds,
    super_frames: SuperFrames,
}

impl StereoPcm {
    pub fn new(left: Vec<f32>, right: Vec<f32>, sample_rate: SampleRate) -> Self {
        assert_eq!(left.len(), right.len());

        let seconds = Frames(left.len() as u64).to_seconds(sample_rate);
        let super_frames = Frames(left.len() as u64).to_super_frames(sample_rate);

        Self { left, right, sample_rate, seconds, super_frames }
    }

    pub fn left(&self) -> &[f32] {
        &self.left
    }

    pub fn right(&self) -> &[f32] {
        &self.right
    }

    pub fn left_right(&self) -> (&[f32], &[f32]) {
        (&self.left, &self.right)
    }

    pub fn sample_rate(&self) -> SampleRate {
        self.sample_rate
    }

    pub fn frames(&self) -> Frames {
        Frames(self.left.len() as u64)
    }

    pub fn super_frames(&self) -> SuperFrames {
        self.super_frames
    }

    pub fn seconds(&self) -> Seconds {
        self.seconds
    }
}
