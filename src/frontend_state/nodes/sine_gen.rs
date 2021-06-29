use atomic_refcell::{AtomicRef, AtomicRefMut};
use basedrop::Handle;

use crate::frontend_state::{Gradient, Param, ParamHandle, ParamType, Unit};
use crate::graph_state::{AudioGraphNode, MonoAudioPortBuffer, ProcInfo, StereoAudioPortBuffer};

use super::{DB_GRADIENT, SMOOTH_MS};

pub struct StereoSineGenNodeHandle {
    pub pitch: ParamHandle,
    pub gain_db: ParamHandle,
}

pub struct StereoSineGenNode {
    pitch: Param,
    gain_amp: Param,

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
        let (pitch, pitch_handle) = Param::from_value(
            ParamType::Numeric {
                min: 20.0,
                max: 20_000.0,
                gradient: Gradient::Exponential,
            },
            Unit::Generic,
            pitch,
            SMOOTH_MS,
            sample_rate,
            coll_handle.clone(),
        );

        let (gain_amp, gain_db_handle) = Param::from_value(
            ParamType::Numeric {
                min: min_db,
                max: max_db,
                gradient: DB_GRADIENT,
            },
            Unit::Decibels,
            gain_db,
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
        let pitch = self.pitch.smoothed(proc_info.frames);
        let gain_amp = self.gain_amp.smoothed(proc_info.frames);

        let (dst_l, dst_r) = stereo_audio_out[0].left_right_mut();

        let period = 2.0 * std::f32::consts::PI * proc_info.sample_rate_recip;
        for i in 0..proc_info.frames {
            // TODO: This algorithm could be optimized.

            self.sample_clock = (self.sample_clock + 1.0) % proc_info.sample_rate;
            let smp = (self.sample_clock * pitch[i] * period).sin() * gain_amp[i];

            // Safe because the scheduler calling this method ensures that all buffers
            // have the length `proc_info.frames`.
            unsafe {
                *dst_l.get_unchecked_mut(i) = smp;
                *dst_r.get_unchecked_mut(i) = smp;
            }
        }
    }
}
