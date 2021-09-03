// Some modified code from baseplug:
//
// https://github.com/wrl/baseplug/blob/trunk/src/smooth.rs
// https://github.com/wrl/baseplug/blob/trunk/LICENSE-APACHE
// https://github.com/wrl/baseplug/blob/trunk/LICENSE-MIT
//
//  Thanks wrl! :)

use std::fmt;
use std::ops;
use std::slice;

use num_traits::Float;
use rusty_daw_time::SampleRate;
use rusty_daw_time::Seconds;

use crate::backend::cpu_id;
use crate::backend::graph::{MonoAudioBlockBuffer, StereoAudioBlockBuffer};
use crate::backend::MAX_BLOCKSIZE;

const SETTLE: f32 = 0.0001f32;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum SmoothStatus {
    Inactive,
    Active,
    Deactivating,
}

impl SmoothStatus {
    #[inline]
    fn is_active(&self) -> bool {
        self != &SmoothStatus::Inactive
    }
}

pub struct SmoothOutput<'a, T> {
    pub values: &'a [T; MAX_BLOCKSIZE],
    pub status: SmoothStatus,
}

impl<'a, T> SmoothOutput<'a, T> {
    #[inline]
    pub fn is_smoothing(&self) -> bool {
        self.status.is_active()
    }
}

impl<'a, I, T> ops::Index<I> for SmoothOutput<'a, T>
where
    I: slice::SliceIndex<[T]>,
{
    type Output = I::Output;

    #[inline]
    fn index(&self, idx: I) -> &I::Output {
        &self.values[idx]
    }
}

pub struct Smooth<T: Float> {
    output: [T; MAX_BLOCKSIZE],
    input: T,

    status: SmoothStatus,

    a: T,
    b: T,
    last_output: T,
}

impl<T> Smooth<T>
where
    T: Float + fmt::Display,
{
    pub fn new(input: T) -> Self {
        Self {
            status: SmoothStatus::Inactive,
            input,
            output: [input; MAX_BLOCKSIZE],

            a: T::one(),
            b: T::zero(),
            last_output: input,
        }
    }

    pub fn reset(&mut self, val: T) {
        *self = Self { a: self.a, b: self.b, ..Self::new(val) };
    }

    pub fn set(&mut self, val: T) {
        self.input = val;
        self.status = SmoothStatus::Active;
    }

    #[inline]
    pub fn dest(&self) -> T {
        self.input
    }

    #[inline]
    pub fn output(&self) -> SmoothOutput<T> {
        SmoothOutput { values: &self.output, status: self.status }
    }

    #[inline]
    pub fn current_value(&self) -> (T, SmoothStatus) {
        (self.last_output, self.status)
    }

    pub fn update_status_with_epsilon(&mut self, epsilon: T) -> SmoothStatus {
        let status = self.status;

        match status {
            SmoothStatus::Active => {
                if (self.input - self.output[0]).abs() < epsilon {
                    self.reset(self.input);
                    self.status = SmoothStatus::Deactivating;
                }
            }

            SmoothStatus::Deactivating => self.status = SmoothStatus::Inactive,

            _ => (),
        };

        self.status
    }

    pub fn process(&mut self, nframes: usize) {
        if self.status != SmoothStatus::Active {
            return;
        }

        let nframes = nframes.min(MAX_BLOCKSIZE);
        let input = self.input * self.a;

        self.output[0] = input + (self.last_output * self.b);

        for i in 1..nframes {
            self.output[i] = input + (self.output[i - 1] * self.b);
        }

        self.last_output = self.output[nframes - 1];
    }

    #[inline]
    pub fn is_active(&self) -> bool {
        self.status.is_active()
    }
}

impl Smooth<f32> {
    pub fn set_speed(&mut self, sample_rate: SampleRate, seconds: Seconds) {
        self.b = (-1.0f32 / (seconds.0 as f32 * sample_rate.0 as f32)).exp();
        self.a = 1.0f32 - self.b;
    }

    #[inline]
    pub fn update_status(&mut self) -> SmoothStatus {
        self.update_status_with_epsilon(SETTLE)
    }
}

impl<T> From<T> for Smooth<T>
where
    T: Float + fmt::Display,
{
    fn from(val: T) -> Self {
        Self::new(val)
    }
}

