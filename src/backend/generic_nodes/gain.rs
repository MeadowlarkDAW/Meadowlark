use rusty_daw_time::SampleRate;

use crate::backend::graph::{clear_audio_outputs, AudioBlockBuffer, AudioGraphNode, ProcInfo};
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
    fn audio_in_ports(&self) -> usize {
        1
    }
    fn audio_out_ports(&self) -> usize {
        1
    }

    fn process(
        &mut self,
        proc_info: &ProcInfo,
        _transport: &TimelineTransport,
        audio_in: &[AudioBlockBuffer<f32>],
        audio_out: &mut [AudioBlockBuffer<f32>],
    ) {
        if audio_in.is_empty() || audio_out.is_empty() {
            // As per the spec, all unused audio output buffers must be cleared to 0.0.
            clear_audio_outputs(audio_out, proc_info);
            return;
        }

        let frames = proc_info.frames();

        let gain_amp = self.gain_amp.smoothed(frames);

        let src = &audio_in[0];
        let dst = &mut audio_out[0];

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
    fn audio_in_ports(&self) -> usize {
        2
    }
    fn audio_out_ports(&self) -> usize {
        2
    }

    fn process(
        &mut self,
        proc_info: &ProcInfo,
        _transport: &TimelineTransport,
        audio_in: &[AudioBlockBuffer<f32>],
        audio_out: &mut [AudioBlockBuffer<f32>],
    ) {
        // Assume the host always connects ports in a stereo pair together.
        if audio_in.len() < 2 || audio_out.len() < 2 {
            // As per the spec, all unused audio output buffers must be cleared to 0.0.
            clear_audio_outputs(audio_out, proc_info);
            return;
        }

        let frames = proc_info.frames();

        let gain_amp = self.gain_amp.smoothed(frames);

        let src_left = &audio_in[0];
        let src_right = &audio_in[1];
        let dst_left = &mut audio_out[0];
        let dst_right = &mut audio_out[1];

        // TODO: SIMD

        if gain_amp.is_smoothing() {
            for i in 0..frames {
                dst_left[i] = src_left[i] * gain_amp[i];
                dst_right[i] = src_right[i] * gain_amp[i];
            }
        } else {
            // We can optimize by using a constant gain (better SIMD load efficiency).
            let gain = gain_amp[0];

            for i in 0..frames {
                dst_left[i] = src_left[i] * gain;
                dst_right[i] = src_right[i] * gain;
            }
        }
    }
}
