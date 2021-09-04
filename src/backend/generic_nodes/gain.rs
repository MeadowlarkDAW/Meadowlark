use rusty_daw_time::SampleRate;

use crate::backend::graph::{AudioGraphNode, ProcBuffers, ProcInfo};
use crate::backend::parameter::{ParamF32, ParamF32Handle, Unit};
use crate::backend::timeline::TimelineTransport;

use super::{DB_GRADIENT, SMOOTH_SECS};

pub struct GainNodeHandle {
    pub gain_db: ParamF32Handle,
}

pub struct MonoGainNode {
    gain_amp: ParamF32,
}

impl MonoGainNode {
    pub fn new(
        gain_db: f32,
        min_db: f32,
        max_db: f32,
        sample_rate: SampleRate,
    ) -> (Self, GainNodeHandle) {
        let (gain_amp, gain_handle) = ParamF32::from_value(
            gain_db,
            min_db,
            max_db,
            DB_GRADIENT,
            Unit::Decibels,
            SMOOTH_SECS,
            sample_rate,
        );

        (Self { gain_amp }, GainNodeHandle { gain_db: gain_handle })
    }
}

impl AudioGraphNode for MonoGainNode {
    fn mono_audio_in_ports(&self) -> usize {
        1
    }
    fn mono_audio_out_ports(&self) -> usize {
        1
    }

    fn process(
        &mut self,
        proc_info: &ProcInfo,
        _transport: &TimelineTransport,
        buffers: &mut ProcBuffers<f32>,
    ) {
        if buffers.mono_audio_in.is_empty() || buffers.mono_audio_out.is_empty() {
            // As per the spec, all unused audio output buffers must be cleared to 0.0.
            buffers.clear_audio_out_buffers(proc_info);
            return;
        }

        let frames = proc_info.frames();

        let gain_amp = self.gain_amp.smoothed(frames);

        // Won't panic because we checked these were not empty earlier.
        let src = buffers.mono_audio_in.first().unwrap();
        let dst = buffers.mono_audio_out.first_mut().unwrap();

        // TODO: SIMD

        if gain_amp.is_smoothing() {
            for i in 0..frames {
                dst.buf[i] = src.buf[i] * gain_amp[i];
            }
        } else {
            // We can optimize by using a constant gain (better SIMD load efficiency).
            let gain = gain_amp[0];

            for i in 0..frames {
                dst.buf[i] = src.buf[i] * gain;
            }
        }
    }
}

pub struct StereoGainNode {
    gain_amp: ParamF32,
}

impl StereoGainNode {
    pub fn new(
        gain_db: f32,
        min_db: f32,
        max_db: f32,
        sample_rate: SampleRate,
    ) -> (Self, GainNodeHandle) {
        let (gain_amp, gain_handle) = ParamF32::from_value(
            gain_db,
            min_db,
            max_db,
            DB_GRADIENT,
            Unit::Decibels,
            SMOOTH_SECS,
            sample_rate,
        );

        (Self { gain_amp }, GainNodeHandle { gain_db: gain_handle })
    }
}

impl AudioGraphNode for StereoGainNode {
    fn stereo_audio_in_ports(&self) -> usize {
        1
    }
    fn stereo_audio_out_ports(&self) -> usize {
        1
    }

    fn process(
        &mut self,
        proc_info: &ProcInfo,
        _transport: &TimelineTransport,
        buffers: &mut ProcBuffers<f32>,
    ) {
        if buffers.stereo_audio_in.is_empty() || buffers.stereo_audio_out.is_empty() {
            // As per the spec, all unused audio output buffers must be cleared to 0.0.
            buffers.clear_audio_out_buffers(proc_info);
            return;
        }

        let frames = proc_info.frames();

        let gain_amp = self.gain_amp.smoothed(frames);

        // Won't panic because we checked these were not empty earlier.
        let src = buffers.stereo_audio_in.first().unwrap();
        let dst = buffers.stereo_audio_out.first_mut().unwrap();

        // TODO: SIMD

        if gain_amp.is_smoothing() {
            for i in 0..frames {
                dst.left[i] = src.left[i] * gain_amp[i];
                dst.right[i] = src.right[i] * gain_amp[i];
            }
        } else {
            // We can optimize by using a constant gain (better SIMD load efficiency).
            let gain = gain_amp[0];

            for i in 0..frames {
                dst.left[i] = src.left[i] * gain;
                dst.right[i] = src.right[i] * gain;
            }
        }
    }
}