impl<T> fmt::Debug for Smooth<T>
where
    T: Float + fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct(concat!("Smooth<", stringify!(T), ">"))
            .field("output[0]", &self.output[0])
            .field("input", &self.input)
            .field("status", &self.status)
            .field("last_output", &self.last_output)
            .finish()
    }
}

impl<'a> SmoothOutput<'a, f32> {
    /// Multiplies each value in the given `buf` by the corresponding value in this smoothed output.
    ///
    /// # Safety
    ///
    /// Data in `buf` after `frames` may be mutated if `frames` is not a multiple of the width of the used SIMD register
    /// (4 `f32`s for SSE2, 8 `f32`s for AVX, etc.) due to optimized looping. Therefore, you must ensure that no
    /// potentially uninitialized data after `frames` is read.
    #[inline]
    pub fn optimized_multiply_mono(&self, buf: &mut MonoAudioBlockBuffer, frames: usize) {
        let is_smoothing = self.is_smoothing();

        if !is_smoothing && self.values[0] == 1.0 {
            // Nothing to do.
            return;
        }

        #[cfg(all(any(target_arch = "x86", target_arch = "x86_64")))]
        {
            if cpu_id::has_avx() {
                // Safe because we checked that the cpu has avx.
                unsafe {
                    simd::optimized_multiply_mono_avx(buf, self.values, frames, is_smoothing);
                }
                return;
            }
        }

        simd::optimized_multiply_mono_fallback(buf, self.values, frames, is_smoothing);
    }

    /// Multiplies each value in the given `buf` buffer by the corresponding value in this smoothed output.
    ///
    /// # Safety
    ///
    /// Data in `buf` after `frames` may be mutated if `frames` is not a multiple of the width of the used
    /// SIMD register (4 `f32`s for SSE2, 8 `f32`s for AVX, etc.) due to optimized looping. Therefore, you must ensure
    /// that no potentially uninitialized data after `frames` is read.
    #[inline]
    pub fn optimized_multiply_stereo(&self, buf: &mut StereoAudioBlockBuffer, frames: usize) {
        let is_smoothing = self.is_smoothing();

        if !is_smoothing && self.values[0] == 1.0 {
            // Nothing to do.
            return;
        }

        #[cfg(all(any(target_arch = "x86", target_arch = "x86_64")))]
        {
            if cpu_id::has_avx() {
                // Safe because we checked that the cpu has avx.
                unsafe {
                    simd::optimized_multiply_stereo_avx(buf, self.values, frames, is_smoothing);
                }
                return;
            }
        }

        simd::optimized_multiply_stereo_fallback(buf, self.values, frames, is_smoothing);
    }

    /// Multiplies each value in the given `buf` buffer by the corresponding value in this smoothed output.
    ///
    /// This is the same as `optimized_multiply_stereo`, except that the buffer `buf` will start
    /// at `offset`, while this smoothed output will start at 0 (as opposed to all buffers starting at 0).
    ///
    /// # Safety
    ///
    /// Data in `buf` after `frames` may be mutated if `frames` is not a multiple of the width of the used
    /// SIMD register (4 `f32`s for SSE2, 8 `f32`s for AVX, etc.) due to optimized looping. Therefore, you must ensure
    /// that no potentially uninitialized data after `frames` is read.
    #[inline]
    pub fn optimized_multiply_offset_stereo(
        &self,
        buf: &mut StereoAudioBlockBuffer,
        frames: usize,
        offset: usize,
    ) {
        let is_smoothing = self.is_smoothing();

        if !is_smoothing && self.values[0] == 1.0 {
            // Nothing to do.
            return;
        }

        #[cfg(all(any(target_arch = "x86", target_arch = "x86_64")))]
        {
            if cpu_id::has_avx() {
                // Safe because we checked that the cpu has avx.
                unsafe {
                    simd::optimized_multiply_offset_stereo_avx(
                        buf,
                        self.values,
                        frames,
                        offset,
                        is_smoothing,
                    );
                }
                return;
            }
        }

        simd::optimized_multiply_offset_stereo_fallback(
            buf,
            self.values,
            frames,
            offset,
            is_smoothing,
        );
    }

