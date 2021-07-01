use atomic_refcell::{AtomicRef, AtomicRefMut};
use basedrop::Handle;

use crate::frontend_state::{ParamF32, ParamF32Handle, Unit};
use crate::graph_state::{
    AudioGraphNode, MonoAudioPortBuffer, ProcInfo, StereoAudioPortBuffer, MAX_BLOCKSIZE,
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
        mono_audio_in: &[AtomicRef<MonoAudioPortBuffer>],
        mono_audio_out: &mut [AtomicRefMut<MonoAudioPortBuffer>],
        _stereo_audio_in: &[AtomicRef<StereoAudioPortBuffer>],
        _stereo_audio_out: &mut [AtomicRefMut<StereoAudioPortBuffer>],
    ) {
        let gain_amp = self.gain_amp.smoothed(proc_info.frames).values;

        let src = &mono_audio_in[0];
        let dst = &mut mono_audio_out[0];

        #[cfg(all(any(target_arch = "x86", target_arch = "x86_64")))]
        {
            if crate::cpu_id::has_avx() {
                // Safe because we checked that the cpu has avx.
                unsafe {
                    simd::mono_gain_avx(proc_info.frames, src, dst, gain_amp);
                }
                return;
            }
        }

        // This will make the compiler elid all bounds checking.
        //
        // TODO: Actually check that the compiler is eliding bounds checking
        // properly.
        let frames = proc_info.frames.min(MAX_BLOCKSIZE);

        // Fallback
        for i in 0..frames {
            dst.buf[i] = src.buf[i] * gain_amp[i];
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
        let gain_amp = self.gain_amp.smoothed(proc_info.frames).values;

        let src = &stereo_audio_in[0];
        let dst = &mut stereo_audio_out[0];

        #[cfg(all(any(target_arch = "x86", target_arch = "x86_64")))]
        {
            if crate::cpu_id::has_avx() {
                // Safe because we checked that the cpu has avx.
                unsafe {
                    simd::stereo_gain_avx(proc_info.frames, src, dst, gain_amp);
                }
                return;
            }
        }

        // This will make the compiler elid all bounds checking.
        //
        // TODO: Actually check that the compiler is eliding bounds checking
        // properly.
        let frames = proc_info.frames.min(MAX_BLOCKSIZE);

        // Fallback
        for i in 0..frames {
            dst.left[i] = src.left[i] * gain_amp[i];
            dst.right[i] = src.right[i] * gain_amp[i];
        }
    }
}

mod simd {
    // Using manual SIMD on such a simple algorithm is probably unecessary, but I'm including it
    // here anyway as an example on how to acheive uber-optimized manual SIMD for future nodes.

    use super::{MonoAudioPortBuffer, StereoAudioPortBuffer, MAX_BLOCKSIZE};
    use crate::cpu_id;

    #[cfg(all(any(target_arch = "x86", target_arch = "x86_64")))]
    #[target_feature(enable = "avx")]
    pub unsafe fn mono_gain_avx(
        frames: usize,
        src: &MonoAudioPortBuffer,
        dst: &mut MonoAudioPortBuffer,
        gain_amp: &[f32; MAX_BLOCKSIZE],
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
    }

    #[cfg(all(any(target_arch = "x86", target_arch = "x86_64")))]
    #[target_feature(enable = "avx")]
    pub unsafe fn stereo_gain_avx(
        frames: usize,
        src: &StereoAudioPortBuffer,
        dst: &mut StereoAudioPortBuffer,
        gain_amp: &[f32; MAX_BLOCKSIZE],
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

        // Looping like this will cause this to process in chunks of cpu_id::AVX_F32_WIDTH.
        //
        // Even if the number of `frames` is not a multiple of cpu_id::AVX_F32_WIDTH, it
        // is more efficient to process it as a block anyway. It doesn't matter if the last block
        // contains uninitialized data because we won't read it anyway.
        for i in (0..frames).step_by(cpu_id::AVX_F32_WIDTH) {
            let src_l_v = _mm256_loadu_ps(&src.left[i]);
            let src_r_v = _mm256_loadu_ps(&src.right[i]);
            let gain_v = _mm256_loadu_ps(&gain_amp[i]);

            let mul_l_v = _mm256_mul_ps(src_l_v, gain_v);
            let mul_r_v = _mm256_mul_ps(src_r_v, gain_v);

            _mm256_storeu_ps(&mut dst.left[i], mul_l_v);
            _mm256_storeu_ps(&mut dst.right[i], mul_r_v);
        }
    }
}
