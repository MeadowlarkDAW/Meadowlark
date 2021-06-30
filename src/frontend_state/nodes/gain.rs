use atomic_refcell::{AtomicRef, AtomicRefMut};
use basedrop::Handle;

use crate::frontend_state::{ParamF32, ParamF32Handle, Unit};
use crate::graph_state::{AudioGraphNode, MonoAudioPortBuffer, ProcInfo, StereoAudioPortBuffer};

use super::{DB_GRADIENT, SMOOTH_MS};

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
        sample_rate: f32,
        coll_handle: Handle,
    ) -> (Self, GainNodeHandle) {
        let (gain_amp, gain_handle) = ParamF32::from_value(
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
            Self { gain_amp },
            GainNodeHandle {
                gain_db: gain_handle,
            },
        )
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
        mono_audio_in: &[AtomicRef<MonoAudioPortBuffer>],
        mono_audio_out: &mut [AtomicRefMut<MonoAudioPortBuffer>],
        _stereo_audio_in: &[AtomicRef<StereoAudioPortBuffer>],
        _stereo_audio_out: &mut [AtomicRefMut<StereoAudioPortBuffer>],
    ) {
        let gain_amp = self.gain_amp.smoothed(proc_info.frames);

        // TODO: Manual SIMD (to take advantage of AVX)

        let src = mono_audio_in[0].get();
        let dst = mono_audio_out[0].get_mut();

        for i in 0..proc_info.frames {
            // Safe because the scheduler calling this method ensures that all buffers
            // have the length `proc_info.frames`.
            unsafe {
                *dst.get_unchecked_mut(i) = *src.get_unchecked(i) * gain_amp[i];
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
        sample_rate: f32,
        coll_handle: Handle,
    ) -> (Self, GainNodeHandle) {
        let (gain_amp, gain_handle) = ParamF32::from_value(
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
            Self { gain_amp },
            GainNodeHandle {
                gain_db: gain_handle,
            },
        )
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
        _mono_audio_in: &[AtomicRef<MonoAudioPortBuffer>],
        _mono_audio_out: &mut [AtomicRefMut<MonoAudioPortBuffer>],
        stereo_audio_in: &[AtomicRef<StereoAudioPortBuffer>],
        stereo_audio_out: &mut [AtomicRefMut<StereoAudioPortBuffer>],
    ) {
        let gain_amp = self.gain_amp.smoothed(proc_info.frames);

        // TODO: Manual SIMD (to take advantage of AVX)

        let (src_l, src_r) = stereo_audio_in[0].left_right();
        let (dst_l, dst_r) = stereo_audio_out[0].left_right_mut();

        for i in 0..proc_info.frames {
            // Safe because the scheduler calling this method ensures that all buffers
            // have the length `proc_info.frames`.
            unsafe {
                *dst_l.get_unchecked_mut(i) = *src_l.get_unchecked(i) * gain_amp[i];
                *dst_r.get_unchecked_mut(i) = *src_r.get_unchecked(i) * gain_amp[i];
            }
        }
    }
}