    /// Multiplies each value in the given `src` buffer by the corresponding value in this smoothed output, and then adds
    /// that value to the corresponding value in the `dst` buffer.
    ///
    /// # Safety
    ///
    /// Data in `dst` after `frames` may be mutated if `frames` is not a multiple of the width of the used
    /// SIMD register (4 `f32`s for SSE2, 8 `f32`s for AVX, etc.) due to optimized looping. Therefore, you must ensure
    /// that no potentially uninitialized data after `frames` is read.
    #[inline]
    pub fn optimized_multiply_then_add_stereo(
        &self,
        src: &StereoAudioBlockBuffer,
        dst: &mut StereoAudioBlockBuffer,
        frames: usize,
    ) {
        let is_smoothing = self.is_smoothing();

        #[cfg(all(any(target_arch = "x86", target_arch = "x86_64")))]
        {
            if cpu_id::has_avx() {
                // Safe because we checked that the cpu has avx.
                unsafe {
                    simd::optimized_multiply_then_add_stereo_avx(
                        src,
                        dst,
                        self.values,
                        frames,
                        is_smoothing,
                    );
                }
                return;
            }
        }

        simd::optimized_multiply_then_add_stereo_fallback(
            src,
            dst,
            self.values,
            frames,
            is_smoothing,
        );
    }

    /// Multiplies each value in the given `src` buffer by the corresponding value in this smoothed output, and then adds
    /// that value to the corresponding value in the `dst` buffer.
    ///
    /// This is the same as `optimized_multiply_then_add_stereo`, except that the buffers `src` and `dst` will start
    /// at `offset`, while `gain` will start at 0 (as opposed to all buffers starting at 0).
    ///
    /// # Safety
    ///
    /// Data in `dst` after `frames` may be mutated if `frames` is not a multiple of the width of the used
    /// SIMD register (4 `f32`s for SSE2, 8 `f32`s for AVX, etc.) due to optimized looping. Therefore, you must ensure
    /// that no potentially uninitialized data after `frames` is read.
    #[inline]
    pub fn optimized_multiply_then_add_offset_stereo(
        &self,
        src: &StereoAudioBlockBuffer,
        dst: &mut StereoAudioBlockBuffer,
        frames: usize,
        offset: usize,
    ) {
        let is_smoothing = self.is_smoothing();

        #[cfg(all(any(target_arch = "x86", target_arch = "x86_64")))]
        {
            if cpu_id::has_avx() {
                // Safe because we checked that the cpu has avx.
                unsafe {
                    simd::optimized_multiply_then_add_offset_stereo_avx(
                        src,
                        dst,
                        self.values,
                        frames,
                        offset,
                        is_smoothing,
                    );
                }
                return;
            }
        }

        simd::optimized_multiply_then_add_offset_stereo_fallback(
            src,
            dst,
            self.values,
            frames,
            offset,
            is_smoothing,
        );
    }
}

mod simd {
    use crate::backend::{
        cpu_id,
        graph::{MonoAudioBlockBuffer, StereoAudioBlockBuffer},
        MAX_BLOCKSIZE,
    };

    pub fn optimized_multiply_mono_fallback(
        buf: &mut MonoAudioBlockBuffer,
        gain: &[f32; MAX_BLOCKSIZE],
        frames: usize,
        is_smoothing: bool,
    ) {
        // Hint to compiler to optimize loops.
        let frames = frames.min(MAX_BLOCKSIZE);

        if is_smoothing {
            for i in 0..frames {
                buf.buf[i] *= gain[i];
            }
        } else {
            // We can optimize by using a constant gain (better SIMD load efficiency).
            let gain_amp = gain[0];

            for i in 0..frames {
                buf.buf[i] *= gain_amp;
            }
        }
    }

