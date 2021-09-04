use rusty_daw_time::SampleRate;

use crate::backend::graph::{AudioGraphNode, ProcBuffers, ProcInfo};
use crate::backend::parameter::{Gradient, ParamF32, ParamF32Handle, Unit};
use crate::backend::timeline::TimelineTransport;

use super::{DB_GRADIENT, SMOOTH_SECS};

pub struct StereoSineGenNodeHandle {
    pub pitch: ParamF32Handle,
    pub gain_db: ParamF32Handle,
}

pub struct StereoSineGenNode {
    pitch: ParamF32,
    gain_amp: ParamF32,

    sample_clock: f32,
}

impl StereoSineGenNode {
    pub fn new(
        pitch: f32,
        gain_db: f32,
        min_db: f32,
        max_db: f32,
        sample_rate: SampleRate,
    ) -> (Self, StereoSineGenNodeHandle) {
        let (pitch, pitch_handle) = ParamF32::from_value(
            pitch,
            20.0,
            20_000.0,
            Gradient::Exponential,
            Unit::Generic,
            SMOOTH_SECS,
            sample_rate,
        );

        let (gain_amp, gain_db_handle) = ParamF32::from_value(
            gain_db,
            min_db,
            max_db,
            DB_GRADIENT,
            Unit::Decibels,
            SMOOTH_SECS,
            sample_rate,
        );

        (
            Self { pitch, gain_amp, sample_clock: 0.0 },
            StereoSineGenNodeHandle { pitch: pitch_handle, gain_db: gain_db_handle },
        )
    }
}

impl AudioGraphNode for StereoSineGenNode {
    fn stereo_audio_out_ports(&self) -> usize {
        1
    }

    fn process(
        &mut self,
        proc_info: &ProcInfo,
        _transport: &TimelineTransport,
        buffers: &mut ProcBuffers<f32>,
    ) {
        if let Some(mut dst) = buffers.stereo_audio_out.first_mut() {
            let frames = proc_info.frames();

            let pitch = self.pitch.smoothed(frames).values;
            let gain_amp = self.gain_amp.smoothed(frames).values;

            let sr = proc_info.sample_rate.0 as f32;

            let period = 2.0 * std::f32::consts::PI * proc_info.sample_rate_recip as f32;
            for i in 0..frames {
                // TODO: This algorithm could be greatly optimized.

                self.sample_clock = (self.sample_clock + 1.0) % sr;
                let smp = (self.sample_clock * pitch[i] * period).sin() * gain_amp[i];

                dst.left[i] = smp;
                dst.right[i] = smp;
            }
        }
    }
}
