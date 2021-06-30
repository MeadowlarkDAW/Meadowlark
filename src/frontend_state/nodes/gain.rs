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
        let gain_amp = self.gain_amp.smoothed(proc_info.frames).values;

        let src = mono_audio_in[0].get();
        let dst = mono_audio_out[0].get_mut();

        #[cfg(all(any(target_arch = "x86", target_arch = "x86_64")))]
        {
            if crate::cpu_id::has_avx() {
                // Safe because we checked that the cpu has avx.
                unsafe {
                    simd::mono_gain_avx(src, dst, gain_amp);
                }
                return;
            }
        }

        // Fallback
        for i in 0..proc_info.frames {
            // Safe because the scheduler calling this method ensures that all buffers
            // have the length `proc_info.frames`.
            unsafe {
                *dst.get_unchecked_mut(i) = *src.get_unchecked(i) * gain_amp.get_unchecked(i);
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
        let gain_amp = self.gain_amp.smoothed(proc_info.frames).values;

        let (src_l, src_r) = stereo_audio_in[0].left_right();
        let (dst_l, dst_r) = stereo_audio_out[0].left_right_mut();

        #[cfg(all(any(target_arch = "x86", target_arch = "x86_64")))]
        {
            if crate::cpu_id::has_avx() {
                // Safe because we checked that the cpu has avx.
                unsafe {
                    simd::stereo_gain_avx(src_l, src_r, dst_l, dst_r, gain_amp);
                }
                return;
            }
        }

        // Fallback
        for i in 0..proc_info.frames {
            // Safe because the scheduler calling this method ensures that all buffers
            // have the length `proc_info.frames`.
            unsafe {
                *dst_l.get_unchecked_mut(i) = *src_l.get_unchecked(i) * gain_amp.get_unchecked(i);
                *dst_r.get_unchecked_mut(i) = *src_r.get_unchecked(i) * gain_amp.get_unchecked(i);
            }
        }
    }
}

mod simd {
    // Using manual SIMD on such a simple algorithm is probably unecessary, but I'm including it
    // here anyway as an example on how to acheive uber-optimized manual SIMD for future nodes.

    use crate::cpu_id;

    #[cfg(all(any(target_arch = "x86", target_arch = "x86_64")))]
    #[target_feature(enable = "avx")]
    pub unsafe fn mono_gain_avx(mut src: &[f32], mut dst: &mut [f32], mut gain: &[f32]) {
        #[cfg(target_arch = "x86")]
        use std::arch::x86::*;
        #[cfg(target_arch = "x86_64")]
        use std::arch::x86_64::*;

        // Hints to compiler that we want to elid all bounds checking on buffers for optimal
        // looping performance.
        //
        // This is safe because the scheduler upholds that all buffers are the same length.
        dst = std::slice::from_raw_parts_mut(dst.as_mut_ptr(), src.len());
        gain = std::slice::from_raw_parts(gain.as_ptr(), src.len());

        while src.len() >= cpu_id::AVX_F32_WIDTH {
            let src_v = _mm256_loadu_ps(src.as_ptr());
            let gain_v = _mm256_loadu_ps(gain.as_ptr());

            let mul_v = _mm256_mul_ps(src_v, gain_v);

            _mm256_storeu_ps(dst.as_mut_ptr(), mul_v);

            src = &src[cpu_id::AVX_F32_WIDTH..];
            dst = &mut dst[cpu_id::AVX_F32_WIDTH..];
            gain = &gain[cpu_id::AVX_F32_WIDTH..];
        }

        // Compute any remaining elements.
        for i in 0..src.len() {
            // This is safe because the scheduler upholds that all buffers are the same length.
            *dst.get_unchecked_mut(i) = *src.get_unchecked(i) * gain.get_unchecked(i);
        }
    }

    #[cfg(all(any(target_arch = "x86", target_arch = "x86_64")))]
    #[target_feature(enable = "avx")]
    pub unsafe fn stereo_gain_avx(
        mut src_l: &[f32],
        mut src_r: &[f32],
        mut dst_l: &mut [f32],
        mut dst_r: &mut [f32],
        mut gain: &[f32],
    ) {
        #[cfg(target_arch = "x86")]
        use std::arch::x86::*;
        #[cfg(target_arch = "x86_64")]
        use std::arch::x86_64::*;

        // Hints to compiler that we want to elid all bounds checking on buffers for optimal
        // looping performance.
        //
        // This is safe because the scheduler upholds that all buffers are the same length.
        src_r = std::slice::from_raw_parts(src_r.as_ptr(), src_l.len());
        dst_l = std::slice::from_raw_parts_mut(dst_l.as_mut_ptr(), src_l.len());
        dst_r = std::slice::from_raw_parts_mut(dst_r.as_mut_ptr(), src_l.len());
        gain = std::slice::from_raw_parts(gain.as_ptr(), src_l.len());

        while src_l.len() >= cpu_id::AVX_F32_WIDTH {
            let src_l_v = _mm256_loadu_ps(src_l.as_ptr());
            let src_r_v = _mm256_loadu_ps(src_r.as_ptr());
            let gain_v = _mm256_loadu_ps(gain.as_ptr());

            let mul_l_v = _mm256_mul_ps(src_l_v, gain_v);
            let mul_r_v = _mm256_mul_ps(src_r_v, gain_v);

            _mm256_storeu_ps(dst_l.as_mut_ptr(), mul_l_v);
            _mm256_storeu_ps(dst_r.as_mut_ptr(), mul_r_v);

            src_l = &src_l[cpu_id::AVX_F32_WIDTH..];
            src_r = &src_r[cpu_id::AVX_F32_WIDTH..];
            dst_l = &mut dst_l[cpu_id::AVX_F32_WIDTH..];
            dst_r = &mut dst_r[cpu_id::AVX_F32_WIDTH..];
            gain = &gain[cpu_id::AVX_F32_WIDTH..];
        }

        // Compute any remaining elements.
        for i in 0..src_l.len() {
            // This is safe because the scheduler upholds that all buffers are the same length.
            *dst_l.get_unchecked_mut(i) = *src_l.get_unchecked(i) * gain.get_unchecked(i);
            *dst_r.get_unchecked_mut(i) = *src_r.get_unchecked(i) * gain.get_unchecked(i);
        }
    }
}