    #[cfg(all(any(target_arch = "x86", target_arch = "x86_64")))]
    #[target_feature(enable = "avx")]
    pub unsafe fn optimized_multiply_mono_avx(
        buf: &mut MonoAudioBlockBuffer,
        gain: &[f32; MAX_BLOCKSIZE],
        frames: usize,
        is_smoothing: bool,
    ) {
        #[cfg(target_arch = "x86")]
        use std::arch::x86::*;
        #[cfg(target_arch = "x86_64")]
        use std::arch::x86_64::*;

        // Hint to compiler to optimize loops.
        let frames = frames.min(MAX_BLOCKSIZE);

        if is_smoothing {
            // Looping like this will cause this to process in chunks of cpu_id::AVX_F32_WIDTH.
            //
            // Even if the number of `frames` is not a multiple of cpu_id::AVX_F32_WIDTH, it
            // is more efficient to process it as a block anyway. It doesn't matter if the last block
            // contains uninitialized data because we stipulated that this data must not be read.
            for i in (0..frames).step_by(cpu_id::AVX_F32_WIDTH) {
                let src_v = _mm256_loadu_ps(&buf.buf[i]);
                let gain_v = _mm256_loadu_ps(&gain[i]);

                let mul_v = _mm256_mul_ps(src_v, gain_v);

                _mm256_storeu_ps(&mut buf.buf[i], mul_v);
            }
        } else {
            // We can optimize by using a constant gain (better SIMD load efficiency).
            let gain_v = _mm256_set1_ps(gain[0]);

            // Looping like this will cause this to process in chunks of cpu_id::AVX_F32_WIDTH.
            //
            // Even if the number of `frames` is not a multiple of cpu_id::AVX_F32_WIDTH, it
            // is more efficient to process it as a block anyway. It doesn't matter if the last block
            // contains uninitialized data because we stipulated that this data must not be read.
            for i in (0..frames).step_by(cpu_id::AVX_F32_WIDTH) {
                let src_v = _mm256_loadu_ps(&buf.buf[i]);

                let mul_v = _mm256_mul_ps(src_v, gain_v);

                _mm256_storeu_ps(&mut buf.buf[i], mul_v);
            }
        }
    }

    pub fn optimized_multiply_stereo_fallback(
        buf: &mut StereoAudioBlockBuffer,
        gain: &[f32; MAX_BLOCKSIZE],
        frames: usize,
        is_smoothing: bool,
    ) {
        // Hint to compiler to optimize loops.
        let frames = frames.min(MAX_BLOCKSIZE);

        if is_smoothing {
            for i in 0..frames {
                buf.left[i] *= gain[i];
                buf.right[i] *= gain[i];
            }
        } else {
            // We can optimize by using a constant gain (better SIMD load efficiency).
            let gain_amp = gain[0];

            for i in 0..frames {
                buf.left[i] *= gain_amp;
                buf.right[i] *= gain_amp;
            }
        }
    }

    #[cfg(all(any(target_arch = "x86", target_arch = "x86_64")))]
    #[target_feature(enable = "avx")]
    pub unsafe fn optimized_multiply_stereo_avx(
        buf: &mut StereoAudioBlockBuffer,
        gain: &[f32; MAX_BLOCKSIZE],
        frames: usize,
        is_smoothing: bool,
    ) {
        #[cfg(target_arch = "x86")]
        use std::arch::x86::*;
        #[cfg(target_arch = "x86_64")]
        use std::arch::x86_64::*;

        // Hint to compiler to optimize loops.
        let frames = frames.min(MAX_BLOCKSIZE);

        if is_smoothing {
            // Looping like this will cause this to process in chunks of cpu_id::AVX_F32_WIDTH.
            //
            // Even if the number of `frames` is not a multiple of cpu_id::AVX_F32_WIDTH, it
            // is more efficient to process it as a block anyway. It doesn't matter if the last block
            // contains uninitialized data because we stipulated that this data must not be read.
            for i in (0..frames).step_by(cpu_id::AVX_F32_WIDTH) {
                let src_left_v = _mm256_loadu_ps(&buf.left[i]);
                let src_right_v = _mm256_loadu_ps(&buf.right[i]);
                let gain_v = _mm256_loadu_ps(&gain[i]);

                let mul_left_v = _mm256_mul_ps(src_left_v, gain_v);
                let mul_right_v = _mm256_mul_ps(src_right_v, gain_v);

                _mm256_storeu_ps(&mut buf.left[i], mul_left_v);
                _mm256_storeu_ps(&mut buf.right[i], mul_right_v);
            }
        } else {
            // We can optimize by using a constant gain (better SIMD load efficiency).
            let gain_v = _mm256_set1_ps(gain[0]);

            // Looping like this will cause this to process in chunks of cpu_id::AVX_F32_WIDTH.
            //
            // Even if the number of `frames` is not a multiple of cpu_id::AVX_F32_WIDTH, it
            // is more efficient to process it as a block anyway. It doesn't matter if the last block
            // contains uninitialized data because we stipulated that this data must not be read.
            for i in (0..frames).step_by(cpu_id::AVX_F32_WIDTH) {
                let src_left_v = _mm256_loadu_ps(&buf.left[i]);
                let src_right_v = _mm256_loadu_ps(&buf.right[i]);

                let mul_left_v = _mm256_mul_ps(src_left_v, gain_v);
                let mul_right_v = _mm256_mul_ps(src_right_v, gain_v);

                _mm256_storeu_ps(&mut buf.left[i], mul_left_v);
                _mm256_storeu_ps(&mut buf.right[i], mul_right_v);
            }
        }
    }

