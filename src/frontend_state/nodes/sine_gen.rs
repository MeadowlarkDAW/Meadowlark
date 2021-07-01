use atomic_refcell::{AtomicRef, AtomicRefMut};
use basedrop::Handle;

use crate::frontend_state::{Gradient, ParamF32, ParamF32Handle, Unit};
use crate::graph_state::{
    AudioGraphNode, MonoAudioPortBuffer, ProcInfo, StereoAudioPortBuffer, MAX_BLOCKSIZE,
};

use super::{DB_GRADIENT, SMOOTH_MS};

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
        sample_rate: f32,
        coll_handle: Handle,
    ) -> (Self, StereoSineGenNodeHandle) {
        let (pitch, pitch_handle) = ParamF32::from_value(
            pitch,
            20.0,
            20_000.0,
            Gradient::Exponential,
            Unit::Generic,
            SMOOTH_MS,
            sample_rate,
            coll_handle.clone(),
        );

        let (gain_amp, gain_db_handle) = ParamF32::from_value(
            gain_db,
            min_db,
            max_db,
            DB_GRADIENT,
            Unit::Decibels,
            SMOOTH_MS,
            sample_rate,
            coll_handle,
        );

        (
            Self {
                pitch,
                gain_amp,
                sample_clock: 0.0,
            },
            StereoSineGenNodeHandle {
                pitch: pitch_handle,
                gain_db: gain_db_handle,
            },
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
        _mono_audio_in: &[AtomicRef<MonoAudioPortBuffer>],
        _mono_audio_out: &mut [AtomicRefMut<MonoAudioPortBuffer>],
        _stereo_audio_in: &[AtomicRef<StereoAudioPortBuffer>],
        stereo_audio_out: &mut [AtomicRefMut<StereoAudioPortBuffer>],
    ) {
        let pitch = self.pitch.smoothed(proc_info.frames).values;
        let gain_amp = self.gain_amp.smoothed(proc_info.frames).values;

        let dst = &mut stereo_audio_out[0];

        // This will make the compiler elid all bounds checking.
        //
        // TODO: Actually check that the compiler is eliding bounds checking
        // properly.
        let frames = proc_info.frames.min(MAX_BLOCKSIZE);

        let period = 2.0 * std::f32::consts::PI * proc_info.sample_rate_recip;
        for i in 0..frames {
            // TODO: This algorithm could be optimized.

            self.sample_clock = (self.sample_clock + 1.0) % proc_info.sample_rate;
            let smp = (self.sample_clock * pitch[i] * period).sin() * gain_amp[i];

            dst.left[i] = smp;
            dst.right[i] = smp;
        }
    }
}
