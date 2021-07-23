use atomic_refcell::{AtomicRef, AtomicRefMut};
use basedrop::Handle;

use crate::backend::graph_interface::{
    AudioGraphNode, MonoAudioPortBuffer, ProcInfo, StereoAudioPortBuffer,
};
use crate::backend::timeline::TimelineTransport;
use crate::backend::{
    cpu_id,
    parameter::{ParamF32, ParamF32Handle, Unit},
    MAX_BLOCKSIZE,
};

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
        _transport: &TimelineTransport,
        mono_audio_in: &[AtomicRef<MonoAudioPortBuffer>],
        mono_audio_out: &mut [AtomicRefMut<MonoAudioPortBuffer>],
        _stereo_audio_in: &[AtomicRef<StereoAudioPortBuffer>],
        _stereo_audio_out: &mut [AtomicRefMut<StereoAudioPortBuffer>],
    ) {
        let gain_amp = self.gain_amp.smoothed(proc_info.frames);

        let src = &mono_audio_in[0];
        let dst = &mut mono_audio_out[0];

        #[cfg(all(any(target_arch = "x86", target_arch = "x86_64")))]
        {
            if cpu_id::has_avx() {
                // Safe because we checked that the cpu has avx.
                unsafe {
                    simd::mono_gain_avx(proc_info.frames, src, dst, &gain_amp);
                }
                return;
            }
        }

        // Fallback

        // This will make the compiler elid all bounds checking.
        //
        // TODO: Actually check that the compiler is eliding bounds checking
        // properly.
        let frames = proc_info.frames.min(MAX_BLOCKSIZE);

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
        _transport: &TimelineTransport,
        _mono_audio_in: &[AtomicRef<MonoAudioPortBuffer>],
        _mono_audio_out: &mut [AtomicRefMut<MonoAudioPortBuffer>],
        stereo_audio_in: &[AtomicRef<StereoAudioPortBuffer>],
        stereo_audio_out: &mut [AtomicRefMut<StereoAudioPortBuffer>],
    ) {
        let gain_amp = self.gain_amp.smoothed(proc_info.frames);

        let src = &stereo_audio_in[0];
        let dst = &mut stereo_audio_out[0];

        #[cfg(all(any(target_arch = "x86", target_arch = "x86_64")))]
        {
            if cpu_id::has_avx() {
                // Safe because we checked that the cpu has avx.
                unsafe {
                    simd::stereo_gain_avx(proc_info.frames, src, dst, &gain_amp);
                }
                return;
            }
        }

        // Fallback

        // This will make the compiler elid all bounds checking.
        //
        // TODO: Actually check that the compiler is eliding bounds checking
        // properly.
        let frames = proc_info.frames.min(MAX_BLOCKSIZE);

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

mod simd {
    // Using manual SIMD on such a simple algorithm is probably unecessary, but I'm including it
    // here anyway as an example on how to acheive uber-optimized manual SIMD for future nodes.

    use super::{MonoAudioPortBuffer, StereoAudioPortBuffer, MAX_BLOCKSIZE};
    use crate::backend::{cpu_id, parameter::SmoothOutput};

    #[cfg(all(any(target_arch = "x86", target_arch = "x86_64")))]
    #[target_feature(enable = "avx")]
    pub unsafe fn mono_gain_avx(
        frames: usize,
        src: &MonoAudioPortBuffer,
        dst: &mut MonoAudioPortBuffer,
        gain_amp: &SmoothOutput<f32>,
    ) {
        #[cfg(target_arch = "x86")]
        use std::arch::x86::*;
        #[cfg(target_arch = "x86_64")]
        use std::arch::x86_64::*;

        // This will make the compiler elid all bounds checking.
        //
        // TODO: Actually check that the compiler is eliding bounds checking
        // properly.
        let frames = frames.min(MAX_BLOCKSIZE);

        if gain_amp.is_smoothing() {
            // Looping like this will cause this to process in chunks of cpu_id::AVX_F32_WIDTH.
            //
            // Even if the number of `frames` is not a multiple of cpu_id::AVX_F32_WIDTH, it
            // is more efficient to process it as a block anyway. It doesn't matter if the last block
            // contains uninitialized data because we won't read it anyway.
            for i in (0..frames).step_by(cpu_id::AVX_F32_WIDTH) {
                let src_v = _mm256_loadu_ps(&src.buf[i]);
                let gain_v = _mm256_loadu_ps(&gain_amp[i]);

                let mul_v = _mm256_mul_ps(src_v, gain_v);

                _mm256_storeu_ps(&mut dst.buf[i], mul_v);
            }
        } else {
            // We can optimize by using a constant gain (better SIMD load efficiency).
            let gain_v = _mm256_set1_ps(gain_amp.values[0]);

            // Looping like this will cause this to process in chunks of cpu_id::AVX_F32_WIDTH.
            //
            // Even if the number of `frames` is not a multiple of cpu_id::AVX_F32_WIDTH, it
            // is more efficient to process it as a block anyway. It doesn't matter if the last block
            // contains uninitialized data because we won't read it anyway.
            for i in (0..frames).step_by(cpu_id::AVX_F32_WIDTH) {
                let src_v = _mm256_loadu_ps(&src.buf[i]);
                let mul_v = _mm256_mul_ps(src_v, gain_v);

                _mm256_storeu_ps(&mut dst.buf[i], mul_v);
            }
        }
    }

    #[cfg(all(any(target_arch = "x86", target_arch = "x86_64")))]
    #[target_feature(enable = "avx")]
    pub unsafe fn stereo_gain_avx(
        frames: usize,
        src: &StereoAudioPortBuffer,
        dst: &mut StereoAudioPortBuffer,
        gain_amp: &SmoothOutput<f32>,
    ) {
        #[cfg(target_arch = "x86")]
        use std::arch::x86::*;
        #[cfg(target_arch = "x86_64")]
        use std::arch::x86_64::*;

        // This will make the compiler elid all bounds checking.
        //
        // TODO: Actually check that the compiler is eliding bounds checking
        // properly.
        let frames = frames.min(MAX_BLOCKSIZE);

        if gain_amp.is_smoothing() {
            // Looping like this will cause this to process in chunks of cpu_id::AVX_F32_WIDTH.
            //
            // Even if the number of `frames` is not a multiple of cpu_id::AVX_F32_WIDTH, it
            // is more efficient to process it as a block anyway. It doesn't matter if the last block
            // contains uninitialized data because we won't read it anyway.
            for i in (0..frames).step_by(cpu_id::AVX_F32_WIDTH) {
                let src_left_v = _mm256_loadu_ps(&src.left[i]);
                let src_right_v = _mm256_loadu_ps(&src.right[i]);

                let gain_v = _mm256_loadu_ps(&gain_amp[i]);

                let mul_left_v = _mm256_mul_ps(src_left_v, gain_v);
                let mul_right_v = _mm256_mul_ps(src_right_v, gain_v);

                _mm256_storeu_ps(&mut dst.left[i], mul_left_v);
                _mm256_storeu_ps(&mut dst.right[i], mul_right_v);
            }
        } else {
            // We can optimize by using a constant gain (better SIMD load efficiency).
            let gain_v = _mm256_set1_ps(gain_amp.values[0]);

            // Looping like this will cause this to process in chunks of cpu_id::AVX_F32_WIDTH.
            //
            // Even if the number of `frames` is not a multiple of cpu_id::AVX_F32_WIDTH, it
            // is more efficient to process it as a block anyway. It doesn't matter if the last block
            // contains uninitialized data because we won't read it anyway.
            for i in (0..frames).step_by(cpu_id::AVX_F32_WIDTH) {
                let src_left_v = _mm256_loadu_ps(&src.left[i]);
                let src_right_v = _mm256_loadu_ps(&src.right[i]);

                let mul_left_v = _mm256_mul_ps(src_left_v, gain_v);
                let mul_right_v = _mm256_mul_ps(src_right_v, gain_v);

                _mm256_storeu_ps(&mut dst.left[i], mul_left_v);
                _mm256_storeu_ps(&mut dst.right[i], mul_right_v);
            }
        }
    }
}