    pub fn optimized_multiply_offset_stereo_fallback(
        buf: &mut StereoAudioBlockBuffer,
        gain: &[f32; MAX_BLOCKSIZE],
        frames: usize,
        offset: usize,
        is_smoothing: bool,
    ) {
        // Hint to compiler to optimize loops.
        let offset = offset.min(MAX_BLOCKSIZE);
        let frames = frames.min(MAX_BLOCKSIZE - offset);

        if is_smoothing {
            for i in 0..frames {
                buf.left[offset + i] *= gain[i];
                buf.right[offset + i] *= gain[i];
            }
        } else {
            // We can optimize by using a constant gain (better SIMD load efficiency).
            let gain_amp = gain[0];

            for i in 0..frames {
                buf.left[offset + i] *= gain_amp;
                buf.right[offset + i] *= gain_amp;
            }
        }
    }

    #[cfg(all(any(target_arch = "x86", target_arch = "x86_64")))]
    #[target_feature(enable = "avx")]
    pub unsafe fn optimized_multiply_offset_stereo_avx(
        buf: &mut StereoAudioBlockBuffer,
        gain: &[f32; MAX_BLOCKSIZE],
        frames: usize,
        offset: usize,
        is_smoothing: bool,
    ) {
        #[cfg(target_arch = "x86")]
        use std::arch::x86::*;
        #[cfg(target_arch = "x86_64")]
        use std::arch::x86_64::*;

        // Hint to compiler to optimize loops.
        let offset = offset.min(MAX_BLOCKSIZE);
        let frames = frames.min(MAX_BLOCKSIZE - offset);

        if is_smoothing {
            // Looping like this will cause this to process in chunks of cpu_id::AVX_F32_WIDTH.
            //
            // Even if the number of `frames` is not a multiple of cpu_id::AVX_F32_WIDTH, it
            // is more efficient to process it as a block anyway. It doesn't matter if the last block
            // contains uninitialized data because we stipulated that this data must not be read.
            for i in (0..frames).step_by(cpu_id::AVX_F32_WIDTH) {
                let src_left_v = _mm256_loadu_ps(&buf.left[offset + i]);
                let src_right_v = _mm256_loadu_ps(&buf.right[offset + i]);
                let gain_v = _mm256_loadu_ps(&gain[i]);

                let mul_left_v = _mm256_mul_ps(src_left_v, gain_v);
                let mul_right_v = _mm256_mul_ps(src_right_v, gain_v);

                _mm256_storeu_ps(&mut buf.left[offset + i], mul_left_v);
                _mm256_storeu_ps(&mut buf.right[offset + i], mul_right_v);
            }
        } else {
            // We can optimize by using a constant gain (better SIMD load efficiency).
            let gain_v = _mm256_set1_ps(gain[0]);

            // Looping like this will cause this to process in chunks of cpu_id::AVX_F32_WIDTH.
            //
            // Even if the number of `frames` is not a multiple of cpu_id::AVX_F32_WIDTH, it
            // is more efficient to process it as a block anyway. It doesn't matter if the last block
            // contains uninitialized data because we stipulated that this data must not be read.
            for i in (0..frames).step_by(cpu_id::AVX_F32_WIDTH) {
                let src_left_v = _mm256_loadu_ps(&buf.left[offset + i]);
                let src_right_v = _mm256_loadu_ps(&buf.right[offset + i]);

                let mul_left_v = _mm256_mul_ps(src_left_v, gain_v);
                let mul_right_v = _mm256_mul_ps(src_right_v, gain_v);

                _mm256_storeu_ps(&mut buf.left[offset + i], mul_left_v);
                _mm256_storeu_ps(&mut buf.right[offset + i], mul_right_v);
            }
        }
    }

    pub fn optimized_multiply_then_add_stereo_fallback(
        src: &StereoAudioBlockBuffer,
        dst: &mut StereoAudioBlockBuffer,
        gain: &[f32; MAX_BLOCKSIZE],
        frames: usize,
        is_smoothing: bool,
    ) {
        // Hint to compiler to optimize loops.
        let frames = frames.min(MAX_BLOCKSIZE);

        if is_smoothing {
            for i in 0..frames {
                dst.left[i] += src.left[i] * gain[i];
                dst.right[i] += src.right[i] * gain[i];
            }
        } else {
            // We can optimize by using a constant gain (better SIMD load efficiency).
            let gain_amp = gain[0];

            for i in 0..frames {
                dst.left[i] += src.left[i] * gain_amp;
                dst.right[i] += src.right[i] * gain_amp;
            }
        }
    }

    #[cfg(all(any(target_arch = "x86", target_arch = "x86_64")))]
    #[target_feature(enable = "avx")]
    pub unsafe fn optimized_multiply_then_add_stereo_avx(
        src: &StereoAudioBlockBuffer,
        dst: &mut StereoAudioBlockBuffer,
        gain: &[f32; MAX_BLOCKSIZE],
        frames: usize,
        is_smoothing: bool,
    ) {
        #[cfg(target_arch = "x86")]
        use std::arch::x86::*;
        #[cfg(target_arch = "x86_64")]
        use std::arch::x86_64::*;

        // Hint to compiler to optimize loops.
        let frames = frames.min(MAX_BLOCKSIZE);

        if is_smoothing {
            // Looping like this will cause this to process in chunks of cpu_id::AVX_F32_WIDTH.
            //
            // Even if the number of `frames` is not a multiple of cpu_id::AVX_F32_WIDTH, it
            // is more efficient to process it as a block anyway. It doesn't matter if the last block
            // contains uninitialized data because we stipulated that this data must not be read.
            for i in (0..frames).step_by(cpu_id::AVX_F32_WIDTH) {
                let src_left_v = _mm256_loadu_ps(&src.left[i]);
                let src_right_v = _mm256_loadu_ps(&src.right[i]);

                let dst_left_v = _mm256_loadu_ps(&dst.left[i]);
                let dst_right_v = _mm256_loadu_ps(&dst.right[i]);

                let gain_v = _mm256_loadu_ps(&gain[i]);

                let mul_src_left_v = _mm256_mul_ps(src_left_v, gain_v);
                let mul_src_right_v = _mm256_mul_ps(src_right_v, gain_v);

                let add_left_v = _mm256_add_ps(dst_left_v, mul_src_left_v);
                let add_right_v = _mm256_add_ps(dst_right_v, mul_src_right_v);

                _mm256_storeu_ps(&mut dst.left[i], add_left_v);
                _mm256_storeu_ps(&mut dst.right[i], add_right_v);
            }
        } else {
            // We can optimize by using a constant gain (better SIMD load efficiency).
            let gain_v = _mm256_set1_ps(gain[0]);

            // Looping like this will cause this to process in chunks of cpu_id::AVX_F32_WIDTH.
            //
            // Even if the number of `frames` is not a multiple of cpu_id::AVX_F32_WIDTH, it
            // is more efficient to process it as a block anyway. It doesn't matter if the last block
            // contains uninitialized data because we stipulated that this data must not be read.
            for i in (0..frames).step_by(cpu_id::AVX_F32_WIDTH) {
                let src_left_v = _mm256_loadu_ps(&src.left[i]);
                let src_right_v = _mm256_loadu_ps(&src.right[i]);

                let dst_left_v = _mm256_loadu_ps(&dst.left[i]);
                let dst_right_v = _mm256_loadu_ps(&dst.right[i]);

                let mul_src_left_v = _mm256_mul_ps(src_left_v, gain_v);
                let mul_src_right_v = _mm256_mul_ps(src_right_v, gain_v);

                let add_left_v = _mm256_add_ps(dst_left_v, mul_src_left_v);
                let add_right_v = _mm256_add_ps(dst_right_v, mul_src_right_v);

                _mm256_storeu_ps(&mut dst.left[i], add_left_v);
                _mm256_storeu_ps(&mut dst.right[i], add_right_v);
            }
        }
    }

    pub fn optimized_multiply_then_add_offset_stereo_fallback(
        src: &StereoAudioBlockBuffer,
        dst: &mut StereoAudioBlockBuffer,
        gain: &[f32; MAX_BLOCKSIZE],
        frames: usize,
        offset: usize,
        is_smoothing: bool,
    ) {
        // Hint to compiler to optimize loops.
        let offset = offset.min(MAX_BLOCKSIZE);
        let frames = frames.min(MAX_BLOCKSIZE - offset);

        if is_smoothing {
            for i in 0..frames {
                dst.left[offset + i] += src.left[offset + i] * gain[i];
                dst.right[offset + i] += src.right[offset + i] * gain[i];
            }
        } else {
            // We can optimize by using a constant gain (better SIMD load efficiency).
            let gain_amp = gain[0];

            for i in 0..frames {
                dst.left[offset + i] += src.left[offset + i] * gain_amp;
                dst.right[offset + i] += src.right[offset + i] * gain_amp;
            }
        }
    }

    #[cfg(all(any(target_arch = "x86", target_arch = "x86_64")))]
    #[target_feature(enable = "avx")]
    pub unsafe fn optimized_multiply_then_add_offset_stereo_avx(
        src: &StereoAudioBlockBuffer,
        dst: &mut StereoAudioBlockBuffer,
        gain: &[f32; MAX_BLOCKSIZE],
        frames: usize,
        offset: usize,
        is_smoothing: bool,
    ) {
        #[cfg(target_arch = "x86")]
        use std::arch::x86::*;
        #[cfg(target_arch = "x86_64")]
        use std::arch::x86_64::*;

        // Hint to compiler to optimize loops.
        let offset = offset.min(MAX_BLOCKSIZE);
        let frames = frames.min(MAX_BLOCKSIZE - offset);

        if is_smoothing {
            // Looping like this will cause this to process in chunks of cpu_id::AVX_F32_WIDTH.
            //
            // Even if the number of `frames` is not a multiple of cpu_id::AVX_F32_WIDTH, it
            // is more efficient to process it as a block anyway. It doesn't matter if the last block
            // contains uninitialized data because we stipulated that this data must not be read.
            for i in (0..frames).step_by(cpu_id::AVX_F32_WIDTH) {
                let src_left_v = _mm256_loadu_ps(&src.left[offset + i]);
                let src_right_v = _mm256_loadu_ps(&src.right[offset + i]);

                let dst_left_v = _mm256_loadu_ps(&dst.left[offset + i]);
                let dst_right_v = _mm256_loadu_ps(&dst.right[offset + i]);

                let gain_v = _mm256_loadu_ps(&gain[i]);

                let mul_src_left_v = _mm256_mul_ps(src_left_v, gain_v);
                let mul_src_right_v = _mm256_mul_ps(src_right_v, gain_v);

                let add_left_v = _mm256_add_ps(dst_left_v, mul_src_left_v);
                let add_right_v = _mm256_add_ps(dst_right_v, mul_src_right_v);

                _mm256_storeu_ps(&mut dst.left[offset + i], add_left_v);
                _mm256_storeu_ps(&mut dst.right[offset + i], add_right_v);
            }
        } else {
            // We can optimize by using a constant gain (better SIMD load efficiency).
            let gain_v = _mm256_set1_ps(gain[0]);

            // Looping like this will cause this to process in chunks of cpu_id::AVX_F32_WIDTH.
            //
            // Even if the number of `frames` is not a multiple of cpu_id::AVX_F32_WIDTH, it
            // is more efficient to process it as a block anyway. It doesn't matter if the last block
            // contains uninitialized data because we stipulated that this data must not be read.
            for i in (0..frames).step_by(cpu_id::AVX_F32_WIDTH) {
                let src_left_v = _mm256_loadu_ps(&src.left[offset + i]);
                let src_right_v = _mm256_loadu_ps(&src.right[offset + i]);

                let dst_left_v = _mm256_loadu_ps(&dst.left[offset + i]);
                let dst_right_v = _mm256_loadu_ps(&dst.right[offset + i]);

                let mul_src_left_v = _mm256_mul_ps(src_left_v, gain_v);
                let mul_src_right_v = _mm256_mul_ps(src_right_v, gain_v);

                let add_left_v = _mm256_add_ps(dst_left_v, mul_src_left_v);
                let add_right_v = _mm256_add_ps(dst_right_v, mul_src_right_v);

                _mm256_storeu_ps(&mut dst.left[offset + i], add_left_v);
                _mm256_storeu_ps(&mut dst.right[offset + i], add_right_v);
            }
        }
    }
}
